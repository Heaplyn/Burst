//! Ring 2 · Elaboration · **Layer 1 — Semantics**
//!
//! Purpose: figure out *what each name means*. After Layer 1, every
//! declaration in the program has a unique [`SymbolId`] in a shared
//! [`SymbolTable`], and every variable/function reference has been checked
//! against the scope stack.
//!
//! Requires: `ast` only (this is the first pass over the parsed tree).
//! Feeds: [`layer2_types`](crate::layer2_types) (types are attached to
//! symbols, not to names).
//!
//! ## Gameplan
//! - [x] `Symbol` + `SymbolTable`    → [`symbol`]
//! - [x] `Scope` + `ScopeStack`      → [`scope`]
//! - [x] Name resolution walker      → [`resolve::Resolve`]
//! - [x] Shadowing policy            → [`shadowing`]
//! - [x] Undefined-name errors       → [`errors::SemanticError::UndefinedName`]
//! - [x] Duplicate-name errors       → [`errors::SemanticError::DuplicateName`]

pub mod errors;
pub mod resolve;
pub mod scope;
pub mod shadowing;
pub mod symbol;

pub use errors::SemanticError;
pub use resolve::{Resolve, ResolvedProgram};
pub use scope::{Scope, ScopeEntry, ScopeStack};
pub use shadowing::{IsShadowingAllowed, ShadowDecision};
pub use symbol::{Symbol, SymbolId, SymbolKind, SymbolTable};
