use nalgebra::{DMatrix, DVector};

/// Diagnostic statistical metrics from a fitted model.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelDiagnostics {
    pub r_squared: f64,
    pub adjusted_r_squared: f64,
    pub rmse: f64,
    pub mae: f64,
    pub aicc: f64,
    pub bic: f64,
    pub loocv_rmse: f64,
    pub residuals: Vec<f64>,
    pub cooks_distances: Vec<f64>,
    pub standard_errors: Vec<f64>,
    pub conf_band_upper: Vec<f32>,
    pub conf_band_lower: Vec<f32>,
    pub pred_band_upper: Vec<f32>,
    pub pred_band_lower: Vec<f32>,
}

/// Computes comprehensive statistical diagnostics for any model given data, predictions, and Jacobian.
pub fn compute_diagnostics<EvalFn, JacFn>(
    x: &[f64],
    y: &[f64],
    params: &[f64],
    eval_fn: EvalFn,
    jac_fn: JacFn,
) -> ModelDiagnostics
where
    EvalFn: Fn(f64, &[f64]) -> f64,
    JacFn: Fn(f64, &[f64]) -> Vec<f64>,
{
    let n = x.len();
    let p = params.len();

    if n == 0 {
        return empty_diagnostics(0);
    }

    let mut y_pred = Vec::with_capacity(n);
    let mut residuals = Vec::with_capacity(n);
    let mut sse = 0.0;
    let mut sum_abs_res = 0.0;
    let y_mean = y.iter().sum::<f64>() / (n as f64);
    let mut sst = 0.0;

    for i in 0..n {
        let yp = eval_fn(x[i], params);
        y_pred.push(yp);
        let res = y[i] - yp;
        residuals.push(res);
        sse += res * res;
        sum_abs_res += res.abs();
        sst += (y[i] - y_mean).powi(2);
    }

    let r_squared = if sst > 1e-15 {
        (1.0 - sse / sst).clamp(0.0, 1.0)
    } else {
        1.0
    };

    let adjusted_r_squared = if n > p && sst > 1e-15 {
        let df_t = (n - 1) as f64;
        let df_e = (n - p) as f64;
        (1.0 - (sse / df_e) / (sst / df_t)).clamp(-1.0, 1.0)
    } else {
        r_squared
    };

    let rmse = (sse / (n as f64)).sqrt();
    let mae = sum_abs_res / (n as f64);

    let n_f = n as f64;
    let p_f = p as f64;
    let safe_sse = sse.max(1e-15);

    let aic = n_f * (safe_sse / n_f).ln() + 2.0 * p_f;
    let aicc = if n > p + 1 {
        aic + (2.0 * p_f * (p_f + 1.0)) / (n_f - p_f - 1.0)
    } else {
        aic + 2.0 * p_f
    };

    let bic = n_f * (safe_sse / n_f).ln() + p_f * n_f.ln();

    // Compute Jacobian matrix J (n x p)
    let mut j_mat = DMatrix::zeros(n, p);
    for i in 0..n {
        let grad = jac_fn(x[i], params);
        for j in 0..p {
            j_mat[(i, j)] = if j < grad.len() { grad[j] } else { 0.0 };
        }
    }

    // Covariance matrix = s^2 * (J^T J)^+
    let jt = j_mat.transpose();
    let jtj = &jt * &j_mat;
    let s2 = if n > p { sse / ((n - p) as f64) } else { sse / n_f };

    let cov_opt = jtj.clone().pseudo_inverse(1e-12).ok();
    let cov = cov_opt.unwrap_or_else(|| DMatrix::identity(p, p) * 1e-6);

    let mut standard_errors = Vec::with_capacity(p);
    for j in 0..p {
        let var = cov[(j, j)] * s2;
        standard_errors.push(if var > 0.0 { var.sqrt() } else { 0.0 });
    }

    // Hat matrix leverage h_i = J_i * Cov * J_i^T
    let mut leverages = Vec::with_capacity(n);
    let mut cooks_distances = Vec::with_capacity(n);
    let mut loocv_sq_sum = 0.0;

    for i in 0..n {
        let row_vec = j_mat.row(i).transpose();
        let h_i = (row_vec.transpose() * &cov * &row_vec)[(0, 0)].clamp(0.0, 0.999);
        leverages.push(h_i);

        let d_i = if p > 0 && s2 > 1e-15 && (1.0 - h_i) > 1e-6 {
            let res = residuals[i];
            (res.powi(2) / (p_f * s2)) * (h_i / (1.0 - h_i).powi(2))
        } else {
            0.0
        };
        cooks_distances.push(d_i);

        let loocv_err = residuals[i] / (1.0 - h_i).max(0.01);
        loocv_sq_sum += loocv_err * loocv_err;
    }

    let loocv_rmse = (loocv_sq_sum / n_f).sqrt();

    // 95% Confidence and Prediction Bands across 200 points in [min(x), max(x)]
    let mut conf_upper = Vec::with_capacity(400);
    let mut conf_lower = Vec::with_capacity(400);
    let mut pred_upper = Vec::with_capacity(400);
    let mut pred_lower = Vec::with_capacity(400);

    let x_min = x.iter().cloned().fold(f64::INFINITY, f64::min);
    let x_max = x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let span = (x_max - x_min).max(1e-4);
    let margin = span * 0.05;
    let plot_min = x_min - margin;
    let plot_max = x_max + margin;
    let t_crit = 1.96; // ~95% confidence standard normal approximation
    let _s = s2.sqrt();

    let grid_steps = 150;
    for k in 0..=grid_steps {
        let gx = plot_min + (k as f64) * (plot_max - plot_min) / (grid_steps as f64);
        let gy = eval_fn(gx, params);
        let grad = jac_fn(gx, params);
        let mut g_vec = DVector::zeros(p);
        for j in 0..p {
            g_vec[j] = if j < grad.len() { grad[j] } else { 0.0 };
        }
        let se_mean = ((g_vec.transpose() * &cov * &g_vec)[(0, 0)] * s2).max(0.0).sqrt();
        let se_pred = (s2 + se_mean * se_mean).sqrt();

        conf_upper.push(gx as f32);
        conf_upper.push((gy + t_crit * se_mean) as f32);

        conf_lower.push(gx as f32);
        conf_lower.push((gy - t_crit * se_mean) as f32);

        pred_upper.push(gx as f32);
        pred_upper.push((gy + t_crit * se_pred) as f32);

        pred_lower.push(gx as f32);
        pred_lower.push((gy - t_crit * se_pred) as f32);
    }

    ModelDiagnostics {
        r_squared,
        adjusted_r_squared,
        rmse,
        mae,
        aicc,
        bic,
        loocv_rmse,
        residuals,
        cooks_distances,
        standard_errors,
        conf_band_upper: conf_upper,
        conf_band_lower: conf_lower,
        pred_band_upper: pred_upper,
        pred_band_lower: pred_lower,
    }
}

/// Huber Iteratively Reweighted Least Squares (IRLS) weight calculation.
pub fn huber_weights(residuals: &[f64]) -> Vec<f64> {
    if residuals.is_empty() {
        return Vec::new();
    }

    let mut abs_res: Vec<f64> = residuals.iter().map(|r| r.abs()).collect();
    abs_res.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if abs_res.len() % 2 == 1 {
        abs_res[abs_res.len() / 2]
    } else {
        0.5 * (abs_res[abs_res.len() / 2 - 1] + abs_res[abs_res.len() / 2])
    };

    let mad = (1.4826 * median).max(1e-6);
    let k = 1.345 * mad;

    residuals
        .iter()
        .map(|r| {
            let abs_r = r.abs();
            if abs_r <= k {
                1.0
            } else {
                k / abs_r
            }
        })
        .collect()
}

fn empty_diagnostics(p: usize) -> ModelDiagnostics {
    ModelDiagnostics {
        r_squared: 0.0,
        adjusted_r_squared: 0.0,
        rmse: 0.0,
        mae: 0.0,
        aicc: 0.0,
        bic: 0.0,
        loocv_rmse: 0.0,
        residuals: Vec::new(),
        cooks_distances: Vec::new(),
        standard_errors: vec![0.0; p],
        conf_band_upper: Vec::new(),
        conf_band_lower: Vec::new(),
        pred_band_upper: Vec::new(),
        pred_band_lower: Vec::new(),
    }
}
