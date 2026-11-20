use nalgebra::{DMatrix, DVector};
use crate::fitting::varpro::{solve_linear_svd, VarProSolver};
use crate::fitting::stats::{compute_diagnostics, huber_weights, ModelDiagnostics};

/// The 15 supported explicit curve-fitting models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FitModelKind {
    Linear,
    Quadratic,
    Cubic,
    Degree4,
    Degree5,
    Exponential,
    Logarithmic,
    Power,
    Sinusoidal,
    Cosinusoidal,
    Tangential,
    Hyperbolic,
    SquareRoot,
    AbsoluteValue,
    Gaussian,
}

impl FitModelKind {
    pub fn name(&self) -> &'static str {
        match self {
            FitModelKind::Linear => "Linear",
            FitModelKind::Quadratic => "Quadratic",
            FitModelKind::Cubic => "Cubic",
            FitModelKind::Degree4 => "Quartic (Degree 4)",
            FitModelKind::Degree5 => "Quintic (Degree 5)",
            FitModelKind::Exponential => "Exponential",
            FitModelKind::Logarithmic => "Logarithmic",
            FitModelKind::Power => "Power",
            FitModelKind::Sinusoidal => "Sinusoidal",
            FitModelKind::Cosinusoidal => "Cosinusoidal",
            FitModelKind::Tangential => "Tangential",
            FitModelKind::Hyperbolic => "Hyperbolic",
            FitModelKind::SquareRoot => "Square Root",
            FitModelKind::AbsoluteValue => "Absolute Value",
            FitModelKind::Gaussian => "Gaussian",
        }
    }

    pub fn min_points(&self) -> usize {
        match self {
            FitModelKind::Linear => 2,
            FitModelKind::Quadratic => 3,
            FitModelKind::Cubic => 4,
            FitModelKind::Degree4 => 5,
            FitModelKind::Degree5 => 6,
            FitModelKind::Exponential => 2,
            FitModelKind::Logarithmic => 2,
            FitModelKind::Power => 2,
            FitModelKind::Sinusoidal => 4,
            FitModelKind::Cosinusoidal => 4,
            FitModelKind::Tangential => 4,
            FitModelKind::Hyperbolic => 3,
            FitModelKind::SquareRoot => 3,
            FitModelKind::AbsoluteValue => 3,
            FitModelKind::Gaussian => 4,
        }
    }

    pub fn all_models() -> &'static [FitModelKind] {
        &[
            FitModelKind::Linear,
            FitModelKind::Quadratic,
            FitModelKind::Cubic,
            FitModelKind::Degree4,
            FitModelKind::Degree5,
            FitModelKind::Exponential,
            FitModelKind::Logarithmic,
            FitModelKind::Power,
            FitModelKind::Sinusoidal,
            FitModelKind::Cosinusoidal,
            FitModelKind::Tangential,
            FitModelKind::Hyperbolic,
            FitModelKind::SquareRoot,
            FitModelKind::AbsoluteValue,
            FitModelKind::Gaussian,
        ]
    }
}

/// Raw result of fitting a model.
#[derive(Debug, Clone)]
pub struct RawFitResult {
    pub kind: FitModelKind,
    pub parameters: Vec<f64>,
    pub parameter_names: Vec<String>,
    pub equation_unicode: String,
    pub equation_latex: String,
    pub diagnostics: ModelDiagnostics,
    pub domain_note: Option<String>,
}

pub struct ModelFitter;

impl ModelFitter {
    /// Fit a model to data points (x, y) with optional Huber IRLS robust weighting.
    pub fn fit(
        kind: FitModelKind,
        x: &[f64],
        y: &[f64],
        use_robust: bool,
    ) -> Result<RawFitResult, String> {
        let n = x.len();
        if n < kind.min_points() {
            return Err(format!(
                "{} requires at least {} points (got {})",
                kind.name(),
                kind.min_points(),
                n
            ));
        }

        // Domain guards
        match kind {
            FitModelKind::Logarithmic => {
                if x.iter().any(|&v| v <= 0.0) {
                    return Err("Logarithmic fit requires all x > 0".into());
                }
            }
            FitModelKind::Power => {
                if x.iter().any(|&v| v <= 0.0) {
                    return Err("Power fit requires all x > 0".into());
                }
            }
            _ => {}
        }

        // Initial fit
        let (params, names) = Self::fit_raw(kind, x, y)?;

        // If robust is requested, run iterations of Huber IRLS reweighting
        let final_params = if use_robust {
            let mut current_p = params;
            for _ in 0..6 {
                let eval_fn = Self::eval_fn(kind);
                let residuals: Vec<f64> = (0..n).map(|i| y[i] - eval_fn(x[i], &current_p)).collect();
                let weights = huber_weights(&residuals);
                let sqrt_w: Vec<f64> = weights.iter().map(|w| w.sqrt()).collect();
                let y_weighted: Vec<f64> = (0..n).map(|i| y[i] * sqrt_w[i]).collect();
                if let Ok((p_new, _)) = Self::fit_raw_weighted(kind, x, &y_weighted, &sqrt_w) {
                    current_p = p_new;
                }
            }
            current_p
        } else {
            params
        };

        let eval_fn = Self::eval_fn(kind);
        let jac_fn = Self::jac_fn(kind);
        let diagnostics = compute_diagnostics(x, y, &final_params, eval_fn, jac_fn);

        let (eq_u, eq_l) = Self::format_equations(kind, &final_params);

        Ok(RawFitResult {
            kind,
            parameters: final_params,
            parameter_names: names,
            equation_unicode: eq_u,
            equation_latex: eq_l,
            diagnostics,
            domain_note: match kind {
                FitModelKind::Logarithmic => Some("Domain: x > 0".into()),
                FitModelKind::Power => Some("Domain: x > 0".into()),
                FitModelKind::SquareRoot => Some("Domain: x ≥ h".into()),
                _ => None,
            },
        })
    }

    /// Evaluates the fitted model at x.
    pub fn eval_fn(kind: FitModelKind) -> fn(f64, &[f64]) -> f64 {
        match kind {
            FitModelKind::Linear => |x, p| p[0] * x + p[1],
            FitModelKind::Quadratic => |x, p| p[0] * x * x + p[1] * x + p[2],
            FitModelKind::Cubic => |x, p| p[0] * x.powi(3) + p[1] * x * x + p[2] * x + p[3],
            FitModelKind::Degree4 => |x, p| p[0] * x.powi(4) + p[1] * x.powi(3) + p[2] * x * x + p[3] * x + p[4],
            FitModelKind::Degree5 => |x, p| p[0] * x.powi(5) + p[1] * x.powi(4) + p[2] * x.powi(3) + p[3] * x * x + p[4] * x + p[5],
            FitModelKind::Exponential => |x, p| p[0] * (p[1] * x).exp(),
            FitModelKind::Logarithmic => |x, p| if x > 0.0 { p[0] + p[1] * x.ln() } else { f64::NAN },
            FitModelKind::Power => |x, p| if x > 0.0 { p[0] * x.powf(p[1]) } else { f64::NAN },
            FitModelKind::Sinusoidal => |x, p| p[0] * (p[1] * x + p[2]).sin() + p[3],
            FitModelKind::Cosinusoidal => |x, p| p[0] * (p[1] * x + p[2]).cos() + p[3],
            FitModelKind::Tangential => |x, p| {
                let angle = p[1] * x + p[2];
                if angle.cos().abs() > 1e-4 { p[0] * angle.tan() + p[3] } else { f64::NAN }
            },
            FitModelKind::Hyperbolic => |x, p| {
                let denom = x - p[1];
                if denom.abs() > 1e-6 { p[0] / denom + p[2] } else { f64::NAN }
            },
            FitModelKind::SquareRoot => |x, p| {
                let diff = x - p[1];
                if diff >= 0.0 { p[0] * diff.sqrt() + p[2] } else { f64::NAN }
            },
            FitModelKind::AbsoluteValue => |x, p| p[0] * (x - p[1]).abs() + p[2],
            FitModelKind::Gaussian => |x, p| {
                let s = p[2].abs().max(1e-6);
                p[0] * (-(x - p[1]).powi(2) / (2.0 * s * s)).exp() + p[3]
            },
        }
    }

    /// Evaluates the gradient of the model with respect to parameters.
    pub fn jac_fn(kind: FitModelKind) -> fn(f64, &[f64]) -> Vec<f64> {
        match kind {
            FitModelKind::Linear => |x, _| vec![x, 1.0],
            FitModelKind::Quadratic => |x, _| vec![x * x, x, 1.0],
            FitModelKind::Cubic => |x, _| vec![x.powi(3), x * x, x, 1.0],
            FitModelKind::Degree4 => |x, _| vec![x.powi(4), x.powi(3), x * x, x, 1.0],
            FitModelKind::Degree5 => |x, _| vec![x.powi(5), x.powi(4), x.powi(3), x * x, x, 1.0],
            FitModelKind::Exponential => |x, p| {
                let exp_val = (p[1] * x).exp();
                vec![exp_val, p[0] * x * exp_val]
            },
            FitModelKind::Logarithmic => |x, _| {
                vec![1.0, if x > 0.0 { x.ln() } else { 0.0 }]
            },
            FitModelKind::Power => |x, p| {
                if x > 0.0 {
                    let pow_val = x.powf(p[1]);
                    vec![pow_val, p[0] * pow_val * x.ln()]
                } else {
                    vec![0.0, 0.0]
                }
            },
            FitModelKind::Sinusoidal => |x, p| {
                let ang = p[1] * x + p[2];
                vec![ang.sin(), p[0] * x * ang.cos(), p[0] * ang.cos(), 1.0]
            },
            FitModelKind::Cosinusoidal => |x, p| {
                let ang = p[1] * x + p[2];
                vec![ang.cos(), -p[0] * x * ang.sin(), -p[0] * ang.sin(), 1.0]
            },
            FitModelKind::Tangential => |x, p| {
                let ang = p[1] * x + p[2];
                let sec2 = 1.0 / (ang.cos().powi(2).max(1e-8));
                vec![ang.tan(), p[0] * x * sec2, p[0] * sec2, 1.0]
            },
            FitModelKind::Hyperbolic => |x, p| {
                let d = x - p[1];
                let d2 = (d * d).max(1e-12);
                vec![1.0 / d, p[0] / d2, 1.0]
            },
            FitModelKind::SquareRoot => |x, p| {
                let diff = (x - p[1]).max(1e-12);
                let sqrt_d = diff.sqrt();
                vec![sqrt_d, -0.5 * p[0] / sqrt_d, 1.0]
            },
            FitModelKind::AbsoluteValue => |x, p| {
                let s = if x >= p[1] { 1.0 } else { -1.0 };
                vec![(x - p[1]).abs(), -p[0] * s, 1.0]
            },
            FitModelKind::Gaussian => |x, p| {
                let s = p[2].abs().max(1e-6);
                let z = (x - p[1]) / s;
                let exp_val = (-0.5 * z * z).exp();
                vec![
                    exp_val,
                    p[0] * exp_val * (z / s),
                    p[0] * exp_val * (z * z / s),
                    1.0,
                ]
            },
        }
    }

    /// Internal fitting dispatch.
    fn fit_raw(kind: FitModelKind, x: &[f64], y: &[f64]) -> Result<(Vec<f64>, Vec<String>), String> {
        match kind {
            FitModelKind::Linear => Self::fit_polynomial(x, y, 1),
            FitModelKind::Quadratic => Self::fit_polynomial(x, y, 2),
            FitModelKind::Cubic => Self::fit_polynomial(x, y, 3),
            FitModelKind::Degree4 => Self::fit_polynomial(x, y, 4),
            FitModelKind::Degree5 => Self::fit_polynomial(x, y, 5),
            FitModelKind::Exponential => Self::fit_exponential(x, y),
            FitModelKind::Logarithmic => Self::fit_logarithmic(x, y),
            FitModelKind::Power => Self::fit_power(x, y),
            FitModelKind::Sinusoidal => Self::fit_sinusoidal(x, y, false),
            FitModelKind::Cosinusoidal => Self::fit_sinusoidal(x, y, true),
            FitModelKind::Tangential => Self::fit_tangential(x, y),
            FitModelKind::Hyperbolic => Self::fit_hyperbolic(x, y),
            FitModelKind::SquareRoot => Self::fit_square_root(x, y),
            FitModelKind::AbsoluteValue => Self::fit_absolute_value(x, y),
            FitModelKind::Gaussian => Self::fit_gaussian(x, y),
        }
    }

    fn fit_raw_weighted(
        kind: FitModelKind,
        x: &[f64],
        y_w: &[f64],
        sqrt_w: &[f64],
    ) -> Result<(Vec<f64>, Vec<String>), String> {
        let n = x.len();
        match kind {
            FitModelKind::Linear | FitModelKind::Quadratic | FitModelKind::Cubic
            | FitModelKind::Degree4 | FitModelKind::Degree5 => {
                let deg = match kind {
                    FitModelKind::Linear => 1,
                    FitModelKind::Quadratic => 2,
                    FitModelKind::Cubic => 3,
                    FitModelKind::Degree4 => 4,
                    FitModelKind::Degree5 => 5,
                    _ => unreachable!(),
                };
                let mut phi = DMatrix::zeros(n, deg + 1);
                for i in 0..n {
                    let w = sqrt_w[i];
                    for d in 0..=deg {
                        let power = (deg - d) as i32;
                        phi[(i, d)] = w * x[i].powi(power);
                    }
                }
                let y_vec = DVector::from_row_slice(y_w);
                let c = solve_linear_svd(&phi, &y_vec).ok_or("Failed weighted poly fit")?;
                let names = if deg == 1 {
                    vec!["m".into(), "b".into()]
                } else {
                    (0..=deg).map(|i| ((b'a' + i as u8) as char).to_string()).collect()
                };
                Ok((c.as_slice().to_vec(), names))
            }
            _ => Self::fit_raw(kind, x, y_w),
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // Polynomial Fit (Scaled Vandermonde SVD, unscaled back to x)
    // ─────────────────────────────────────────────────────────────────
    fn fit_polynomial(x: &[f64], y: &[f64], degree: usize) -> Result<(Vec<f64>, Vec<String>), String> {
        let n = x.len();
        let mean_x = x.iter().sum::<f64>() / (n as f64);
        let var_x = x.iter().map(|&v| (v - mean_x).powi(2)).sum::<f64>() / (n as f64);
        let scale_x = var_x.sqrt().max(1e-6);

        let u: Vec<f64> = x.iter().map(|&v| (v - mean_x) / scale_x).collect();

        let mut phi = DMatrix::zeros(n, degree + 1);
        for i in 0..n {
            let mut u_pow = 1.0;
            for d in (0..=degree).rev() {
                phi[(i, d)] = u_pow;
                u_pow *= u[i];
            }
        }

        let y_vec = DVector::from_row_slice(y);
        let c_u = solve_linear_svd(&phi, &y_vec).ok_or("Failed SVD on polynomial system")?;

        let mut c_x = vec![0.0; degree + 1];

        for k in 0..=degree {
            let p_deg = degree - k;
            let coeff_u = c_u[k];
            let inv_scale = (1.0 / scale_x).powi(p_deg as i32);
            for j in 0..=p_deg {
                let binom = Self::binomial(p_deg, j) as f64;
                let term = coeff_u * inv_scale * binom * (-mean_x).powi((p_deg - j) as i32);
                let target_idx = degree - j;
                c_x[target_idx] += term;
            }
        }

        let names: Vec<String> = match degree {
            1 => vec!["m".into(), "b".into()],
            _ => (0..=degree).map(|i| ((b'a' + i as u8) as char).to_string()).collect(),
        };

        Ok((c_x, names))
    }

    fn binomial(n: usize, k: usize) -> u64 {
        if k > n { return 0; }
        if k == 0 || k == n { return 1; }
        let mut c = 1;
        for i in 0..k {
            c = c * (n - i) as u64 / (i + 1) as u64;
        }
        c
    }

    // ─────────────────────────────────────────────────────────────────
    // Exponential Fit: y = a * exp(b * x)
    // ─────────────────────────────────────────────────────────────────
    fn fit_exponential(x: &[f64], y: &[f64]) -> Result<(Vec<f64>, Vec<String>), String> {
        let _n = x.len();
        let all_pos = y.iter().all(|&v| v > 0.0);
        let (init_a, init_b) = if all_pos {
            let ln_y: Vec<f64> = y.iter().map(|&v| v.ln()).collect();
            let (poly, _) = Self::fit_polynomial(x, &ln_y, 1)?;
            (poly[1].exp(), poly[0])
        } else {
            let y_min = y.iter().cloned().fold(f64::INFINITY, f64::min);
            let shift = (1.0 - y_min).max(0.1);
            let ln_y: Vec<f64> = y.iter().map(|&v| (v + shift).ln()).collect();
            let (poly, _) = Self::fit_polynomial(x, &ln_y, 1)?;
            (poly[1].exp(), poly[0])
        };

        let phi_fn = |alpha: &[f64], x_pts: &[f64]| {
            let b = alpha[0];
            let mut mat = DMatrix::zeros(x_pts.len(), 1);
            for i in 0..x_pts.len() {
                mat[(i, 0)] = (b * x_pts[i]).exp().clamp(-1e20, 1e20);
            }
            mat
        };

        let (best_b_vec, best_c, _) = VarProSolver::optimize_nonlinear(vec![init_b], x, y, phi_fn, 50, 1e-8);
        let a = if best_c.len() > 0 { best_c[0] } else { init_a };
        let b = best_b_vec[0];

        Ok((vec![a, b], vec!["a".into(), "b".into()]))
    }

    // ─────────────────────────────────────────────────────────────────
    // Logarithmic Fit: y = a + b * ln(x)
    // ─────────────────────────────────────────────────────────────────
    fn fit_logarithmic(x: &[f64], y: &[f64]) -> Result<(Vec<f64>, Vec<String>), String> {
        let n = x.len();
        let mut phi = DMatrix::zeros(n, 2);
        for i in 0..n {
            phi[(i, 0)] = 1.0;
            phi[(i, 1)] = x[i].ln();
        }
        let y_vec = DVector::from_row_slice(y);
        let c = solve_linear_svd(&phi, &y_vec).ok_or("Failed SVD on logarithmic fit")?;
        Ok((vec![c[0], c[1]], vec!["a".into(), "b".into()]))
    }

    // ─────────────────────────────────────────────────────────────────
    // Power Fit: y = a * x^b
    // ─────────────────────────────────────────────────────────────────
    fn fit_power(x: &[f64], y: &[f64]) -> Result<(Vec<f64>, Vec<String>), String> {
        let _n = x.len();
        let all_pos = y.iter().all(|&v| v > 0.0);
        let (init_a, init_b) = if all_pos {
            let ln_x: Vec<f64> = x.iter().map(|&v| v.ln()).collect();
            let ln_y: Vec<f64> = y.iter().map(|&v| v.ln()).collect();
            let (poly, _) = Self::fit_polynomial(&ln_x, &ln_y, 1)?;
            (poly[1].exp(), poly[0])
        } else {
            (1.0, 1.0)
        };

        let phi_fn = |alpha: &[f64], x_pts: &[f64]| {
            let b = alpha[0];
            let mut mat = DMatrix::zeros(x_pts.len(), 1);
            for i in 0..x_pts.len() {
                mat[(i, 0)] = if x_pts[i] > 0.0 { x_pts[i].powf(b).clamp(-1e20, 1e20) } else { 0.0 };
            }
            mat
        };

        let (best_b_vec, best_c, _) = VarProSolver::optimize_nonlinear(vec![init_b], x, y, phi_fn, 50, 1e-8);
        let a = if best_c.len() > 0 { best_c[0] } else { init_a };
        let b = best_b_vec[0];

        Ok((vec![a, b], vec!["a".into(), "b".into()]))
    }

    // ─────────────────────────────────────────────────────────────────
    // Periodic: Sinusoidal & Cosinusoidal (Lomb-Scargle + VarPro + LM)
    // ─────────────────────────────────────────────────────────────────
    fn fit_sinusoidal(x: &[f64], y: &[f64], is_cos: bool) -> Result<(Vec<f64>, Vec<String>), String> {
        let n = x.len();
        let x_min = x.iter().cloned().fold(f64::INFINITY, f64::min);
        let x_max = x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let t_span = (x_max - x_min).max(1e-4);

        let f_min = 1.0 / (2.0 * t_span);
        let f_max = (n as f64) / (2.0 * t_span);
        let n_freq = 60;
        let mut candidate_b = Vec::with_capacity(n_freq);
        for i in 0..n_freq {
            let freq = f_min + (i as f64) * (f_max - f_min) / (n_freq as f64);
            candidate_b.push(2.0 * std::f64::consts::PI * freq);
        }

        let phi_fn = move |alpha: &[f64], x_pts: &[f64]| {
            let b = alpha[0];
            let c = alpha[1];
            let mut mat = DMatrix::zeros(x_pts.len(), 2);
            for i in 0..x_pts.len() {
                let ang = b * x_pts[i] + c;
                mat[(i, 0)] = if is_cos { ang.cos() } else { ang.sin() };
                mat[(i, 1)] = 1.0;
            }
            mat
        };

        let mut best_sse = f64::INFINITY;
        let mut best_alpha = vec![candidate_b[0], 0.0];
        let mut best_lin = DVector::zeros(2);

        for &b_trial in candidate_b.iter().step_by(2) {
            for &c_trial in &[0.0, std::f64::consts::FRAC_PI_2, std::f64::consts::PI] {
                if let Some((lin, _, sse)) = VarProSolver::evaluate_projection(&[b_trial, c_trial], x, y, phi_fn) {
                    if sse < best_sse {
                        best_sse = sse;
                        best_alpha = vec![b_trial, c_trial];
                        best_lin = lin;
                    }
                }
            }
        }

        let (opt_alpha, opt_lin, _) = VarProSolver::optimize_nonlinear(best_alpha, x, y, phi_fn, 60, 1e-8);
        let a = if opt_lin.len() >= 2 { opt_lin[0] } else { best_lin[0] };
        let b = opt_alpha[0];
        let c = opt_alpha[1];
        let d = if opt_lin.len() >= 2 { opt_lin[1] } else { best_lin[1] };

        Ok((vec![a, b, c, d], vec!["a".into(), "b".into(), "c".into(), "d".into()]))
    }

    // ─────────────────────────────────────────────────────────────────
    // Tangential Fit: y = a * tan(b * x + c) + d
    // ─────────────────────────────────────────────────────────────────
    fn fit_tangential(x: &[f64], y: &[f64]) -> Result<(Vec<f64>, Vec<String>), String> {
        let _n = x.len();
        let x_min = x.iter().cloned().fold(f64::INFINITY, f64::min);
        let x_max = x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let t_span = (x_max - x_min).max(1e-4);

        let init_b = std::f64::consts::PI / (2.0 * t_span);
        let init_c = 0.0;

        let phi_fn = |alpha: &[f64], x_pts: &[f64]| {
            let b = alpha[0];
            let c = alpha[1];
            let mut mat = DMatrix::zeros(x_pts.len(), 2);
            for i in 0..x_pts.len() {
                let ang = b * x_pts[i] + c;
                let cos_val = ang.cos();
                mat[(i, 0)] = if cos_val.abs() > 1e-3 { ang.tan().clamp(-1e4, 1e4) } else { 1e4 * cos_val.signum() };
                mat[(i, 1)] = 1.0;
            }
            mat
        };

        let (opt_alpha, opt_lin, _) = VarProSolver::optimize_nonlinear(vec![init_b, init_c], x, y, phi_fn, 60, 1e-8);
        let a = if opt_lin.len() >= 2 { opt_lin[0] } else { 1.0 };
        let b = opt_alpha[0];
        let c = opt_alpha[1];
        let d = if opt_lin.len() >= 2 { opt_lin[1] } else { 0.0 };

        Ok((vec![a, b, c, d], vec!["a".into(), "b".into(), "c".into(), "d".into()]))
    }

    // ─────────────────────────────────────────────────────────────────
    // Hyperbolic Fit: y = a / (x - h) + k
    // ─────────────────────────────────────────────────────────────────
    fn fit_hyperbolic(x: &[f64], y: &[f64]) -> Result<(Vec<f64>, Vec<String>), String> {
        let n = x.len();
        let x_min = x.iter().cloned().fold(f64::INFINITY, f64::min);
        let x_max = x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let span = (x_max - x_min).max(1e-4);

        let mut candidate_h = Vec::new();
        candidate_h.push(x_min - 0.2 * span);
        candidate_h.push(x_max + 0.2 * span);
        for i in 0..(n - 1) {
            candidate_h.push(0.5 * (x[i] + x[i + 1]));
        }

        let phi_fn = |alpha: &[f64], x_pts: &[f64]| {
            let h = alpha[0];
            let mut mat = DMatrix::zeros(x_pts.len(), 2);
            for i in 0..x_pts.len() {
                let diff = x_pts[i] - h;
                mat[(i, 0)] = if diff.abs() > 1e-6 { 1.0 / diff } else { 1e6 * diff.signum() };
                mat[(i, 1)] = 1.0;
            }
            mat
        };

        let mut best_sse = f64::INFINITY;
        let mut best_h = candidate_h[0];
        for &h_trial in &candidate_h {
            if let Some((_, _, sse)) = VarProSolver::evaluate_projection(&[h_trial], x, y, phi_fn) {
                if sse < best_sse {
                    best_sse = sse;
                    best_h = h_trial;
                }
            }
        }

        let (opt_alpha, opt_lin, _) = VarProSolver::optimize_nonlinear(vec![best_h], x, y, phi_fn, 50, 1e-8);
        let a = if opt_lin.len() >= 2 { opt_lin[0] } else { 1.0 };
        let h = opt_alpha[0];
        let k = if opt_lin.len() >= 2 { opt_lin[1] } else { 0.0 };

        Ok((vec![a, h, k], vec!["a".into(), "h".into(), "k".into()]))
    }

    // ─────────────────────────────────────────────────────────────────
    // Square Root Fit: y = a * sqrt(x - h) + k (h <= min(x))
    // ─────────────────────────────────────────────────────────────────
    fn fit_square_root(x: &[f64], y: &[f64]) -> Result<(Vec<f64>, Vec<String>), String> {
        let x_min = x.iter().cloned().fold(f64::INFINITY, f64::min);
        let span = (x.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - x_min).max(1e-4);

        let phi_fn = move |alpha: &[f64], x_pts: &[f64]| {
            let delta = alpha[0];
            let h = x_min - delta * delta;
            let mut mat = DMatrix::zeros(x_pts.len(), 2);
            for i in 0..x_pts.len() {
                let diff = (x_pts[i] - h).max(0.0);
                mat[(i, 0)] = diff.sqrt();
                mat[(i, 1)] = 1.0;
            }
            mat
        };

        let (opt_delta, opt_lin, _) = VarProSolver::optimize_nonlinear(vec![0.1 * span.sqrt()], x, y, phi_fn, 50, 1e-8);
        let h = x_min - opt_delta[0] * opt_delta[0];
        let a = if opt_lin.len() >= 2 { opt_lin[0] } else { 1.0 };
        let k = if opt_lin.len() >= 2 { opt_lin[1] } else { 0.0 };

        Ok((vec![a, h, k], vec!["a".into(), "h".into(), "k".into()]))
    }

    // ─────────────────────────────────────────────────────────────────
    // Absolute Value Fit: y = a * |x - h| + k
    // ─────────────────────────────────────────────────────────────────
    fn fit_absolute_value(x: &[f64], y: &[f64]) -> Result<(Vec<f64>, Vec<String>), String> {
        let _n = x.len();
        let phi_fn = |alpha: &[f64], x_pts: &[f64]| {
            let h = alpha[0];
            let mut mat = DMatrix::zeros(x_pts.len(), 2);
            for i in 0..x_pts.len() {
                mat[(i, 0)] = (x_pts[i] - h).abs();
                mat[(i, 1)] = 1.0;
            }
            mat
        };

        let mut best_sse = f64::INFINITY;
        let mut best_h = x[0];
        for &h_trial in x {
            if let Some((_, _, sse)) = VarProSolver::evaluate_projection(&[h_trial], x, y, phi_fn) {
                if sse < best_sse {
                    best_sse = sse;
                    best_h = h_trial;
                }
            }
        }

        let (opt_alpha, opt_lin, _) = VarProSolver::optimize_nonlinear(vec![best_h], x, y, phi_fn, 40, 1e-8);
        let a = if opt_lin.len() >= 2 { opt_lin[0] } else { 1.0 };
        let h = opt_alpha[0];
        let k = if opt_lin.len() >= 2 { opt_lin[1] } else { 0.0 };

        Ok((vec![a, h, k], vec!["a".into(), "h".into(), "k".into()]))
    }

    // ─────────────────────────────────────────────────────────────────
    // Gaussian Fit: y = a * exp(-(x - mu)^2 / (2 * sigma^2)) + c
    // ─────────────────────────────────────────────────────────────────
    fn fit_gaussian(x: &[f64], y: &[f64]) -> Result<(Vec<f64>, Vec<String>), String> {
        let n = x.len();
        let y_min = y.iter().cloned().fold(f64::INFINITY, f64::min);
        let y_max = y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let peak_idx = (0..n).max_by(|&i, &j| y[i].partial_cmp(&y[j]).unwrap()).unwrap_or(0);
        let init_mu = x[peak_idx];

        let span = (x.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - x.iter().cloned().fold(f64::INFINITY, f64::min)).max(1e-4);
        let init_sigma = (span / 4.0).max(1e-3);

        let phi_fn = |alpha: &[f64], x_pts: &[f64]| {
            let mu = alpha[0];
            let sigma = alpha[1].exp();
            let mut mat = DMatrix::zeros(x_pts.len(), 2);
            for i in 0..x_pts.len() {
                let z = (x_pts[i] - mu) / sigma;
                mat[(i, 0)] = (-0.5 * z * z).exp();
                mat[(i, 1)] = 1.0;
            }
            mat
        };

        let (opt_alpha, opt_lin, _) = VarProSolver::optimize_nonlinear(vec![init_mu, init_sigma.ln()], x, y, phi_fn, 60, 1e-8);
        let a = if opt_lin.len() >= 2 { opt_lin[0] } else { y_max - y_min };
        let mu = opt_alpha[0];
        let sigma = opt_alpha[1].exp();
        let c = if opt_lin.len() >= 2 { opt_lin[1] } else { y_min };

        Ok((vec![a, mu, sigma, c], vec!["a".into(), "μ".into(), "σ".into(), "c".into()]))
    }

    // ─────────────────────────────────────────────────────────────────
    // Clean Equation Formatting (Unicode + LaTeX, No "+ -" Artifacts)
    // ─────────────────────────────────────────────────────────────────
    fn format_equations(kind: FitModelKind, p: &[f64]) -> (String, String) {
        match kind {
            FitModelKind::Linear => {
                let m = p[0];
                let b = p[1];
                let u = format!("y = {}x {}", Self::fmt_val(m), Self::fmt_signed(b));
                let l = format!("y = {}x {}", Self::fmt_latex(m), Self::fmt_signed_latex(b));
                (u, l)
            }
            FitModelKind::Quadratic => {
                let (a, b, c) = (p[0], p[1], p[2]);
                let u = format!("y = {}x² {}x {}", Self::fmt_val(a), Self::fmt_signed(b), Self::fmt_signed(c));
                let l = format!("y = {}x^2 {}x {}", Self::fmt_latex(a), Self::fmt_signed_latex(b), Self::fmt_signed_latex(c));
                (u, l)
            }
            FitModelKind::Cubic => {
                let (a, b, c, d) = (p[0], p[1], p[2], p[3]);
                let u = format!("y = {}x³ {}x² {}x {}", Self::fmt_val(a), Self::fmt_signed(b), Self::fmt_signed(c), Self::fmt_signed(d));
                let l = format!("y = {}x^3 {}x^2 {}x {}", Self::fmt_latex(a), Self::fmt_signed_latex(b), Self::fmt_signed_latex(c), Self::fmt_signed_latex(d));
                (u, l)
            }
            FitModelKind::Degree4 => {
                let u = format!("y = {}x⁴ {}x³ {}x² {}x {}", Self::fmt_val(p[0]), Self::fmt_signed(p[1]), Self::fmt_signed(p[2]), Self::fmt_signed(p[3]), Self::fmt_signed(p[4]));
                let l = format!("y = {}x^4 {}x^3 {}x^2 {}x {}", Self::fmt_latex(p[0]), Self::fmt_signed_latex(p[1]), Self::fmt_signed_latex(p[2]), Self::fmt_signed_latex(p[3]), Self::fmt_signed_latex(p[4]));
                (u, l)
            }
            FitModelKind::Degree5 => {
                let u = format!("y = {}x⁵ {}x⁴ {}x³ {}x² {}x {}", Self::fmt_val(p[0]), Self::fmt_signed(p[1]), Self::fmt_signed(p[2]), Self::fmt_signed(p[3]), Self::fmt_signed(p[4]), Self::fmt_signed(p[5]));
                let l = format!("y = {}x^5 {}x^4 {}x^3 {}x^2 {}x {}", Self::fmt_latex(p[0]), Self::fmt_signed_latex(p[1]), Self::fmt_signed_latex(p[2]), Self::fmt_signed_latex(p[3]), Self::fmt_signed_latex(p[4]), Self::fmt_signed_latex(p[5]));
                (u, l)
            }
            FitModelKind::Exponential => {
                let (a, b) = (p[0], p[1]);
                let u = format!("y = {}·e^({}x)", Self::fmt_val(a), Self::fmt_val(b));
                let l = format!("y = {} e^{{{}x}}", Self::fmt_latex(a), Self::fmt_latex(b));
                (u, l)
            }
            FitModelKind::Logarithmic => {
                let (a, b) = (p[0], p[1]);
                let u = format!("y = {} {}ln(x)", Self::fmt_val(a), Self::fmt_signed(b));
                let l = format!("y = {} {} \\ln(x)", Self::fmt_latex(a), Self::fmt_signed_latex(b));
                (u, l)
            }
            FitModelKind::Power => {
                let (a, b) = (p[0], p[1]);
                let u = format!("y = {}·x^({})", Self::fmt_val(a), Self::fmt_val(b));
                let l = format!("y = {} x^{{{}}}", Self::fmt_latex(a), Self::fmt_latex(b));
                (u, l)
            }
            FitModelKind::Sinusoidal => {
                let (a, b, c, d) = (p[0], p[1], p[2], p[3]);
                let u = format!("y = {}·sin({}x {}) {}", Self::fmt_val(a), Self::fmt_val(b), Self::fmt_signed(c), Self::fmt_signed(d));
                let l = format!("y = {} \\sin({}x {}) {}", Self::fmt_latex(a), Self::fmt_latex(b), Self::fmt_signed_latex(c), Self::fmt_signed_latex(d));
                (u, l)
            }
            FitModelKind::Cosinusoidal => {
                let (a, b, c, d) = (p[0], p[1], p[2], p[3]);
                let u = format!("y = {}·cos({}x {}) {}", Self::fmt_val(a), Self::fmt_val(b), Self::fmt_signed(c), Self::fmt_signed(d));
                let l = format!("y = {} \\cos({}x {}) {}", Self::fmt_latex(a), Self::fmt_latex(b), Self::fmt_signed_latex(c), Self::fmt_signed_latex(d));
                (u, l)
            }
            FitModelKind::Tangential => {
                let (a, b, c, d) = (p[0], p[1], p[2], p[3]);
                let u = format!("y = {}·tan({}x {}) {}", Self::fmt_val(a), Self::fmt_val(b), Self::fmt_signed(c), Self::fmt_signed(d));
                let l = format!("y = {} \\tan({}x {}) {}", Self::fmt_latex(a), Self::fmt_latex(b), Self::fmt_signed_latex(c), Self::fmt_signed_latex(d));
                (u, l)
            }
            FitModelKind::Hyperbolic => {
                let (a, h, k) = (p[0], p[1], p[2]);
                let u = format!("y = {} / (x {}) {}", Self::fmt_val(a), Self::fmt_signed(-h), Self::fmt_signed(k));
                let l = format!("y = \\frac{{{}}}{{x {}}} {}", Self::fmt_latex(a), Self::fmt_signed_latex(-h), Self::fmt_signed_latex(k));
                (u, l)
            }
            FitModelKind::SquareRoot => {
                let (a, h, k) = (p[0], p[1], p[2]);
                let u = format!("y = {}·√(x {}) {}", Self::fmt_val(a), Self::fmt_signed(-h), Self::fmt_signed(k));
                let l = format!("y = {} \\sqrt{{x {}}} {}", Self::fmt_latex(a), Self::fmt_signed_latex(-h), Self::fmt_signed_latex(k));
                (u, l)
            }
            FitModelKind::AbsoluteValue => {
                let (a, h, k) = (p[0], p[1], p[2]);
                let u = format!("y = {}·|x {}| {}", Self::fmt_val(a), Self::fmt_signed(-h), Self::fmt_signed(k));
                let l = format!("y = {} |x {}| {}", Self::fmt_latex(a), Self::fmt_signed_latex(-h), Self::fmt_signed_latex(k));
                (u, l)
            }
            FitModelKind::Gaussian => {
                let (a, mu, sigma, c) = (p[0], p[1], p[2], p[3]);
                let u = format!("y = {}·exp(-(x - {})² / (2·{}²)) {}", Self::fmt_val(a), Self::fmt_val(mu), Self::fmt_val(sigma), Self::fmt_signed(c));
                let l = format!("y = {} \\exp\\!\\left(-\\frac{{(x - {})^2}}{{2 \\cdot {}^2}}\\right) {}", Self::fmt_latex(a), Self::fmt_latex(mu), Self::fmt_latex(sigma), Self::fmt_signed_latex(c));
                (u, l)
            }
        }
    }

    fn fmt_val(v: f64) -> String {
        if !v.is_finite() { return format!("{}", v); }
        let abs_v = v.abs();
        if abs_v == 0.0 {
            "0".into()
        } else if abs_v < 1e-4 || abs_v >= 1e5 {
            format!("{:.4e}", v)
        } else {
            let s = format!("{:.4}", v);
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        }
    }

    fn fmt_signed(v: f64) -> String {
        if !v.is_finite() { return format!("+ {}", v); }
        if v < 0.0 {
            format!("- {}", Self::fmt_val(-v))
        } else {
            format!("+ {}", Self::fmt_val(v))
        }
    }

    fn fmt_latex(v: f64) -> String {
        if !v.is_finite() { return format!("{}", v); }
        let abs_v = v.abs();
        if abs_v == 0.0 {
            "0".into()
        } else if abs_v < 1e-4 || abs_v >= 1e5 {
            let s = format!("{:.4e}", v);
            let parts: Vec<&str> = s.split('e').collect();
            if parts.len() == 2 {
                let exp: i32 = parts[1].parse().unwrap_or(0);
                format!("{} \\times 10^{{{}}}", parts[0], exp)
            } else {
                s
            }
        } else {
            let s = format!("{:.4}", v);
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        }
    }

    fn fmt_signed_latex(v: f64) -> String {
        if !v.is_finite() { return format!("+ {}", v); }
        if v < 0.0 {
            format!("- {}", Self::fmt_latex(-v))
        } else {
            format!("+ {}", Self::fmt_latex(v))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_fit_exact() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![5.0, 7.0, 9.0, 11.0, 13.0];
        let res = ModelFitter::fit(FitModelKind::Linear, &x, &y, false).unwrap();
        assert!((res.parameters[0] - 2.0).abs() < 1e-6);
        assert!((res.parameters[1] - 3.0).abs() < 1e-6);
        assert!((res.diagnostics.r_squared - 1.0).abs() < 1e-6);
        assert!(!res.equation_unicode.contains("+ -"));
    }

    #[test]
    fn test_quadratic_fit_exact() {
        let x = vec![-2.0, -1.0, 0.0, 1.0, 2.0, 3.0];
        let y: Vec<f64> = x.iter().map(|&v| v * v - 2.0 * v + 1.0).collect();
        let res = ModelFitter::fit(FitModelKind::Quadratic, &x, &y, false).unwrap();
        assert!((res.parameters[0] - 1.0).abs() < 1e-5);
        assert!((res.parameters[1] - (-2.0)).abs() < 1e-5);
        assert!((res.parameters[2] - 1.0).abs() < 1e-5);
        assert!((res.diagnostics.r_squared - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cubic_fit() {
        let x: Vec<f64> = vec![-2.0, -1.0, 0.0, 1.0, 2.0, 3.0];
        let y: Vec<f64> = x.iter().map(|&v| v.powi(3) - v + 2.0).collect();
        let res = ModelFitter::fit(FitModelKind::Cubic, &x, &y, false).unwrap();
        assert!((res.parameters[0] - 1.0).abs() < 1e-4);
        assert!((res.parameters[1] - 0.0).abs() < 1e-4);
        assert!((res.parameters[2] - (-1.0)).abs() < 1e-4);
        assert!((res.parameters[3] - 2.0).abs() < 1e-4);
        assert!(res.diagnostics.r_squared > 0.999);
    }

    #[test]
    fn test_degree4_fit() {
        let x: Vec<f64> = vec![-2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0];
        let y: Vec<f64> = x.iter().map(|&v| 0.5 * v.powi(4) - v.powi(2) + 1.0).collect();
        let res = ModelFitter::fit(FitModelKind::Degree4, &x, &y, false).unwrap();
        assert!((res.parameters[0] - 0.5).abs() < 1e-4);
        assert!(res.diagnostics.r_squared > 0.999);
    }

    #[test]
    fn test_degree5_fit() {
        let x: Vec<f64> = vec![-3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0];
        let y: Vec<f64> = x.iter().map(|&v| 0.1 * v.powi(5) - 0.5 * v.powi(3) + 2.0).collect();
        let res = ModelFitter::fit(FitModelKind::Degree5, &x, &y, false).unwrap();
        assert!((res.parameters[0] - 0.1).abs() < 1e-3);
        assert!(res.diagnostics.r_squared > 0.999);
    }

    #[test]
    fn test_exponential_fit() {
        let x: Vec<f64> = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y: Vec<f64> = x.iter().map(|&v| 2.0 * (0.5 * v).exp()).collect();
        let res = ModelFitter::fit(FitModelKind::Exponential, &x, &y, false).unwrap();
        assert!((res.parameters[0] - 2.0).abs() < 1e-2);
        assert!((res.parameters[1] - 0.5).abs() < 1e-2);
        assert!(res.diagnostics.r_squared > 0.99);
    }

    #[test]
    fn test_logarithmic_fit() {
        let x: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y: Vec<f64> = x.iter().map(|&v| 3.0 + 2.0 * v.ln()).collect();
        let res = ModelFitter::fit(FitModelKind::Logarithmic, &x, &y, false).unwrap();
        assert!((res.parameters[0] - 3.0).abs() < 1e-6);
        assert!((res.parameters[1] - 2.0).abs() < 1e-6);
        assert!((res.diagnostics.r_squared - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_logarithmic_domain_guard() {
        let x = vec![-1.0, 1.0, 2.0];
        let y = vec![1.0, 2.0, 3.0];
        assert!(ModelFitter::fit(FitModelKind::Logarithmic, &x, &y, false).is_err());
    }

    #[test]
    fn test_power_fit() {
        let x: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y: Vec<f64> = x.iter().map(|&v| 2.5 * v.powf(1.5)).collect();
        let res = ModelFitter::fit(FitModelKind::Power, &x, &y, false).unwrap();
        assert!((res.parameters[0] - 2.5).abs() < 1e-2);
        assert!((res.parameters[1] - 1.5).abs() < 1e-2);
        assert!(res.diagnostics.r_squared > 0.99);
    }

    #[test]
    fn test_sinusoidal_fit() {
        let x: Vec<f64> = (0..20).map(|i| i as f64 * 0.3).collect();
        let y: Vec<f64> = x.iter().map(|&v| 3.0 * (2.0 * v + 0.5).sin() + 1.0).collect();
        let res = ModelFitter::fit(FitModelKind::Sinusoidal, &x, &y, false).unwrap();
        assert!(res.diagnostics.r_squared > 0.95);
    }

    #[test]
    fn test_cosinusoidal_fit() {
        let x: Vec<f64> = (0..20).map(|i| i as f64 * 0.4).collect();
        let y: Vec<f64> = x.iter().map(|&v| 2.0 * (1.5 * v + 0.3).cos() + 4.0).collect();
        let res = ModelFitter::fit(FitModelKind::Cosinusoidal, &x, &y, false).unwrap();
        assert!(res.diagnostics.r_squared > 0.95);
    }

    #[test]
    fn test_hyperbolic_fit() {
        let x: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let y: Vec<f64> = x.iter().map(|&v| 3.0 / (v - 0.5) + 2.0).collect();
        let res = ModelFitter::fit(FitModelKind::Hyperbolic, &x, &y, false).unwrap();
        assert!((res.parameters[0] - 3.0).abs() < 0.2);
        assert!((res.parameters[1] - 0.5).abs() < 0.1);
        assert!((res.parameters[2] - 2.0).abs() < 0.2);
        assert!(res.diagnostics.r_squared > 0.98);
    }

    #[test]
    fn test_square_root_fit() {
        let x: Vec<f64> = vec![2.0, 3.0, 5.0, 7.0, 10.0];
        let y: Vec<f64> = x.iter().map(|&v| 2.0 * (v - 1.0).sqrt() + 3.0).collect();
        let res = ModelFitter::fit(FitModelKind::SquareRoot, &x, &y, false).unwrap();
        assert!(res.diagnostics.r_squared > 0.98);
    }

    #[test]
    fn test_absolute_value_fit() {
        let x: Vec<f64> = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y: Vec<f64> = x.iter().map(|&v| 1.5 * (v - 2.0).abs() + 1.0).collect();
        let res = ModelFitter::fit(FitModelKind::AbsoluteValue, &x, &y, false).unwrap();
        assert!((res.parameters[0] - 1.5).abs() < 1e-4);
        assert!((res.parameters[1] - 2.0).abs() < 1e-4);
        assert!((res.parameters[2] - 1.0).abs() < 1e-4);
        assert!(res.diagnostics.r_squared > 0.999);
    }

    #[test]
    fn test_gaussian_fit() {
        let x: Vec<f64> = (-2..10).map(|i| i as f64 * 0.5).collect();
        let y: Vec<f64> = x.iter().map(|&v| 4.0 * (-(v - 2.0).powi(2) / 2.0).exp() + 0.5).collect();
        let res = ModelFitter::fit(FitModelKind::Gaussian, &x, &y, false).unwrap();
        assert!((res.parameters[0] - 4.0).abs() < 0.2);
        assert!((res.parameters[1] - 2.0).abs() < 0.2);
        assert!(res.diagnostics.r_squared > 0.98);
    }

    #[test]
    fn test_huber_outlier_robustness() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let mut y: Vec<f64> = x.iter().map(|&v| 2.0 * v + 1.0).collect();
        y[8] = 100.0; // severe outlier at high leverage point x=9

        let non_robust = ModelFitter::fit(FitModelKind::Linear, &x, &y, false).unwrap();
        let robust = ModelFitter::fit(FitModelKind::Linear, &x, &y, true).unwrap();

        let non_robust_err = (non_robust.parameters[0] - 2.0).abs();
        let robust_err = (robust.parameters[0] - 2.0).abs();
        assert!(robust_err < non_robust_err, "Robust err {} should be < non-robust err {}", robust_err, non_robust_err);
        assert!(robust_err < 0.2);
    }
}
