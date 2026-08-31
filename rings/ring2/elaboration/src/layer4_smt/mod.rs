//! Ring 2 · Elaboration · **Layer 4 — Solver (from-scratch, no Z3)**
//!
//! Purpose: given assumptions + a goal, decide whether the goal is
//! satisfiable. `Unsat` means the goal is impossible — the caller (Layer 5)
//! can erase the runtime check that would have guarded against it.
//!
//! Requires: [`layer3_refinements`](crate::layer3_refinements) (proof
//! obligations to solve).
//! Feeds:    [`layer5_optimize`](crate::layer5_optimize).
//!
//! ## Gameplan
//! - [x] Solver backend                → [`backend::Query`]
//! - [x] SAT / UNSAT / UNKNOWN verdict → [`backend::SolverVerdict`]
//! - [x] Proof caching                 → [`cache::ProofCache`]
//! - [x] Constraint → SMT-LIB (legacy) → [`translate`]
//!
//! ## Internal
//! - [`normalize`] — `Expression` → `Prop` / `Term` normal form
//! - [`interval`]  — abstract interval domain used by propagation

pub mod backend;
pub mod cache;
pub mod interval;
pub mod normalize;
pub mod translate;

pub use backend::{Query, SolverVerdict};
pub use cache::ProofCache;
pub use interval::{Interval, IntervalStore};
pub use normalize::{Atom, Prop, Term, ToProp, ToTerm};
