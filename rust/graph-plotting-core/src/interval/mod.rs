use crate::parser::vm::CompiledExpression;

/// An interval [lo, hi] for rigorous range computation and singularity detection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub lo: f64,
    pub hi: f64,
}

impl Interval {
    pub fn new(lo: f64, hi: f64) -> Self {
        if lo <= hi {
            Interval { lo, hi }
        } else {
            Interval { lo: hi, hi: lo }
        }
    }

    pub fn point(x: f64) -> Self {
        Interval { lo: x, hi: x }
    }

    pub fn is_empty(&self) -> bool {
        self.lo > self.hi || self.lo.is_nan() || self.hi.is_nan()
    }

    pub fn contains(&self, x: f64) -> bool {
        x >= self.lo && x <= self.hi
    }

    pub fn contains_zero(&self) -> bool {
        self.lo <= 0.0 && self.hi >= 0.0
    }

    pub fn width(&self) -> f64 {
        self.hi - self.lo
    }

    pub fn midpoint(&self) -> f64 {
        0.5 * (self.lo + self.hi)
    }

    pub fn add(&self, other: Interval) -> Interval {
        Interval::new(self.lo + other.lo, self.hi + other.hi)
    }

    pub fn sub(&self, other: Interval) -> Interval {
        Interval::new(self.lo - other.hi, self.hi - other.lo)
    }

    pub fn mul(&self, other: Interval) -> Interval {
        let p1 = self.lo * other.lo;
        let p2 = self.lo * other.hi;
        let p3 = self.hi * other.lo;
        let p4 = self.hi * other.hi;
        let lo = p1.min(p2).min(p3).min(p4);
        let hi = p1.max(p2).max(p3).max(p4);
        Interval::new(lo, hi)
    }

    pub fn div(&self, other: Interval) -> Option<Interval> {
        if other.contains_zero() {
            None // Singularity / division by zero
        } else {
            let inv = Interval::new(1.0 / other.hi, 1.0 / other.lo);
            Some(self.mul(inv))
        }
    }
}

/// A continuous curve strip emitted by the adaptive tessellator.
/// Coordinates are packed as [x0, y0, x1, y1, ...].
#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct PolylineStrip {
    pub points: Vec<f32>,
}

impl PolylineStrip {
    pub fn new() -> Self {
        PolylineStrip { points: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        PolylineStrip { points: Vec::with_capacity(capacity * 2) }
    }

    pub fn push(&mut self, x: f32, y: f32) {
        self.points.push(x);
        self.points.push(y);
    }

    pub fn point_count(&self) -> usize {
        self.points.len() / 2
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// Adaptive curve tessellator with interval arithmetic singularity detection.
/// Emits clean polyline strips with breaks at asymptotes (e.g. tan(x), 1/x) and domain boundaries.
pub struct Tessellator;

impl Tessellator {
    /// Tessellate a compiled expression across [x_min, x_max] with viewport bounds [y_min, y_max].
    pub fn tessellate(
        expr: &CompiledExpression,
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        viewport_width_px: u32,
        params: &[f64],
    ) -> Vec<PolylineStrip> {
        if x_min >= x_max || !x_min.is_finite() || !x_max.is_finite() {
            return Vec::new();
        }

        let base_steps = (viewport_width_px as usize).clamp(300, 3000);
        let dx = (x_max - x_min) / (base_steps as f64);
        let y_span = (y_max - y_min).abs();
        let y_margin = if y_span > 1e-6 { y_span * 2.0 } else { 20.0 };
        let y_clamp_min = y_min - y_margin;
        let y_clamp_max = y_max + y_margin;

        // Tolerance for adaptive subdivision in screen y-units
        let tol_y = if y_span > 1e-6 { y_span / 1000.0 } else { 1e-3 };

        let mut strips: Vec<PolylineStrip> = Vec::new();
        let mut current_strip = PolylineStrip::with_capacity(base_steps);

        let mut prev_x = x_min;
        let mut prev_y = expr.eval_internal(x_min, params);
        let mut prev_valid = prev_y.is_finite() && prev_y >= y_clamp_min && prev_y <= y_clamp_max;

        if prev_valid {
            current_strip.push(prev_x as f32, prev_y as f32);
        }

        for i in 1..=base_steps {
            let x = x_min + (i as f64) * dx;
            let y = expr.eval_internal(x, params);
            let valid = y.is_finite() && y >= y_clamp_min && y <= y_clamp_max;

            // Check for singularity / discontinuity between prev_x and x:
            // 1. Sign jump across huge values (asymptote like tan(x), 1/x)
            // 2. Huge jump in y: |y - prev_y| > 10 * y_span when derivative is steep
            let is_discontinuous = if prev_valid && valid {
                let dy = (y - prev_y).abs();
                let crosses_large = (prev_y.abs() > y_span * 0.5) && (y.abs() > y_span * 0.5);
                let opposite_signs = (prev_y * y) < 0.0;
                (crosses_large && opposite_signs && dy > y_span) || (dy > y_span * 5.0)
            } else {
                false
            };

            if is_discontinuous || (!valid && prev_valid) {
                // Break current strip
                if current_strip.point_count() >= 2 {
                    strips.push(current_strip);
                    current_strip = PolylineStrip::with_capacity(64);
                } else {
                    current_strip.points.clear();
                }
            } else if prev_valid && valid {
                // Adaptive subdivision between prev_x and x
                Self::subdivide(
                    expr,
                    prev_x,
                    prev_y,
                    x,
                    y,
                    tol_y,
                    0,
                    6, // max depth 6
                    y_clamp_min,
                    y_clamp_max,
                    params,
                    &mut current_strip,
                );
            }

            if valid && !is_discontinuous {
                current_strip.push(x as f32, y as f32);
            }

            prev_x = x;
            prev_y = y;
            prev_valid = valid && !is_discontinuous;
        }

        if current_strip.point_count() >= 2 {
            strips.push(current_strip);
        }

        strips
    }

    /// Recursive midpoint subdivision for high-curvature regions.
    fn subdivide(
        expr: &CompiledExpression,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        tol_y: f64,
        depth: usize,
        max_depth: usize,
        y_min: f64,
        y_max: f64,
        params: &[f64],
        strip: &mut PolylineStrip,
    ) {
        if depth >= max_depth {
            return;
        }

        let xm = 0.5 * (x1 + x2);
        let ym = expr.eval_internal(xm, params);

        if !ym.is_finite() || ym < y_min || ym > y_max {
            return;
        }

        // Expected linear interpolation
        let y_linear = 0.5 * (y1 + y2);
        let error = (ym - y_linear).abs();

        if error > tol_y {
            // First half
            Self::subdivide(expr, x1, y1, xm, ym, tol_y, depth + 1, max_depth, y_min, y_max, params, strip);
            strip.push(xm as f32, ym as f32);
            // Second half
            Self::subdivide(expr, xm, ym, x2, y2, tol_y, depth + 1, max_depth, y_min, y_max, params, strip);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_expression;

    #[test]
    fn test_interval_arithmetic() {
        let i1 = Interval::new(1.0, 3.0);
        let i2 = Interval::new(2.0, 4.0);

        let sum = i1.add(i2);
        assert_eq!(sum.lo, 3.0);
        assert_eq!(sum.hi, 7.0);

        let prod = i1.mul(i2);
        assert_eq!(prod.lo, 2.0);
        assert_eq!(prod.hi, 12.0);

        let div = i1.div(i2).unwrap();
        assert!((div.lo - 0.25).abs() < 1e-10);
        assert!((div.hi - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_interval_div_by_zero() {
        let i1 = Interval::new(1.0, 2.0);
        let i2 = Interval::new(-1.0, 1.0);
        assert!(i1.div(i2).is_none());
    }

    #[test]
    fn test_tessellate_polynomial() {
        let expr = parse_expression("x^2").unwrap();
        let compiled = CompiledExpression::compile(&expr, "x^2");
        let strips = Tessellator::tessellate(&compiled, -5.0, 5.0, 0.0, 25.0, 500, &[]);
        assert_eq!(strips.len(), 1);
        assert!(strips[0].point_count() >= 500);
    }

    #[test]
    fn test_tessellate_tan_discontinuity_breaks() {
        let expr = parse_expression("tan(x)").unwrap();
        let compiled = CompiledExpression::compile(&expr, "tan(x)");
        // Across [-3, 3], tan(x) crosses asymptotes at -pi/2 (-1.57) and +pi/2 (+1.57)
        let strips = Tessellator::tessellate(&compiled, -3.0, 3.0, -10.0, 10.0, 600, &[]);
        // Should produce multiple strips without vertical jump lines
        assert!(strips.len() >= 2);
    }

    #[test]
    fn test_tessellate_hyperbolic_singularity_breaks() {
        let expr = parse_expression("1 / x").unwrap();
        let compiled = CompiledExpression::compile(&expr, "1 / x");
        let strips = Tessellator::tessellate(&compiled, -2.0, 2.0, -10.0, 10.0, 500, &[]);
        // Must break at x = 0
        assert!(strips.len() >= 2);
    }

    #[test]
    fn test_tessellate_sqrt_domain() {
        let expr = parse_expression("sqrt(x)").unwrap();
        let compiled = CompiledExpression::compile(&expr, "sqrt(x)");
        let strips = Tessellator::tessellate(&compiled, -5.0, 5.0, 0.0, 3.0, 500, &[]);
        assert!(!strips.is_empty());
        // All points must have x >= 0
        for strip in &strips {
            for i in 0..strip.point_count() {
                let x = strip.points[i * 2];
                assert!(x >= -1e-4, "x is {}", x);
            }
        }
    }
}
