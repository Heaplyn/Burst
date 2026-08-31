//! Ring 2 · Elaboration · **Layer 4 (SMT) — SMT-LIB serialization**
//!
//! Keeps the *legacy* `TranslateToSmt` for external tools that want a stringy
//! SMT-LIB view of a predicate (e.g. dropping into a real Z3 later, or for
//! debug output). The solver itself doesn't use this — it works over the
//! typed [`Prop`](super::normalize::Prop) representation directly.
//!
//! Requires: [`ast::Expression`], [`crate::context::ElaborationContext`].

use ast::Expression;

use crate::context::ElaborationContext;

impl ElaborationContext {
    /// Turn an expression into a prefix SMT-LIB string.
    pub fn TranslateToSmt(&mut self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::LiteralInt(val) => Ok(val.to_string()),
            Expression::LiteralBool(true) => Ok("true".to_string()),
            Expression::LiteralBool(false) => Ok("false".to_string()),
            Expression::Variable(name) => Ok(name.clone()),
            Expression::BinaryOp { Op, Lhs, Rhs } => {
                let l = self.TranslateToSmt(Lhs)?;
                let r = self.TranslateToSmt(Rhs)?;
                let smt_op = match Op.as_str() {
                    "+" => "+",
                    "-" => "-",
                    "*" => "*",
                    "/" => "/",
                    "==" => "=",
                    "!=" => "distinct",
                    "<" => "<",
                    ">" => ">",
                    "<=" => "<=",
                    ">=" => ">=",
                    "&&" => "and",
                    "||" => "or",
                    _ => return Err(format!("Unsupported SMT operator: {}", Op)),
                };
                Ok(format!("({} {} {})", smt_op, l, r))
            }
            Expression::UnaryOp { Op, Target } if Op == "!" => {
                Ok(format!("(not {})", self.TranslateToSmt(Target)?))
            }
            Expression::FunctionCall { Name, Args } => {
                let mut parts = Vec::new();
                for a in Args {
                    parts.push(self.TranslateToSmt(a)?);
                }
                Ok(format!("({} {})", Name, parts.join(" ")))
            }
            _ => Ok(format!("{:?}", expr)),
        }
    }
}
