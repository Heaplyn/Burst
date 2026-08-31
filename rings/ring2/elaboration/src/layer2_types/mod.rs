//! Ring 2 · Elaboration · **Layer 2 — Types**
//!
//! Purpose: assign a concrete `Type` to every value-producing layer, then
//! check every operator / assignment / call / return is used at a compatible
//! type.
//!
//! Requires: [`layer1_semantics`](crate::layer1_semantics) (a resolved symbol
//! table so `Variable("x")` has a declared type to look up).
//! Feeds: [`layer3_refinements`](crate::layer3_refinements) (refinements need
//! typed expressions to attach to).
//!
//! ## Gameplan
//! - [x] Expression type inference → [`infer::InferExpression`]
//! - [x] Operator checking         → [`check::CheckBinaryOp`]
//! - [x] Assignment checking       → [`check::CheckAssignment`]
//! - [x] Function-call checking    → [`check::CheckFunctionCall`]
//! - [x] Return checking           → [`check::CheckReturn`]
//! - [x] Type errors               → [`errors::TypeError`]
//! - [x] Typed AST (side table)    → [`typed_ast::TypeTable`], walker → [`walk::CheckProgram`]

pub mod check;
pub mod errors;
pub mod infer;
pub mod typed_ast;
pub mod walk;

pub use check::{CheckAssignment, CheckBinaryOp, CheckFunctionCall, CheckReturn};
pub use errors::TypeError;
pub use infer::InferExpression;
pub use typed_ast::{TypeEnv, TypeTable, TypedExpr};
pub use walk::{CheckProgram, TypedProgram};
