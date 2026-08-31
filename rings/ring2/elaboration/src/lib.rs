#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! Ring 2 · **Elaboration crate hub**
//!
//! The elaboration crate turns the raw layer tree from Ring 1 into a verified,
//! optimizable program. Work is organized into **five layers**, each with its
//! own module. Every layer's `mod.rs` names what it *requires* and what it
//! *feeds*, so the dependency direction is obvious from the tree:
//!
//! ```text
//! layer1_semantics   (needs: ast)
//!         ↓
//! layer2_types       (needs: layer1)
//!         ↓
//! layer3_refinements (needs: layer2)
//!         ↓
//! layer4_smt         (needs: layer3)      ← from-scratch solver, no Z3
//!         ↓
//! layer5_optimize    (needs: layer4)
//! ```
//!
//! [`pipeline::RunAll`] runs all five in order.

pub mod context;

pub mod layer1_semantics;
pub mod layer2_types;
pub mod layer3_refinements;
pub mod layer4_smt;
pub mod layer5_optimize;

pub mod pipeline;

pub use context::ElaborationContext;
pub use pipeline::{Elaborated, RunAll};
