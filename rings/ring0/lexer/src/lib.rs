#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! Ring 0 · **Lexer crate hub**
//!
//! Modules (with file-level docs stating what each requires):
//! - [`token`]    — token definitions (`Token`, `TokenKind`, `Variable<T>`)
//! - [`lexer`]    — the `Lexer` struct + cursor helpers + `Iterator` impl
//! - [`scan`]     — the per-character `NextToken` scanner
//! - [`keywords`] — the keyword table + bit-precise-type recognizer

pub mod keywords;
pub mod lexer;
pub mod scan;
pub mod token;

// Flat re-exports so `use lexer::Lexer;` and `use lexer::token::*;` keep working.
pub use lexer::Lexer;
pub use token::{Token, TokenKind};

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
