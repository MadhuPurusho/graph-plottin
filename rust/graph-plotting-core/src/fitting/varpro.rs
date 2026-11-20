use nalgebra::{DMatrix, DVector};

/// Solves a linear least-squares problem Phi * c = y via Singular Value Decomposition (SVD).
/// Returns optimal linear coefficients c, or None if singular.
pub fn solve_linear_svd(phi: &DMatrix<f64>, y: &DVector<f64>) -> Option<DVector<f64>> {
    let svd = phi.clone().svd(true, true);
    let u = svd.u.as_ref()?;
    let v_t = svd.v_t.as_ref()?;
    let singular_values = &svd.singular_values;

    let max_sv = singular_values.iter().cloned().fold(0.0_f64, f64::max);
    let eps = 1e-12 * max_sv.max(1.0);

    // Compute pseudo-inverse: c = V * Sigma^+ * U^T * y
    let u_t_y = u.transpose() * y;
    let mut s_inv = DVector::zeros(singular_values.len());
    for i in 0..singular_values.len() {
        if singular_values[i] > eps {
            s_inv[i] = u_t_y[i] / singular_values[i];
        } else {
            s_inv[i] = 0.0;
        }
    }

    let c = v_t.transpose() * s_inv;
    Some(c)
}

/// Variable Projection (VarPro) state for optimizing nonlinear parameters alpha
/// while eliminating linear parameters c analytically.
pub struct VarProSolver;

impl VarProSolver {
    /// Given data x, y and a basis function generator phi_fn(alpha, x) -> DMatrix,
    /// compute the optimal linear coefficients c(alpha) and the projected residual vector r(alpha) = y - Phi * c.
    pub fn evaluate_projection<F>(
        alpha: &[f64],
        x: &[f64],
        y: &[f64],
        phi_fn: F,
    ) -> Option<(DVector<f64>, DVector<f64>, f64)>
    where
        F: Fn(&[f64], &[f64]) -> DMatrix<f64>,
    {
        let _n = x.len();
        let y_vec = DVector::from_row_slice(y);
        let phi = phi_fn(alpha, x);

        let c = solve_linear_svd(&phi, &y_vec)?;
        let y_hat = &phi * &c;
        let residual = &y_vec - y_hat;
        let sse = residual.norm_squared();

        Some((c, residual, sse))
    }

    /// Levenberg-Marquardt optimization of nonlinear parameters alpha using finite-difference Jacobian.
    pub fn optimize_nonlinear<F>(
        init_alpha: Vec<f64>,
        x: &[f64],
        y: &[f64],
        phi_fn: F,
        max_iter: usize,
        tol: f64,
    ) -> (Vec<f64>, DVector<f64>, f64)
    where
        F: Fn(&[f64], &[f64]) -> DMatrix<f64> + Copy,
    {
        let mut alpha = init_alpha;
        let p_nl = alpha.len();
        let n = x.len();

        let (mut best_c, mut best_r, mut best_sse) = match Self::evaluate_projection(&alpha, x, y, phi_fn) {
            Some(res) => res,
            None => return (alpha, DVector::zeros(n), f64::NAN),
        };

        if p_nl == 0 {
            return (alpha, best_c, best_sse);
        }

        let mut lambda = 1e-3;

        for _ in 0..max_iter {
            // Compute Jacobian J of residual with respect to alpha via central differences
            let mut j = DMatrix::zeros(n, p_nl);
            for k in 0..p_nl {
                let h = (alpha[k].abs() * 1e-5).max(1e-6);
                let mut alpha_plus = alpha.clone();
                alpha_plus[k] += h;
                let mut alpha_minus = alpha.clone();
                alpha_minus[k] -= h;

                let r_plus = Self::evaluate_projection(&alpha_plus, x, y, phi_fn)
                    .map(|(_, r, _)| r)
                    .unwrap_or_else(|| best_r.clone());
                let r_minus = Self::evaluate_projection(&alpha_minus, x, y, phi_fn)
                    .map(|(_, r, _)| r)
                    .unwrap_or_else(|| best_r.clone());

                let diff = (r_plus - r_minus) / (2.0 * h);
                for i in 0..n {
                    j[(i, k)] = diff[i];
                }
            }

            // Normal equations: (J^T J + lambda * diag(J^T J)) delta = - J^T r
            let jt = j.transpose();
            let mut jtj = &jt * &j;
            for k in 0..p_nl {
                jtj[(k, k)] += lambda * (jtj[(k, k)].abs().max(1e-4));
            }
            let jtr = &jt * &best_r;

            // Solve for step delta
            let delta = match jtj.clone().lu().solve(&(-jtr)) {
                Some(d) => d,
                None => {
                    lambda *= 5.0;
                    continue;
                }
            };

            let mut candidate_alpha = alpha.clone();
            for k in 0..p_nl {
                candidate_alpha[k] += delta[k];
            }

            if let Some((cand_c, cand_r, cand_sse)) = Self::evaluate_projection(&candidate_alpha, x, y, phi_fn) {
                if cand_sse < best_sse {
                    // Step accepted
                    alpha = candidate_alpha;
                    best_c = cand_c;
                    best_r = cand_r;
                    let diff_sse = best_sse - cand_sse;
                    best_sse = cand_sse;
                    lambda = (lambda * 0.2).max(1e-7);

                    if diff_sse < tol || delta.norm() < tol {
                        break;
                    }
                } else {
                    // Step rejected
                    lambda = (lambda * 5.0).min(1e8);
                }
            } else {
                lambda = (lambda * 5.0).min(1e8);
            }
        }

        (alpha, best_c, best_sse)
    }
}
