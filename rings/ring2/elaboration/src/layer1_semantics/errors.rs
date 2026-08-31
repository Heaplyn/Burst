//! Ring 2 · Elaboration · **Layer 1 (Semantics) — errors**
//!
//! Requires: [`ast::SourceLocation`].

use ast::SourceLocation;

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticError {
    /// A name was used that no in-scope declaration matches.
    UndefinedName { Name: String, At: SourceLocation },
    /// Two declarations in the same scope share a name.
    DuplicateName {
        Name: String,
        First: SourceLocation,
        Second: SourceLocation,
    },
    /// A shadowing attempt was rejected by policy.
    ShadowingDenied {
        Name: String,
        Outer: SourceLocation,
        Inner: SourceLocation,
    },
}
