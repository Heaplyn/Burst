//! Ring 2 · Elaboration · **Layer 1 (Semantics) — Shadowing policy**
//!
//! LayerScript follows Rust-ish rules for now:
//!   - Same-scope duplicate: **error** (handled in [`Scope::Insert`]).
//!   - Nested-scope shadow: **allowed silently**.
//!
//! Split out as its own file so a future "strict mode" that forbids all
//! shadowing (or a per-kind policy) is a one-file change.
//!
//! Requires: nothing.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShadowDecision {
    Allow,
    Deny,
}

/// Decide whether a new inner declaration may shadow an outer one.
///
/// `SameScope` is `true` when both declarations live in the *same* scope —
/// that case is not shadowing, it's a duplicate, and this function should
/// never be called with `true` (the caller checks duplicates first). Kept as
/// a parameter for future policy that treats the two cases differently.
pub fn IsShadowingAllowed(_SameScope: bool) -> ShadowDecision {
    ShadowDecision::Allow
}
