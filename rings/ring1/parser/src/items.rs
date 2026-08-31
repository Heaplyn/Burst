//! Top-level item parsing: functions and structs.

use ast::*;
use lexer::token::{Token, TokenKind};

use crate::Parser;

impl Parser {
    /// Parses a struct definition.
    pub fn ParseStruct(&mut self) -> Result<Layer, String> {
        self.Advance(); // consume 'struct'
        let Name = match self.Advance().map(|t| &t.Kind) {
            Some(TokenKind::Ident(n)) => n.clone(),
            _ => return Err("Expected struct name".to_string()),
        };

        self.Consume(TokenKind::OpenBrace, "Expected '{' before struct fields")?;
        let mut Fields = Vec::new();
        while !self.Check(TokenKind::CloseBrace) && !self.IsAtEnd() {
            let (FieldName, FieldType) = self.ParseNameAndType("field")?;
            Fields.push(StructField { Name: FieldName, Type_: FieldType });
            self.Match(TokenKind::Comma);
            self.Match(TokenKind::Semicolon);
        }
        self.Consume(TokenKind::CloseBrace, "Expected '}' after struct fields")?;

        Ok(LayerBuilder::New(
            LayerKind::Struct {
                Name,
                Fields,
                IsPacked: false,
            },
            SourceLocation::Builtin(),
        )
        .Build())
    }

    /// Parses a function or procedure.
    pub fn ParseFunction(&mut self) -> Result<Layer, String> {
        self.Advance(); // consume keyword
        

        let Name = if let Some(TokenKind::Ident(name)) = self.Peek().map(|t| &t.Kind) {
            let n = name.clone();
            self.Advance();
            n
        } else {
            "".to_string()
        };

        self.Consume(TokenKind::OpenParen, "Expected '(' after function name")?;
        let mut Params = Vec::new();
        if !self.Check(TokenKind::CloseParen) {
            loop {
                if self.Match(TokenKind::Comma) {
                    //self.Advance(); // consume the comma or 'None'
                    continue;
                }
                let (ParamName, ParamType) = self.ParseNameAndType("parameter")?;
                let Val: Type;
                if config::DebugMode.load(std::sync::atomic::Ordering::Relaxed) {
                    println!("Param Name: {:?}, Param Type: {:?}", ParamName, ParamType);
                }
                if ParamType == Type::Null {
                    //self.Advance();
                    continue;
                }
                Val = ParamType.clone();
                Params.push(Param {
                    Name: ParamName,
                    Type_: ParamType,
                    Value: Val,
                });
                if self.Match(TokenKind::Invalid) {
                    self.Advance();
                    continue;
                }
                if self.Match(TokenKind::And) {
                    continue;
                }
                if self.Match(TokenKind::CloseParen) {
                    break;
                }
            }
        } else {
            self.Consume(TokenKind::CloseParen, "Expected ')' after parameters")?;
        }
        

        //self.Consume(TokenKind::CloseParen, "Expected ')' after parameters")?;

        let mut ReturnType = None;
        if let Some(TokenKind::Arrow) = self.Peek().map(|t| &t.Kind) {
            self.Advance();
            ReturnType = Some(self.ParseType()?);
        }
        //println!("Return Type: {:?}", ReturnType);
        self.Consume(TokenKind::OpenBrace, "Expected '{' before function body")?;
        let mut Body = Vec::new();
        while !self.Check(TokenKind::CloseBrace) && !self.IsAtEnd() {
            Body.push(self.ParseStatement()?);
        }
        self.Consume(TokenKind::CloseBrace, "Expected '}' after function body")?;

        Ok(LayerBuilder::New(
            LayerKind::Function {
                Name,
                Params,
                ReturnType,
                IsUnsafe: false,
                IsExtern: false,
            },
            SourceLocation::Builtin(),
        )
        .WithChildren(Body)
        .Build())
    }
}
