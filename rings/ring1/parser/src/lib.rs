#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! Parser (Ring 1): turns a token stream into a layer tree.
//!
//! The `impl Parser` is split across modules for readability:
//! - [`cursor`]      — token peeking/advancing/matching
//! - [`items`]       — top-level items (functions, structs)
//! - [`statements`]  — statements & control flow
//! - [`expressions`] — expression parsing (precedence climbing)
//! - [`types`]       — type parsing and the name+type form

use ast::*;
use lexer::token::{Token, TokenKind};

pub mod cursor;
pub mod expressions;
pub mod items;
pub mod statements;
pub mod types;
pub mod error;

#[derive(Debug, PartialEq)]
/// The thing that builds our layer tree.
pub struct Parser {
    /// The list of tokens from the lexer.
    pub Tokens: Vec<Token>,
    /// Where we are in the list.
    pub Position: usize,
    /// Every token consumed by Advance, in the order it happened.
    pub History: Vec<Token>,
    /// The current layer being built.
    pub CurrentLayer: Layer,
}

impl Parser {
    /// Starts a new parser.
    pub fn New(Tokens: Vec<Token>,CurrentLayer: Layer) -> Self {
        Self {
            Tokens,
            Position: 0,
            History: Vec::new(),
            CurrentLayer,
        }
    }

    /// Main entry point: produces the root `Program` layer.
    pub fn Parse(&mut self) -> Result<Layer, String> {
        let mut Program = LayerBuilder::New(LayerKind::Program, SourceLocation::Builtin()).Build();

        while !self.IsAtEnd() {
            Program.Children.push(self.ParseItem()?);
        }

        Ok(Program)
    }

    /// Identifies what kind of top-level thing we have.
    pub fn ParseItem(&mut self) -> Result<Layer, String> {
        let Tok = self.Peek().map(|t| &t.Kind);
        match Tok {
            Some(TokenKind::Function) => self.ParseFunction(),
            Some(TokenKind::Struct) => self.ParseStruct(),
            _ => self.ParseStatement(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::Lexer;
    use lexer::token::Token;

    #[test]
    fn test_parse_basic_function() {
        let code = "function test() { panic; }";
        let tokens: Vec<Token> = Lexer::New(code).collect();
        let mut parser = Parser::New(tokens, LayerBuilder::New(LayerKind::Program, SourceLocation::Builtin()).Build());
        let ast = parser.Parse().unwrap();
        assert!(matches!(ast.Kind, LayerKind::Program));
        assert_eq!(ast.Children.len(), 1);
    }

    #[test]
    fn test_parse_complex_expressions_and_return() {
        let code = "function test() { return ptr[index]; }";
        let tokens: Vec<Token> = Lexer::New(code).collect();
        let mut parser = Parser::New(tokens, LayerBuilder::New(LayerKind::Program, SourceLocation::Builtin()).Build());
        let ast = parser.Parse().unwrap();
        assert!(matches!(ast.Kind, LayerKind::Program));

        let func = &ast.Children[0];
        assert_eq!(func.Children.len(), 1);
        let ret_stmt = &func.Children[0];
        assert!(matches!(ret_stmt.Kind, LayerKind::Return { .. }));
        if let LayerKind::Return { Value: Some(expr) } = &ret_stmt.Kind {
            assert!(matches!(expr, Expression::IndexAccess { .. }));
        } else {
            panic!("Expected return statement with index access expression");
        }
    }
}
