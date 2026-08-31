//! Ring 0 · AST · **Type**
//!
//! The `Type` enum — every static/structural type LayerScript understands.
//! Requires: nothing (except `Expression`, which is defined in a sibling module
//! and is required only because refinements (`where …`) carry an expression).

use crate::expr::Expression;

/// The actual types for LayerScript.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Bit-precise stuff like i32 or b8.
    BitPrecise(char, u32),
    /// Names for types we made ourselves.
    Named(String),
    /// Pointers for bare-metal stuff.
    Pointer(Box<Type>),
    /// Arrays for holding a bunch of things.
    Array(Box<Type>, usize),
    /// Refinements for SMT checks like `x < 10`.
    ///
    /// Third field is an optional **`else` fallback**: if the predicate
    /// evaluates to `false` at runtime for the bound value, the fallback
    /// expression is evaluated and used instead of failing. `None` means
    /// "no fallback — a violation is an error".
    ///
    /// Syntax:
    ///   `val: u32 where val >= 10 && val <= 1000`         → fallback = None
    ///   `val: u32 where val >= 10 && val <= 1000 else 0`  → fallback = Some(0)
    Where(Box<Type>, Box<Expression>, Option<Box<Expression>>),
    /// Unit type — no value.
    Unit,
    /// No explicit annotation; Ring 2 infers it from the initializer.
    Inferred,
    /// References for borrowing values.
    Reference(Box<Type>),
    /// Null value (placeholder — prefer `Option`/refinements).
    Null,
}
