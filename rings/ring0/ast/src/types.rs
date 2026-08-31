//! Ring 0 · AST · **types module hub**
//!
//! Kept as a single import surface (`ast::*` still resolves everything) while
//! the actual definitions live in topic files:
//!
//! - [`ty`]         — the `Type` enum
//! - [`expr`]       — the `Expression` enum
//! - [`decl`]       — `Param`, `StructField`, `EnumVariant`, `GenericParam`
//! - [`pattern`]    — `Pattern`
//! - [`hook`]       — `VariableHook`, `HookKind`
//! - [`storage`]    — `TypeStorage`, `VariableStorage`, `TypeDefinition`, `TypeKind`, `VariableDefinition`
//! - [`source`]     — `SourceLocation`
//! - [`metadata`]   — `MetadataValue`, `Directive`, `OptimizationHints`, `ObservabilityFlags`, …
//! - [`constraint`] — `Constraint`
//! - [`trace`]      — `TraceInfo`, `TraceContext`
//!
//! The `pub use` re-exports below preserve `use ast::*;` compatibility.

pub use crate::constraint::*;
pub use crate::decl::*;
pub use crate::expr::*;
pub use crate::hook::*;
pub use crate::metadata::*;
pub use crate::pattern::*;
pub use crate::source::*;
pub use crate::storage::*;
pub use crate::trace::*;
pub use crate::ty::*;
