//! Translation of LayerScript expressions into prefix SMT-LIB strings.

use ast::Expression;

use crate::context::ElaborationContext;

impl ElaborationContext {
    /// Turns an expression into prefix SMT strings.
    pub fn TranslateToSmt(&mut self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::LiteralInt(val) => Ok(val.to_string()),
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
                    "as" => "as",
                    "&&" => "and",
                    "||" => "or",
                    _ => return Err(format!("Unsupported SMT operator: {}", Op)),
                };
                Ok(format!("({} {} {})", smt_op, l, r))
            }
            Expression::FunctionCall { Name, Args } => {
                let mut args_str = Vec::new();
                for a in Args {
                    args_str.push(self.TranslateToSmt(a)?);
                }
                Ok(format!("({} {})", Name, args_str.join(" ")))
            }
            _ => Ok(format!("{:?}", expr)),
        }
    }
}
