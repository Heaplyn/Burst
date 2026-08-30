# Phase 3 — Elaboration & Constraints

> **Goal:** Turn the layer tree into verifiable logic — extract constraints, prove them with an SMT solver, build the POMSET, and compute the observability boundary.
> **Owns crate:** [`rings/ring2/elaboration`](../../rings/ring2/elaboration/src/lib.rs).
> **Milestone:** `cargo test -p elaboration` passes constraint + verification tests; a provably-safe access compiles with no check, a provably-unsafe one fails with a counterexample.

---

## Current state

[`ElaborationContext`](../../rings/ring2/elaboration/src/lib.rs) is a first sketch:

```rust
pub struct ElaborationContext {
    pub Constraints: Vec<String>,   // SMT-LIB assertion strings
    pub KnownVars: HashSet<String>,
}
```

- `ElaborateLayer` recurses the tree: `Function` collects params and translates `where` clauses; `VariableBinding` records the name and any refinement; `Panic` and `Conditional` print goals/branch constraints.
- `TranslateToSmt` lowers `LiteralInt`, `Variable`, `BinaryOp`, and `FunctionCall` to prefix SMT-LIB (see [Refined Types](../Type%20System%20and%20Coercions/Refined%20Types.md)).

**Gaps found in the code — this phase is where they get real:**
- **No solver.** Constraints are built as strings and never checked. There is no Z3 dependency and no `sat`/`unsat` decision.
- **No POMSET.** The `Constraint::POMSET` AST variant and `Layer::AddDependency` exist but nothing builds the graph.
- **No observability analysis.** `ObservabilityFlags` are never computed from data flow.
- **No `Type::Inferred` resolution.** Inferred bindings pass through untyped.
- `ElaborateLayer` returns `Ok(())` regardless of whether proofs hold.

---

## 3.1 Constraint extraction

- [ ] Refined-type constraints (`where idx < N`) — partially done; extend beyond `Function`/`VariableBinding` to array indexing and `panic` guards.
- [ ] Safety constraints at each `panic` / bounds access / `unreachable` (`Constraint::Safety`).
- [ ] Dependency & POMSET ordering constraints between sibling layers.

## 3.2 SMT integration (Z3)

- [ ] Add the `z3` crate to [`elaboration/Cargo.toml`](../../rings/ring2/elaboration/Cargo.toml).
- [ ] Emit full SMT-LIB: `declare-const` per variable (bit-precise → `(_ BitVec N)`), `assert` the `where` clauses, then negate the safety goal.
- [ ] Query; on `unsat` (failure impossible) **erase the check**, on `sat` surface the counterexample as a compile error with the failing inputs.
- [ ] Honor `@strict` (undecidable = error) vs `@silent` (undecidable = runtime check) directives.

## 3.3 POMSET graph builder

- [ ] Build a node per side-effecting layer; add edges from data/ordering constraints.
- [ ] Detect cycles; compute a topological order; mark independent nodes as parallelizable (feeds [[Phase 4 - Execution Engine]] scheduler and [POMSET and Task Scheduler](../Execution%20Model/POMSET%20and%20Task%20Scheduler.md)).

## 3.4 Observability analysis

- [ ] Propagate observable values (returns, hardware writes, `interrupt`, output) through the tree.
- [ ] Set `AffectsOutput` / `AffectsHardware` / `ObservableToTrace` per layer.
- [ ] Identify foldable traces for [[Phase 7 - Advanced Features]] (see [Observability and Trace Folding](../Compiler%20Mechanics/Observability%20and%20Trace%20Folding.md)).

---

## Acceptance criteria

- `function get<N: usize>(a:[i32;N], i:usize where i<N) -> i32 { return a[i]; }` verifies and emits no bounds check.
- The same function with `i <= N` fails compilation and prints a counterexample (e.g. `i = N`).

## See also
- [[Phase 2 - Layer Tree]] · [[Phase 4 - Execution Engine]] · [Elaboration and Constraints](../Compiler%20Mechanics/Elaboration%20and%20Constraints.md)
