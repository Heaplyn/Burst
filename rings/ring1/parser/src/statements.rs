//! Statement parsing: control flow, bindings, returns, and interrupts.

use ast::*;
use lexer::token::TokenKind;

use crate::Parser;

impl Parser {
    /// Identifies what kind of logic statement we have.
    pub fn ParseStatement(&mut self) -> Result<Layer, String> {
        let Tok = self.Peek().map(|t| &t.Kind);
        match Tok {
            Some(TokenKind::Panic) => {
                self.Advance();
                self.Match(TokenKind::Semicolon);
                Ok(LayerBuilder::New(LayerKind::Panic, SourceLocation::Builtin()).Build())
            }
            Some(TokenKind::Unreachable) => {
                self.Advance();
                self.Match(TokenKind::Semicolon);
                Ok(LayerBuilder::New(LayerKind::Unreachable, SourceLocation::Builtin()).Build())
            }
            Some(TokenKind::Let(true) | TokenKind::Let(false)) => {
                let TokenBufferPrevious = self.Peek();
                if config::DebugMode.load(std::sync::atomic::Ordering::Relaxed) {
                    //println!("Token: {:?}", TokenBufferPrevious);
                }
                if TokenBufferPrevious.is_none() {
                    return Err("Unexpected end of input".to_string());
                }
                let TokenBuffer = TokenBufferPrevious.unwrap();

                if TokenBuffer.Kind == TokenKind::Let(true) {
                } else if TokenBuffer.Kind == TokenKind::Let(false) {
                    // handle immutable let binding
                }
                if config::DebugMode.load(std::sync::atomic::Ordering::Relaxed) {
                    //println!("Var binding");
                }
                self.ParseVariableBinding()
            }
            Some(TokenKind::Return) => self.ParseReturn(),
            Some(TokenKind::Havoc) => {
                self.Advance();
                let Expr = self.ParseExpression()?;
                self.Match(TokenKind::Semicolon);
                Ok(LayerBuilder::New(LayerKind::Havoc { Target: Expr }, SourceLocation::Builtin()).Build())
            }
            Some(TokenKind::Interrupt) => self.ParseInterrupt(),
            Some(TokenKind::If) => self.ParseIf(),
            Some(TokenKind::While) => self.ParseWhile(),
            Some(TokenKind::OpenBrace) => {
                self.Advance();
                let mut Children = Vec::new();
                while !self.Check(TokenKind::CloseBrace) && !self.IsAtEnd() {
                    Children.push(self.ParseStatement()?);
                }
                self.Consume(TokenKind::CloseBrace, "Expected '}' after block")?;
                Ok(LayerBuilder::New(LayerKind::Block, SourceLocation::Builtin())
                    .WithChildren(Children)
                    .Build())
            }
            _ => {
                let Expr = self.ParseExpression()?;
                if self.Match(TokenKind::Equal) {
                    let Value = self.ParseExpression()?;
                    self.Match(TokenKind::Semicolon);
                    Ok(LayerBuilder::New(
                        LayerKind::Assignment { Target: Expr, Value },
                        SourceLocation::Builtin(),
                    )
                    .Build())
                } else {
                    self.Match(TokenKind::Semicolon);
                    Ok(LayerBuilder::New(LayerKind::Expression(Expr), SourceLocation::Builtin()).Build())
                }
            }
        }
    }

    /// Parses return statements.
    pub fn ParseReturn(&mut self) -> Result<Layer, String> {
        self.Advance(); // consume 'return'
        let Value = if self.Check(TokenKind::Semicolon) {
            None
        } else {
            Some(self.ParseExpression()?)
        };
        self.Match(TokenKind::Semicolon);
        Ok(LayerBuilder::New(LayerKind::Return { Value }, SourceLocation::Builtin()).Build())
    }

    /// Parses branching logic.
    pub fn ParseIf(&mut self) -> Result<Layer, String> {
        self.Advance(); // 'if'
        self.Consume(TokenKind::OpenParen, "Expected '('")?;
        let Condition = self.ParseExpression()?;
        self.Consume(TokenKind::CloseParen, "Expected ')'")?;
        let mut Children = vec![self.ParseStatement()?];
        let mut HasElse = false;
        if self.Match(TokenKind::Else) {
            HasElse = true;
            Children.push(self.ParseStatement()?);
        }
        Ok(LayerBuilder::New(
            LayerKind::Conditional { Condition, HasElse },
            SourceLocation::Builtin(),
        )
        .WithChildren(Children)
        .Build())
    }

    /// Parses loop logic.
    pub fn ParseWhile(&mut self) -> Result<Layer, String> {
        self.Advance(); // 'while'
        self.Consume(TokenKind::OpenParen, "Expected '('")?;
        let Condition = self.ParseExpression()?;
        self.Consume(TokenKind::CloseParen, "Expected ')'")?;
        let Body = self.ParseStatement()?;
        Ok(LayerBuilder::New(
            LayerKind::Loop {
                Label: None,
                Kind: LoopKind::While(Condition),
            },
            SourceLocation::Builtin(),
        )
        .WithChild(Body)
        .Build())
    }

    /// Parses `interrupt 'syscall'` blocks.
    pub fn ParseInterrupt(&mut self) -> Result<Layer, String> {
        self.Advance(); // consume 'interrupt'
        let Syscall = match self.Advance().map(|t| &t.Kind) {
            Some(TokenKind::StringLiteral(s)) | Some(TokenKind::Ident(s)) => s.clone(),
            _ => return Err("Expected syscall name in interrupt".to_string()),
        };

        self.Match(TokenKind::Comma);
        if !self.Check(TokenKind::OpenBrace) {
            let _ = self.ParseExpression();
        }

        self.Consume(TokenKind::OpenBrace, "Expected '{' for interrupt body")?;
        let mut Body = Vec::new();
        while !self.Check(TokenKind::CloseBrace) && !self.IsAtEnd() {
            Body.push(self.ParseStatement()?);
        }
        self.Consume(TokenKind::CloseBrace, "Expected '}' after interrupt body")?;

        Ok(LayerBuilder::New(LayerKind::Interrupt { Syscall }, SourceLocation::Builtin())
            .WithChildren(Body)
            .Build())
    }

    /// Parses variable bindings with optional hooks.
    pub fn ParseVariableBinding(&mut self) -> Result<Layer, String> {
        let is_mutable = matches!(self.Advance().map(|t| &t.Kind), Some(TokenKind::Let(true)));

        let (Name, TypeAnnotation) = self.ParseNameAndType("variable")?;

        let mut InitialValue = None;
        if self.Match(TokenKind::Equal) {
            InitialValue = Some(self.ParseExpression()?);
        }

        let mut Hooks = Vec::new();
        if self.Check(TokenKind::OpenBrace) {
            self.Advance();
            while !self.Check(TokenKind::CloseBrace) && !self.IsAtEnd() {
                let hook_type = match self.Advance().map(|t| &t.Kind) {
                    Some(TokenKind::Ident(n)) if n == "on_change" => HookKind::OnChange,
                    Some(TokenKind::Ident(n)) if n == "on_read" => HookKind::OnRead,
                    _ => return Err("Expected hook type".to_string()),
                };
                self.Consume(TokenKind::Colon, "Expected ':' after hook type")?;
                let hook_layer = self.ParseFunction()?;
                Hooks.push(VariableHook {
                    Kind: hook_type,
                    Body: vec![hook_layer.Kind],
                });
                self.Match(TokenKind::Comma);
            }
            self.Consume(TokenKind::CloseBrace, "Expected '}' after hooks")?;
        }

        self.Match(TokenKind::Semicolon);

        let var_def = VariableDefinition {
            Name: Name.clone(),
            TypeAnnotation: Some(TypeAnnotation.clone()),
            IsMutable: is_mutable,
            Value: InitialValue.clone().unwrap_or(Expression::Variable("Invalid".to_string())),
        };
        if config::DebugMode.load(std::sync::atomic::Ordering::Relaxed) {
            //println!("Var added: {:?}", var_def);
        }
        self.CurrentLayer.AddVariable(var_def);

        Ok(LayerBuilder::New(
            LayerKind::VariableBinding {
                Name,
                TypeAnnotation: Some(TypeAnnotation),
                IsMutable: is_mutable,
                Hooks,
                InitialValue,
            },
            SourceLocation::Builtin(),
        )
        .Build())
    }
}
