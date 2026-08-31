//! Ring 2 · Elaboration · **Pipeline orchestrator**
//!
//! Runs the five elaboration layers in order and returns the fully-elaborated
//! program (the input tree, rewritten by Layer 5).
//!
//! ```text
//! parser AST
//!    │
//!    ▼
//! Layer 1: Resolve             — name → symbol
//!    │
//!    ▼
//! Layer 2: CheckProgram        — type each expression
//!    │
//!    ▼
//! Layer 3: Propagate           — collect refinement obligations
//!    │
//!    ▼
//! Layer 4: Query (solver)      — sat / unsat / unknown per obligation
//!    │
//!    ▼
//! Layer 5: Fold + RemoveDead   — erase what we proved unnecessary
//!    │
//!    ▼
//! Optimized AST → code_runner
//! ```

use ast::Layer;

use crate::layer1_semantics::{Resolve, ResolvedProgram};
use crate::layer2_types::{CheckProgram, TypedProgram};
use crate::layer3_refinements::{ConstraintGraph, Propagate};
use crate::layer4_smt::{Query, SolverVerdict};
use crate::layer5_optimize::{EraseProvenSafe, ErasureSet, Fold, RemoveDeadBranches};

/// The full elaboration output: the rewritten tree plus every diagnostic
/// each layer Produced.
#[derive(Debug)]
pub struct Elaborated {
    pub Program: Layer,
    pub Resolved: ResolvedProgram,
    pub Typed: TypedProgram,
    pub Constraints: ConstraintGraph,
    pub Erasures: ErasureSet,
}

/// Public entry point.
///
/// Returns an `Elaborated` even on internal errors — the caller inspects the
/// error vectors on `Resolved.Errors` / `Typed.Errors` and decides how to
/// surface them (fatal in `@strict`, warning otherwise).
pub fn RunAll(Root: &Layer) -> Elaborated {
    // Layer 1: names.
    let Resolved = Resolve(Root);

    // Layer 2: types.
    let Typed = CheckProgram(Root, &Resolved);

    // Layer 3: refinements.
    let Constraints_ = Propagate(Root);

    // Layer 4 is invoked inside Layer 5 (once per obligation) via the
    // solver's `Query`. `EraseProvenSafe` batches that up for us.
    let Erasures = EraseProvenSafe(&Constraints_);

    // Layer 5: optimize the tree.
    let Folded = Fold(Root);
    let Pruned = RemoveDeadBranches(&Folded);

    Elaborated {
        Program: Pruned,
        Resolved: Resolved,
        Typed: Typed,
        Constraints: Constraints_,
        Erasures: Erasures,
    }
}

// Bring the solver into scope so downstream users of the pipeline module
// have direct access to a single query without pulling from another submodule.
pub use crate::layer4_smt::SolverVerdict as Verdict;

pub fn SolveOnce(assumptions: &[ast::Expression], goal: &ast::Expression) -> SolverVerdict {
    Query(assumptions, goal)
}
