#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use ast::{Layer, LayerKind};

pub struct ElaborationContext {
    pub Constraints: Vec<String>,
}

impl ElaborationContext {
    pub fn New() -> Self {
        Self { Constraints: Vec::new() }
    }

    pub fn ElaborateLayer(&mut self, L: &Layer) -> Result<(), String> {
        // 1. Process local layer kind
        match &L.Kind {
            LayerKind::Panic => {
                self.Constraints.push("panic_state".to_string());
            }
            LayerKind::Unreachable => {
                self.Constraints.push("unreachable_state".to_string());
            }
            LayerKind::Function { Name, .. } => {
                println!("Elaborating function: {}", Name);
            }
            _ => {}
        }

        // 2. Recursively elaborate children
        for Child in &L.Children {
            self.ElaborateLayer(Child)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::{LayerBuilder, SourceLocation};

    #[test]
    fn test_elaboration_basic() {
        let mut Ctx = ElaborationContext::New();

        let PanicLayer = LayerBuilder::New(LayerKind::Panic, SourceLocation::Builtin()).Build();
        Ctx.ElaborateLayer(&PanicLayer).unwrap();
        assert_eq!(Ctx.Constraints, vec!["panic_state".to_string()]);
        
        let UnreachableLayer = LayerBuilder::New(LayerKind::Unreachable, SourceLocation::Builtin()).Build();
        Ctx.ElaborateLayer(&UnreachableLayer).unwrap();
        assert_eq!(Ctx.Constraints, vec!["panic_state".to_string(), "unreachable_state".to_string()]);
    }
}
