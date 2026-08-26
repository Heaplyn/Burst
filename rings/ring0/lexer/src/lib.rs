#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[derive(Debug, Clone, PartialEq)]
pub enum token {
    // Keywords
    fn_,
    where_,
    havoc,
    interrupt,
    unreachable,
    panic,
    as_,
    let_,
    match_,
    struct_,
    enum_,

    // Identifiers and Literals
    ident(String),
    int_literal(i64),
    float_literal(f64),
    bit_precise_type { kind: char, bits: u32 },

    // Operators and Symbols
    plus,
    minus,
    star,
    slash,
    percent,
    equal,
    arrow,          // ->
    open_paren,      // (
    close_paren,     // )
    open_brace,      // {
    close_brace,     // }
    open_bracket,    // [
    close_bracket,   // ]
    colon,
    semicolon,
    comma,
}

pub struct lexer<'a> {
    _source: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            _source: source,
            chars: source.chars().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Option<token> {
        self.skip_whitespace();

        let ch = self.chars.next()?;

        match ch {
            // Symbols
            '+' => Some(token::plus),
            '-' => {
                if self.chars.peek() == Some(&'>') {
                    self.chars.next();
                    Some(token::arrow)
                } else {
                    Some(token::minus)
                }
            }
            '*' => Some(token::star),
            '/' => Some(token::slash),
            '%' => Some(token::percent),
            '=' => Some(token::equal),
            '(' => Some(token::open_paren),
            ')' => Some(token::close_paren),
            '{' => Some(token::open_brace),
            '}' => Some(token::close_brace),
            '[' => Some(token::open_bracket),
            ']' => Some(token::close_bracket),
            ':' => Some(token::colon),
            ';' => Some(token::semicolon),
            ',' => Some(token::comma),

            // Identifiers, Keywords, and Types
            c if c.is_alphabetic() || c == '_' => {
                let mut identifier = String::new();
                identifier.push(c);
                while let Some(&next_char) = self.chars.peek() {
                    if next_char.is_alphanumeric() || next_char == '_' {
                        identifier.push(self.chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                // Check for keywords
                match identifier.as_str() {
                    "fn" => Some(token::fn_),
                    "where" => Some(token::where_),
                    "havoc" => Some(token::havoc),
                    "interrupt" => Some(token::interrupt),
                    "unreachable" => Some(token::unreachable),
                    "panic" => Some(token::panic),
                    "as" => Some(token::as_),
                    "let" => Some(token::let_),
                    "match" => Some(token::match_),
                    "struct" => Some(token::struct_),
                    "enum" => Some(token::enum_),
                    other => {
                        // Check if it's a bit-precise type (e.g., i32, u16, b8, f64)
                        if (other.starts_with('i') || other.starts_with('u') || other.starts_with('b') || other.starts_with('f')) 
                            && other.len() > 1 
                            && other[1..].chars().all(|c| c.is_numeric()) 
                        {
                            let kind = other.chars().next().unwrap();
                            let bits = other[1..].parse::<u32>().unwrap_or(32);
                            Some(token::bit_precise_type { kind, bits })
                        } else {
                            Some(token::ident(other.to_string()))
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
                    Some(token::float_literal(val))
                } else {
                    let val = num.parse::<i64>().unwrap_or(0);
                    Some(token::int_literal(val))
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

impl<'a> Iterator for lexer<'a> {
    type Item = token;

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
        let tokens: Vec<token> = lexer::new(code).collect();
        assert_eq!(
            tokens,
            vec![
                token::fn_,
                token::ident("main".to_string()),
                token::open_paren,
                token::close_paren,
                token::open_brace,
                token::let_,
                token::ident("x".to_string()),
                token::colon,
                token::bit_precise_type { kind: 'i', bits: 32 },
                token::equal,
                token::int_literal(10),
                token::semicolon,
                token::havoc,
                token::ident("x".to_string()),
                token::semicolon,
                token::close_brace,
            ]
        );
    }
}
