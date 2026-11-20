use super::ast::{Expr, BinOp, MathFn, Span};
use super::lexer::{Token, TokenKind, Lexer, LexError};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at {}-{}: {}", self.span.start, self.span.end, self.message)
    }
}

impl std::error::Error for ParseError {}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self {
        ParseError { message: e.message, span: e.span }
    }
}

/// Pratt parser for math expressions.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn parse(input: &str) -> Result<Expr, ParseError> {
        let tokens = Lexer::new(input).tokenize()?;
        let mut parser = Parser { tokens, pos: 0 };
        let expr = parser.parse_expr(0)?;
        if !parser.at_eof() {
            let tok = &parser.tokens[parser.pos];
            return Err(ParseError {
                message: format!("Unexpected token {:?}", tok.kind),
                span: tok.span,
            });
        }
        Ok(expr)
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_prefix()?;

        loop {
            if self.at_eof() { break; }
            let tok = &self.tokens[self.pos];

            // Check for postfix/infix
            match &tok.kind {
                TokenKind::Plus | TokenKind::Minus |
                TokenKind::Star | TokenKind::Slash |
                TokenKind::Caret => {
                    let (l_bp, r_bp) = infix_binding_power(&tok.kind);
                    if l_bp < min_bp { break; }
                    let op = match &tok.kind {
                        TokenKind::Plus => BinOp::Add,
                        TokenKind::Minus => BinOp::Sub,
                        TokenKind::Star => {
                            // Check if this is an implicit * before a function name
                            // If so, check if next ident is a function name and there's a paren after
                            // We need to handle: 2*sin(x) vs x*y
                            // The * was inserted by lexer for implicit mul
                            // If next token is a function ident followed by (, consume * and let
                            // the prefix parser handle the function call
                            if self.is_implicit_function_mul() {
                                self.pos += 1; // consume the *
                                let rhs = self.parse_expr(r_bp)?;
                                lhs = Expr::mul(lhs, rhs);
                                continue;
                            }
                            BinOp::Mul
                        }
                        TokenKind::Slash => BinOp::Div,
                        TokenKind::Caret => BinOp::Pow,
                        _ => unreachable!(),
                    };
                    self.pos += 1;
                    let rhs = self.parse_expr(r_bp)?;
                    lhs = Expr::BinOp {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    };
                }
                _ => break,
            }
        }

        Ok(lhs)
    }

    /// Check if current token is * inserted for implicit mul before a function call.
    fn is_implicit_function_mul(&self) -> bool {
        if self.pos + 1 >= self.tokens.len() { return false; }
        if let TokenKind::Star = &self.tokens[self.pos].kind {
            if let TokenKind::Ident(name) = &self.tokens[self.pos + 1].kind {
                if MathFn::from_name(name).is_some() {
                    // Check if followed by (
                    if self.pos + 2 < self.tokens.len() {
                        // The lexer may have inserted another * between the ident and (
                        if let TokenKind::LParen = &self.tokens[self.pos + 2].kind {
                            return true;
                        }
                        if let TokenKind::Star = &self.tokens[self.pos + 2].kind {
                            if self.pos + 3 < self.tokens.len() {
                                if let TokenKind::LParen = &self.tokens[self.pos + 3].kind {
                                    return true;
                                }
                            }
                        }
                    }
                }
                // Constants like pi, e, tau
                if is_constant(name) {
                    return false; // treat as normal mul
                }
            }
        }
        false
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        let tok = self.tokens[self.pos].clone();
        match &tok.kind {
            TokenKind::Number(v) => {
                self.pos += 1;
                Ok(Expr::Num(*v))
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.pos += 1;

                // Check for constant
                if let Some(val) = constant_value(&name) {
                    return Ok(Expr::Num(val));
                }

                // Check for function call
                if let Some(func) = MathFn::from_name(&name) {
                    // Consume implicit * before ( if present
                    if self.pos < self.tokens.len() && matches!(self.tokens[self.pos].kind, TokenKind::Star) {
                        if self.pos + 1 < self.tokens.len() && matches!(self.tokens[self.pos + 1].kind, TokenKind::LParen) {
                            self.pos += 1; // consume *
                        }
                    }
                    self.expect(TokenKind::LParen)?;
                    let mut args = vec![self.parse_expr(0)?];
                    while self.peek_is(TokenKind::Comma) {
                        self.pos += 1; // consume comma
                        args.push(self.parse_expr(0)?);
                    }
                    self.expect(TokenKind::RParen)?;

                    if args.len() != func.arity() {
                        return Err(ParseError {
                            message: format!("{}() expects {} argument(s), got {}", func.name(), func.arity(), args.len()),
                            span: tok.span,
                        });
                    }

                    return Ok(Expr::FnCall { func, args });
                }

                // Variable
                if name == "x" {
                    Ok(Expr::Var)
                } else {
                    Ok(Expr::Param(name))
                }
            }
            TokenKind::Minus => {
                self.pos += 1;
                let expr = self.parse_expr(PREFIX_BP)?;
                Ok(Expr::neg(expr))
            }
            TokenKind::Plus => {
                self.pos += 1;
                self.parse_expr(PREFIX_BP)
            }
            TokenKind::LParen => {
                self.pos += 1;
                let expr = self.parse_expr(0)?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::Pipe => {
                self.pos += 1;
                let expr = self.parse_expr(0)?;
                self.expect(TokenKind::Pipe)?;
                Ok(Expr::Abs(Box::new(expr)))
            }
            _ => Err(ParseError {
                message: format!("Unexpected token {:?}", tok.kind),
                span: tok.span,
            }),
        }
    }

    fn expect(&mut self, expected: TokenKind) -> Result<Token, ParseError> {
        if self.pos >= self.tokens.len() {
            return Err(ParseError {
                message: format!("Expected {:?}, found end of input", expected),
                span: Span::new(self.pos, self.pos),
            });
        }
        let tok = self.tokens[self.pos].clone();
        if std::mem::discriminant(&tok.kind) == std::mem::discriminant(&expected) {
            self.pos += 1;
            Ok(tok)
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, found {:?}", expected, tok.kind),
                span: tok.span,
            })
        }
    }

    fn peek_is(&self, kind: TokenKind) -> bool {
        if self.pos >= self.tokens.len() { return false; }
        std::mem::discriminant(&self.tokens[self.pos].kind) == std::mem::discriminant(&kind)
    }

    fn at_eof(&self) -> bool {
        self.pos >= self.tokens.len() || matches!(self.tokens[self.pos].kind, TokenKind::Eof)
    }
}

const PREFIX_BP: u8 = 5; // unary prefix binding power (binds tighter than * and +, looser than ^)

fn infix_binding_power(kind: &TokenKind) -> (u8, u8) {
    match kind {
        TokenKind::Plus | TokenKind::Minus => (1, 2),
        TokenKind::Star | TokenKind::Slash => (3, 4),
        TokenKind::Caret => (7, 6), // right-associative, higher precedence than unary minus
        _ => (0, 0),
    }
}

fn is_constant(name: &str) -> bool {
    matches!(name, "pi" | "tau" | "e")
}

fn constant_value(name: &str) -> Option<f64> {
    match name {
        "pi" => Some(std::f64::consts::PI),
        "tau" => Some(std::f64::consts::TAU),
        "e" => Some(std::f64::consts::E),
        _ => None,
    }
}

/// Convenience function to parse an expression string.
pub fn parse_expression(input: &str) -> Result<Expr, ParseError> {
    Parser::parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_add() {
        let e = parse_expression("1 + 2").unwrap();
        assert_eq!(format!("{}", e), "(1 + 2)");
    }

    #[test]
    fn test_precedence() {
        let e = parse_expression("1 + 2 * 3").unwrap();
        assert_eq!(format!("{}", e), "(1 + (2 * 3))");
    }

    #[test]
    fn test_power_right_assoc() {
        let e = parse_expression("x^2^3").unwrap();
        // Should be x^(2^3)
        assert_eq!(format!("{}", e), "(x ^ (2 ^ 3))");
    }

    #[test]
    fn test_function_call() {
        let e = parse_expression("sin(x)").unwrap();
        assert_eq!(format!("{}", e), "sin(x)");
    }

    #[test]
    fn test_implicit_mul() {
        let e = parse_expression("2x").unwrap();
        assert_eq!(format!("{}", e), "(2 * x)");
    }

    #[test]
    fn test_implicit_mul_parens() {
        let e = parse_expression("(x+1)(x-2)").unwrap();
        // Should parse as (x+1) * (x-2)
        match &e {
            Expr::BinOp { op: BinOp::Mul, .. } => {},
            _ => panic!("Expected multiplication, got {:?}", e),
        }
    }

    #[test]
    fn test_implicit_mul_func() {
        let e = parse_expression("2sin(x)").unwrap();
        match &e {
            Expr::BinOp { op: BinOp::Mul, lhs, rhs } => {
                assert!(matches!(**lhs, Expr::Num(v) if v == 2.0));
                assert!(matches!(**rhs, Expr::FnCall { func: MathFn::Sin, .. }));
            },
            _ => panic!("Expected 2*sin(x), got {:?}", e),
        }
    }

    #[test]
    fn test_abs() {
        let e = parse_expression("|x - 1|").unwrap();
        assert!(matches!(e, Expr::Abs(_)));
    }

    #[test]
    fn test_unary_neg() {
        let e = parse_expression("-x").unwrap();
        assert!(matches!(e, Expr::UnaryNeg(_)));
    }

    #[test]
    fn test_pi_constant() {
        let e = parse_expression("pi").unwrap();
        assert!(matches!(e, Expr::Num(v) if (v - std::f64::consts::PI).abs() < 1e-15));
    }

    #[test]
    fn test_nested_functions() {
        let e = parse_expression("sin(cos(x))").unwrap();
        match e {
            Expr::FnCall { func: MathFn::Sin, args } => {
                assert!(matches!(&args[0], Expr::FnCall { func: MathFn::Cos, .. }));
            },
            _ => panic!("Expected sin(cos(x))"),
        }
    }

    #[test]
    fn test_two_arg_function() {
        let e = parse_expression("max(x, 0)").unwrap();
        match e {
            Expr::FnCall { func: MathFn::Max, args } => {
                assert_eq!(args.len(), 2);
            },
            _ => panic!("Expected max(x, 0)"),
        }
    }

    #[test]
    fn test_complex_expression() {
        // Should parse without error
        let _e = parse_expression("2*x^3 + sin(x)/cos(x) - exp(-x^2)").unwrap();
    }
}
