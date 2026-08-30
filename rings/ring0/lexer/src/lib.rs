#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod token;

use token::{Token, TokenKind};

/// the thing that turns text into a stream of tokens
pub struct Lexer<'a> {
    /// the actual text we are reading
    pub Source: &'a str,
    /// character iterator with lookahead
    pub Chars: std::iter::Peekable<std::str::Chars<'a>>,
    /// token we saved for later
    pub PeekedToken: Option<Token>,
    /// line count for error messages
    pub CurrentLine: usize,
    /// column count for error messages
    pub CurrentColumn: usize,
}

impl Token {
    pub fn New(Kind: TokenKind, Line: usize, Column: usize) -> Self {
        Self { Kind, Line, Column }
    }
}
impl<'a> Lexer<'a> {
    /// starts a new lexer at line 1 column 1
    pub fn New(Source: &'a str) -> Self {
        Self {
            Source: Source,
            Chars: Source.chars().peekable(),
            PeekedToken: None,
            CurrentLine: 1,
            CurrentColumn: 1,
        }
    }

    /// pulls a char and keeps tracking in sync
    fn AdvanceChar(&mut self) -> Option<char> {
        let ch = self.Chars.next()?;
        if ch == '\n' {
            self.CurrentLine += 1;
            self.CurrentColumn = 1;
        } else {
            self.CurrentColumn += 1;
        }
        Some(ch)
    }
    
    /// looks at the next char without pulling it
    fn PeekChar(&mut self) -> Option<&char> {
        self.Chars.peek()
    }

    /// identifies the next token in the stream
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
                        if c == '\n' { break; }
                    }
                    return self.NextToken();
                } else {
                    TokenKind::Slash
                }
            }
            '%' => TokenKind::Percent,
            '=' => TokenKind::Equal,
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

            '\'' => {
                let mut s = String::new();
                while let Some(&c) = self.PeekChar() {
                    if c == '\'' {
                        self.AdvanceChar();
                        return Some(Token { Kind: TokenKind::StringLiteral(s), Line: line, Column: col });
                    }
                    s.push(self.AdvanceChar().unwrap());
                }
                return None;
            }

            c if c.is_alphabetic() || c == '_' => {
                let mut identifier = String::new();
                identifier.push(c);
                while let Some(&next_char) = self.PeekChar() {
                    if next_char.is_alphanumeric() || next_char == '_' {
                        identifier.push(self.AdvanceChar().unwrap());
                    } else {
                        break;
                    }
                }

                match identifier.as_str() {
                    "function" | "fn" => TokenKind::Function,
                    "var" => TokenKind::Let(true),
                    "let" => TokenKind::Let(false),
                    "mut" => TokenKind::Mut,
                    "where" => TokenKind::Where,
                    "havoc" => TokenKind::Havoc,
                    "interrupt" => TokenKind::Interrupt,
                    "unreachable" => TokenKind::Unreachable,
                    "panic" => TokenKind::Panic,
                    "as" => TokenKind::As,
                    "match" => TokenKind::Match,
                    "struct" => TokenKind::Struct,
                    "enum" => TokenKind::Enum,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "while" => TokenKind::While,
                    "for" => TokenKind::For,
                    "loop" => TokenKind::Loop,
                    "return" => TokenKind::Return,
                    "goto" => TokenKind::Goto,
                    other => {
                        if (other.starts_with('i') || other.starts_with('u') || other.starts_with('b') || other.starts_with('f'))
                            && other.len() > 1
                            && other[1..].chars().all(|c| c.is_numeric())
                        {
                            let kind = other.chars().next().unwrap();
                            let bits = other[1..].parse::<u32>().unwrap_or(32);
                            TokenKind::BitPreciseType { Kind: kind, Bits: bits }
                        } else {
                            TokenKind::Ident(other.to_string())
                        }
                    }
                }
            }

            c if c.is_numeric() => {
                let mut num = String::new();
                num.push(c);
                let mut is_float = false;

                while let Some(&next_char) = self.PeekChar() {
                    if next_char.is_numeric() {
                        num.push(self.AdvanceChar().unwrap());
                    } else if next_char == '.' {
                        is_float = true;
                        num.push(self.AdvanceChar().unwrap());
                    } else {
                        break;
                    }
                }

                if is_float {
                    let val = num.parse::<f64>().unwrap_or(0.0);
                    TokenKind::FloatLiteral(val)
                } else {
                    let val = num.parse::<i64>().unwrap_or(0);
                    TokenKind::IntLiteral(val)
                }
            }

            _ => return None,
        };

        Some(Token { Kind: kind, Line: line, Column: col })
    }

    /// checks the next token without pulling it from the stream
    pub fn PeekToken(&mut self) -> Option<Token> {
        if self.PeekedToken.is_none() {
            self.PeekedToken = self.NextToken();
        }
        self.PeekedToken.clone()
    }

    /// clears out the spaces and newlines
    fn SkipWhitespace(&mut self) {
        while let Some(&ch) = self.PeekChar() {
            if ch.is_whitespace() {
                self.AdvanceChar();
            } else {
                break;
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.NextToken()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let Code = "fn main() { var x: i32 = 10; havoc x; }";
        let Tokens: Vec<Token> = Lexer::New(Code).collect();
        assert_eq!(Tokens[5].Kind, TokenKind::Let(true));
    }
}
