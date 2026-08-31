//! Ring 2 · Elaboration · **Layer 3 (Refinements) — Constraint representation**
//!
//! In-memory shape of the propositions we track through the tree. Two flavors:
//!   - **Assumption** — a predicate we know to be true at this program point
//!     (e.g. a caller-guaranteed `where` clause on a parameter).
//!   - **Goal** — a predicate we're trying to *prove impossible* to justify
//!     erasing a runtime check. (`unsat` means "safe".)
//!
//! Goals carry a `GoalId` so Layer 4's verdicts can be looked up and Layer 5
//! knows which checks to erase.
//!
//! Requires: [`ast::Expression`], [`ast::SourceLocation`].

use ast::{Expression, SourceLocation};

/// A stable identifier for a proof obligation, assigned as the propagator
/// discovers them. Layer 4 populates `HashMap<GoalId, SolverVerdict>` and
/// Layer 5 reads that map to decide what to erase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GoalId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub enum RefinementConstraint {
    /// Something the caller guaranteed / a branch condition we're inside.
    Assumption {
        Predicate: Expression,
        At: SourceLocation,
    },
    /// Something we want proved impossible.
    Goal {
        Id: GoalId,
        Predicate: Expression,
        At: SourceLocation,
    },
}

/// A collected set of assumptions + a single goal — the shape Layer 4 wants
/// as input to a solver query.
#[derive(Debug, Clone)]
pub struct ProofObligation {
    pub Id: GoalId,
    pub Assumptions: Vec<Expression>,
    pub Goal: Expression,
    pub At: SourceLocation,
}
