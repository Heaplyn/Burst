#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use ast::{Layer, LayerKind, Expression, Type};
use std::collections::HashSet;

/// the context for analyzing layers and constraints
pub struct ElaborationContext {
    /// collected smt assertions
    pub Constraints: Vec<String>,
    /// variables we know about in this scope
    pub KnownVars: HashSet<String>,
}

impl ElaborationContext {
    /// starts a fresh context
    pub fn New() -> Self {
        Self {
            Constraints: Vec::new(),
            KnownVars: HashSet::new(),
        }
    }

    /// walks the layer tree and finds constraints
    pub fn ElaborateLayer(&mut self, L: &Layer) -> Result<(), String> {
        match &L.Kind {
            LayerKind::Function { Name, Params, .. } => {
                println!("Elaborating function: {}", Name);
                for p in Params {
                    self.KnownVars.insert(p.Name.clone());
                    if let Type::Where(_base, constraint) = &p.Type_ {
                        let smt = self.TranslateToSmt(constraint)?;
                        self.Constraints.push(format!("(assert {})", smt));
                    }
                }
            }
            LayerKind::VariableBinding { Name, TypeAnnotation, .. } => {
                self.KnownVars.insert(Name.clone());
                if let Some(Type::Where(_, constraint)) = TypeAnnotation {
                    let smt = self.TranslateToSmt(constraint)?;
                    self.Constraints.push(format!("(assert {})", smt));
                }
            }
            LayerKind::Panic => {
                println!("🔍 SMT Goal: Is 'panic' unreachable?");
            }
            LayerKind::Conditional { Condition, .. } => {
                let smt = self.TranslateToSmt(Condition)?;
                println!("   Branch Constraint: {}", smt);
            }
            _ => {}
        }

        for Child in &L.Children {
            self.ElaborateLayer(Child)?;
        }
        Ok(())
    }

    /// turns an expression into prefix smt strings
    pub fn TranslateToSmt(&mut self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::LiteralInt(val) => Ok(val.to_string()),
            Expression::Variable(name) => Ok(name.clone()),
            Expression::BinaryOp { Op, Lhs, Rhs } => {
                let l = self.TranslateToSmt(Lhs)?;
                let r = self.TranslateToSmt(Rhs)?;
                let smt_op = match Op.as_str() {
                    "+" => "+", "-" => "-", "*" => "*", "/" => "/", "==" => "=", "<" => "<", ">" => ">", "<=" => "<=", ">=" => ">=", "as" => "as",
                    _ => return Err(format!("Unsupported SMT operator: {}", Op)),
                };
                Ok(format!("({} {} {})", smt_op, l, r))
            }
            Expression::FunctionCall { Name, Args } => {
                 let mut args_str = Vec::new();
                 for a in Args { args_str.push(self.TranslateToSmt(a)?); }
                 Ok(format!("({} {})", Name, args_str.join(" ")))
            }
            _ => Ok(format!("{:?}", expr)),
        }
    }
}
