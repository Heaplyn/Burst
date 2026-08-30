# Phase 4 — Execution Engine

> **Goal:** Run LayerScript programs — first a tree-walking interpreter, then a bytecode VM, then native codegen, with the POMSET scheduler for parallel execution.
> **Owns crate:** [`rings/ring3/code_runner`](../../rings/ring3/code_runner/src/lib.rs).
> **Milestone:** `cargo run -- compile x.layerscript` runs a real program and returns the correct value.

---

## Current state

`code_runner` is a working **tree-walking interpreter** for a small subset:

- `CodeRunner { Context, Trace, Config }` with `RunCode` → `RunLayer` recursion.
- `EvaluateExpression` handles `LiteralInt/Float/Bool`, `Variable` lookup, and `BinaryOp`; `EvaluateBinaryOp` does int `+ - * /` (with divide-by-zero guard) and float `+`.
- `RunLayer` handles `Program`, `Function` (push/pop `Frame`, run body, check return type), `VariableBinding`, and `Return`.
- `ExecutionContext` holds global `Variables` + a `Stack` of `Frame`s; `Value` is `Unit/Int/Float/Bool/String/Array/Struct`.

Verified: `eval "function main() { var x = 30; }"` → `Execution Result: Unit`.

**Gaps found in the code (fix in this phase):**
- **Hook bodies never execute.** In `RunLayer`'s `VariableBinding` arm, the temporary hook layer is built with `Children: vec![]`, so `on_change`/`on_assign` functions run over an empty body.
- **Hook order is inverted vs. the spec.** The code runs `on_assign` *before* the store and `on_change` *after*; the [Hooks Runtime](../Execution%20Model/Variable%20Behavior%20Hooks%20Runtime.md) defines `on_change` (pre-store, transforms) then store then `on_assign` (post-store).
- **No control-flow execution.** `Conditional`, `Loop`, `MatchArm`, `Block`(as statement) hit the `_ =>` "Unsupported layer kind" error.
- **No function calls.** `Expression::FunctionCall` is unsupported in `EvaluateExpression`.
- **`havoc` is commented out**; `Invalidate`/`KeepCache` are stubs.
- **Stray debug `print!("GetVar …")`** in `ExecutionContext::GetVariable` — remove.
- `FindMainFunction` exists but `RunCode` runs all layers instead of entering `main`.

## 4.1 Interpreter (development & testing)

- [ ] Execute `Conditional` (evaluate condition → run the taken branch) and `Loop` (`While`/`For`/`Infinite`).
- [ ] Evaluate `FunctionCall`: resolve the function layer, bind args into a new `Frame`, run, return.
- [ ] Fix hook execution: populate hook-body `Children`, run `on_change` pre-store (use its return), store, then `on_assign`.
- [ ] Implement `havoc` and re-enable the commented arm.
- [ ] Enter via `FindMainFunction` when present.

## 4.2 Bytecode VM (balanced performance)

- [ ] Define a bytecode ISA; lower the layer tree to it.
- [ ] Stack-based VM with a frame stack, call/return, and debugging hooks.

## 4.3 Assembly generation (production)

- [ ] x86-64 instruction selection, linear-scan register allocation, prologue/epilogue, object emission + linking. (This is the first **Codegen** work — currently ❌ not started.)

## 4.4 POMSET scheduler (parallel execution)

- [ ] Work-stealing scheduler that runs independent POMSET nodes ([[Phase 3 - Elaboration and Constraints]] §3.3) in parallel with join synchronization, preserving observability order.

---

## Acceptance criteria

- A program with an `if`, a `while`, and a called helper function returns the right `Value`.
- A clamping `on_change` hook actually changes the stored value.

## See also
- [[Phase 3 - Elaboration and Constraints]] · [[Phase 5 - Standard Library and Runtime]] · [layertrace Runtime](../Execution%20Model/layertrace%20Runtime.md)
