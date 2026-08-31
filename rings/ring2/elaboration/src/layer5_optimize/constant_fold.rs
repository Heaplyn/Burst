//! Ring 2 · Elaboration · **Layer 5 (Optimization) — constant folding**
//!
//! Recursively evaluate the constant sub-expressions of the tree — anything
//! whose leaves are all literals collapses to a single literal.

use ast::{Expression, Layer, LayerKind};

/// Rewrite `Root` — return a new tree with constant sub-expressions folded.
pub fn Fold(Root: &Layer) -> Layer {
    let mut node = Root.clone();
    FoldLayer(&mut node);
    node
}

fn FoldLayer(L: &mut Layer) {
    match &mut L.Kind {
        LayerKind::Expression(e) => *e = FoldExpr(e.clone()),
        LayerKind::Assignment { Target, Value } => {
            *Target = FoldExpr(Target.clone());
            *Value = FoldExpr(Value.clone());
        }
        LayerKind::VariableBinding { InitialValue, .. } => {
            if let Some(v) = InitialValue {
                *v = FoldExpr(v.clone());
            }
        }
        LayerKind::Return { Value } => {
            if let Some(v) = Value {
                *v = FoldExpr(v.clone());
            }
        }
        LayerKind::Conditional { Condition, .. } => {
            *Condition = FoldExpr(Condition.clone());
        }
        _ => {}
    }
    for c in &mut L.Children {
        FoldLayer(c);
    }
}

/// Fold one expression. Any surviving `BinaryOp`/`UnaryOp` with all-literal
/// operands becomes a literal.
pub fn FoldExpr(e: Expression) -> Expression {
    match e {
        Expression::BinaryOp { Op, Lhs, Rhs } => {
            let l = FoldExpr(*Lhs);
            let r = FoldExpr(*Rhs);
            FoldBinary(&Op, l, r)
        }
        Expression::UnaryOp { Op, Target } => {
            let t = FoldExpr(*Target);
            FoldUnary(&Op, t)
        }
        Expression::FunctionCall { Name, Args } => Expression::FunctionCall {
            Name,
            Args: Args.into_iter().map(FoldExpr).collect(),
        },
        other => other,
    }
}

fn FoldBinary(Op: &str, L: Expression, R: Expression) -> Expression {
    // Integer arithmetic.
    if let (Expression::LiteralInt(a), Expression::LiteralInt(b)) = (&L, &R) {
        let r = match Op {
            "+" => Some(a.wrapping_add(*b)),
            "-" => Some(a.wrapping_sub(*b)),
            "*" => Some(a.wrapping_mul(*b)),
            "/" if *b != 0 => Some(a.wrapping_div(*b)),
            "%" if *b != 0 => Some(a.wrapping_rem(*b)),
            _ => None,
        };
        if let Some(v) = r {
            return Expression::LiteralInt(v);
        }
        // Integer comparisons.
        let b = match Op {
            "==" => Some(a == b),
            "!=" => Some(a != b),
            "<" => Some(a < b),
            "<=" => Some(a <= b),
            ">" => Some(a > b),
            ">=" => Some(a >= b),
            _ => None,
        };
        if let Some(v) = b {
            return Expression::LiteralBool(v);
        }
    }
    // Boolean short-circuiting.
    if Op == "&&" {
        if let Expression::LiteralBool(false) = L {
            return Expression::LiteralBool(false);
        }
        if let Expression::LiteralBool(true) = L {
            return R;
        }
    }
    if Op == "||" {
        if let Expression::LiteralBool(true) = L {
            return Expression::LiteralBool(true);
        }
        if let Expression::LiteralBool(false) = L {
            return R;
        }
    }
    Expression::BinaryOp {
        Op: Op.to_string(),
        Lhs: Box::new(L),
        Rhs: Box::new(R),
    }
}

fn FoldUnary(Op: &str, T: Expression) -> Expression {
    match (Op, &T) {
        ("-", Expression::LiteralInt(n)) => Expression::LiteralInt(n.wrapping_neg()),
        ("!", Expression::LiteralBool(b)) => Expression::LiteralBool(!*b),
        _ => Expression::UnaryOp {
            Op: Op.to_string(),
            Target: Box::new(T),
        },
    }
}
