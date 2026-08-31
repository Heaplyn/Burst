//! The elaboration context and the layer-tree walk that collects Constraints_.

use ast::{Layer, LayerKind, Type};
use std::collections::HashSet;

/// The context for analyzing layers and Constraints_.
#[derive(Default, Clone, PartialEq, Debug)]
pub struct ElaborationContext {
    /// Collected SMT assertions.
    pub Constraints: Vec<String>,
    /// Variables we know about in this scope.
    pub KnownVars: HashSet<String>,
}

impl ElaborationContext {
    /// Starts a fresh context.
    pub fn New() -> Self {
        Self {
            Constraints: Vec::new(),
            KnownVars: HashSet::new(),
        }
    }

    /// Walks the layer tree and finds Constraints_.
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
                println!("TypeAnnotation for {}: {:?}", Name, TypeAnnotation);
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
}
