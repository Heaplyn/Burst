//! Parsing of types and the shared "name + type" declaration form.

use ast::*;
use lexer::token::{Token, TokenKind};

use crate::Parser;

impl Parser {
    /// Handles both "name: type" and "type name", plus inferred "name = ..." (variables only).
    pub fn ParseNameAndType(&mut self, context: &str) -> Result<(String, Type), String> {
        if context == "variable" {
            if let Some(TokenKind::Ident(_)) = self.Peek().map(|t| &t.Kind) {
                let next_ends_decl = matches!(
                    self.PeekAt(1).map(|t| &t.Kind),
                    Some(TokenKind::Equal) | Some(TokenKind::Semicolon) | Some(TokenKind::OpenBrace)
                );
                if next_ends_decl {
                    let name = match self.Advance().map(|t| &t.Kind) {
                        Some(TokenKind::Ident(n)) => n.clone(),
                        _ => unreachable!(),
                    };
                    return Ok((name, Type::Inferred)); // leave `=`/`;`/`{` for the caller
                }
            }
        }

        let mut found_type_colon = false;
        if let Some(TokenKind::Ident(_)) = self.Peek().map(|t| &t.Kind) {
            if let Some(TokenKind::Colon) = self.PeekAt(1).map(|t| &t.Kind) {
                if !matches!(self.PeekAt(2).map(|t| &t.Kind), Some(TokenKind::Where)) {
                    found_type_colon = true;
                }
            }
        }
        let BeforeName = match self.Peek().map(|t| &t.Kind) {
            Some(TokenKind::Ident(n)) => n.clone(),
            Some(TokenKind::IntLiteral(n)) => n.to_string(),
            Some(TokenKind::FloatLiteral(n)) => n.to_string(),
            Some(TokenKind::StringLiteral(n)) => n.clone(),
            Some(TokenKind::BitPreciseType { Kind, Bits }) => format!("{}{}", Kind, Bits),
            Some(other) => format!("{:?}", other),
            None => "EOF".to_string(),
        };
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
                Some(TokenKind::IntLiteral(_)) => BeforeName,
                Some(TokenKind::FloatLiteral(_)) => BeforeName,
                Some(TokenKind::StringLiteral(_)) => BeforeName,
                Some(TokenKind::BitPreciseType { .. }) => BeforeName,
                _ => return Err(format!("Expected {:?} name after type. Found {:?}", t, self.Peek())),
            };
            (n, t)
        };

        let has_refinement = if self.Match(TokenKind::Where) {
            true
        } else if matches!(self.Peek().map(|t| &t.Kind), Some(TokenKind::Colon))
            && matches!(self.PeekAt(1).map(|t| &t.Kind), Some(TokenKind::Where))
        {
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
    

    /// Parses type names and pointer levels.
    pub fn ParseType(&mut self) -> Result<Type, String> {
        //println!("Parsing type: {:?}", self.Peek());
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
            Some(TokenKind::Equal) => {
                self.Advance();
                let _Left = self.ParseType()?;
                let Right = self.Peek();
                if let Some(Token { Kind: TokenKind::Ident(_), .. }) = Right {
                    println!("Self type: {:?}", self.ParseType()?);
                    return Ok(self.ParseType()?);
                } else {
                    return Err("Expected type after '='".to_string());
                }
            }
            Some(TokenKind::And) => {
                self.Advance();
                let Left = self.PeekAt(-2);
                let Inner = self.ParseType()?;
                Type::Reference(Box::new(Inner))
            }
            Some(TokenKind::Arrow) => {
                self.Advance();
                let ReturnType = self.ParseType()?;
                ReturnType
            }
            
            _ => return Err(format!("Expected type. Found {:?}\nLine:{:?}", self.Peek(), self.Peek().map(|t| t.Line))),
        };
        
        let CurrentPeek = self.PeekAt(0);
        let CurrentKind = CurrentPeek.as_ref().map(|t| &t.Kind).unwrap_or(&self.Peek().as_ref().map(|t| &t.Kind).unwrap_or(&TokenKind::End));
        
        if config::DebugMode.load(std::sync::atomic::Ordering::Relaxed) {
            println!("CurrentKind: {:?}", CurrentKind);
        }
        if CurrentKind == &TokenKind::Where {
            self.Advance(); // consume 'where'
            let ConstraintExpr = self.ParseExpression().unwrap_or(Expression::Invalid);
            //println!("ConstraintExpr: {:?}", ConstraintExpr);
            BaseType = Type::Where(Box::new(BaseType), Box::new(ConstraintExpr.clone())); 
            if self.EvaluateExpression(&ConstraintExpr) == Ok(false) {
                return Err("Unsatisfiable 'where' constraint".to_string());
            }
        }


        Ok(BaseType)
    }
}
