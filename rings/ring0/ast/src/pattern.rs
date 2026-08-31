//! Ring 0 · AST · **Pattern**
//!
//! Patterns used by `match` arms and by pattern-binding `let`s.
//! Requires: [`Expression`](crate::expr::Expression).

use crate::expr::Expression;

/// Pattern matching variants.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Matches anything (`_`).
    Wildcard,
    /// Matches a specific value.
    Literal(Expression),
    /// Matches and binds to a name.
    Variable(String, Expression),
    /// Matches an enum variant.
    Variant(String, Option<Box<Pattern>>),
}
