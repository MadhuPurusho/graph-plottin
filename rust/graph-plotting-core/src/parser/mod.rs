pub mod ast;
pub mod lexer;
pub mod parser;
pub mod diff;
pub mod vm;

pub use ast::{Expr, BinOp, MathFn, Span};
pub use lexer::{Lexer, Token, TokenKind, LexError};
pub use parser::{Parser, ParseError, parse_expression};
pub use diff::{differentiate, differentiate2, simplify};
pub use vm::{CompiledExpression, OpCode};
