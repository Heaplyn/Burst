//! Ring 2 · Elaboration · **Layer 4 (Solver) — proof cache**
//!
//! Optimization pipelines re-ask the solver about the same `(assumptions,
//! goal)` pair many times. The cache keys on a stable hash of the pair so a
//! second identical query is O(1).
//!
//! Requires: [`backend::SolverVerdict`](super::backend::SolverVerdict).

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ast::Expression;

use super::backend::SolverVerdict;

#[derive(Debug, Default, Clone)]
pub struct ProofCache {
    entries: HashMap<u64, SolverVerdict>,
}

impl ProofCache {
    pub fn New() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Hash `(assumptions, goal)` into a lookup key.
    ///
    /// We use `Debug`-formatted strings under the hood — cheap and stable
    /// enough for a first pass. A future upgrade would derive `Hash` on
    /// `Expression` directly and skip the string round-trip.
    fn Key(assumptions: &[Expression], goal: &Expression) -> u64 {
        let mut h = DefaultHasher::new();
        format!("{:?}", goal).hash(&mut h);
        for a in assumptions {
            format!("{:?}", a).hash(&mut h);
        }
        h.finish()
    }

    pub fn Get(&self, assumptions: &[Expression], goal: &Expression) -> Option<&SolverVerdict> {
        self.entries.get(&Self::Key(assumptions, goal))
    }

    pub fn Put(&mut self, assumptions: &[Expression], goal: &Expression, verdict: SolverVerdict) {
        self.entries.insert(Self::Key(assumptions, goal), verdict);
    }
}
