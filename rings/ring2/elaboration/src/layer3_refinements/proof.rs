//! Ring 2 · Elaboration · **Layer 3 (Refinements) — local (cheap) proof rules**
//!
//! Before we spend a full solver call on a goal, try a handful of syntactic
//! rules. When a rule fires we can skip the whole Layer 4 round trip.
//!
//! Requires: [`ast::Expression`], [`constraint::ProofObligation`](super::constraint::ProofObligation).

use ast::Expression;

use super::constraint::ProofObligation;

/// A local proof result. `Some(true)` = goal is trivially proven false
/// (unsat, safe to erase). `Some(false)` = trivially reachable (definitely
/// sat). `None` = we don't know; hand it to the SMT layer.
pub fn TryLocal(o: &ProofObligation) -> Option<bool> {
    // Rule 1: goal is a literal false → trivially unsat → proven safe.
    if let Expression::LiteralBool(false) = o.Goal {
        return Some(true);
    }
    // Rule 2: goal is a literal true and no contradictory assumptions → sat.
    if let Expression::LiteralBool(true) = o.Goal {
        // If the caller explicitly assumed `false`, everything is unsat.
        for a in &o.Assumptions {
            if matches!(a, Expression::LiteralBool(false)) {
                return Some(true);
            }
        }
        // Otherwise it's clearly reachable.
        return Some(false);
    }
    // Rule 3: goal appears verbatim as an assumption → sat (definitely reachable).
    for a in &o.Assumptions {
        if a == &o.Goal {
            return Some(false);
        }
    }
    None
}
