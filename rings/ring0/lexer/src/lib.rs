#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod token;

use token::Token;
pub struct Lexer<'a> {
    pub Source: &'a str,
    pub Chars: std::iter::Peekable<std::str::Chars<'a>>,
    pub PeekedToken: Option<Token>,
}

impl<'a> Lexer<'a> {
    pub fn New(Source: &'a str) -> Self {
        Self {
            Source: Source,
            Chars: Source.chars().peekable(),
            PeekedToken: None,
        }
    }

    pub fn NextToken(&mut self) -> Option<token::Token> {
        if let Some(tok) = self.PeekedToken.take() {
            return Some(tok);
        }
        self.SkipWhitespace();

        let Ch = self.Chars.next()?;

        match Ch {
            // Symbols
            '+' => Some(Token::Plus),
            '-' => {
                if self.Chars.peek() == Some(&'>') {
                    self.Chars.next();
                    Some(Token::Arrow)
                } else {
                    Some(Token::Minus)
                }
            }
            '*' => Some(Token::Star),
            '/' => {
                if self.Chars.peek() == Some(&'/') {
                    // Single-line comment
                    while let Some(c) = self.Chars.next() {
                        if c == '\n' { break; }
                    }
                    self.NextToken()
                } else {
                    Some(Token::Slash)
                }
            }
            '%' => Some(Token::Percent),
            '=' => Some(Token::Equal),
            '(' => Some(Token::OpenParen),
            ')' => Some(Token::CloseParen),
            '{' => Some(Token::OpenBrace),
            '}' => Some(Token::CloseBrace),
            '[' => Some(Token::OpenBracket),
            ']' => Some(Token::CloseBracket),
            '<' => {
                if self.Chars.peek() == Some(&'=') {
                    self.Chars.next();
                    Some(Token::LessEqual)
                } else {
                    Some(Token::Less)
                }
            }
            '>' => {
                if self.Chars.peek() == Some(&'=') {
                    self.Chars.next();
                    Some(Token::GreaterEqual)
                } else {
                    Some(Token::Greater)
                }
            }
            ':' => {
                if self.Chars.peek() == Some(&'=') {
                    self.Chars.next();
                    Some(Token::TypeSet)
                } else {
                    Some(Token::Colon)
                }
            }
            ';' => Some(Token::Semicolon),
            ',' => Some(Token::Comma),
            '.' => Some(Token::Dot),

            // String Literals
            '\'' => {
                let mut s = String::new();
                while let Some(&c) = self.Chars.peek() {
                    if c == '\'' {
                        self.Chars.next();
                        return Some(Token::StringLiteral(s));
                    }
                    s.push(self.Chars.next().unwrap());
                }
                None // Unclosed string
            }

            // Identifiers, Keywords, and Types
            C if C.is_alphabetic() || C == '_' => {
                let mut Identifier = String::new();
                Identifier.push(C);
                while let Some(&NextChar) = self.Chars.peek() {
                    if NextChar.is_alphanumeric() || NextChar == '_' {
                        Identifier.push(self.Chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                // Check for keywords
                match Identifier.as_str() {
                    "function" => Some(Token::Function),
                    "fn" => Some(Token::Function),
                    "where" => Some(Token::Where),
                    "havoc" => Some(Token::Havoc),
                    "interrupt" => Some(Token::Interrupt),
                    "unreachable" => Some(Token::Unreachable),
                    "panic" => Some(Token::Panic),
                    "as" => Some(Token::As),
                    "var" => Some(Token::Var),
                    "let" => Some(Token::Var),
                    "match" => Some(Token::Match),
                    "struct" => Some(Token::Struct),
                    "enum" => Some(Token::Enum),
                    Other => {
                        // Check if it's a bit-precise type (e.g., i32, u16, b8, f64)
                        if (Other.starts_with('i') || Other.starts_with('u') || Other.starts_with('b') || Other.starts_with('f'))
                            && Other.len() > 1
                            && Other[1..].chars().all(|c| c.is_numeric())
                        {
                            let Kind = Other.chars().next().unwrap();
                            let Bits = Other[1..].parse::<u32>().unwrap_or(32);
                            Some(Token::BitPreciseType { Kind, Bits })
                        } else {
                            Some(Token::Ident(Other.to_string()))
                        }
                    }
                }
            }

            // Numeric Literals
            C if C.is_numeric() => {
                let mut Num = String::new();
                Num.push(C);
                let mut IsFloat = false;

                while let Some(&NextChar) = self.Chars.peek() {
                    if NextChar.is_numeric() {
                        Num.push(self.Chars.next().unwrap());
                    } else if NextChar == '.' {
                        IsFloat = true;
                        Num.push(self.Chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                if IsFloat {
                    let Val = Num.parse::<f64>().unwrap_or(0.0);
                    Some(Token::FloatLiteral(Val))
                } else {
                    let Val = Num.parse::<i64>().unwrap_or(0);
                    Some(Token::IntLiteral(Val))
                }
            }

            _ => None, // Unknown character, skip or return None
        }
    }

    pub fn PeekToken(&mut self) -> Option<Token> {
        if self.PeekedToken.is_none() {
            self.PeekedToken = self.NextToken();
        }
        self.PeekedToken.clone()
    }

    fn SkipWhitespace(&mut self) {
        while let Some(&Ch) = self.Chars.peek() {
            if Ch.is_whitespace() {
                self.Chars.next();
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
        let Code = "fn main() { let x: i32 = 10; havoc x; }";
        let Tokens: Vec<Token> = Lexer::New(Code).collect();
        println!("{:?}", Tokens);
        assert_eq!(
            Tokens,
            vec![
                Token::Function,
                Token::Ident("main".to_string()),
                Token::OpenParen,
                Token::CloseParen,
                Token::OpenBrace,
                Token::Var,
                Token::Ident("x".to_string()),
                Token::Colon,
                Token::BitPreciseType { Kind: 'i', Bits: 32 },
                Token::Equal,
                Token::IntLiteral(10),
                Token::Semicolon,
                Token::Havoc,
                Token::Ident("x".to_string()),
                Token::Semicolon,
                Token::CloseBrace,
            ]
        );
    }
}
