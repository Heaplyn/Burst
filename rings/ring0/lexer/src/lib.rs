#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Fn,
    Where,
    Havoc,
    Interrupt,
    Unreachable,
    Panic,
    As,
    Let,
    Match,
    Struct,
    Enum,

    // Identifiers and Literals
    Ident(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    BitPreciseType { kind: char, bits: u32 },

    // Operators and Symbols
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    Arrow,          // ->
    OpenParen,      // (
    CloseParen,     // )
    OpenBrace,      // {
    CloseBrace,     // }
    OpenBracket,    // [
    CloseBracket,   // ]
    Colon,
    Semicolon,
    Comma,
}

pub struct Lexer<'a> {
    _source: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            _source: source,
            chars: source.chars().peekable(),
        }
    }


    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        let ch = self.chars.next()?;

        match ch {
            // Symbols
            '+' => Some(Token::Plus),
            '-' => {
                if self.chars.peek() == Some(&'>') {
                    self.chars.next();
                    Some(Token::Arrow)
                } else {
                    Some(Token::Minus)
                }
            }
            '*' => Some(Token::Star),
            '/' => Some(Token::Slash),
            '%' => Some(Token::Percent),
            '=' => Some(Token::Equal),
            '(' => Some(Token::OpenParen),
            ')' => Some(Token::CloseParen),
            '{' => Some(Token::OpenBrace),
            '}' => Some(Token::CloseBrace),
            '[' => Some(Token::OpenBracket),
            ']' => Some(Token::CloseBracket),
            ':' => Some(Token::Colon),
            ';' => Some(Token::Semicolon),
            ',' => Some(Token::Comma),

            // Identifiers, Keywords, and Types
            c if c.is_alphabetic() || c == '_' => {
                let mut ident = String::new();
                ident.push(c);
                while let Some(&next_char) = self.chars.peek() {
                    if next_char.is_alphanumeric() || next_char == '_' {
                        ident.push(self.chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                // Check for keywords
                match ident.as_str() {
                    "fn" => Some(Token::Fn),
                    "where" => Some(Token::Where),
                    "havoc" => Some(Token::Havoc),
                    "interrupt" => Some(Token::Interrupt),
                    "unreachable" => Some(Token::Unreachable),
                    "panic" => Some(Token::Panic),
                    "as" => Some(Token::As),
                    "let" => Some(Token::Let),
                    "match" => Some(Token::Match),
                    "struct" => Some(Token::Struct),
                    "enum" => Some(Token::Enum),
                    other => {
                        // Check if it's a bit-precise type (e.g., i32, u16, b8, f64)
                        if (other.starts_with('i') || other.starts_with('u') || other.starts_with('b') || other.starts_with('f')) 
                            && other.len() > 1 
                            && other[1..].chars().all(|c| c.is_numeric()) 
                        {
                            let kind = other.chars().next().unwrap();
                            let bits = other[1..].parse::<u32>().unwrap_or(32);
                            Some(Token::BitPreciseType { kind, bits })
                        } else {
                            Some(Token::Ident(other.to_string()))
                        }
                    }
                }
            }

            // Numeric Literals
            c if c.is_numeric() => {
                let mut num = String::new();
                num.push(c);
                let mut is_float = false;

                while let Some(&next_char) = self.chars.peek() {
                    if next_char.is_numeric() {
                        num.push(self.chars.next().unwrap());
                    } else if next_char == '.' {
                        is_float = true;
                        num.push(self.chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                if is_float {
                    let val = num.parse::<f64>().unwrap_or(0.0);
                    Some(Token::FloatLiteral(val))
                } else {
                    let val = num.parse::<i64>().unwrap_or(0);
                    Some(Token::IntLiteral(val))
                }
            }

            _ => None, // Unknown character, skip or return None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.chars.peek() {
            if ch.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let code = "fn main() { let x: i32 = 10; havoc x; }";
        let tokens: Vec<Token> = Lexer::new(code).collect();
        assert_eq!(
            tokens,
            vec![
                Token::Fn,
                Token::Ident("main".to_string()),
                Token::OpenParen,
                Token::CloseParen,
                Token::OpenBrace,
                Token::Let,
                Token::Ident("x".to_string()),
                Token::Colon,
                Token::BitPreciseType { kind: 'i', bits: 32 },
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
