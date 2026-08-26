#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use ast::*;
use lexer::token::Token;

pub struct Parser {
    pub Tokens: Vec<Token>,
    pub Position: usize,
}

impl Parser {
    pub fn New(Tokens: Vec<Token>) -> Self {
        Self { Tokens, Position: 0 }
    }

    pub fn Parse(&mut self) -> Result<Layer, String> {
        let mut Program = LayerBuilder::New(LayerKind::Program, SourceLocation::Builtin()).Build();

        while !self.IsAtEnd() {
            Program.Children.push(self.ParseItem()?);
        }

        Ok(Program)
    }

    fn ParseItem(&mut self) -> Result<Layer, String> {
        let Tok = self.Peek();
        match Tok {
            Some(Token::Function) => self.ParseFunction(),
            Some(Token::Struct) => self.ParseStruct(),
            _ => self.ParseStatement(),
        }
    }

    fn ParseStruct(&mut self) -> Result<Layer, String> {
        self.Advance(); // consume 'struct'
        let Name = match self.Advance() {
            Some(Token::Ident(n)) => n.clone(),
            _ => return Err("Expected struct name".to_string()),
        };

        self.Consume(Token::OpenBrace, "Expected '{' before struct fields")?;
        let mut Fields = Vec::new();
        while !self.Check(Token::CloseBrace) && !self.IsAtEnd() {
            let mut is_colon_style = false;
            let mut offset = 0;
            while let Some(t) = self.PeekAt(offset) {
                if matches!(t, Token::Colon) { is_colon_style = true; break; }
                if matches!(t, Token::Comma) || matches!(t, Token::Semicolon) || matches!(t, Token::CloseBrace) { break; }
                offset += 1;
            }

            if is_colon_style {
                let FieldName = match self.Advance() {
                    Some(Token::Ident(n)) => n.clone(),
                    _ => return Err("Expected field name before ':'".to_string()),
                };
                self.Advance(); // consume colon
                let FieldType = self.ParseType()?;
                Fields.push(StructField { Name: FieldName, Type_: FieldType });
            } else {
                let FieldType = self.ParseType()?;
                let FieldName = match self.Advance() {
                    Some(Token::Ident(n)) => n.clone(),
                    _ => return Err(format!("Expected field name after type. Found {:?}", self.Peek())),
                };
                Fields.push(StructField { Name: FieldName, Type_: FieldType });
            }

            self.Match(Token::Comma);
            self.Match(Token::Semicolon);
        }
        self.Consume(Token::CloseBrace, "Expected '}' after struct fields")?;

        Ok(LayerBuilder::New(
            LayerKind::Struct {
                Name,
                Fields,
                IsPacked: false,
            },
            SourceLocation::Builtin(),
        ).Build())
    }

    fn ParseFunction(&mut self) -> Result<Layer, String> {
        self.Advance(); // Consume 'function'

        let Name = match self.Advance() {
            Some(Token::Ident(name)) => name.clone(),
            _ => return Err("Expected function name".to_string()),
        };

        self.Consume(Token::OpenParen, "Expected '(' after function name")?;
        let mut Params = Vec::new();
        if !self.Check(Token::CloseParen) {
            loop {
                // Determine parameter syntax: "type name" or "name: type"
                let mut is_colon_style = false;
                let mut offset = 0;
                while let Some(t) = self.PeekAt(offset) {
                    if matches!(t, Token::Colon) { is_colon_style = true; break; }
                    if matches!(t, Token::Comma) || matches!(t, Token::CloseParen) { break; }
                    offset += 1;
                }

                if is_colon_style {
                    let ParamName = match self.Advance() {
                        Some(Token::Ident(n)) => n.clone(),
                        _ => return Err("Expected parameter name before ':'".to_string()),
                    };
                    self.Advance(); // consume colon
                    let ParamType = self.ParseType()?;
                    Params.push(Param { Name: ParamName, Type_: ParamType });
                } else {
                    let ParamType = self.ParseType()?;
                    let ParamName = match self.Advance() {
                        Some(Token::Ident(n)) => n.clone(),
                        _ => return Err(format!("Expected parameter name after type. Found {:?}", self.Peek())),
                    };
                    Params.push(Param { Name: ParamName, Type_: ParamType });
                }

                if !self.Match(Token::Comma) { break; }
            }
        }
        self.Consume(Token::CloseParen, "Expected ')' after parameters")?;

        let mut ReturnType = None;
        if let Some(Token::Arrow) = self.Peek() {
            self.Advance();
            ReturnType = Some(self.ParseType()?);
        }

        self.Consume(Token::OpenBrace, "Expected '{' before function body")?;
        let mut Body = Vec::new();
        while !self.Check(Token::CloseBrace) && !self.IsAtEnd() {
            Body.push(self.ParseStatement()?);
        }
        self.Consume(Token::CloseBrace, "Expected '}' after function body")?;

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

    fn ParseStatement(&mut self) -> Result<Layer, String> {
        let Tok = self.Peek();
        match Tok {
            Some(Token::Panic) => {
                self.Advance();
                self.Match(Token::Semicolon);
                Ok(LayerBuilder::New(LayerKind::Panic, SourceLocation::Builtin()).Build())
            }
            Some(Token::Unreachable) => {
                self.Advance();
                self.Match(Token::Semicolon);
                Ok(LayerBuilder::New(LayerKind::Unreachable, SourceLocation::Builtin()).Build())
            }
            Some(Token::Var) => self.ParseVariableBinding(),
            Some(Token::Havoc) => {
                self.Advance();
                let Expr = self.ParseExpression()?;
                self.Match(Token::Semicolon);
                Ok(LayerBuilder::New(LayerKind::Havoc { Target: Expr }, SourceLocation::Builtin()).Build())
            }
            Some(Token::Interrupt) => self.ParseInterrupt(),
            Some(Token::OpenBrace) => {
                self.Advance();
                let mut Children = Vec::new();
                while !self.Check(Token::CloseBrace) && !self.IsAtEnd() {
                    Children.push(self.ParseStatement()?);
                }
                self.Consume(Token::CloseBrace, "Expected '}' after block")?;
                Ok(LayerBuilder::New(LayerKind::Block, SourceLocation::Builtin()).WithChildren(Children).Build())
            }
            _ => {
                let Expr = self.ParseExpression()?;
                if self.Match(Token::Equal) {
                    let Value = self.ParseExpression()?;
                    self.Match(Token::Semicolon);
                    Ok(LayerBuilder::New(LayerKind::Assignment { Target: Expr, Value }, SourceLocation::Builtin()).Build())
                } else {
                    self.Match(Token::Semicolon);
                    Ok(LayerBuilder::New(LayerKind::Expression(Expr), SourceLocation::Builtin()).Build())
                }
            }
        }
    }

    fn ParseInterrupt(&mut self) -> Result<Layer, String> {
        self.Advance(); // consume 'interrupt'
        let Syscall = match self.Advance() {
            Some(Token::StringLiteral(s)) => s.clone(),
            Some(Token::Ident(i)) => i.clone(),
            _ => return Err("Expected syscall name in interrupt".to_string()),
        };

        self.Match(Token::Comma);
        if !self.Check(Token::OpenBrace) {
             let _ = self.ParseExpression();
        }

        self.Consume(Token::OpenBrace, "Expected '{' for interrupt body")?;
        let mut Body = Vec::new();
        while !self.Check(Token::CloseBrace) && !self.IsAtEnd() {
            Body.push(self.ParseStatement()?);
        }
        self.Consume(Token::CloseBrace, "Expected '}' after interrupt body")?;

        Ok(LayerBuilder::New(LayerKind::Interrupt { Syscall }, SourceLocation::Builtin()).WithChildren(Body).Build())
    }

    fn ParseVariableBinding(&mut self) -> Result<Layer, String> {
        self.Advance(); // consume 'var' or 'let'

        let mut is_colon_style = false;
        let mut offset = 0;
        while let Some(t) = self.PeekAt(offset) {
            if matches!(t, Token::Colon) { is_colon_style = true; break; }
            if matches!(t, Token::Equal) || matches!(t, Token::Semicolon) { break; }
            offset += 1;
        }

        let (Name, TypeAnnotation) = if is_colon_style {
            let n = match self.Advance() {
                Some(Token::Ident(name)) => name.clone(),
                _ => return Err("Expected variable name before ':'".to_string()),
            };
            self.Advance(); // consume colon
            let ty = self.ParseType()?;
            (n, Some(ty))
        } else {
            let ty = self.ParseType()?;
            let n = match self.Advance() {
                 Some(Token::Ident(name)) => name.clone(),
                 _ => return Err(format!("Expected variable name after type. Found {:?}", self.Peek())),
            };
            (n, Some(ty))
        };

        let mut InitialValue = None;
        if self.Match(Token::Equal) {
            InitialValue = Some(self.ParseExpression()?);
        }

        self.Match(Token::Semicolon);

        Ok(LayerBuilder::New(
            LayerKind::VariableBinding {
                Name,
                TypeAnnotation,
                IsMutable: false,
                Hooks: Vec::new(),
                InitialValue,
            },
            SourceLocation::Builtin(),
        ).Build())
    }

    fn ParseExpression(&mut self) -> Result<Expression, String> {
        self.ParseBinary(0)
    }

    fn ParseBinary(&mut self, Precedence: u8) -> Result<Expression, String> {
        let mut Expr = self.ParsePrimary()?;

        while let Some(tok) = self.Peek() {
            let next_prec = self.TokenPrecedence(tok);
            if next_prec <= Precedence {
                break;
            }

            let op_tok = self.Advance().unwrap().clone();
            let Rhs = self.ParseBinary(next_prec)?;
            Expr = Expression::BinaryOp {
                Op: match op_tok {
                    Token::Plus => "+".to_string(),
                    Token::Minus => "-".to_string(),
                    Token::Star => "*".to_string(),
                    Token::Slash => "/".to_string(),
                    Token::Percent => "%".to_string(),
                    Token::Equal => "=".to_string(),
                    Token::Less => "<".to_string(),
                    Token::Greater => ">".to_string(),
                    Token::LessEqual => "<=".to_string(),
                    Token::GreaterEqual => ">=".to_string(),
                    Token::As => "as".to_string(),
                    _ => format!("{:?}", op_tok),
                },
                Lhs: Box::new(Expr),
                Rhs: Box::new(Rhs),
            };
        }

        Ok(Expr)
    }

    fn TokenPrecedence(&self, Tok: &Token) -> u8 {
        match Tok {
            Token::Equal => 1,
            Token::Less | Token::Greater | Token::LessEqual | Token::GreaterEqual => 2,
            Token::Plus | Token::Minus => 3,
            Token::Star | Token::Slash | Token::Percent => 4,
            Token::As => 5,
            _ => 0,
        }
    }

    fn ParsePrimary(&mut self) -> Result<Expression, String> {
        let Tok = self.Peek().cloned();
        match Tok {
            Some(Token::IntLiteral(val)) => { self.Advance(); Ok(Expression::LiteralInt(val)) }
            Some(Token::FloatLiteral(val)) => { self.Advance(); Ok(Expression::LiteralFloat(val)) }
            Some(Token::StringLiteral(val)) => { self.Advance(); Ok(Expression::LiteralString(val)) }
            Some(Token::Ident(name)) => {
                self.Advance();
                let mut Expr = Expression::Variable(name.clone());
                while let Some(Token::Dot) = self.Peek() {
                    self.Advance();
                    let member = match self.Advance() {
                        Some(Token::Ident(m)) => m.clone(),
                        _ => return Err("Expected member name after '.'".to_string()),
                    };
                    Expr = Expression::Variable(format!("{}.{}", name, member));
                }
                if self.Check(Token::OpenParen) {
                    self.Advance();
                    let mut Args = Vec::new();
                    if !self.Check(Token::CloseParen) {
                        loop {
                            Args.push(self.ParseExpression()?);
                            if !self.Match(Token::Comma) { break; }
                        }
                    }
                    self.Consume(Token::CloseParen, "Expected ')' after arguments")?;
                    let Name = if let Expression::Variable(n) = Expr { n } else { format!("{:?}", Expr) };
                    Ok(Expression::FunctionCall { Name, Args })
                } else {
                    Ok(Expr)
                }
            }
            Some(Token::BitPreciseType { Kind, Bits }) => {
                println!("Parsing bit-precise integer type: Kind={}, Bits={}", Kind, Bits);
                    
                self.Advance();
                if (Kind == 'i') {
                    Ok(Expression::LiteralInt(<i64>::from(Bits)))
                } else {
                    Ok(Expression::BitPreciseType { Kind, Bits })
                }
            }
            Some(Token::Star) => {
                self.Advance();
                let Target = self.ParsePrimary()?;
                Ok(Expression::UnaryOp { Op: "*".to_string(), Target: Box::new(Target) })
            }
            Some(Token::OpenParen) => {
                self.Advance();
                let Expr = self.ParseExpression()?;
                self.Consume(Token::CloseParen, "Expected ')'")?;
                Ok(Expr)
            }
            _ => Err(format!("Unexpected token in expression: {:?}", Tok)),
        }
    }

    fn ParseType(&mut self) -> Result<Type, String> {
        let mut BaseType = match self.Peek() {
            Some(Token::Star) => {
                self.Advance();
                let Inner = self.ParseType()?;
                Type::Pointer(Box::new(Inner))
            }
            Some(Token::BitPreciseType { Kind, Bits }) => {
                let k = *Kind;
                let b = *Bits;
                self.Advance();
                Type::BitPrecise(k, b)
            }
            Some(Token::Ident(name)) => {
                let n = name.clone();
                self.Advance();
                if self.Match(Token::Star) {
                    Type::Pointer(Box::new(Type::Named(n)))
                } else {
                    Type::Named(n)
                }
            }
            _ => return Err(format!("Expected type. Found {:?}", self.Peek())),
        };

        if self.Match(Token::Where) {
            let ConstraintExpr = self.ParseExpression()?;
            BaseType = Type::Where(Box::new(BaseType), Box::new(ConstraintExpr));
        }

        Ok(BaseType)
    }

    fn Peek(&self) -> Option<&Token> {
        self.Tokens.get(self.Position)
    }

    fn PeekAt(&self, Offset: usize) -> Option<&Token> {
        self.Tokens.get(self.Position + Offset)
    }

    fn Advance(&mut self) -> Option<&Token> {
        if !self.IsAtEnd() {
            self.Position += 1;
        }
        self.Tokens.get(self.Position - 1)
    }

    fn Check(&self, Target: Token) -> bool {
        self.Peek() == Some(&Target)
    }

    fn Match(&mut self, Target: Token) -> bool {
        if self.Check(Target) {
            self.Advance();
            true
        } else {
            false
        }
    }

    fn IsAtEnd(&self) -> bool {
        self.Position >= self.Tokens.len()
    }

    fn Consume(&mut self, Target: Token, Message: &str) -> Result<(), String> {
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
        if let LayerKind::Function { Name, .. } = &ast.Children[0].Kind {
            assert_eq!(Name, "test");
        } else {
            panic!("Expected function layer");
        }
    }

    #[test]
    fn test_parse_refinement() {
        let code = "fn proc(x: u32 where x > 0) {}";
        let tokens: Vec<Token> = Lexer::New(code).collect();
        let mut parser = Parser::New(tokens);
        let ast = parser.Parse().unwrap();

        assert_eq!(ast.Children.len(), 1);
    }

    #[test]
    fn test_parse_struct() {
        let code = "struct Point { x: i32, y: i32 }";
        let tokens: Vec<Token> = Lexer::New(code).collect();
        let mut parser = Parser::New(tokens);
        let ast = parser.Parse().unwrap();

        if let LayerKind::Struct { Name, Fields, .. } = &ast.Children[0].Kind {
            assert_eq!(Name, "Point");
            assert_eq!(Fields.len(), 2);
        } else {
            panic!("Expected struct layer");
        }
    }
}
