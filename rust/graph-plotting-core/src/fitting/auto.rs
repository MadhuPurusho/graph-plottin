use rayon::prelude::*;
use crate::fitting::models::{FitModelKind, ModelFitter, RawFitResult};

/// Auto Fit engine racing all valid fit models in parallel and ranking by AICc.
pub struct AutoFitter;

impl AutoFitter {
    /// Races all domain-valid models in parallel with Rayon and ranks candidates by AICc.
    pub fn auto_fit(x: &[f64], y: &[f64], use_robust: bool) -> Result<(RawFitResult, Vec<RawFitResult>), String> {
        let n = x.len();
        if n < 3 {
            return Err("Auto Fit requires at least 3 points".into());
        }

        let models = FitModelKind::all_models();

        let mut results: Vec<RawFitResult> = models
            .par_iter()
            .filter_map(|&kind| {
                if n < kind.min_points() {
                    return None;
                }
                // Try fitting
                match ModelFitter::fit(kind, x, y, use_robust) {
                    Ok(res) => {
                        // Sanity checks on parameters and diagnostics
                        if res.parameters.iter().all(|p| p.is_finite())
                            && res.diagnostics.aicc.is_finite()
                            && res.diagnostics.r_squared >= -0.5
                        {
                            Some(res)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            })
            .collect();

        if results.is_empty() {
            return Err("No models could successfully fit the dataset".into());
        }

        // Rank by AICc ascending (lower is better), tie-broken by LOOCV RMSE
        results.sort_by(|a, b| {
            a.diagnostics
                .aicc
                .partial_cmp(&b.diagnostics.aicc)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.diagnostics
                        .loocv_rmse
                        .partial_cmp(&b.diagnostics.loocv_rmse)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        let winner = results[0].clone();
        Ok((winner, results))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_fit_ranks_linear() {
        // Linear data with slight noise
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let y: Vec<f64> = x.iter().map(|&v| 2.5 * v - 1.0).collect();
        let (winner, candidates) = AutoFitter::auto_fit(&x, &y, false).unwrap();
        assert!(!candidates.is_empty());
        // Linear should have AICc near minimum and high R^2
        assert!(winner.diagnostics.r_squared > 0.99);
    }
}
