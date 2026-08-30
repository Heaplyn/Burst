# Phase 5 — Standard Library & Runtime

> **Goal:** A core library and runtime that real programs can build on — types, core functions, the hardware interface, and `layertrace` support.
> **Touches:** a new `stdlib` (likely `rings/ring2` or a sibling crate), plus runtime hooks in [`code_runner`](../../rings/ring3/code_runner/src/lib.rs).
> **Milestone:** `cargo build` produces a working runtime library the compiler links against.

---

## Current state

There is no stdlib crate yet. The interpreter's `Value` enum (`Unit/Int/Float/Bool/String/Array/Struct`) is the entire "runtime" today, and the [Core Library Reference](../API%20and%20Standard%20Library/Core%20Library%20Reference.md) / [Runtime and Compiler API](../API%20and%20Standard%20Library/Runtime%20and%20Compiler%20API.md) notes describe the target surface. This phase makes those notes real.

---

## 5.1 Core types

- [ ] Integer (`i8..i128`, `u8..u128`) and float (`f32/f64/f128`) families as first-class, with the [Built-in Types Reference](../API%20and%20Standard%20Library/Built-in%20Types%20Reference.md) semantics.
- [ ] `bool`, `String`, `Array`, `Slice`.
- [ ] `Option<T>` and `Result<T, E>` (the [Syntax and Grammar](../Language%20Specification/Syntax%20and%20Grammar.md) `enum Result<T>` example is the model).
- [ ] Predefined refinements: `NonZero<T>`, `Positive<T>`, `Index<N>`, `Aligned<T,A>`.

## 5.2 Core functions

- [ ] `trace!` / print / debug (used throughout the [tutorials](../Tutorials%20and%20Examples/Simple%20Scripts.md)).
- [ ] Math (`sin`, `cos`, `sqrt`, …), memory (`alloc`, `free`, `copy`), string (`len`, `concat`), array (`len`, `push`, `pop`).

## 5.3 Hardware interface

- [ ] Register access, interrupt handlers, memory-mapped I/O, atomics, SIMD intrinsics, inline `asm!` — the surface behind [Hardware and Havoc](../Bare%20Metal%20Interfacing/Hardware%20and%20Havoc.md) and [Memory Layout and Packing](../Bare%20Metal%20Interfacing/Memory%20Layout%20and%20Packing.md).
- [ ] FFI plumbing for `extern` (see [FFI and extern](../Bare%20Metal%20Interfacing/FFI%20and%20extern.md)).

## 5.4 Runtime support

- [ ] `layertrace` implementation backing the [layertrace Runtime](../Execution%20Model/layertrace%20Runtime.md) API (`current`, `root`, `push`/`pop`, `lookup_type`, `get_metadata`). The AST already carries `TraceInfo` per layer.
- [ ] Runtime type info, stack unwinding, panic handling, allocation.
- [ ] Variable-hook runtime per the [Hooks Runtime](../Execution%20Model/Variable%20Behavior%20Hooks%20Runtime.md) ordering (coordinate with [[Phase 4 - Execution Engine]] §4.1).

---

## Acceptance criteria

- A program can `import` core types, print, and use `Option`/`Result`.
- `layertrace.current().kind` returns the correct layer kind at runtime.

## See also
- [[Phase 4 - Execution Engine]] · [[Phase 6 - Tooling and Developer Experience]] · [Core Library Reference](../API%20and%20Standard%20Library/Core%20Library%20Reference.md)
