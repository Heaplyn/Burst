//! Ring 2 · Elaboration · **Layer 5 (Optimization) — remove proven-impossible branches**
//!
//! For each `Conditional { Condition, .. }` layer, ask the solver whether the
//! condition can hold under the current assumptions. `Unsat` → delete the
//! then-branch. Ask about the negation for the else-branch.
//!
//! Requires: [`layer4_smt::Query`](crate::layer4_smt::Query).

use ast::{Expression, Layer, LayerKind};

use crate::layer4_smt::{Query, SolverVerdict};

/// Rewrite `Root` — return a new tree with dead branches Pruned.
pub fn RemoveDeadBranches(Root: &Layer) -> Layer {
    let mut node = Root.clone();
    Walk(&mut node, &mut Vec::new());
    node
}

fn Walk(L: &mut Layer, assumptions: &mut Vec<Expression>) {
    // If this is a conditional, decide each arm's fate.
    if let LayerKind::Conditional { Condition, .. } = &L.Kind.clone() {
        let mut kept: Vec<Layer> = Vec::new();

        // Then-arm: alive if `Condition` is not Unsat.
        if let Some(ThenArm) = L.Children.get(0) {
            let CanTake = matches!(
                Query(assumptions, Condition),
                SolverVerdict::Sat { .. } | SolverVerdict::Unknown
            );
            if CanTake {
                let mut arm = ThenArm.clone();
                assumptions.push(Condition.clone());
                Walk(&mut arm, assumptions);
                assumptions.pop();
                kept.push(arm);
            }
        }

        // Else-arm: alive if `!Condition` is not Unsat.
        if let Some(ElseArm) = L.Children.get(1) {
            let neg = Negate(Condition.clone());
            let CanTake = matches!(
                Query(assumptions, &neg),
                SolverVerdict::Sat { .. } | SolverVerdict::Unknown
            );
            if CanTake {
                let mut arm = ElseArm.clone();
                assumptions.push(neg);
                Walk(&mut arm, assumptions);
                assumptions.pop();
                kept.push(arm);
            }
        }

        L.Children = kept;
        return;
    }

    // Otherwise: recurse into children.
    for c in &mut L.Children {
        Walk(c, assumptions);
    }
}

fn Negate(e: Expression) -> Expression {
    Expression::UnaryOp { Op: "!".to_string(), Target: Box::new(e) }
}
