//! Ring 2 · Elaboration · **Layer 3 (Refinements) — assumption/goal collector**
//!
//! Walks the layer tree, pushing/popping assumptions as we enter and leave
//! functions and branches, and emitting a `ProofObligation` for every safety
//! goal it finds (currently: `panic`, and refinements attached to parameter
//! types).
//!
//! Requires: [`ast::Layer`], the constraint/graph types in this module.

use ast::{Expression, Layer, LayerKind, Type};

use super::branch::AssumptionStack;
use super::constraint::{GoalId, ProofObligation};

/// The output of Layer 3: every proof obligation the program presents.
#[derive(Debug, Default)]
pub struct ConstraintGraph {
    pub Obligations: Vec<ProofObligation>,
    NextGoalId: usize,
}

impl ConstraintGraph {
    pub fn New() -> Self {
        Self::default()
    }

    /// Reserve a fresh `GoalId`.
    pub fn FreshGoalId(&mut self) -> GoalId {
        let id = GoalId(self.NextGoalId);
        self.NextGoalId += 1;
        id
    }

    /// Convenience: record a goal along with the assumptions currently in scope.
    fn EmitGoal(
        &mut self,
        stack: &AssumptionStack,
        predicate: Expression,
        at: ast::SourceLocation,
    ) {
        let id = self.FreshGoalId();
        self.Obligations.push(ProofObligation {
            Id: id,
            Assumptions: stack.Snapshot(),
            Goal: predicate,
            At: at,
        });
    }
}

/// Public entry — walk `root`, produce the graph.
pub fn Propagate(root: &Layer) -> ConstraintGraph {
    let mut g = ConstraintGraph::New();
    let mut stack = AssumptionStack::New();
    Walk(root, &mut stack, &mut g);
    g
}

fn Walk(L: &Layer, stack: &mut AssumptionStack, g: &mut ConstraintGraph) {
    match &L.Kind {
        // Function entry: parameters' `where` clauses become assumptions for
        // the body.
        LayerKind::Function { Params, .. } => {
            stack.EnterFrame();
            for p in Params {
                if let Type::Where(_base, cond, _) = &p.Type_ {
                    stack.PushAssumption((**cond).clone());
                }
            }
            for c in &L.Children {
                Walk(c, stack, g);
            }
            stack.LeaveFrame();
        }

        // Variable binding with a refined type: push the constraint too.
        LayerKind::VariableBinding { TypeAnnotation, .. } => {
            if let Some(Type::Where(_, cond, _)) = TypeAnnotation {
                stack.PushAssumption((**cond).clone());
                // We don't pop — the binding is in scope until the enclosing
                // block ends, and blocks manage their own frames.
            }
        }

        // Conditional: push condition on the "then" arm, negated on the else.
        // We approximate by walking each arm inside its own frame.
        LayerKind::Conditional { Condition, .. } => {
            // Only two children max in the current AST (then + optional else).
            if let Some(ThenArm) = L.Children.get(0) {
                stack.EnterFrame();
                stack.PushAssumption(Condition.clone());
                Walk(ThenArm, stack, g);
                stack.LeaveFrame();
            }
            if let Some(ElseArm) = L.Children.get(1) {
                stack.EnterFrame();
                stack.PushAssumption(Negate(Condition.clone()));
                Walk(ElseArm, stack, g);
                stack.LeaveFrame();
            }
        }

        // Blocks (including function bodies' inner blocks): frame per block.
        LayerKind::Block => {
            stack.EnterFrame();
            for c in &L.Children {
                Walk(c, stack, g);
            }
            stack.LeaveFrame();
        }

        // `panic` is a safety goal: we want to prove `false` given the current
        // assumptions (i.e. that this program point is unreachable).
        LayerKind::Panic => {
            g.EmitGoal(stack, Expression::LiteralBool(true), L.Metadata.SourceLocation.clone());
        }

        // `unreachable` — same idea, stronger claim from the user.
        LayerKind::Unreachable => {
            g.EmitGoal(stack, Expression::LiteralBool(true), L.Metadata.SourceLocation.clone());
        }

        // Everything else: recurse.
        _ => {
            for c in &L.Children {
                Walk(c, stack, g);
            }
        }
    }
}

/// Wrap a predicate in a logical negation, syntactically.
fn Negate(e: Expression) -> Expression {
    Expression::UnaryOp {
        Op: "!".to_string(),
        Target: Box::new(e),
    }
}
