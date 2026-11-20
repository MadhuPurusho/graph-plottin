use super::ast::Span;

/// Token types for math expression lexing.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    Comma,
    Pipe,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Lexer for math expressions with implicit multiplication support.
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    pipe_open: bool,
    last_was_closing_pipe: bool,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            pipe_open: false,
            last_was_closing_pipe: false,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();
            if self.pos >= self.chars.len() {
                tokens.push(Token::new(TokenKind::Eof, Span::new(self.pos, self.pos)));
                break;
            }

            let start = self.pos;
            let ch = self.chars[self.pos];

            let is_closing_pipe = ch == '|' && self.pipe_open;
            let is_opening_pipe = ch == '|' && !self.pipe_open;

            let token = match ch {
                '+' => { self.pos += 1; Token::new(TokenKind::Plus, Span::new(start, self.pos)) }
                '-' => { self.pos += 1; Token::new(TokenKind::Minus, Span::new(start, self.pos)) }
                '*' => { self.pos += 1; Token::new(TokenKind::Star, Span::new(start, self.pos)) }
                '/' => { self.pos += 1; Token::new(TokenKind::Slash, Span::new(start, self.pos)) }
                '^' => { self.pos += 1; Token::new(TokenKind::Caret, Span::new(start, self.pos)) }
                '(' => { self.pos += 1; Token::new(TokenKind::LParen, Span::new(start, self.pos)) }
                ')' => { self.pos += 1; Token::new(TokenKind::RParen, Span::new(start, self.pos)) }
                ',' => { self.pos += 1; Token::new(TokenKind::Comma, Span::new(start, self.pos)) }
                '|' => {
                    self.pos += 1;
                    self.pipe_open = !self.pipe_open;
                    Token::new(TokenKind::Pipe, Span::new(start, self.pos))
                }
                'π' => {
                    self.pos += 1;
                    Token::new(TokenKind::Ident("pi".to_string()), Span::new(start, self.pos))
                }
                'τ' => {
                    self.pos += 1;
                    Token::new(TokenKind::Ident("tau".to_string()), Span::new(start, self.pos))
                }
                c if c.is_ascii_digit() || c == '.' => self.lex_number(start)?,
                c if c.is_ascii_alphabetic() || c == '_' => self.lex_ident(start),
                _ => return Err(LexError {
                    message: format!("Unexpected character '{}'", ch),
                    span: Span::new(start, start + 1),
                }),
            };

            // Insert implicit multiplication tokens
            if let Some(last) = tokens.last() {
                if !is_closing_pipe && needs_implicit_mul(&last.kind, &token.kind, is_opening_pipe, self.last_was_closing_pipe) {
                    tokens.push(Token::new(TokenKind::Star, Span::new(start, start)));
                }
            }

            self.last_was_closing_pipe = is_closing_pipe;
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn lex_number(&mut self, start: usize) -> Result<Token, LexError> {
        let mut has_dot = false;
        let mut has_e = false;

        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch.is_ascii_digit() {
                self.pos += 1;
            } else if ch == '.' && !has_dot && !has_e {
                has_dot = true;
                self.pos += 1;
            } else if (ch == 'e' || ch == 'E') && !has_e {
                // Check this is scientific notation, not Euler's number
                // Only treat as sci notation if followed by digit or +/-
                if self.pos + 1 < self.chars.len() {
                    let next = self.chars[self.pos + 1];
                    if next.is_ascii_digit() || next == '+' || next == '-' {
                        has_e = true;
                        self.pos += 1;
                        if self.pos < self.chars.len() && (self.chars[self.pos] == '+' || self.chars[self.pos] == '-') {
                            self.pos += 1;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let text: String = self.chars[start..self.pos].iter().collect();
        let value = text.parse::<f64>().map_err(|_| LexError {
            message: format!("Invalid number '{}'", text),
            span: Span::new(start, self.pos),
        })?;

        Ok(Token::new(TokenKind::Number(value), Span::new(start, self.pos)))
    }

    fn lex_ident(&mut self, start: usize) -> Token {
        while self.pos < self.chars.len() && (self.chars[self.pos].is_ascii_alphanumeric() || self.chars[self.pos] == '_') {
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        Token::new(TokenKind::Ident(text), Span::new(start, self.pos))
    }
}

/// Check if implicit multiplication should be inserted between two tokens.
fn needs_implicit_mul(left: &TokenKind, right: &TokenKind, is_right_opening_pipe: bool, last_was_closing_pipe: bool) -> bool {
    if matches!(right, TokenKind::Pipe) && !is_right_opening_pipe {
        return false;
    }

    match (left, right) {
        // 2x, 2sin(x), 2(x+1), 2|x|
        (TokenKind::Number(_), TokenKind::Ident(_)) => true,
        (TokenKind::Number(_), TokenKind::LParen) => true,
        (TokenKind::Number(_), TokenKind::Pipe) => is_right_opening_pipe,
        // )(, )x, )2, )|x|
        (TokenKind::RParen, TokenKind::LParen) => true,
        (TokenKind::RParen, TokenKind::Ident(_)) => true,
        (TokenKind::RParen, TokenKind::Number(_)) => true,
        (TokenKind::RParen, TokenKind::Pipe) => is_right_opening_pipe,
        // x(x+1)
        (TokenKind::Ident(_), TokenKind::LParen) => true,
        (TokenKind::Ident(_), TokenKind::Number(_)) => true,
        (TokenKind::Ident(_), TokenKind::Ident(_)) => true,
        (TokenKind::Ident(_), TokenKind::Pipe) => is_right_opening_pipe,
        // |x|y, |x|2, |x|(, |x||y| - only if left was a closing pipe!
        (TokenKind::Pipe, TokenKind::Number(_)) => last_was_closing_pipe,
        (TokenKind::Pipe, TokenKind::Ident(_)) => last_was_closing_pipe,
        (TokenKind::Pipe, TokenKind::LParen) => last_was_closing_pipe,
        (TokenKind::Pipe, TokenKind::Pipe) => last_was_closing_pipe && is_right_opening_pipe,
        _ => false,
    }
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lex error at {}-{}: {}", self.span.start, self.span.end, self.message)
    }
}

impl std::error::Error for LexError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_kinds(input: &str) -> Vec<TokenKind> {
        Lexer::new(input).tokenize().unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_basic_tokens() {
        let kinds = token_kinds("1 + 2");
        assert_eq!(kinds, vec![
            TokenKind::Number(1.0),
            TokenKind::Plus,
            TokenKind::Number(2.0),
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_implicit_mul_number_ident() {
        let kinds = token_kinds("2x");
        assert_eq!(kinds, vec![
            TokenKind::Number(2.0),
            TokenKind::Star,
            TokenKind::Ident("x".into()),
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_implicit_mul_parens() {
        let kinds = token_kinds("(x+1)(x-2)");
        // Should have * between ) and (
        assert!(kinds.contains(&TokenKind::Star));
    }

    #[test]
    fn test_scientific_notation() {
        let kinds = token_kinds("1.5e3");
        assert_eq!(kinds[0], TokenKind::Number(1500.0));
    }

    #[test]
    fn test_pi_symbol() {
        let kinds = token_kinds("π");
        assert_eq!(kinds[0], TokenKind::Ident("pi".into()));
    }
}
