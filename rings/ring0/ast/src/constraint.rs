//! Ring 0 · AST · **Constraints**
//!
//! Logical rules attached to a layer: refinements, safety obligations, POMSET
//! ordering. Consumed by Ring 2 elaboration.
//! Requires: [`LayerId`](crate::LayerId).

use crate::LayerId;

/// Logical rules for SMT and POMSET.
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// Refined types like `x > 0`.
    RefinedType {
        Variable: String,
        Condition: String,
    },
    /// Making sure things are safe.
    Safety {
        Condition: String,
        ErrorMessage: String,
    },
    /// Partial ordering for parallel tasks.
    POMSET { Before: LayerId, After: LayerId },
}
