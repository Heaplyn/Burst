//! Ring 2 · Elaboration · **Layer 3 — Refinements**
//!
//! Purpose: extract every `where` predicate + every safety goal from the
//! Typed AST, propagate assumptions through branches, discharge trivial ones
//! locally, and hand the rest to Layer 4.
//!
//! Requires: [`layer2_types`](crate::layer2_types).
//! Feeds:    [`layer4_smt`](crate::layer4_smt).
//!
//! ## Gameplan
//! - [x] Constraint representation → [`constraint::RefinementConstraint`]
//! - [x] Constraint propagation    → [`propagate::Propagate`]
//! - [x] Branch Constraints_        → [`branch::AssumptionStack`]
//! - [x] Basic local proof rules   → [`proof::TryLocal`]

pub mod branch;
pub mod constraint;
pub mod proof;
pub mod propagate;

pub use branch::AssumptionStack;
pub use constraint::{GoalId, ProofObligation, RefinementConstraint};
pub use proof::TryLocal;
pub use propagate::{ConstraintGraph, Propagate};
