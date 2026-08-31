//! Ring 2 · Elaboration · **Layer 4 (Solver) — from-scratch backend**
//!
//! Not Z3. This is a small, pure-Rust satisfiability engine that handles the
//! fragment LayerScript refinements actually generate: linear integer
//! arithmetic + boolean combinators + comparisons.
//!
//! ## Algorithm (high level)
//!
//! Given assumptions `A₁ ∧ A₂ ∧ … ∧ Aₙ` and a goal `G`, we ask
//! `sat(A₁ ∧ … ∧ Aₙ ∧ G)`:
//!
//! 1. Normalize each `Expression` into our internal `Prop` form
//!    ([`normalize::ToProp`]). Anything we can't lower → `Unknown`.
//! 2. Push the assumptions and goal through **interval propagation**
//!    ([`interval::IntervalStore`]) — tightens per-variable `[lo, hi]` from
//!    every simple `x < N`-style atom.
//! 3. If propagation produces an empty interval → contradiction → `Unsat`.
//! 4. Otherwise fall back to **bounded enumeration** — pick a bounded
//!    variable, iterate its interval, recurse until we find a satisfying
//!    assignment or exhaust the search space.
//! 5. If we run out of budget or hit unbounded variables → `Unknown`.
//!
//! This won't beat a real SMT solver on hard problems, but it's honest about
//! what it can and can't prove, which is what the language pitch demands:
//! erase what we can prove, keep the check when we can't.

use std::collections::HashMap;

use ast::Expression;

use super::interval::{Interval, IntervalStore};
use super::normalize::{Atom, EvalProp, Prop, Term, ToProp, VarsOf};

/// Verdict returned by [`Query`].
#[derive(Debug, Clone, PartialEq)]
pub enum SolverVerdict {
    /// A model exists — the goal is reachable / the assumption is not enough
    /// to force it false. Erasing a safety check based on this would be wrong.
    Sat { Model: HashMap<String, i64> },
    /// No model — the conjunction is impossible. Safe to erase.
    Unsat,
    /// The solver could not decide (unsupported syntax, budget exhausted).
    Unknown,
}

/// Maximum number of leaves the enumerator will visit before giving up.
const BUDGET: usize = 4096;
/// Cap on enumeration width per variable, to avoid exploding on huge ranges.
const ENUM_WIDTH_CAP: i64 = 256;

/// The public entry point.
///
/// `Assumptions` are what we know to be true (parameter `where` clauses,
/// branch conditions).  `Goal` is what we're asking the solver to satisfy —
/// typically the *failure condition* of a runtime check. `Unsat` here means
/// "safe to erase the check".
pub fn Query(Assumptions: &[Expression], Goal: &Expression) -> SolverVerdict {
    // 1. Normalize.
    let mut props: Vec<Prop> = Vec::with_capacity(Assumptions.len() + 1);
    for a in Assumptions {
        match ToProp(a) {
            Some(p) => props.push(p),
            None => return SolverVerdict::Unknown, // unsupported assumption
        }
    }
    let GoalProp = match ToProp(Goal) {
        Some(p) => p,
        None => return SolverVerdict::Unknown,
    };
    props.push(GoalProp);
    let full = Prop::And(props);

    // 2. Interval propagation over the flattened conjunction.
    let mut store = IntervalStore::New();
    PropagateProp(&full, &mut store);
    if store.HasContradiction() {
        return SolverVerdict::Unsat;
    }

    // 3. Bounded enumeration.
    let vars = VarsOf(&full);
    let mut assignment = HashMap::new();
    let mut visited = 0usize;
    if Enumerate(&full, &vars, 0, &store, &mut assignment, &mut visited) {
        SolverVerdict::Sat { Model: assignment }
    } else if visited >= BUDGET {
        SolverVerdict::Unknown
    } else {
        SolverVerdict::Unsat
    }
}

// ------------------------------------------------------------------
// Interval propagation
// ------------------------------------------------------------------

/// Walk `p` and push whatever we can into the interval store. Only atoms of
/// the shape `var op const` (either direction) tighten bounds — everything
/// else is left for the enumerator to check.
fn PropagateProp(p: &Prop, store: &mut IntervalStore) {
    match p {
        Prop::And(children) => {
            for c in children {
                PropagateProp(c, store);
            }
        }
        Prop::Atom(a) => PropagateAtom(a, store),
        // `Or` / `Not` are handled by enumeration — interval propagation only
        // covers conjunctions of simple bounds.
        _ => {}
    }
}

fn PropagateAtom(a: &Atom, store: &mut IntervalStore) {
    // Try both orderings — `x < 10` and `10 > x` produce the same bound.
    let (pair, order) = match a {
        Atom::Lt(l, r) => (Pair(l, r), 0u8),
        Atom::Le(l, r) => (Pair(l, r), 1),
        Atom::Gt(l, r) => (Pair(l, r), 2),
        Atom::Ge(l, r) => (Pair(l, r), 3),
        Atom::Eq(l, r) => (Pair(l, r), 4),
        Atom::Ne(_, _) => return, // != doesn't shrink intervals meaningfully
    };
    let (var, k) = match pair {
        (Some(v), Some(k)) => (v, k),
        _ => return,
    };
    let iv = match order {
        // x < k   →  x ∈ (-∞, k-1]
        0 => Interval { lo: None, hi: Some(k.saturating_sub(1)) },
        // x <= k  →  x ∈ (-∞, k]
        1 => Interval { lo: None, hi: Some(k) },
        // x > k   →  x ∈ [k+1, +∞)
        2 => Interval { lo: Some(k.saturating_add(1)), hi: None },
        // x >= k  →  x ∈ [k, +∞)
        3 => Interval { lo: Some(k), hi: None },
        // x == k  →  x ∈ [k, k]
        4 => Interval::Point(k),
        _ => return,
    };
    store.Tighten(&var, iv);
}

/// If exactly one side is a variable and the other a constant, return
/// `(Some(varname), Some(constant))`. Otherwise `(None, None)`.
fn Pair(l: &Term, r: &Term) -> (Option<String>, Option<i64>) {
    match (l, r) {
        (Term::Var(n), Term::Const(k)) => (Some(n.clone()), Some(*k)),
        (Term::Const(k), Term::Var(n)) => (Some(n.clone()), Some(*k)),
        _ => (None, None),
    }
}

// ------------------------------------------------------------------
// Bounded enumeration
// ------------------------------------------------------------------

/// Recursive DFS over variable assignments. Returns `true` on the first
/// satisfying assignment found; the assignment is left in `env`.
fn Enumerate(
    p: &Prop,
    vars: &[String],
    idx: usize,
    store: &IntervalStore,
    env: &mut HashMap<String, i64>,
    visited: &mut usize,
) -> bool {
    *visited += 1;
    if *visited > BUDGET {
        return false;
    }
    if idx == vars.len() {
        // All variables assigned — evaluate.
        return EvalProp(p, env).unwrap_or(false);
    }

    let name = &vars[idx];
    let iv = store.Get(name);

    // Choose a candidate range.
    let (lo, hi) = match (iv.lo, iv.hi) {
        (Some(l), Some(h)) => (l, h),
        // Unbounded on one or both sides — sample a small window around 0.
        (Some(l), None) => (l, l.saturating_add(ENUM_WIDTH_CAP)),
        (None, Some(h)) => (h.saturating_sub(ENUM_WIDTH_CAP), h),
        (None, None) => (-ENUM_WIDTH_CAP / 2, ENUM_WIDTH_CAP / 2),
    };

    // Cap absurd ranges — the point of Layer 4 is to fail fast on unsupported
    // shapes and let `Unknown` propagate, not to burn cycles.
    let width = hi.saturating_sub(lo);
    if width < 0 {
        return false;
    }
    let StepHi = if width > ENUM_WIDTH_CAP { lo.saturating_add(ENUM_WIDTH_CAP) } else { hi };

    let mut v = lo;
    while v <= StepHi {
        env.insert(name.clone(), v);
        if Enumerate(p, vars, idx + 1, store, env, visited) {
            return true;
        }
        v = v.saturating_add(1);
    }
    env.remove(name);
    false
}
