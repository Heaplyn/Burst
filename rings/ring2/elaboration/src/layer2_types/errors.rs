//! Ring 2 · Elaboration · **Layer 2 (Types) — errors**
//!
//! Requires: [`ast::Type`], [`ast::SourceLocation`].

use ast::{SourceLocation, Type};

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    /// Types were expected to match but didn't (assignment, argument, return).
    Mismatch {
        Expected: Type,
        Found: Type,
        At: SourceLocation,
    },
    /// An operator was applied to operands it doesn't support.
    BadOperator {
        Op: String,
        Lhs: Type,
        Rhs: Type,
        At: SourceLocation,
    },
    /// A function was called with the wrong number of arguments.
    ArityMismatch {
        Name: String,
        Expected: usize,
        Found: usize,
        At: SourceLocation,
    },
    /// A required check isn't implemented yet.
    NotImplemented(String),
}
