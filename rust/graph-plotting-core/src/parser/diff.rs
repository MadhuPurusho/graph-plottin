use super::ast::{Expr, BinOp, MathFn};

/// Symbolically differentiate an expression with respect to x.
pub fn differentiate(expr: &Expr) -> Expr {
    let raw = diff(expr);
    simplify(&raw)
}

/// Compute second derivative.
pub fn differentiate2(expr: &Expr) -> Expr {
    let first = differentiate(expr);
    differentiate(&first)
}

fn diff(expr: &Expr) -> Expr {
    match expr {
        Expr::Num(_) => Expr::Num(0.0),
        Expr::Var => Expr::Num(1.0),
        Expr::Param(_) => Expr::Num(0.0),
        Expr::UnaryNeg(e) => Expr::neg(diff(e)),
        Expr::Abs(e) => {
            // d/dx |f(x)| = f(x)/|f(x)| * f'(x) = sign(f(x)) * f'(x)
            Expr::mul(
                Expr::call(MathFn::Sign, vec![*e.clone()]),
                diff(e),
            )
        }
        Expr::BinOp { op, lhs, rhs } => {
            match op {
                BinOp::Add => Expr::add(diff(lhs), diff(rhs)),
                BinOp::Sub => Expr::sub(diff(lhs), diff(rhs)),
                BinOp::Mul => {
                    // Product rule: f'g + fg'
                    Expr::add(
                        Expr::mul(diff(lhs), *rhs.clone()),
                        Expr::mul(*lhs.clone(), diff(rhs)),
                    )
                }
                BinOp::Div => {
                    // Quotient rule: (f'g - fg') / g^2
                    Expr::div(
                        Expr::sub(
                            Expr::mul(diff(lhs), *rhs.clone()),
                            Expr::mul(*lhs.clone(), diff(rhs)),
                        ),
                        Expr::pow(*rhs.clone(), Expr::Num(2.0)),
                    )
                }
                BinOp::Pow => {
                    // Cases:
                    // 1) f(x)^c where c is constant: c * f(x)^(c-1) * f'(x)
                    // 2) c^g(x) where c is constant: c^g(x) * ln(c) * g'(x)
                    // 3) f(x)^g(x): exp(g*ln(f)) differentiated
                    if rhs.is_constant() {
                        // Power rule
                        Expr::mul(
                            Expr::mul(
                                *rhs.clone(),
                                Expr::pow(*lhs.clone(), Expr::sub(*rhs.clone(), Expr::Num(1.0))),
                            ),
                            diff(lhs),
                        )
                    } else if lhs.is_constant() {
                        // c^g(x) = c^g * ln(c) * g'
                        Expr::mul(
                            Expr::mul(
                                Expr::pow(*lhs.clone(), *rhs.clone()),
                                Expr::call(MathFn::Ln, vec![*lhs.clone()]),
                            ),
                            diff(rhs),
                        )
                    } else {
                        // General: f^g = exp(g*ln(f))
                        // d/dx = f^g * (g' * ln(f) + g * f'/f)
                        Expr::mul(
                            Expr::pow(*lhs.clone(), *rhs.clone()),
                            Expr::add(
                                Expr::mul(diff(rhs), Expr::call(MathFn::Ln, vec![*lhs.clone()])),
                                Expr::mul(*rhs.clone(), Expr::div(diff(lhs), *lhs.clone())),
                            ),
                        )
                    }
                }
            }
        }
        Expr::FnCall { func, args } => {
            // Chain rule: outer'(inner) * inner'
            let inner = &args[0];
            let inner_diff = diff(inner);

            let outer_diff = match func {
                MathFn::Sin => Expr::call(MathFn::Cos, vec![inner.clone()]),
                MathFn::Cos => Expr::neg(Expr::call(MathFn::Sin, vec![inner.clone()])),
                MathFn::Tan => {
                    // sec^2(x)
                    Expr::pow(Expr::call(MathFn::Sec, vec![inner.clone()]), Expr::Num(2.0))
                }
                MathFn::Csc => {
                    // -csc(x)*cot(x)
                    Expr::neg(Expr::mul(
                        Expr::call(MathFn::Csc, vec![inner.clone()]),
                        Expr::call(MathFn::Cot, vec![inner.clone()]),
                    ))
                }
                MathFn::Sec => {
                    // sec(x)*tan(x)
                    Expr::mul(
                        Expr::call(MathFn::Sec, vec![inner.clone()]),
                        Expr::call(MathFn::Tan, vec![inner.clone()]),
                    )
                }
                MathFn::Cot => {
                    // -csc^2(x)
                    Expr::neg(Expr::pow(Expr::call(MathFn::Csc, vec![inner.clone()]), Expr::Num(2.0)))
                }
                MathFn::Asin => {
                    // 1/sqrt(1-x^2)
                    Expr::div(
                        Expr::Num(1.0),
                        Expr::call(MathFn::Sqrt, vec![Expr::sub(Expr::Num(1.0), Expr::pow(inner.clone(), Expr::Num(2.0)))]),
                    )
                }
                MathFn::Acos => {
                    // -1/sqrt(1-x^2)
                    Expr::neg(Expr::div(
                        Expr::Num(1.0),
                        Expr::call(MathFn::Sqrt, vec![Expr::sub(Expr::Num(1.0), Expr::pow(inner.clone(), Expr::Num(2.0)))]),
                    ))
                }
                MathFn::Atan => {
                    // 1/(1+x^2)
                    Expr::div(
                        Expr::Num(1.0),
                        Expr::add(Expr::Num(1.0), Expr::pow(inner.clone(), Expr::Num(2.0))),
                    )
                }
                MathFn::Sinh => Expr::call(MathFn::Cosh, vec![inner.clone()]),
                MathFn::Cosh => Expr::call(MathFn::Sinh, vec![inner.clone()]),
                MathFn::Tanh => {
                    // 1 - tanh^2(x) = sech^2(x)
                    Expr::sub(
                        Expr::Num(1.0),
                        Expr::pow(Expr::call(MathFn::Tanh, vec![inner.clone()]), Expr::Num(2.0)),
                    )
                }
                MathFn::Exp => Expr::call(MathFn::Exp, vec![inner.clone()]),
                MathFn::Ln => Expr::div(Expr::Num(1.0), inner.clone()),
                MathFn::Log10 => {
                    // 1/(x * ln(10))
                    Expr::div(Expr::Num(1.0), Expr::mul(inner.clone(), Expr::Num(10.0_f64.ln())))
                }
                MathFn::Log2 => {
                    // 1/(x * ln(2))
                    Expr::div(Expr::Num(1.0), Expr::mul(inner.clone(), Expr::Num(2.0_f64.ln())))
                }
                MathFn::Sqrt => {
                    // 1/(2*sqrt(x))
                    Expr::div(Expr::Num(1.0), Expr::mul(Expr::Num(2.0), Expr::call(MathFn::Sqrt, vec![inner.clone()])))
                }
                MathFn::Cbrt => {
                    // 1/(3*x^(2/3))
                    Expr::div(
                        Expr::Num(1.0),
                        Expr::mul(Expr::Num(3.0), Expr::pow(inner.clone(), Expr::div(Expr::Num(2.0), Expr::Num(3.0)))),
                    )
                }
                MathFn::Abs => Expr::call(MathFn::Sign, vec![inner.clone()]),
                // For floor, ceil, round, sign — derivative is 0 (almost everywhere)
                MathFn::Floor | MathFn::Ceil | MathFn::Round | MathFn::Sign => {
                    return Expr::Num(0.0);
                }
                MathFn::Min | MathFn::Max | MathFn::Pow | MathFn::Gamma => {
                    // For 2-arg functions, return 0 (not differentiable in simple symbolic form)
                    return Expr::Num(0.0);
                }
            };

            // Apply chain rule
            Expr::mul(outer_diff, inner_diff)
        }
    }
}

/// Algebraic simplification.
pub fn simplify(expr: &Expr) -> Expr {
    match expr {
        Expr::Num(v) => Expr::Num(*v),
        Expr::Var => Expr::Var,
        Expr::Param(p) => Expr::Param(p.clone()),
        Expr::UnaryNeg(e) => {
            let e = simplify(e);
            match e {
                Expr::Num(v) => Expr::Num(-v),
                Expr::UnaryNeg(inner) => *inner,
                _ => Expr::neg(e),
            }
        }
        Expr::Abs(e) => {
            let e = simplify(e);
            match e {
                Expr::Num(v) => Expr::Num(v.abs()),
                _ => Expr::Abs(Box::new(e)),
            }
        }
        Expr::BinOp { op, lhs, rhs } => {
            let l = simplify(lhs);
            let r = simplify(rhs);

            // Constant folding
            if let (Expr::Num(a), Expr::Num(b)) = (&l, &r) {
                return match op {
                    BinOp::Add => Expr::Num(a + b),
                    BinOp::Sub => Expr::Num(a - b),
                    BinOp::Mul => Expr::Num(a * b),
                    BinOp::Div => {
                        if *b != 0.0 { Expr::Num(a / b) }
                        else { Expr::div(l, r) }
                    }
                    BinOp::Pow => Expr::Num(a.powf(*b)),
                };
            }

            match op {
                BinOp::Add => {
                    if l.is_zero() { return r; }
                    if r.is_zero() { return l; }
                    // a + (-b) => a - b
                    if let Expr::UnaryNeg(inner) = &r {
                        return simplify(&Expr::sub(l, *inner.clone()));
                    }
                    Expr::add(l, r)
                }
                BinOp::Sub => {
                    if r.is_zero() { return l; }
                    if l.is_zero() { return simplify(&Expr::neg(r)); }
                    // a - (-b) => a + b
                    if let Expr::UnaryNeg(inner) = &r {
                        return simplify(&Expr::add(l, *inner.clone()));
                    }
                    Expr::sub(l, r)
                }
                BinOp::Mul => {
                    if l.is_zero() || r.is_zero() { return Expr::Num(0.0); }
                    if l.is_one() { return r; }
                    if r.is_one() { return l; }
                    if let Expr::Num(v) = &l { if *v == -1.0 { return simplify(&Expr::neg(r)); } }
                    if let Expr::Num(v) = &r { if *v == -1.0 { return simplify(&Expr::neg(l)); } }
                    Expr::mul(l, r)
                }
                BinOp::Div => {
                    if l.is_zero() { return Expr::Num(0.0); }
                    if r.is_one() { return l; }
                    Expr::div(l, r)
                }
                BinOp::Pow => {
                    if r.is_zero() { return Expr::Num(1.0); }
                    if r.is_one() { return l; }
                    if l.is_zero() { return Expr::Num(0.0); }
                    if l.is_one() { return Expr::Num(1.0); }
                    Expr::pow(l, r)
                }
            }
        }
        Expr::FnCall { func, args } => {
            let args: Vec<Expr> = args.iter().map(|a| simplify(a)).collect();
            // Constant fold single-arg functions
            if args.len() == 1 {
                if let Expr::Num(v) = &args[0] {
                    let result = eval_math_fn(*func, *v);
                    if result.is_finite() {
                        return Expr::Num(result);
                    }
                }
            }
            Expr::FnCall { func: *func, args }
        }
    }
}

fn eval_math_fn(func: MathFn, x: f64) -> f64 {
    match func {
        MathFn::Sin => x.sin(),
        MathFn::Cos => x.cos(),
        MathFn::Tan => x.tan(),
        MathFn::Csc => 1.0 / x.sin(),
        MathFn::Sec => 1.0 / x.cos(),
        MathFn::Cot => 1.0 / x.tan(),
        MathFn::Asin => x.asin(),
        MathFn::Acos => x.acos(),
        MathFn::Atan => x.atan(),
        MathFn::Sinh => x.sinh(),
        MathFn::Cosh => x.cosh(),
        MathFn::Tanh => x.tanh(),
        MathFn::Exp => x.exp(),
        MathFn::Ln => x.ln(),
        MathFn::Log10 => x.log10(),
        MathFn::Log2 => x.log2(),
        MathFn::Sqrt => x.sqrt(),
        MathFn::Cbrt => x.cbrt(),
        MathFn::Abs => x.abs(),
        MathFn::Floor => x.floor(),
        MathFn::Ceil => x.ceil(),
        MathFn::Round => x.round(),
        MathFn::Sign => {
            if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 }
        }
        _ => f64::NAN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser::parse_expression;

    fn eval_simple(expr: &Expr, x: f64) -> f64 {
        match expr {
            Expr::Num(v) => *v,
            Expr::Var => x,
            Expr::Param(_) => 0.0,
            Expr::UnaryNeg(e) => -eval_simple(e, x),
            Expr::Abs(e) => eval_simple(e, x).abs(),
            Expr::BinOp { op, lhs, rhs } => {
                let l = eval_simple(lhs, x);
                let r = eval_simple(rhs, x);
                match op {
                    BinOp::Add => l + r,
                    BinOp::Sub => l - r,
                    BinOp::Mul => l * r,
                    BinOp::Div => l / r,
                    BinOp::Pow => l.powf(r),
                }
            }
            Expr::FnCall { func, args } => {
                let a = eval_simple(&args[0], x);
                eval_math_fn(*func, a)
            }
        }
    }

    fn numerical_derivative(expr: &Expr, x: f64, h: f64) -> f64 {
        (eval_simple(expr, x + h) - eval_simple(expr, x - h)) / (2.0 * h)
    }

    #[test]
    fn test_symbolic_matches_central_difference() {
        let expressions = ["x^4 - 2*x^2 + 5", "sin(x) * cos(x)", "exp(-x^2)"];
        for expr_str in expressions {
            let e = parse_expression(expr_str).unwrap();
            let d = differentiate(&e);
            for &x in &[0.5, 1.2, -0.7] {
                let exact = eval_simple(&d, x);
                let num = numerical_derivative(&e, x, 1e-6);
                assert!((exact - num).abs() < 1e-4, "Failed for {} at {}", expr_str, x);
            }
        }
    }

    #[test]
    fn test_diff_polynomial() {
        let e = parse_expression("x^3").unwrap();
        let d = differentiate(&e);
        // d/dx x^3 = 3x^2
        let val = eval_simple(&d, 2.0);
        assert!((val - 12.0).abs() < 1e-10, "got {}", val);
    }

    #[test]
    fn test_diff_sin() {
        let e = parse_expression("sin(x)").unwrap();
        let d = differentiate(&e);
        // d/dx sin(x) = cos(x)
        let val = eval_simple(&d, 1.0);
        let expected = 1.0_f64.cos();
        assert!((val - expected).abs() < 1e-10);
    }

    #[test]
    fn test_diff_product() {
        let e = parse_expression("x * sin(x)").unwrap();
        let d = differentiate(&e);
        // d/dx (x*sin(x)) = sin(x) + x*cos(x)
        let x: f64 = 1.5;
        let expected = x.sin() + x * x.cos();
        let val = eval_simple(&d, x);
        assert!((val - expected).abs() < 1e-10);
    }

    #[test]
    fn test_diff_exp() {
        let e = parse_expression("exp(x)").unwrap();
        let d = differentiate(&e);
        let x = 1.0;
        let val = eval_simple(&d, x);
        assert!((val - x.exp()).abs() < 1e-10);
    }

    #[test]
    fn test_simplify_zero_add() {
        let e = Expr::add(Expr::Num(0.0), Expr::Var);
        let s = simplify(&e);
        assert_eq!(s, Expr::Var);
    }

    #[test]
    fn test_simplify_one_mul() {
        let e = Expr::mul(Expr::Num(1.0), Expr::Var);
        let s = simplify(&e);
        assert_eq!(s, Expr::Var);
    }

    #[test]
    fn test_simplify_constant_fold() {
        let e = Expr::add(Expr::Num(2.0), Expr::Num(3.0));
        let s = simplify(&e);
        assert_eq!(s, Expr::Num(5.0));
    }
}
