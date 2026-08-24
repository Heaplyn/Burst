use ast::Stmt;


pub struct ElaborationContext {
    pub constraints: Vec<String>,
}

impl ElaborationContext {
    pub fn new() -> Self {
        Self { constraints: Vec::new() }
    }

    pub fn elaborate_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Panic => {
                self.constraints.push("panic_state".to_string());
                Ok(())
            }
            Stmt::Unreachable => {
                self.constraints.push("unreachable_state".to_string());
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elaboration_basic() {
        let mut ctx = ElaborationContext::new();
        ctx.elaborate_statement(&Stmt::Panic).unwrap();
        assert_eq!(ctx.constraints, vec!["panic_state".to_string()]);
    }
}
