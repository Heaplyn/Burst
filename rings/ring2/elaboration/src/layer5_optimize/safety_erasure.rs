//! Ring 2 · Elaboration · **Layer 5 (Optimization) — erase provably-safe checks**
//!
//! Walks Layer 3's `ConstraintGraph` — for every `ProofObligation` where
//! Layer 4 returned `Unsat`, mark the goal as erased. Downstream (code_runner
//! / codegen) reads the erasure set and skips emitting the check.
//!
//! Requires: [`layer3_refinements::ConstraintGraph`](crate::layer3_refinements::ConstraintGraph),
//! [`layer4_smt`](crate::layer4_smt).

use crate::layer3_refinements::{ConstraintGraph, GoalId};
use crate::layer4_smt::{Query, SolverVerdict};

/// The set of goals we've proven impossible; whoever emits code should elide
/// the corresponding runtime check.
#[derive(Debug, Default, Clone)]
pub struct ErasureSet {
    pub Erased: std::collections::HashSet<GoalId>,
}

impl ErasureSet {
    pub fn New() -> Self {
        Self::default()
    }

    pub fn Contains(&self, id: GoalId) -> bool {
        self.Erased.contains(&id)
    }
}

/// Ask the solver about every obligation. Anything `Unsat` is added to the
/// erasure set.
pub fn EraseProvenSafe(graph: &ConstraintGraph) -> ErasureSet {
    let mut set = ErasureSet::New();
    for ob in &graph.Obligations {
        if matches!(Query(&ob.Assumptions, &ob.Goal), SolverVerdict::Unsat) {
            set.Erased.insert(ob.Id);
        }
    }
    set
}
