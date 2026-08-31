//! Ring 2 · Elaboration · **Layer 5 — Optimization**
//!
//! Purpose: use the verdicts from Layer 4 to shrink the program — erase
//! safety checks that can never fail, delete branches that can never be
//! taken, fold constants, drop unobservable code.
//!
//! Requires: [`layer4_smt`](crate::layer4_smt) (verdicts),
//! [`layer2_types`](crate::layer2_types) (typed tree for folding),
//! [`layer3_refinements`](crate::layer3_refinements) (obligations).
//! Feeds:    code_runner / future codegen.
//!
//! ## Gameplan
//! - [x] Remove proven-impossible branches → [`dead_branch::RemoveDeadBranches`]
//! - [x] Remove proven safety checks       → [`safety_erasure::EraseProvenSafe`]
//! - [x] Constant folding                  → [`constant_fold::Fold`]
//! - [x] Dead-code elimination             → [`dce::Eliminate`]

pub mod constant_fold;
pub mod dce;
pub mod dead_branch;
pub mod safety_erasure;

pub use constant_fold::{Fold, FoldExpr};
pub use dce::Eliminate;
pub use dead_branch::RemoveDeadBranches;
pub use safety_erasure::{ErasureSet, EraseProvenSafe};
