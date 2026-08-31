//! Ring 2 · Elaboration · **Layer 4 (Solver) — interval domain**
//!
//! A minimal abstract interpretation over integer intervals. Each variable
//! gets an inclusive `[lo, hi]` (using `Option<i64>` for open bounds), and
//! atomic Constraints_ are propagated into interval tightenings.
//!
//! Interval domain is coarse — it can prove `x >= 0 && x < 10 && x == 15`
//! contradictory, but it can't prove `x + y == y + x` alone. That's fine: it
//! catches the majority of linear-integer refinements which is what the
//! language cares about (bounds, non-zero, sign, alignment residues).

use std::collections::HashMap;

/// A closed-or-open integer interval. `None` on either bound means unbounded
/// in that direction (`(-∞, hi]` or `[lo, +∞)`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub lo: Option<i64>,
    pub hi: Option<i64>,
}

impl Interval {
    pub fn Top() -> Self {
        Self { lo: None, hi: None }
    }

    pub fn Point(n: i64) -> Self {
        Self { lo: Some(n), hi: Some(n) }
    }

    /// `true` if the interval is empty (contains no integer).
    pub fn IsEmpty(&self) -> bool {
        matches!((self.lo, self.hi), (Some(l), Some(h)) if l > h)
    }

    /// Tighten by intersecting with another interval.
    pub fn Meet(&self, other: &Interval) -> Interval {
        Interval {
            lo: match (self.lo, other.lo) {
                (None, x) | (x, None) => x,
                (Some(a), Some(b)) => Some(a.max(b)),
            },
            hi: match (self.hi, other.hi) {
                (None, x) | (x, None) => x,
                (Some(a), Some(b)) => Some(a.min(b)),
            },
        }
    }
}

/// A per-variable interval store. Missing entries default to `Top`.
#[derive(Debug, Default, Clone)]
pub struct IntervalStore {
    map: HashMap<String, Interval>,
}

impl IntervalStore {
    pub fn New() -> Self {
        Self::default()
    }

    pub fn Get(&self, name: &str) -> Interval {
        self.map.get(name).copied().unwrap_or_else(Interval::Top)
    }

    /// Meet the current interval for `name` with `other`. Returns the new
    /// interval — the caller checks emptiness to detect contradiction.
    pub fn Tighten(&mut self, name: &str, other: Interval) -> Interval {
        let cur = self.Get(name);
        let met = cur.Meet(&other);
        self.map.insert(name.to_string(), met);
        met
    }

    /// True iff *any* variable in the store has an empty interval.
    pub fn HasContradiction(&self) -> bool {
        self.map.values().any(|i| i.IsEmpty())
    }

    /// Convenience for the search phase: pick a variable that isn't fully
    /// pinned and return `(name, lo, hi)` for enumeration. `None` if every
    /// variable is either fully pinned or fully unbounded.
    pub fn PickBoundedFree(&self) -> Option<(String, i64, i64)> {
        for (name, iv) in &self.map {
            if let (Some(l), Some(h)) = (iv.lo, iv.hi) {
                if l < h {
                    return Some((name.clone(), l, h));
                }
            }
        }
        None
    }

    pub fn Iter(&self) -> impl Iterator<Item = (&String, &Interval)> {
        self.map.iter()
    }
}
