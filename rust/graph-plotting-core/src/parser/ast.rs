use std::fmt;

/// Span in source text for error diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// Supported binary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Pow => write!(f, "^"),
        }
    }
}

/// Supported math functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathFn {
    Sin, Cos, Tan, Csc, Sec, Cot,
    Asin, Acos, Atan,
    Sinh, Cosh, Tanh,
    Exp, Ln, Log10, Log2,
    Sqrt, Cbrt,
    Abs, Floor, Ceil, Round, Sign,
    Min, Max, Pow,
    Gamma,
}

impl MathFn {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "sin" => Some(MathFn::Sin),
            "cos" => Some(MathFn::Cos),
            "tan" => Some(MathFn::Tan),
            "csc" => Some(MathFn::Csc),
            "sec" => Some(MathFn::Sec),
            "cot" => Some(MathFn::Cot),
            "asin" | "arcsin" => Some(MathFn::Asin),
            "acos" | "arccos" => Some(MathFn::Acos),
            "atan" | "arctan" => Some(MathFn::Atan),
            "sinh" => Some(MathFn::Sinh),
            "cosh" => Some(MathFn::Cosh),
            "tanh" => Some(MathFn::Tanh),
            "exp" => Some(MathFn::Exp),
            "ln" => Some(MathFn::Ln),
            "log" | "log10" => Some(MathFn::Log10),
            "log2" => Some(MathFn::Log2),
            "sqrt" => Some(MathFn::Sqrt),
            "cbrt" => Some(MathFn::Cbrt),
            "abs" => Some(MathFn::Abs),
            "floor" => Some(MathFn::Floor),
            "ceil" => Some(MathFn::Ceil),
            "round" => Some(MathFn::Round),
            "sign" | "sgn" => Some(MathFn::Sign),
            "min" => Some(MathFn::Min),
            "max" => Some(MathFn::Max),
            "pow" => Some(MathFn::Pow),
            "gamma" => Some(MathFn::Gamma),
            _ => None,
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            MathFn::Min | MathFn::Max | MathFn::Pow => 2,
            _ => 1,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            MathFn::Sin => "sin", MathFn::Cos => "cos", MathFn::Tan => "tan",
            MathFn::Csc => "csc", MathFn::Sec => "sec", MathFn::Cot => "cot",
            MathFn::Asin => "asin", MathFn::Acos => "acos", MathFn::Atan => "atan",
            MathFn::Sinh => "sinh", MathFn::Cosh => "cosh", MathFn::Tanh => "tanh",
            MathFn::Exp => "exp", MathFn::Ln => "ln", MathFn::Log10 => "log10",
            MathFn::Log2 => "log2", MathFn::Sqrt => "sqrt", MathFn::Cbrt => "cbrt",
            MathFn::Abs => "abs", MathFn::Floor => "floor", MathFn::Ceil => "ceil",
            MathFn::Round => "round", MathFn::Sign => "sign",
            MathFn::Min => "min", MathFn::Max => "max", MathFn::Pow => "pow",
            MathFn::Gamma => "gamma",
        }
    }
}

/// Expression AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Numeric literal.
    Num(f64),
    /// Variable (always 'x').
    Var,
    /// Named parameter (a, b, c, ...).
    Param(String),
    /// Binary operation.
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// Unary negation.
    UnaryNeg(Box<Expr>),
    /// Function call.
    FnCall {
        func: MathFn,
        args: Vec<Expr>,
    },
    /// Absolute value |expr|.
    Abs(Box<Expr>),
}

impl Expr {
    pub fn num(v: f64) -> Self { Expr::Num(v) }
    pub fn var() -> Self { Expr::Var }
    pub fn param(name: &str) -> Self { Expr::Param(name.to_string()) }

    pub fn add(a: Expr, b: Expr) -> Self {
        Expr::BinOp { op: BinOp::Add, lhs: Box::new(a), rhs: Box::new(b) }
    }
    pub fn sub(a: Expr, b: Expr) -> Self {
        Expr::BinOp { op: BinOp::Sub, lhs: Box::new(a), rhs: Box::new(b) }
    }
    pub fn mul(a: Expr, b: Expr) -> Self {
        Expr::BinOp { op: BinOp::Mul, lhs: Box::new(a), rhs: Box::new(b) }
    }
    pub fn div(a: Expr, b: Expr) -> Self {
        Expr::BinOp { op: BinOp::Div, lhs: Box::new(a), rhs: Box::new(b) }
    }
    pub fn pow(a: Expr, b: Expr) -> Self {
        Expr::BinOp { op: BinOp::Pow, lhs: Box::new(a), rhs: Box::new(b) }
    }
    pub fn neg(a: Expr) -> Self { Expr::UnaryNeg(Box::new(a)) }

    pub fn call(func: MathFn, args: Vec<Expr>) -> Self {
        Expr::FnCall { func, args }
    }

    /// Check if expression is a constant (no Var).
    pub fn is_constant(&self) -> bool {
        match self {
            Expr::Num(_) => true,
            Expr::Var => false,
            Expr::Param(_) => true,
            Expr::BinOp { lhs, rhs, .. } => lhs.is_constant() && rhs.is_constant(),
            Expr::UnaryNeg(e) => e.is_constant(),
            Expr::FnCall { args, .. } => args.iter().all(|a| a.is_constant()),
            Expr::Abs(e) => e.is_constant(),
        }
    }

    /// Check if this is zero.
    pub fn is_zero(&self) -> bool {
        matches!(self, Expr::Num(v) if *v == 0.0)
    }

    /// Check if this is one.
    pub fn is_one(&self) -> bool {
        matches!(self, Expr::Num(v) if *v == 1.0)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Num(v) => {
                if *v == std::f64::consts::PI { write!(f, "π") }
                else if *v == std::f64::consts::TAU { write!(f, "τ") }
                else if *v == std::f64::consts::E { write!(f, "e") }
                else if v.fract() == 0.0 && v.abs() < 1e15 { write!(f, "{}", *v as i64) }
                else { write!(f, "{}", v) }
            }
            Expr::Var => write!(f, "x"),
            Expr::Param(name) => write!(f, "{}", name),
            Expr::BinOp { op, lhs, rhs } => write!(f, "({} {} {})", lhs, op, rhs),
            Expr::UnaryNeg(e) => write!(f, "(-{})", e),
            Expr::FnCall { func, args } => {
                write!(f, "{}(", func.name())?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", a)?;
                }
                write!(f, ")")
            }
            Expr::Abs(e) => write!(f, "|{}|", e),
        }
    }
}
