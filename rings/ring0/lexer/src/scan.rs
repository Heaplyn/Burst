//! Ring 0 · Lexer · **NextToken (scanner)**
//!
//! The big per-character match that produces one token at a time.
//! Requires: [`Lexer`](crate::lexer::Lexer), [`token`](crate::token),
//! [`keywords::LookupKeyword`](crate::keywords::LookupKeyword).

use crate::keywords::LookupKeyword;
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};

impl<'a> Lexer<'a> {
    /// Identifies the next token in the stream.
    pub fn NextToken(&mut self) -> Option<Token> {
        if let Some(tok) = self.PeekedToken.take() {
            return Some(tok);
        }
        self.SkipWhitespace();

        let line = self.CurrentLine;
        let col = self.CurrentColumn;

        let ch = self.AdvanceChar()?;

        let kind = match ch {
            '+' => TokenKind::Plus,
            '-' => {
                if self.PeekChar() == Some(&'>') {
                    self.AdvanceChar();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => TokenKind::Star,
            '/' => {
                if self.PeekChar() == Some(&'/') {
                    while let Some(c) = self.AdvanceChar() {
                        if c == '\n' {
                            break;
                        }
                    }
                    return self.NextToken();
                } else {
                    TokenKind::Slash
                }
            }
            '%' => TokenKind::Percent,
            '=' => {
                if self.PeekChar() == Some(&'=') {
                    self.AdvanceChar();
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                }
            }
            '!' => {
                if self.PeekChar() == Some(&'=') {
                    self.AdvanceChar();
                    TokenKind::NotEqual
                } else {
                    TokenKind::Not
                }
            }
            '(' => TokenKind::OpenParen,
            ')' => TokenKind::CloseParen,
            '{' => TokenKind::OpenBrace,
            '}' => TokenKind::CloseBrace,
            '[' => TokenKind::OpenBracket,
            ']' => TokenKind::CloseBracket,
            '<' => {
                if self.PeekChar() == Some(&'=') {
                    self.AdvanceChar();
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                if self.PeekChar() == Some(&'=') {
                    self.AdvanceChar();
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            ':' => {
                if self.PeekChar() == Some(&'=') {
                    self.AdvanceChar();
                    TokenKind::TypeSet
                } else {
                    TokenKind::Colon
                }
            }
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            '&' => {
                if self.PeekChar() == Some(&'&') {
                    self.AdvanceChar();
                }
                TokenKind::And
            }
            '|' => {
                if self.PeekChar() == Some(&'|') {
                    self.AdvanceChar();
                }
                TokenKind::Or
            }

            '\'' | '"' => return ScanString(self, ch, line, col),

            c if c.is_alphabetic() || c == '_' => return Some(ScanIdent(self, c, line, col)),

            c if c.is_numeric() => ScanNumber(self, c),

            _ => return None,
        };

        Some(Token { Kind: kind, Line: line, Column: col })
    }
}

/// Scans a quoted string until the matching closing quote.
fn ScanString<'a>(lex: &mut Lexer<'a>, quote: char, line: usize, col: usize) -> Option<Token> {
    let mut s = String::new();
    while let Some(&c) = lex.PeekChar() {
        if c == quote {
            lex.AdvanceChar();
            return Some(Token { Kind: TokenKind::StringLiteral(s), Line: line, Column: col });
        }
        s.push(lex.AdvanceChar().unwrap());
    }
    None
}

/// Scans an identifier / keyword / bit-precise type starting from `first`.
fn ScanIdent<'a>(lex: &mut Lexer<'a>, first: char, line: usize, col: usize) -> Token {
    let mut identifier = String::new();
    identifier.push(first);
    while let Some(&next_char) = lex.PeekChar() {
        if next_char.is_alphanumeric() || next_char == '_' {
            identifier.push(lex.AdvanceChar().unwrap());
        } else {
            break;
        }
    }
    let kind = LookupKeyword(&identifier).unwrap_or(TokenKind::Ident(identifier));
    Token { Kind: kind, Line: line, Column: col }
}

/// Scans an integer or float literal.
fn ScanNumber<'a>(lex: &mut Lexer<'a>, first: char) -> TokenKind {
    let mut num = String::new();
    num.push(first);
    let mut is_float = false;

    while let Some(&next_char) = lex.PeekChar() {
        if next_char.is_numeric() {
            num.push(lex.AdvanceChar().unwrap());
        } else if next_char == '.' {
            is_float = true;
            num.push(lex.AdvanceChar().unwrap());
        } else {
            break;
        }
    }

    if is_float {
        TokenKind::FloatLiteral(num.parse::<f64>().unwrap_or(0.0))
    } else {
        TokenKind::IntLiteral(num.parse::<i64>().unwrap_or(0))
    }
}
