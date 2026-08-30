# Phase 8 — Self-Hosting

> **Goal:** Rewrite the LayerScript compiler in LayerScript and bootstrap it off the Rust implementation.
> **Milestone:** `layerscript compile layerscript.layerscript` produces a working compiler.

---

## Why this is last

Self-hosting is the final proof that the language is complete and fast enough to build itself. It depends on essentially everything before it: a full parser ([[Phase 1 - Parser]]), the layer tree ([[Phase 2 - Layer Tree]]), verification ([[Phase 3 - Elaboration and Constraints]]), codegen ([[Phase 4 - Execution Engine]]), a stdlib ([[Phase 5 - Standard Library and Runtime]]), and tooling ([[Phase 6 - Tooling and Developer Experience]]).

## 8.1 Core compiler in LayerScript

Port each ring, mirroring the Rust layout ([Codebase Reference](../Compiler%20Mechanics/Codebase%20Reference.md)):

- [ ] Lexer (from [`ring0/lexer`](../../rings/ring0/lexer/src/lib.rs))
- [ ] AST / Layer types (from [`ring0/ast`](../../rings/ring0/ast/src/lib.rs))
- [ ] Parser (from [`ring1/parser`](../../rings/ring1/parser/src/lib.rs))
- [ ] Type checker + elaborator (from [`ring2/elaboration`](../../rings/ring2/elaboration/src/lib.rs))
- [ ] Code generator

## 8.2 Bootstrapping

- [ ] **Stage 0:** the current Rust compiler.
- [ ] **Stage 1:** LayerScript compiler written in LayerScript, compiled by Stage 0.
- [ ] **Stage 2:** Stage 1 compiles itself.
- [ ] Verify **Stage 1 output == Stage 2 output** (fixpoint), then drop the Rust dependency.

## 8.3 Testing & validation

- [ ] Self-hosting test suite, source compatibility tests, performance validation, binary-size checks.

---

## Acceptance criteria

- The Stage 1 and Stage 2 binaries are byte-identical.
- The self-hosted compiler passes the same test suite as the Rust one.

## See also
- [[Complete Gameplan]] · [[Phase 7 - Advanced Features]] · [Compiler Implementation](../Compiler%20Mechanics/Compiler%20Implementation.md)
