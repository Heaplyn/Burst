#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use ast::statement;

pub struct elaboration_context {
    pub constraints: Vec<String>,
}

impl elaboration_context {
    pub fn new() -> Self {
        Self { constraints: Vec::new() }
    }

    pub fn elaborate_statement(&mut self, stmt: &statement) -> Result<(), String> {
        match stmt {
            statement::panic => {
                self.constraints.push("panic_state".to_string());
                Ok(())
            }
            statement::unreachable => {
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
        let mut ctx = elaboration_context::new();
        ctx.elaborate_statement(&statement::panic).unwrap();
        assert_eq!(ctx.constraints, vec!["panic_state".to_string()]);
    }
}
