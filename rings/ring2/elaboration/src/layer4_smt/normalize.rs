//! Ring 2 · Elaboration · **Layer 4 (Solver) — normal-form language**
//!
//! To reason about `Expression`s we lower them into a small, restricted
//! representation the solver understands:
//!
//! - `Term` — an arithmetic expression over integer variables and constants.
//! - `Atom` — a comparison between two `Term`s.
//! - `Prop` — a boolean combination of atoms.
//!
//! This is *not* a full model of LayerScript expressions; anything we don't
//! recognize (floats, strings, calls, structs, pointers, arrays) causes
//! normalization to return `None`, which the solver treats as `Unknown`.
//!
//! Requires: [`ast::Expression`].

use ast::Expression;

/// An arithmetic term. All variables are treated as integers for now.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Term {
    Const(i64),
    Var(String),
    Add(Box<Term>, Box<Term>),
    Sub(Box<Term>, Box<Term>),
    Mul(Box<Term>, Box<Term>),
    Neg(Box<Term>),
}

/// A single atomic proposition — a comparison of two terms.
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Eq(Term, Term),
    Ne(Term, Term),
    Lt(Term, Term),
    Le(Term, Term),
    Gt(Term, Term),
    Ge(Term, Term),
}

/// A boolean formula. `Not` is push-down (De Morgan) during normalization so
/// the propagator only ever sees atoms wrapped in `And`/`Or`.
#[derive(Debug, Clone, PartialEq)]
pub enum Prop {
    True,
    False,
    Atom(Atom),
    And(Vec<Prop>),
    Or(Vec<Prop>),
    Not(Box<Prop>),
}

// ------------------------------------------------------------------
// Expression → Term / Prop lowering
// ------------------------------------------------------------------

/// Lower an `Expression` we believe to be numeric.
///
/// Returns `None` if the expression uses a feature the solver can't reason
/// about (floats, function calls, member access, …).
pub fn ToTerm(e: &Expression) -> Option<Term> {
    match e {
        Expression::LiteralInt(n) => Some(Term::Const(*n)),
        Expression::LiteralBool(b) => Some(Term::Const(if *b { 1 } else { 0 })),
        Expression::Variable(n) => Some(Term::Var(n.clone())),
        Expression::BinaryOp { Op, Lhs, Rhs } => {
            let l = ToTerm(Lhs)?;
            let r = ToTerm(Rhs)?;
            match Op.as_str() {
                "+" => Some(Term::Add(Box::new(l), Box::new(r))),
                "-" => Some(Term::Sub(Box::new(l), Box::new(r))),
                "*" => Some(Term::Mul(Box::new(l), Box::new(r))),
                _ => None, // '/', '%', bitwise, etc. — unsupported for now
            }
        }
        Expression::UnaryOp { Op, Target } if Op == "-" => {
            let t = ToTerm(Target)?;
            Some(Term::Neg(Box::new(t)))
        }
        _ => None,
    }
}

/// Lower an `Expression` we believe to be boolean.
///
/// Returns `None` on unsupported cases (float comparisons, calls, etc.).
pub fn ToProp(e: &Expression) -> Option<Prop> {
    match e {
        Expression::LiteralBool(true) => Some(Prop::True),
        Expression::LiteralBool(false) => Some(Prop::False),

        Expression::BinaryOp { Op, Lhs, Rhs } => match Op.as_str() {
            // Boolean combinators — recurse on each side.
            "&&" => Some(Prop::And(vec![ToProp(Lhs)?, ToProp(Rhs)?])),
            "||" => Some(Prop::Or(vec![ToProp(Lhs)?, ToProp(Rhs)?])),
            // Comparisons — lower to atoms.
            "==" => Some(Prop::Atom(Atom::Eq(ToTerm(Lhs)?, ToTerm(Rhs)?))),
            "!=" => Some(Prop::Atom(Atom::Ne(ToTerm(Lhs)?, ToTerm(Rhs)?))),
            "<" => Some(Prop::Atom(Atom::Lt(ToTerm(Lhs)?, ToTerm(Rhs)?))),
            "<=" => Some(Prop::Atom(Atom::Le(ToTerm(Lhs)?, ToTerm(Rhs)?))),
            ">" => Some(Prop::Atom(Atom::Gt(ToTerm(Lhs)?, ToTerm(Rhs)?))),
            ">=" => Some(Prop::Atom(Atom::Ge(ToTerm(Lhs)?, ToTerm(Rhs)?))),
            _ => None,
        },

        Expression::UnaryOp { Op, Target } if Op == "!" => {
            Some(Prop::Not(Box::new(ToProp(Target)?)))
        }

        // A bare variable used as a boolean.
        Expression::Variable(n) => Some(Prop::Atom(Atom::Ne(Term::Var(n.clone()), Term::Const(0)))),

        _ => None,
    }
}

/// Evaluate a term under a variable assignment.
pub fn EvalTerm(t: &Term, env: &std::collections::HashMap<String, i64>) -> Option<i64> {
    match t {
        Term::Const(n) => Some(*n),
        Term::Var(n) => env.get(n).copied(),
        Term::Add(a, b) => Some(EvalTerm(a, env)?.wrapping_add(EvalTerm(b, env)?)),
        Term::Sub(a, b) => Some(EvalTerm(a, env)?.wrapping_sub(EvalTerm(b, env)?)),
        Term::Mul(a, b) => Some(EvalTerm(a, env)?.wrapping_mul(EvalTerm(b, env)?)),
        Term::Neg(a) => Some(EvalTerm(a, env)?.wrapping_neg()),
    }
}

/// Evaluate an atom under a variable assignment.
pub fn EvalAtom(a: &Atom, env: &std::collections::HashMap<String, i64>) -> Option<bool> {
    let (l, r) = match a {
        Atom::Eq(l, r) | Atom::Ne(l, r) | Atom::Lt(l, r) | Atom::Le(l, r) | Atom::Gt(l, r) | Atom::Ge(l, r) => {
            (EvalTerm(l, env)?, EvalTerm(r, env)?)
        }
    };
    Some(match a {
        Atom::Eq(_, _) => l == r,
        Atom::Ne(_, _) => l != r,
        Atom::Lt(_, _) => l < r,
        Atom::Le(_, _) => l <= r,
        Atom::Gt(_, _) => l > r,
        Atom::Ge(_, _) => l >= r,
    })
}

/// Evaluate a full proposition under an assignment.
pub fn EvalProp(p: &Prop, env: &std::collections::HashMap<String, i64>) -> Option<bool> {
    match p {
        Prop::True => Some(true),
        Prop::False => Some(false),
        Prop::Atom(a) => EvalAtom(a, env),
        Prop::And(ps) => {
            for p in ps {
                if !EvalProp(p, env)? {
                    return Some(false);
                }
            }
            Some(true)
        }
        Prop::Or(ps) => {
            for p in ps {
                if EvalProp(p, env)? {
                    return Some(true);
                }
            }
            Some(false)
        }
        Prop::Not(p) => Some(!EvalProp(p, env)?),
    }
}

/// Collect every variable name referenced by a proposition. Used by the
/// solver to know which variables to search over.
pub fn VarsOf(p: &Prop) -> Vec<String> {
    let mut out = Vec::new();
    CollectProp(p, &mut out);
    out.sort();
    out.dedup();
    out
}

fn CollectProp(p: &Prop, out: &mut Vec<String>) {
    match p {
        Prop::True | Prop::False => {}
        Prop::Atom(a) => match a {
            Atom::Eq(l, r) | Atom::Ne(l, r) | Atom::Lt(l, r) | Atom::Le(l, r) | Atom::Gt(l, r) | Atom::Ge(l, r) => {
                CollectTerm(l, out);
                CollectTerm(r, out);
            }
        },
        Prop::And(ps) | Prop::Or(ps) => {
            for p in ps {
                CollectProp(p, out);
            }
        }
        Prop::Not(p) => CollectProp(p, out),
    }
}

fn CollectTerm(t: &Term, out: &mut Vec<String>) {
    match t {
        Term::Const(_) => {}
        Term::Var(n) => out.push(n.clone()),
        Term::Add(a, b) | Term::Sub(a, b) | Term::Mul(a, b) => {
            CollectTerm(a, out);
            CollectTerm(b, out);
        }
        Term::Neg(a) => CollectTerm(a, out),
    }
}
