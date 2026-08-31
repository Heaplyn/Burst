//! Expression parsing: precedence climbing, atoms, and postfix operators.

use std::any::Any;

use ast::*;
use lexer::token::TokenKind;

use crate::Parser;

impl Parser {
    pub fn EvaluateActualType(&mut self, val: Expression) -> Box<dyn Any> {
        match val {
            Expression::Variable(Name) => {
                let peek_kind = self.Peek().map(|t| &t.Kind);
                if peek_kind == Some(&TokenKind::End) {
                    return Box::new("Cannot evaluate variable as boolean".to_string());
                }
                let Expr = self.GetVariable(&Name.to_string().as_str());
                match (&Expr) {
                    Expression::LiteralBool(val) => Box::new(*val),
                    Expression::LiteralInt(val) => Box::new(*val),
                    Expression::LiteralFloat(val) => Box::new(*val),
                    Expression::LiteralString(val) => Box::new(val.clone()),
                    _ => Box::new("Cannot evaluate variable as boolean".to_string()),
                }
            }
            Expression::LiteralBool(val) => Box::new(val),
            Expression::LiteralInt(val) => Box::new(val),
            Expression::LiteralFloat(val) => Box::new(val),
            Expression::LiteralString(val) => Box::new(val.clone()),
            _ => Box::new("Cannot evaluate variable as boolean".to_string()),
        }
    }

    /// Evaluates an expression as a boolean.
    /// Returns Ok(true) or Ok(false) if the expression can be evaluated as a boolean.
    /// Returns Err if the expression cannot be evaluated as a boolean.
    pub fn EvaluateExpression(&mut self, expr: &Expression) -> Result<bool, String> {
        match expr {
            Expression::LiteralInt(n) => Ok(*n != 0),
            Expression::LiteralBool(b) => Ok(*b),
            Expression::LiteralString(_) => Err("Cannot evaluate string as boolean".to_string()),
            Expression::Variable(name) => {
                let Var = self.GetVariable(name);
                let EvalResult = self.EvaluateActualType(Var);
                if let Some(&b) = EvalResult.downcast_ref::<bool>() {
                    Ok(b)
                } else {
                    Err("Cannot evaluate variable as boolean".to_string())
                }
            }
            Expression::BinaryOp { Op, Lhs, Rhs } => {
                let l = self.EvaluateExpression(Lhs);
                let r = self.EvaluateExpression(Rhs);
                match Op.as_str() {
                    "&&" => Ok(l? && r?),
                    "||" => Ok(l? || r?),
                    "==" => Ok(l? == r?),
                    "!=" => Ok(l? != r?),
                    "<" => Ok(l? < r?),
                    "<=" => Ok(l? <= r?),
                    ">" => Ok(l? > r?),
                    ">=" => Ok(l? >= r?),
                    _ => Err("Unknown binary operator".to_string()),
                }
            }
            _ => Err("Cannot evaluate expression as boolean".to_string()),
        }
    }

    /// Get variable of name n from current Context
    pub fn GetVariable(&self, Name: &str) -> Expression {
        let Vars = &self.CurrentLayer.VariableStorage.Variables;
        //println!("vars: {:?}", Vars);
        if !(Vars.contains_key(Name)) {
            return Expression::Variable("Invalid".to_string());
        }
        let Var = Vars.get(Name).unwrap_or_else(|| panic!("Variable '{}' not found", Name));
        return Var.Value.clone();
    }

    /// Entry point for expressions.
    pub fn ParseExpression(&mut self) -> Result<Expression, String> {
        self.ParseBinary(0)
    }

    /// Handles operator precedence for math and logic (precedence climbing).
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
                    TokenKind::And => "&&".to_string(),
                    TokenKind::Or => "||".to_string(),
                    TokenKind::As => "as".to_string(),
                    _ => format!("{:?}", op_tok),
                },
                Lhs: Box::new(Expr),
                Rhs: Box::new(Rhs),
            };
        }

        Ok(Expr)
    }

    /// The order-of-operations table.
    pub fn TokenPrecedence(&self, Tok: &TokenKind) -> u8 {
        match Tok {
            TokenKind::Or => 2,
            TokenKind::And => 3,
            TokenKind::Less | TokenKind::Greater | TokenKind::LessEqual | TokenKind::GreaterEqual => 4,
            TokenKind::Plus | TokenKind::Minus => 5,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => 6,
            TokenKind::As => 7,
            _ => 0,
        }
    }

    /// Parses atoms like numbers, strings, and idents.
    pub fn ParseAtom(&mut self) -> Result<Expression, String> {
        let Tok = self.Peek().cloned();
        match Tok.as_ref().map(|t| &t.Kind) {
            Some(TokenKind::IntLiteral(val)) => {
                let v = *val;
                self.Advance();
                Ok(Expression::LiteralInt(v))
            }
            Some(TokenKind::FloatLiteral(val)) => {
                let v = *val;
                self.Advance();
                Ok(Expression::LiteralFloat(v))
            }
            Some(TokenKind::StringLiteral(val)) => {
                let v = val.clone();
                self.Advance();
                Ok(Expression::LiteralString(v))
            }
            Some(TokenKind::Ident(name)) => {
                let n = name.clone();
                self.Advance();
                Ok(Expression::Variable(n))
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
                Ok(Expression::UnaryOp {
                    Op: "*".to_string(),
                    Target: Box::new(Target),
                })
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

    /// Parses primary expressions with postfix operators (`.`, `()`, `[]`).
    pub fn ParsePrimary(&mut self) -> Result<Expression, String> {
        let mut Expr = self.ParseAtom()?;

        loop {
            match self.Peek().map(|t| &t.Kind) {
                Some(TokenKind::Dot) => {
                    self.Advance();
                    let member = match self.Advance().map(|t| &t.Kind) {
                        Some(TokenKind::Ident(m)) => m.clone(),
                        _ => return Err("Expected member name after '.'".to_string()),
                    };
                    Expr = Expression::MemberAccess {
                        Target: Box::new(Expr),
                        Member: member,
                    };
                }
                Some(TokenKind::OpenParen) => {
                    self.Advance();
                    let mut Args = Vec::new();
                    if !self.Check(TokenKind::CloseParen) {
                        loop {
                            Args.push(self.ParseExpression()?);
                            if !self.Match(TokenKind::Comma) {
                                break;
                            }
                        }
                    }
                    self.Consume(TokenKind::CloseParen, "Expected ')' after arguments")?;
                    let Name = match Expr {
                        Expression::Variable(n) => n,
                        Expression::MemberAccess { Target, Member } => {
                            fn to_string(e: &Expression) -> Result<String, String> {
                                match e {
                                    Expression::Variable(n) => Ok(n.clone()),
                                    Expression::MemberAccess { Target, Member } => {
                                        Ok(format!("{}.{}", to_string(Target)?, Member))
                                    }
                                    _ => Err("Invalid function target".to_string()),
                                }
                            }
                            to_string(&Expression::MemberAccess {
                                Target: Target.clone(),
                                Member: Member.clone(),
                            })?
                        }
                        _ => return Err("Expected function name before '('".to_string()),
                    };
                    Expr = Expression::FunctionCall { Name, Args };
                }
                Some(TokenKind::OpenBracket) => {
                    self.Advance();
                    let Index = self.ParseExpression()?;
                    self.Consume(TokenKind::CloseBracket, "Expected ']' after index")?;
                    Expr = Expression::IndexAccess {
                        Target: Box::new(Expr),
                        Index: Box::new(Index),
                    };
                }
                _ => break,
            }
        }

        Ok(Expr)
    }
}
