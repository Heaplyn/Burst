# Phase 7 — Advanced Features

> **Goal:** Deliver the features that make LayerScript *LayerScript* — zero-cost refined types, POMSET optimization, hardware-specific codegen, and a compiler-plugin system.
> **Touches:** [`ring2/elaboration`](../../rings/ring2/elaboration/src/lib.rs) (proofs, folding) and the codegen work from [[Phase 4 - Execution Engine]].
> **Milestone:** LayerScript demonstrates a measurable speedup vs. C on a benchmark suite.

---

## Dependencies

This phase builds on [[Phase 3 - Elaboration and Constraints]] (SMT + POMSET + observability) and [[Phase 4 - Execution Engine]] (codegen). It is where the [Principle of Most Speed](../Home.md) is actually cashed out.

## 7.1 Refined types (zero-cost)

- [ ] Compile-time proof checking wired to the SMT solver ([Refined Types](../Type%20System%20and%20Coercions/Refined%20Types.md)).
- [ ] `where` clauses accepted everywhere (params, fields, locals, returns).
- [ ] **Proof erasure**: once `unsat`, drop the runtime check entirely — the headline feature (safe array indexing with no bounds check).

## 7.2 POMSET optimization

- [ ] Automatic parallelization of independent nodes.
- [ ] **Trace folding** — delete traces outside the [observability boundary](../Compiler%20Mechanics/Observability%20and%20Trace%20Folding.md).
- [ ] Constant folding, dead-code elimination, loop reduction, inlining, vectorization, register-allocation tuning.

## 7.3 Hardware-specific features

- [ ] CPU feature detection, SIMD auto-vectorization, cache management, interrupt handlers, zero-copy I/O, memory barriers, atomics ([Hardware and Havoc](../Bare%20Metal%20Interfacing/Hardware%20and%20Havoc.md)).

## 7.4 Compiler plugins

- [ ] Plugin system for custom passes, linters, transformations, and alternate codegen backends — cleanly layered on the ring architecture.

---

## Acceptance criteria

- A bounds-checked-in-source hot loop compiles to check-free machine code and beats the equivalent C build on a benchmark.
- Trace folding measurably shrinks output on a program with dead computation.

## See also
- [[Phase 3 - Elaboration and Constraints]] · [[Phase 4 - Execution Engine]] · [[Phase 8 - Self-Hosting]] · [Observability and Trace Folding](../Compiler%20Mechanics/Observability%20and%20Trace%20Folding.md)
