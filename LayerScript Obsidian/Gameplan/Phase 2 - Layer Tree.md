# Phase 2 — Build the Layer Tree

> **Goal:** A complete, richly-annotated [Layer](../Language%20Specification/Layer%20System.md) system — every construct is a layer, every layer carries the metadata later rings need.
> **Owns crate:** [`rings/ring0/ast`](../../rings/ring0/ast/src/lib.rs) (`lib.rs` + `types.rs`).
> **Milestone:** `cargo test -p ast` passes layer-construction and type-storage tests.

---

## Current state

The `Layer` struct is already comprehensive ([`ast/src/lib.rs`](../../rings/ring0/ast/src/lib.rs)):

```rust
pub struct Layer {
    pub Id: LayerId,               // unique, from the atomic LayerAddress counter
    pub Kind: LayerKind,
    pub Metadata: LayerMetadata,   // source loc, docs, directives, optimization hints, custom map
    pub Children: Vec<Layer>,
    pub Constraints: Vec<Constraint>,
    pub Observability: ObservabilityFlags,
    pub TypeStorage: TypeStorage,
    pub TraceInfo: TraceInfo,
}
```

`LayerKind` already covers `Program`, `Function`, `VariableBinding`, `Assignment`, `Expression`, `Block`, `Loop`, `Conditional`, `MatchArm`, `Panic`, `Unreachable`, `Havoc`, `Interrupt`, `Struct`, `Enum`, `Return`, plus a `VariableHook(VariableHook, Expression)` variant. `LayerBuilder` mints IDs via the `LayerAddress: AtomicUsize` counter.

**Gaps found in the code:**
- `Layer` has **no `Parent` link** — traversal is top-down only. Inheritance/observability need upward walks (Phase 3), so add parent tracking (an index/`LayerId`, not a back-reference, to keep `PartialEq`/`Clone` cheap).
- `Metadata.Custom` and `Optimization` exist but nothing populates them yet.
- Hooks: the AST has `HookKind::{OnChange, OnRead, OnAssign, OnDrop, OnError}` but the parser only reads `on_change`/`on_read` — align the two.

---

## 2.1 Complete layer kinds

- [ ] Ensure every `Expression` and statement the parser emits has a home in `LayerKind` (mostly done).
- [ ] Populate `ObservabilityFlags` at construction (default is `ObservableToTrace: true`, everything else false).
- [ ] Fold `BinaryOp` enum usage (currently ops are stringly-typed in `Expression::BinaryOp`).

## 2.2 Type storage & inheritance

Backed by [`TypeStorage`](../Type%20System%20and%20Coercions/Type%20Storage%20and%20Inheritance.md) (`DefinedTypes` + `TypeAliases`). See [Type Checking and Inference](../Compiler%20Mechanics/Type%20Checking%20and%20Inference.md).

- [ ] Seed the root `Program` layer with built-in primitives (`i8..i128`, `u8..u128`, `f32..f128`, `b1..bN`).
- [ ] Implement outward lookup (current → parents → built-ins) — needs the parent link from 2.0.
- [ ] Type aliasing (`type X = …`) and generic parameters.
- [ ] Resolve `Type::Inferred` placeholders (shared with [[Phase 3 - Elaboration and Constraints]]).

## 2.3 Layer tree construction

- [ ] `Program` root already built in `Parser::Parse`.
- [ ] Add parent linking during construction (or in a post-parse pass).
- [ ] Validate structure: no cycles, correct depth/ancestry (feeds `TraceInfo.Depth`).

## 2.4 Metadata & hooks

- [ ] Fill `SourceLocation` from real token positions (shared with [[Phase 1 - Parser]] task 1.4).
- [ ] Parse the remaining hook kinds and validate signatures against the variable's type (see [Hooks Runtime](../Execution%20Model/Variable%20Behavior%20Hooks%20Runtime.md)).
- [ ] Directive handling (`@inline`, `@cold`, `@strict`, `@silent`) → `Metadata.Directives`.

---

## Acceptance criteria

- `{:#?}` on a parsed program shows parent-linked layers with real source locations.
- A type defined in an outer scope resolves from an inner scope; an inner redefinition shadows it.

## See also
- [[Phase 1 - Parser]] · [[Phase 3 - Elaboration and Constraints]] · [Layer System](../Language%20Specification/Layer%20System.md)
