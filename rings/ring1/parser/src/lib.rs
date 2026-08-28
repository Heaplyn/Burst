#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use ast::*;
use lexer::token::{Token, TokenKind};

/// the thing that builds our layer tree
pub struct Parser {
    /// the list of tokens from the lexer
    pub Tokens: Vec<Token>,
    /// where we are in the list
    pub Position: usize,
}

impl Parser {
    /// starts a new parser
    pub fn New(Tokens: Vec<Token>) -> Self {
        Self { Tokens, Position: 0 }
    }

    /// main entry point to get the program layer
    pub fn Parse(&mut self) -> Result<Layer, String> {
        let mut Program = LayerBuilder::New(LayerKind::Program, SourceLocation::Builtin()).Build();

        while !self.IsAtEnd() {
            Program.Children.push(self.ParseItem()?);
        }

        Ok(Program)
    }

    /// identifies what kind of top level thing we have
    pub fn ParseItem(&mut self) -> Result<Layer, String> {
        let Tok = self.Peek().map(|t| &t.Kind);
        match Tok {
            Some(TokenKind::Function) => self.ParseFunction(),
            Some(TokenKind::Struct) => self.ParseStruct(),
            _ => self.ParseStatement(),
        }
    }

    /// parses a struct definition
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
        ).Build())
    }

    /// handles both "name: type" and "type name"
    pub fn ParseNameAndType(&mut self, context: &str) -> Result<(String, Type), String> {
        let mut found_type_colon = false;
        if let Some(TokenKind::Ident(_)) = self.Peek().map(|t| &t.Kind) {
            if let Some(TokenKind::Colon) = self.PeekAt(1).map(|t| &t.Kind) {
                if !matches!(self.PeekAt(2).map(|t| &t.Kind), Some(TokenKind::Where)) {
                    found_type_colon = true;
                }
            }
        }

        let (name, mut ty) = if found_type_colon {
            let n = match self.Advance().map(|t| &t.Kind) {
                Some(TokenKind::Ident(name)) => name.clone(),
                _ => unreachable!(),
            };
            self.Advance(); // consume ':'
            let t = self.ParseType()?;
            (n, t)
        } else {
            let t = self.ParseType()?;
            let n = match self.Advance().map(|t| &t.Kind) {
                Some(TokenKind::Ident(name)) => name.clone(),
                _ => return Err(format!("Expected {} name after type. Found {:?}", context, self.Peek())),
            };
            (n, t)
        };

        let has_refinement = if self.Match(TokenKind::Where) {
            true
        } else if matches!(self.Peek().map(|t| &t.Kind), Some(TokenKind::Colon)) && matches!(self.PeekAt(1).map(|t| &t.Kind), Some(TokenKind::Where)) {
            self.Advance(); // consume ':'
            self.Advance(); // consume 'where'
            true
        } else {
            false
        };

        if has_refinement {
            let ConstraintExpr = self.ParseExpression()?;
            ty = Type::Where(Box::new(ty), Box::new(ConstraintExpr));
        }

        Ok((name, ty))
    }

    /// parses a function or proc
    pub fn ParseFunction(&mut self) -> Result<Layer, String> {
        self.Advance(); // consume keyword

        let Name = match self.Advance().map(|t| &t.Kind) {
            Some(TokenKind::Ident(name)) => name.clone(),
            _ => return Err("Expected function name".to_string()),
        };

        self.Consume(TokenKind::OpenParen, "Expected '(' after function name")?;
        let mut Params = Vec::new();
        if !self.Check(TokenKind::CloseParen) {
            loop {
                let (ParamName, ParamType) = self.ParseNameAndType("parameter")?;
                Params.push(Param { Name: ParamName, Type_: ParamType });
                if !self.Match(TokenKind::Comma) { break; }
            }
        }
        self.Consume(TokenKind::CloseParen, "Expected ')' after parameters")?;

        let mut ReturnType = None;
        if let Some(TokenKind::Arrow) = self.Peek().map(|t| &t.Kind) {
            self.Advance();
            ReturnType = Some(self.ParseType()?);
        }

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
        ).WithChildren(Body).Build())
    }

    /// identifies what kind of logic statement we have
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
            Some(TokenKind::Let(true|false)) => self.ParseVariableBinding(),
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
                Ok(LayerBuilder::New(LayerKind::Block, SourceLocation::Builtin()).WithChildren(Children).Build())
            }
            _ => {
                let Expr = self.ParseExpression()?;
                if self.Match(TokenKind::Equal) {
                    let Value = self.ParseExpression()?;
                    self.Match(TokenKind::Semicolon);
                    Ok(LayerBuilder::New(LayerKind::Assignment { Target: Expr, Value }, SourceLocation::Builtin()).Build())
                } else {
                    self.Match(TokenKind::Semicolon);
                    Ok(LayerBuilder::New(LayerKind::Expression(Expr), SourceLocation::Builtin()).Build())
                }
            }
        }
    }

    /// parses branching logic
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
        Ok(LayerBuilder::New(LayerKind::Conditional { Condition, HasElse }, SourceLocation::Builtin()).WithChildren(Children).Build())
    }

    /// parses loop logic
    pub fn ParseWhile(&mut self) -> Result<Layer, String> {
        self.Advance(); // 'while'
        self.Consume(TokenKind::OpenParen, "Expected '('")?;
        let Condition = self.ParseExpression()?;
        self.Consume(TokenKind::CloseParen, "Expected ')'")?;
        let Body = self.ParseStatement()?;
        Ok(LayerBuilder::New(LayerKind::Loop { Label: None, Kind: LoopKind::While(Condition) }, SourceLocation::Builtin()).WithChild(Body).Build())
    }

    /// parses interrupt 'syscall' blocks
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

        Ok(LayerBuilder::New(LayerKind::Interrupt { Syscall }, SourceLocation::Builtin()).WithChildren(Body).Build())
    }

    /// parses variable bindings with optional hooks
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
                Hooks.push(VariableHook { Kind: hook_type, Body: vec![hook_layer.Kind] });
                self.Match(TokenKind::Comma);
            }
            self.Consume(TokenKind::CloseBrace, "Expected '}' after hooks")?;
        }

        self.Match(TokenKind::Semicolon);

        Ok(LayerBuilder::New(
            LayerKind::VariableBinding {
                Name,
                TypeAnnotation: Some(TypeAnnotation),
                IsMutable: is_mutable,
                Hooks,
                InitialValue,
            },
            SourceLocation::Builtin(),
        ).Build())
    }

    /// entry point for expressions
    pub fn ParseExpression(&mut self) -> Result<Expression, String> {
        self.ParseBinary(0)
    }

    /// handles operator precedence for math and logic
    pub fn ParseBinary(&mut self, Precedence: u8) -> Result<Expression, String> {
        let mut Expr = self.ParsePrimary()?;

        while let Some(tok) = self.Peek().map(|t| &t.Kind) {
            let next_prec = self.TokenPrecedence(tok);
            if next_prec <= Precedence {
                break;
            }

            let op_tok = self.Advance().unwrap().Kind.clone();
            let Rhs = self.ParseBinary(next_prec)?;
            Expr = Expression::BinaryOp {
                Op: match op_tok {
                    TokenKind::Plus => "+".to_string(),
                    TokenKind::Minus => "-".to_string(),
                    TokenKind::Star => "*".to_string(),
                    TokenKind::Slash => "/".to_string(),
                    TokenKind::Percent => "%".to_string(),
                    TokenKind::Equal => "=".to_string(),
                    TokenKind::Less => "<".to_string(),
                    TokenKind::Greater => ">".to_string(),
                    TokenKind::LessEqual => "<=".to_string(),
                    TokenKind::GreaterEqual => ">=".to_string(),
                    TokenKind::As => "as".to_string(),
                    _ => format!("{:?}", op_tok),
                },
                Lhs: Box::new(Expr),
                Rhs: Box::new(Rhs),
            };
        }

        Ok(Expr)
    }

    /// the order of operations table
    pub fn TokenPrecedence(&self, Tok: &TokenKind) -> u8 {
        match Tok {
            TokenKind::Equal => 1,
            TokenKind::Less | TokenKind::Greater | TokenKind::LessEqual | TokenKind::GreaterEqual => 2,
            TokenKind::Plus | TokenKind::Minus => 3,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => 4,
            TokenKind::As => 5,
            _ => 0,
        }
    }

    /// parses atoms like numbers, strings, and idents
    pub fn ParsePrimary(&mut self) -> Result<Expression, String> {
        let Tok = self.Peek().cloned();
        match Tok.as_ref().map(|t| &t.Kind) {
            Some(TokenKind::IntLiteral(val)) => { let v = *val; self.Advance(); Ok(Expression::LiteralInt(v)) }
            Some(TokenKind::FloatLiteral(val)) => { let v = *val; self.Advance(); Ok(Expression::LiteralFloat(v)) }
            Some(TokenKind::StringLiteral(val)) => { let v = val.clone(); self.Advance(); Ok(Expression::LiteralString(v)) }
            Some(TokenKind::Ident(name)) => {
                let n = name.clone();
                self.Advance();
                let mut Expr = Expression::Variable(n.clone());
                while let Some(TokenKind::Dot) = self.Peek().map(|t| &t.Kind) {
                    self.Advance();
                    let member = match self.Advance().map(|t| &t.Kind) {
                        Some(TokenKind::Ident(m)) => m.clone(),
                        _ => return Err("Expected member name after '.'".to_string()),
                    };
                    Expr = Expression::Variable(format!("{}.{}", n, member));
                }
                if self.Check(TokenKind::OpenParen) {
                    self.Advance();
                    let mut Args = Vec::new();
                    if !self.Check(TokenKind::CloseParen) {
                        loop {
                            Args.push(self.ParseExpression()?);
                            if !self.Match(TokenKind::Comma) { break; }
                        }
                    }
                    self.Consume(TokenKind::CloseParen, "Expected ')' after arguments")?;
                    let Name = if let Expression::Variable(n) = Expr { n } else { format!("{:?}", Expr) };
                    Ok(Expression::FunctionCall { Name, Args })
                } else {
                    Ok(Expr)
                }
            }
            Some(TokenKind::BitPreciseType { Kind, Bits }) => {
                let k = *Kind;
                let b = *Bits;
                self.Advance();
                if k == 'i' {
                    Ok(Expression::LiteralInt(<i64>::from(b)))
                } else {
                    Ok(Expression::BitPreciseType { Kind: k, Bits: b })
                }
            }
            Some(TokenKind::Star) => {
                self.Advance();
                let Target = self.ParsePrimary()?;
                Ok(Expression::UnaryOp { Op: "*".to_string(), Target: Box::new(Target) })
            }
            Some(TokenKind::OpenParen) => {
                self.Advance();
                let Expr = self.ParseExpression()?;
                self.Consume(TokenKind::CloseParen, "Expected ')'")?;
                Ok(Expr)
            }
            _ => Err(format!("Unexpected token in expression: {:?}", Tok)),
        }
    }

    /// parses type names and pointer levels
    pub fn ParseType(&mut self) -> Result<Type, String> {
        let mut BaseType = match self.Peek().map(|t| &t.Kind) {
            Some(TokenKind::Star) => {
                self.Advance();
                let Inner = self.ParseType()?;
                Type::Pointer(Box::new(Inner))
            }
            Some(TokenKind::BitPreciseType { Kind, Bits }) => {
                let k = *Kind;
                let b = *Bits;
                self.Advance();
                Type::BitPrecise(k, b)
            }
            Some(TokenKind::Ident(name)) => {
                let n = name.clone();
                self.Advance();
                if self.Match(TokenKind::Star) {
                    Type::Pointer(Box::new(Type::Named(n)))
                } else {
                    Type::Named(n)
                }
            }
            _ => return Err(format!("Expected type. Found {:?}", self.Peek())),
        };

        if self.Match(TokenKind::Where) {
            let ConstraintExpr = self.ParseExpression()?;
            BaseType = Type::Where(Box::new(BaseType), Box::new(ConstraintExpr));
        }

        Ok(BaseType)
    }

    /// look at the current token
    pub fn Peek(&self) -> Option<&Token> {
        self.Tokens.get(self.Position)
    }

    /// look ahead by a specific amount
    pub fn PeekAt(&self, Offset: usize) -> Option<&Token> {
        self.Tokens.get(self.Position + Offset)
    }

    /// pull the next token from the list
    pub fn Advance(&mut self) -> Option<&Token> {
        if !self.IsAtEnd() {
            self.Position += 1;
        }
        self.Tokens.get(self.Position - 1)
    }

    /// just check without consuming
    pub fn Check(&self, Target: TokenKind) -> bool {
        self.Peek().map(|t| &t.Kind) == Some(&Target)
    }

    /// consume if it matches
    pub fn Match(&mut self, Target: TokenKind) -> bool {
        if self.Check(Target) {
            self.Advance();
            true
        } else {
            false
        }
    }

    /// checked if we're done
    pub fn IsAtEnd(&self) -> bool {
        self.Position >= self.Tokens.len()
    }

    /// enforce a specific token or error out
    pub fn Consume(&mut self, Target: TokenKind, Message: &str) -> Result<(), String> {
        if self.Check(Target) {
            self.Advance();
            Ok(())
        } else {
            Err(format!("{}. Found {:?}", Message, self.Peek()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::Lexer;

    #[test]
    fn test_parse_basic_function() {
        let code = "function test() { panic; }";
        let tokens: Vec<Token> = Lexer::New(code).collect();
        let mut parser = Parser::New(tokens);
        let ast = parser.Parse().unwrap();
        assert!(matches!(ast.Kind, LayerKind::Program));
        assert_eq!(ast.Children.len(), 1);
    }
}
