#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! Elaboration (Ring 2): walks the layer tree, collects refinement constraints,
//! and lowers expressions to SMT-LIB.
//!
//! - [`context`] — `ElaborationContext` + the tree walk
//! - [`smt`]     — expression → SMT-LIB translation

pub mod context;
pub mod smt;

pub use context::ElaborationContext;
