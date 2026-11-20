use super::ast::{Expr, BinOp, MathFn};

/// Bytecode opcodes for the expression VM.
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    PushConst(f64),
    LoadX,
    LoadParam(usize), // index into params array
    Add, Sub, Mul, Div, Pow, Neg,
    // Math functions
    Sin, Cos, Tan, Csc, Sec, Cot,
    Asin, Acos, Atan,
    Sinh, Cosh, Tanh,
    Exp, Ln, Log10, Log2,
    Sqrt, Cbrt,
    Abs, Floor, Ceil, Round, Sign,
    Min, Max, FnPow,
    Gamma,
}

/// A compiled expression that can be evaluated efficiently.
#[derive(uniffi::Object, Debug, Clone)]
pub struct CompiledExpression {
    pub(crate) code: Vec<OpCode>,
    pub(crate) param_names: Vec<String>,
    pub(crate) source: String,
}

#[uniffi::export]
impl CompiledExpression {
    /// Evaluate the compiled expression at x with free parameters.
    pub fn eval(&self, x: f64, params: Vec<f64>) -> f64 {
        self.eval_internal(x, &params)
    }

    /// List free parameter names (excluding x).
    pub fn free_parameters(&self) -> Vec<String> {
        self.param_names.clone()
    }

    /// Original expression source code.
    pub fn source(&self) -> String {
        self.source.clone()
    }
}

impl CompiledExpression {
    /// Compile an AST expression into bytecode.
    pub fn compile(expr: &Expr, source: &str) -> Self {
        let mut code = Vec::new();
        let mut param_names = Vec::new();
        compile_expr(expr, &mut code, &mut param_names);
        CompiledExpression {
            code,
            param_names,
            source: source.to_string(),
        }
    }

    /// Evaluate with borrowed slice for internal Rust speed.
    pub fn eval_internal(&self, x: f64, params: &[f64]) -> f64 {
        let mut stack: Vec<f64> = Vec::with_capacity(32);

        for op in &self.code {
            match op {
                OpCode::PushConst(v) => stack.push(*v),
                OpCode::LoadX => stack.push(x),
                OpCode::LoadParam(idx) => {
                    stack.push(if *idx < params.len() { params[*idx] } else { 0.0 });
                }
                OpCode::Add => { let b = stack.pop().unwrap_or(0.0); let a = stack.pop().unwrap_or(0.0); stack.push(a + b); }
                OpCode::Sub => { let b = stack.pop().unwrap_or(0.0); let a = stack.pop().unwrap_or(0.0); stack.push(a - b); }
                OpCode::Mul => { let b = stack.pop().unwrap_or(0.0); let a = stack.pop().unwrap_or(0.0); stack.push(a * b); }
                OpCode::Div => { let b = stack.pop().unwrap_or(0.0); let a = stack.pop().unwrap_or(0.0); stack.push(a / b); }
                OpCode::Pow => { let b = stack.pop().unwrap_or(0.0); let a = stack.pop().unwrap_or(0.0); stack.push(a.powf(b)); }
                OpCode::Neg => { let a = stack.pop().unwrap_or(0.0); stack.push(-a); }
                OpCode::Sin => { let a = stack.pop().unwrap_or(0.0); stack.push(a.sin()); }
                OpCode::Cos => { let a = stack.pop().unwrap_or(0.0); stack.push(a.cos()); }
                OpCode::Tan => { let a = stack.pop().unwrap_or(0.0); stack.push(a.tan()); }
                OpCode::Csc => { let a = stack.pop().unwrap_or(0.0); stack.push(1.0 / a.sin()); }
                OpCode::Sec => { let a = stack.pop().unwrap_or(0.0); stack.push(1.0 / a.cos()); }
                OpCode::Cot => { let a = stack.pop().unwrap_or(0.0); stack.push(1.0 / a.tan()); }
                OpCode::Asin => { let a = stack.pop().unwrap_or(0.0); stack.push(a.asin()); }
                OpCode::Acos => { let a = stack.pop().unwrap_or(0.0); stack.push(a.acos()); }
                OpCode::Atan => { let a = stack.pop().unwrap_or(0.0); stack.push(a.atan()); }
                OpCode::Sinh => { let a = stack.pop().unwrap_or(0.0); stack.push(a.sinh()); }
                OpCode::Cosh => { let a = stack.pop().unwrap_or(0.0); stack.push(a.cosh()); }
                OpCode::Tanh => { let a = stack.pop().unwrap_or(0.0); stack.push(a.tanh()); }
                OpCode::Exp => { let a = stack.pop().unwrap_or(0.0); stack.push(a.exp()); }
                OpCode::Ln => { let a = stack.pop().unwrap_or(0.0); stack.push(a.ln()); }
                OpCode::Log10 => { let a = stack.pop().unwrap_or(0.0); stack.push(a.log10()); }
                OpCode::Log2 => { let a = stack.pop().unwrap_or(0.0); stack.push(a.log2()); }
                OpCode::Sqrt => { let a = stack.pop().unwrap_or(0.0); stack.push(a.sqrt()); }
                OpCode::Cbrt => { let a = stack.pop().unwrap_or(0.0); stack.push(a.cbrt()); }
                OpCode::Abs => { let a = stack.pop().unwrap_or(0.0); stack.push(a.abs()); }
                OpCode::Floor => { let a = stack.pop().unwrap_or(0.0); stack.push(a.floor()); }
                OpCode::Ceil => { let a = stack.pop().unwrap_or(0.0); stack.push(a.ceil()); }
                OpCode::Round => { let a = stack.pop().unwrap_or(0.0); stack.push(a.round()); }
                OpCode::Sign => {
                    let a = stack.pop().unwrap_or(0.0);
                    stack.push(if a > 0.0 { 1.0 } else if a < 0.0 { -1.0 } else { 0.0 });
                }
                OpCode::Min => { let b = stack.pop().unwrap_or(0.0); let a = stack.pop().unwrap_or(0.0); stack.push(a.min(b)); }
                OpCode::Max => { let b = stack.pop().unwrap_or(0.0); let a = stack.pop().unwrap_or(0.0); stack.push(a.max(b)); }
                OpCode::FnPow => { let b = stack.pop().unwrap_or(0.0); let a = stack.pop().unwrap_or(0.0); stack.push(a.powf(b)); }
                OpCode::Gamma => {
                    let a = stack.pop().unwrap_or(0.0);
                    stack.push(gamma_fn(a));
                }
            }
        }

        stack.pop().unwrap_or(f64::NAN)
    }

    /// Evaluate at x with no parameters.
    pub fn eval_x(&self, x: f64) -> f64 {
        self.eval_internal(x, &[])
    }

    /// Get the parameter names.
    pub fn param_names(&self) -> &[String] {
        &self.param_names
    }
}

fn compile_expr(expr: &Expr, code: &mut Vec<OpCode>, param_names: &mut Vec<String>) {
    match expr {
        Expr::Num(v) => code.push(OpCode::PushConst(*v)),
        Expr::Var => code.push(OpCode::LoadX),
        Expr::Param(name) => {
            let idx = if let Some(pos) = param_names.iter().position(|n| n == name) {
                pos
            } else {
                let pos = param_names.len();
                param_names.push(name.clone());
                pos
            };
            code.push(OpCode::LoadParam(idx));
        }
        Expr::UnaryNeg(e) => {
            compile_expr(e, code, param_names);
            code.push(OpCode::Neg);
        }
        Expr::Abs(e) => {
            compile_expr(e, code, param_names);
            code.push(OpCode::Abs);
        }
        Expr::BinOp { op, lhs, rhs } => {
            compile_expr(lhs, code, param_names);
            compile_expr(rhs, code, param_names);
            match op {
                BinOp::Add => code.push(OpCode::Add),
                BinOp::Sub => code.push(OpCode::Sub),
                BinOp::Mul => code.push(OpCode::Mul),
                BinOp::Div => code.push(OpCode::Div),
                BinOp::Pow => code.push(OpCode::Pow),
            }
        }
        Expr::FnCall { func, args } => {
            for arg in args {
                compile_expr(arg, code, param_names);
            }
            match func {
                MathFn::Sin => code.push(OpCode::Sin),
                MathFn::Cos => code.push(OpCode::Cos),
                MathFn::Tan => code.push(OpCode::Tan),
                MathFn::Csc => code.push(OpCode::Csc),
                MathFn::Sec => code.push(OpCode::Sec),
                MathFn::Cot => code.push(OpCode::Cot),
                MathFn::Asin => code.push(OpCode::Asin),
                MathFn::Acos => code.push(OpCode::Acos),
                MathFn::Atan => code.push(OpCode::Atan),
                MathFn::Sinh => code.push(OpCode::Sinh),
                MathFn::Cosh => code.push(OpCode::Cosh),
                MathFn::Tanh => code.push(OpCode::Tanh),
                MathFn::Exp => code.push(OpCode::Exp),
                MathFn::Ln => code.push(OpCode::Ln),
                MathFn::Log10 => code.push(OpCode::Log10),
                MathFn::Log2 => code.push(OpCode::Log2),
                MathFn::Sqrt => code.push(OpCode::Sqrt),
                MathFn::Cbrt => code.push(OpCode::Cbrt),
                MathFn::Abs => code.push(OpCode::Abs),
                MathFn::Floor => code.push(OpCode::Floor),
                MathFn::Ceil => code.push(OpCode::Ceil),
                MathFn::Round => code.push(OpCode::Round),
                MathFn::Sign => code.push(OpCode::Sign),
                MathFn::Min => code.push(OpCode::Min),
                MathFn::Max => code.push(OpCode::Max),
                MathFn::Pow => code.push(OpCode::FnPow),
                MathFn::Gamma => code.push(OpCode::Gamma),
            }
        }
    }
}

/// Lanczos approximation of the Gamma function.
fn gamma_fn(x: f64) -> f64 {
    if x <= 0.0 && x == x.floor() {
        return f64::INFINITY;
    }
    if x < 0.5 {
        // Reflection formula
        std::f64::consts::PI / ((std::f64::consts::PI * x).sin() * gamma_fn(1.0 - x))
    } else {
        let x = x - 1.0;
        let g = 7.0;
        let c = [
            0.99999999999980993,
            676.5203681218851,
            -1259.1392167224028,
            771.32342877765313,
            -176.61502916214059,
            12.507343278686905,
            -0.13857109526572012,
            9.9843695780195716e-6,
            1.5056327351493116e-7,
        ];

        let mut sum = c[0];
        for (i, &ci) in c.iter().enumerate().skip(1) {
            sum += ci / (x + i as f64);
        }

        let t = x + g + 0.5;
        (2.0 * std::f64::consts::PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser::parse_expression;

    #[test]
    fn test_vm_basic() {
        let expr = parse_expression("x^2 + 1").unwrap();
        let compiled = CompiledExpression::compile(&expr, "x^2 + 1");
        assert!((compiled.eval_x(3.0) - 10.0).abs() < 1e-10);
        assert!((compiled.eval_x(0.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_vm_trig() {
        let expr = parse_expression("sin(x)").unwrap();
        let compiled = CompiledExpression::compile(&expr, "sin(x)");
        let x = 1.0;
        assert!((compiled.eval_x(x) - x.sin()).abs() < 1e-15);
    }

    #[test]
    fn test_vm_params() {
        let expr = parse_expression("a*x + b").unwrap();
        let compiled = CompiledExpression::compile(&expr, "a*x + b");
        assert_eq!(compiled.param_names(), &["a", "b"]);
        // a=2, b=3, x=5 => 13
        assert!((compiled.eval_internal(5.0, &[2.0, 3.0]) - 13.0).abs() < 1e-10);
    }

    #[test]
    fn test_vm_complex() {
        let expr = parse_expression("exp(-x^2)").unwrap();
        let compiled = CompiledExpression::compile(&expr, "exp(-x^2)");
        let x = 1.0;
        let expected = (-1.0_f64).exp();
        assert!((compiled.eval_x(x) - expected).abs() < 1e-10);
    }

    #[test]
    fn test_vm_gamma() {
        let expr = parse_expression("gamma(x)").unwrap();
        let compiled = CompiledExpression::compile(&expr, "gamma(x)");
        // gamma(5) = 4! = 24
        assert!((compiled.eval_x(5.0) - 24.0).abs() < 1e-6);
        // gamma(1) = 1
        assert!((compiled.eval_x(1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_vm_nested() {
        let expr = parse_expression("sin(cos(x))").unwrap();
        let compiled = CompiledExpression::compile(&expr, "sin(cos(x))");
        let x: f64 = 0.5;
        let expected = x.cos().sin();
        assert!((compiled.eval_x(x) - expected).abs() < 1e-15);
    }

    #[test]
    fn test_vm_abs() {
        let expr = parse_expression("|x - 2|").unwrap();
        let compiled = CompiledExpression::compile(&expr, "|x - 2|");
        assert!((compiled.eval_x(1.0) - 1.0).abs() < 1e-10);
        assert!((compiled.eval_x(3.0) - 1.0).abs() < 1e-10);
    }
}
