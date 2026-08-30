# Phase 1 — Complete the Parser

> **Goal:** Parse entire LayerScript files into complete, correct [Layer](../Language%20Specification/Layer%20System.md) trees.
> **Owns crate:** [`rings/ring1/parser`](../../rings/ring1/parser/src/lib.rs) (with token support from [`rings/ring0/lexer`](../../rings/ring0/lexer/src/lib.rs)).
> **Milestone:** `cargo test -p parser` passes expression, statement, and type tests.

---

## Why this is the critical path

Every later ring consumes the layer tree the parser produces. Elaboration can't extract constraints, and the interpreter can't run, from syntax the parser silently drops. Right now the parser handles the *skeleton* of a program (functions, structs, bindings, a few statements) but bails on most real expressions — so this phase unblocks everything.

---

## Current state (as of this codebase)

**Working:**
- Top-level dispatch — [`ParseItem`](../../rings/ring1/parser/src/lib.rs) routes `function`/`struct`/statement.
- `ParseFunction`, `ParseStruct`, `ParseReturn`, `ParseIf`, `ParseWhile`, `ParseInterrupt`.
- `ParseVariableBinding` with typed, and now **inferred**, bindings (`Type::Inferred`); hooks limited to `on_change` / `on_read`.
- `ParseNameAndType` handles `name: Type`, `Type name`, and the inferred `name = …` form.
- Precedence-climbing `ParseBinary` + `ParsePrimary` postfix (`.`, `()`, `[]`).

**Gaps found in the code (fix in this phase):**
- **Lexer can't produce `==`, `!=`, `&&`, `||`, `!`.** `=` lexes as a single `Equal` (see [`lexer/src/lib.rs`](../../rings/ring0/lexer/src/lib.rs) line ~85), so comparisons/logic in the [operator table](../Language%20Specification/Syntax%20and%20Grammar.md) are currently unlexable. This is a Phase-1 blocker even though it lives in Ring 0.
- **`match`, `for`, `loop`, `goto`, `enum`** have tokens but no parse path — they fall through `ParseStatement` to the expression branch and error.
- **Unary `-` and `!`** aren't handled in `ParseAtom` (only `*` deref is).
- **Generics** (`<T>`, `Vec<T>`) aren't parsed.
- `TokenPrecedence` conflates assignment `=` with comparison and omits bitwise ops.

---

## 1.1 Expression parsing

- [ ] Extend the lexer to emit `EqualEqual`, `NotEqual`, `AndAnd`, `OrOr`, `Not`, ` Amp`, `Pipe`, `Caret`, `Shl`, `Shr` (multi-char lookahead like the existing `->` / `<=` handling).
- [ ] Add unary `-` and `!` to `ParseAtom`.
- [x] Complete `TokenPrecedence` to match the [Syntax and Grammar](../Language%20Specification/Syntax%20and%20Grammar.md) precedence table (14 levels).
- [x] Confirm function calls, `struct.field`, and `arr[idx]` postfix chains compose (already in `ParsePrimary`).

**Design note:** keep the single precedence-climbing loop; just widen the operator/precedence tables. Represent every binary op as `Expression::BinaryOp { Op: String, .. }` for now — a later pass can intern these into the `BinaryOp` enum already declared in [`ast/src/lib.rs`](../../rings/ring0/ast/src/lib.rs).

## 1.2 Statement parsing

- [ ] `match` → `LayerKind::MatchArm` children (see [Pattern Matching](../Language%20Specification/Pattern%20Matching.md)).
- [ ] `for` and `loop` → `LayerKind::Loop { Kind: For / Infinite }` (the `LoopKind` variants already exist).
- [x] Wire assignment vs. expression-statement (already handled in the `_` arm of `ParseStatement`).
- [x] `havoc` (done) and `panic`/`unreachable` (done) — verify semicolon handling.

## 1.3 Type parsing

- [x] Bit-precise (`i32`, `b8`, …) — done via `BitPreciseType`.
- [ ] `struct` / `enum` type references — `enum` parsing still missing.
- [x] Refined types (`where …`) — done in `ParseNameAndType`.
- [ ] Generics (`T`, `Vec<T>`) and function types (`fn(i32) -> i32`).

## 1.4 Error recovery & reporting

- [ ] Thread the token's `Line`/`Column` (already on every `Token`) into `SourceLocation` instead of `SourceLocation::Builtin()` everywhere.
- [ ] "expected X, found Y" with the **consumed** token (the current `else` arm of `ParseNameAndType` reports `Peek()` one token late — see the parser-fix discussion).
- [ ] Synchronize-on-error (skip to next `;`/`}`) so one mistake doesn't cascade.

---

## Acceptance criteria

- `cargo run -- eval "function f(){ var x = 1 + 2 * 3; if (x == 7) { return x; } }"` parses without error and the AST shows the correct precedence.
- Every keyword in [`token.rs`](../../rings/ring0/lexer/src/token.rs) has a parse path or an explicit "not yet" error with a line number.

## See also
- [[Phase 2 - Layer Tree]] · [Parser and Lexer](../Compiler%20Mechanics/Parser%20and%20Lexer.md) · [Codebase Reference](../Compiler%20Mechanics/Codebase%20Reference.md)
