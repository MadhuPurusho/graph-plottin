use crate::parser::vm::CompiledExpression;

/// Result of a root-finding operation.
#[derive(Debug, Clone, PartialEq)]
pub struct RootResult {
    pub x: f64,
    pub converged: bool,
    pub iterations: usize,
}

/// Extremum type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtremumType {
    LocalMinimum,
    LocalMaximum,
    InflectionPoint,
}

/// Result of an extremum search.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtremumResult {
    pub x: f64,
    pub y: f64,
    pub kind: ExtremumType,
}

/// Result of numerical integration.
#[derive(Debug, Clone, PartialEq)]
pub struct IntegralResult {
    pub value: f64,
    pub error_estimate: f64,
}

/// Calculus engine implementing Brent's root-finding, extrema detection,
/// intersection finding, and adaptive Gauss-Kronrod (G7-K15) numerical integration.
pub struct Calculus;

impl Calculus {
    /// Brent's root-finding algorithm for finding a root of f(x) = 0 in [a, b].
    /// Requires f(a) and f(b) to have opposite signs, or searches around `guess`.
    pub fn brent_root<F>(f: F, mut a: f64, mut b: f64, tol: f64, max_iter: usize) -> Option<RootResult>
    where
        F: Fn(f64) -> f64,
    {
        let mut fa = f(a);
        let mut fb = f(b);

        if !fa.is_finite() || !fb.is_finite() {
            return None;
        }

        if fa.abs() < tol {
            return Some(RootResult { x: a, converged: true, iterations: 0 });
        }
        if fb.abs() < tol {
            return Some(RootResult { x: b, converged: true, iterations: 0 });
        }

        if fa * fb > 0.0 {
            // Bracketing failed
            return None;
        }

        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }

        let mut c = a;
        let mut fc = fa;
        let mut mflag = true;
        let mut d = 0.0;

        for iter in 1..=max_iter {
            let mut s = if (fa - fc).abs() > 1e-15 && (fb - fc).abs() > 1e-15 {
                // Inverse quadratic interpolation
                let term1 = (a * fb * fc) / ((fa - fb) * (fa - fc));
                let term2 = (b * fa * fc) / ((fb - fa) * (fb - fc));
                let term3 = (c * fa * fb) / ((fc - fa) * (fc - fb));
                term1 + term2 + term3
            } else {
                // Secant method
                b - fb * (b - a) / (fb - fa)
            };

            // Check conditions for accepting interpolation or falling back to bisection
            let cond1 = if a < b { s < (3.0 * a + b) / 4.0 || s > b } else { s > (3.0 * a + b) / 4.0 || s < b };
            let cond2 = mflag && (s - b).abs() >= (b - c).abs() / 2.0;
            let cond3 = !mflag && (s - b).abs() >= (c - d).abs() / 2.0;
            let cond4 = mflag && (b - c).abs() < tol;
            let cond5 = !mflag && (c - d).abs() < tol;

            if cond1 || cond2 || cond3 || cond4 || cond5 {
                s = 0.5 * (a + b);
                mflag = true;
            } else {
                mflag = false;
            }

            let fs = f(s);
            if !fs.is_finite() {
                return None;
            }

            d = c;
            c = b;
            fc = fb;

            if fa * fs < 0.0 {
                b = s;
                fb = fs;
            } else {
                a = s;
                fa = fs;
            }

            if fa.abs() < fb.abs() {
                std::mem::swap(&mut a, &mut b);
                std::mem::swap(&mut fa, &mut fb);
            }

            if fb.abs() < tol || (b - a).abs() < tol {
                return Some(RootResult { x: b, converged: true, iterations: iter });
            }
        }

        Some(RootResult { x: b, converged: false, iterations: max_iter })
    }

    /// Find a root of a compiled expression near `guess` within [x_min, x_max].
    pub fn find_root_near(
        expr: &CompiledExpression,
        guess: f64,
        x_min: f64,
        x_max: f64,
        params: &[f64],
        tol: f64,
    ) -> Option<RootResult> {
        let f = |x: f64| expr.eval_internal(x, params);

        // First test immediate guess
        let fg = f(guess);
        if fg.is_finite() && fg.abs() < tol {
            return Some(RootResult { x: guess, converged: true, iterations: 0 });
        }

        // Bracket search expanding outward from guess
        let step = (x_max - x_min) / 100.0;
        let mut delta = step.max(1e-4);

        for _ in 0..50 {
            let left = (guess - delta).max(x_min);
            let right = (guess + delta).min(x_max);

            let fl = f(left);
            let fr = f(right);

            if fl.is_finite() && fr.is_finite() && fl * fr <= 0.0 {
                return Self::brent_root(&f, left, right, tol, 100);
            }

            delta *= 1.5;
            if (guess - delta < x_min) && (guess + delta > x_max) {
                break;
            }
        }

        // Full scan across subintervals
        let n_sub = 50;
        let dx = (x_max - x_min) / (n_sub as f64);
        for i in 0..n_sub {
            let x0 = x_min + (i as f64) * dx;
            let x1 = x0 + dx;
            let y0 = f(x0);
            let y1 = f(x1);
            if y0.is_finite() && y1.is_finite() && y0 * y1 <= 0.0 {
                if let Some(r) = Self::brent_root(&f, x0, x1, tol, 100) {
                    return Some(r);
                }
            }
        }

        None
    }

    /// Find all local extrema in [x_min, x_max] by finding zeros of the first derivative.
    pub fn find_extrema(
        expr: &CompiledExpression,
        x_min: f64,
        x_max: f64,
        params: &[f64],
    ) -> Vec<ExtremumResult> {
        let f = |x: f64| expr.eval_internal(x, params);
        let h = 1e-6;
        let df = |x: f64| (f(x + h) - f(x - h)) / (2.0 * h);
        let d2f = |x: f64| (f(x + h) - 2.0 * f(x) + f(x - h)) / (h * h);

        let n_sub = 100;
        let dx = (x_max - x_min) / (n_sub as f64);
        let mut extrema = Vec::new();

        let x_mesh: Vec<f64> = (0..=n_sub).map(|i| x_min + (i as f64) * dx).collect();
        let dy_mesh: Vec<f64> = x_mesh.iter().map(|&x| df(x)).collect();

        for i in 0..n_sub {
            let x0 = x_mesh[i];
            let x1 = x_mesh[i + 1];
            let dy0 = dy_mesh[i];
            let dy1 = dy_mesh[i + 1];

            let candidate = if dy0.abs() < 1e-6 {
                Some(x0)
            } else if dy1.abs() < 1e-6 {
                Some(x1)
            } else if dy0.is_finite() && dy1.is_finite() && dy0 * dy1 < 0.0 {
                Self::brent_root(&df, x0, x1, 1e-8, 60).map(|r| r.x)
            } else {
                None
            };

            if let Some(x_ext) = candidate {
                // Deduplicate
                if extrema.iter().any(|e: &ExtremumResult| (e.x - x_ext).abs() < 1e-4) {
                    continue;
                }
                let y_ext = f(x_ext);
                let curve = d2f(x_ext);
                let kind = if curve > 1e-6 {
                    ExtremumType::LocalMinimum
                } else if curve < -1e-6 {
                    ExtremumType::LocalMaximum
                } else {
                    ExtremumType::InflectionPoint
                };
                extrema.push(ExtremumResult { x: x_ext, y: y_ext, kind });
            }
        }

        extrema
    }

    /// Find all intersection points between two expressions in [x_min, x_max].
    pub fn find_intersections(
        expr1: &CompiledExpression,
        expr2: &CompiledExpression,
        x_min: f64,
        x_max: f64,
        params: &[f64],
    ) -> Vec<f64> {
        let diff_fn = |x: f64| expr1.eval_internal(x, params) - expr2.eval_internal(x, params);
        let n_sub = 100;
        let dx = (x_max - x_min) / (n_sub as f64);
        let mut roots = Vec::new();

        for i in 0..n_sub {
            let x0 = x_min + (i as f64) * dx;
            let x1 = x0 + dx;
            let y0 = diff_fn(x0);
            let y1 = diff_fn(x1);

            if y0.is_finite() && y1.is_finite() && y0 * y1 <= 0.0 {
                if let Some(r) = Self::brent_root(&diff_fn, x0, x1, 1e-9, 80) {
                    if !roots.iter().any(|&rx: &f64| (rx - r.x).abs() < 1e-5) {
                        roots.push(r.x);
                    }
                }
            }
        }

        roots
    }

    /// Adaptive Gauss-Kronrod G7-K15 numerical integration of f(x) over [a, b].
    pub fn integrate<F>(f: F, a: f64, b: f64, tol: f64) -> IntegralResult
    where
        F: Fn(f64) -> f64 + Copy,
    {
        if (a - b).abs() < 1e-15 {
            return IntegralResult { value: 0.0, error_estimate: 0.0 };
        }

        let sign = if a <= b { 1.0 } else { -1.0 };
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };

        let (val, err) = Self::adaptive_gk(f, lo, hi, tol, 0, 12);
        IntegralResult {
            value: sign * val,
            error_estimate: err,
        }
    }

    /// Recursive Gauss-Kronrod adaptive subinterval integrator.
    fn adaptive_gk<F>(f: F, a: f64, b: f64, tol: f64, depth: usize, max_depth: usize) -> (f64, f64)
    where
        F: Fn(f64) -> f64 + Copy,
    {
        // 15-point Kronrod nodes and weights for interval [-1, 1]
        // Symmetry: nodes are +- x_k
        const NODES_15: [f64; 8] = [
            0.0,
            0.2077849550078985,
            0.4058451513773972,
            0.5860872354676911,
            0.7415311855993944,
            0.8648644233597691,
            0.9491079123427585,
            0.9914553711208126,
        ];
        const WEIGHTS_K15: [f64; 8] = [
            0.2094821410847278,
            0.2044329400752989,
            0.1903505780647854,
            0.1690047266392679,
            0.1406532597155259,
            0.1047900103222502,
            0.0630920926299786,
            0.0229353220105292,
        ];

        let half_len = 0.5 * (b - a);
        let center = 0.5 * (a + b);

        let mut f_vals = [0.0; 15];
        f_vals[0] = f(center);

        for i in 1..8 {
            let x_offset = half_len * NODES_15[i];
            f_vals[2 * i - 1] = f(center - x_offset);
            f_vals[2 * i] = f(center + x_offset);
        }

        // Kronrod 15-point sum
        let mut kronrod_sum = f_vals[0] * WEIGHTS_K15[0];
        for i in 1..8 {
            kronrod_sum += (f_vals[2 * i - 1] + f_vals[2 * i]) * WEIGHTS_K15[i];
        }
        let k15 = kronrod_sum * half_len;

        // Gauss 7-point sum (nodes at indices 2, 4, 6 in NODES_15 corresponding to G7 roots)
        // Standard G7 abscissae: 0, 0.4058451514, 0.7415311856, 0.9491079123
        const G7_WEIGHT_0: f64 = 0.4179591836734694;
        const G7_WEIGHTS_POS: [f64; 3] = [
            0.3818300505051189, // for node at 0.405845 (index 2)
            0.2797053914892767, // for node at 0.741531 (index 4)
            0.1294849661688697, // for node at 0.949108 (index 6)
        ];

        let mut gauss_sum = f_vals[0] * G7_WEIGHT_0;
        gauss_sum += (f_vals[3] + f_vals[4]) * G7_WEIGHTS_POS[0];
        gauss_sum += (f_vals[7] + f_vals[8]) * G7_WEIGHTS_POS[1];
        gauss_sum += (f_vals[11] + f_vals[12]) * G7_WEIGHTS_POS[2];
        let g7 = gauss_sum * half_len;

        let error = (k15 - g7).abs();

        if error <= tol || depth >= max_depth {
            (k15, error)
        } else {
            // Subdivide
            let (v1, e1) = Self::adaptive_gk(f, a, center, 0.5 * tol, depth + 1, max_depth);
            let (v2, e2) = Self::adaptive_gk(f, center, b, 0.5 * tol, depth + 1, max_depth);
            (v1 + v2, e1 + e2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_expression;

    #[test]
    fn test_brent_root() {
        // Find root of x^2 - 4 in [0, 5] => root is 2.0
        let f = |x: f64| x * x - 4.0;
        let res = Calculus::brent_root(f, 0.0, 5.0, 1e-12, 100).unwrap();
        assert!(res.converged);
        assert!((res.x - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_find_root_near() {
        let expr = parse_expression("sin(x)").unwrap();
        let compiled = CompiledExpression::compile(&expr, "sin(x)");
        // Root near 3.0 should be pi = 3.1415926535...
        let res = Calculus::find_root_near(&compiled, 3.0, 0.0, 6.0, &[], 1e-10).unwrap();
        assert!((res.x - std::f64::consts::PI).abs() < 1e-8);
    }

    #[test]
    fn test_extrema_detection() {
        let expr = parse_expression("x^3 - 3*x").unwrap();
        let compiled = CompiledExpression::compile(&expr, "x^3 - 3*x");
        // Extrema at x = -1 (local max, y=2) and x = 1 (local min, y=-2)
        let extrema = Calculus::find_extrema(&compiled, -2.5, 2.5, &[]);
        assert_eq!(extrema.len(), 2);
        let min = extrema.iter().find(|e| e.kind == ExtremumType::LocalMinimum).unwrap();
        let max = extrema.iter().find(|e| e.kind == ExtremumType::LocalMaximum).unwrap();
        assert!((min.x - 1.0).abs() < 1e-4);
        assert!((min.y - (-2.0)).abs() < 1e-4);
        assert!((max.x - (-1.0)).abs() < 1e-4);
        assert!((max.y - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_intersections() {
        let e1 = parse_expression("x^2").unwrap();
        let e2 = parse_expression("x + 2").unwrap();
        let c1 = CompiledExpression::compile(&e1, "x^2");
        let c2 = CompiledExpression::compile(&e2, "x + 2");
        // x^2 = x + 2 => x^2 - x - 2 = 0 => (x-2)(x+1) = 0 => x = -1, 2
        let roots = Calculus::find_intersections(&c1, &c2, -3.0, 4.0, &[]);
        assert_eq!(roots.len(), 2);
        assert!(roots.iter().any(|&r| (r - (-1.0)).abs() < 1e-4));
        assert!(roots.iter().any(|&r| (r - 2.0).abs() < 1e-4));
    }

    #[test]
    fn test_integrate_polynomial() {
        // Integrate 3x^2 from 0 to 2 = [x^3]_0^2 = 8.0
        let f = |x: f64| 3.0 * x * x;
        let res = Calculus::integrate(f, 0.0, 2.0, 1e-9);
        assert!((res.value - 8.0).abs() < 1e-8);
    }

    #[test]
    fn test_integrate_sin() {
        // Integrate sin(x) from 0 to pi = [-cos(x)]_0^pi = 1 - (-1) = 2.0
        let f = |x: f64| x.sin();
        let res = Calculus::integrate(f, 0.0, std::f64::consts::PI, 1e-9);
        assert!((res.value - 2.0).abs() < 1e-8);
    }
}
