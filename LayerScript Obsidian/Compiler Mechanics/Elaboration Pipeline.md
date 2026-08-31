# Elaboration Pipeline

The [Ring 2 elaboration crate](../Compiler%20Mechanics/Codebase%20Reference.md) is now split into **five explicit layers**, each in its own module, with a `pipeline::RunAll` orchestrator that runs them in order and hands the rewritten tree to the interpreter.

```
parser AST
   │
   ▼
Layer 1: layer1_semantics   — name → symbol, scope stack, undefined/duplicate/shadow errors
   │
   ▼
Layer 2: layer2_types       — bidirectional type inference, operator/assignment/call/return checks
   │
   ▼
Layer 3: layer3_refinements — assumption/goal propagation over branches + local proof rules
   │
   ▼
Layer 4: layer4_smt         — from-scratch solver (interval propagation + bounded enumeration)
   │
   ▼
Layer 5: layer5_optimize    — dead-branch removal · safety erasure · constant folding · DCE
   │
   ▼
Optimized AST → code_runner
```

Every layer's `mod.rs` names what it *requires* and what it *feeds*, so the dependency direction is visible from the tree.

---

## Layer 1 — Semantics

Files under [`rings/ring2/elaboration/src/layer1_semantics/`](../../rings/ring2/elaboration/src/layer1_semantics):

| File | Owns |
| :--- | :--- |
| `symbol.rs`    | `Symbol`, `SymbolId`, `SymbolKind`, `SymbolTable` arena |
| `scope.rs`     | `Scope`, `ScopeStack` (push/pop/insert/lookup) |
| `shadowing.rs` | `IsShadowingAllowed` policy hook |
| `resolve.rs`   | `Resolve(&Layer) -> ResolvedProgram` walker |
| `errors.rs`    | `SemanticError` (undefined / duplicate / shadow denied) |

Walks the AST, opens a fresh scope at each function/block, interns each declaration as a `Symbol`, and checks every `Expression::Variable` reference. Seeds the root scope with built-in names (`print`, `println`, `type`) so calls to them aren't flagged undefined.

## Layer 2 — Types

Files under [`layer2_types/`](../../rings/ring2/elaboration/src/layer2_types):

| File | Owns |
| :--- | :--- |
| `infer.rs`      | `InferExpression(&Expr, &TypeEnv) -> Result<Type, TypeError>` |
| `check.rs`      | `CheckBinaryOp` / `CheckAssignment` / `CheckFunctionCall` / `CheckReturn` |
| `typed_ast.rs`  | `TypeEnv` (scope-stacked name→type), `TypeTable` (per-`LayerId` cache), `TypedExpr` |
| `walk.rs`       | `CheckProgram(&Layer, &ResolvedProgram) -> TypedProgram` walker |
| `errors.rs`     | `TypeError` (mismatch / bad operator / arity / not-implemented) |

A small bidirectional inferrer: bottom-up per expression, widen numeric ops to the wider operand, comparisons yield `b1`, logical operators demand `b1`s, dereference peels a `Pointer`/`Reference`.

## Layer 3 — Refinements

Files under [`layer3_refinements/`](../../rings/ring2/elaboration/src/layer3_refinements):

| File | Owns |
| :--- | :--- |
| `constraint.rs` | `RefinementConstraint`, `GoalId`, `ProofObligation` |
| `branch.rs`     | `AssumptionStack` (frames enter/leave with lexical scopes) |
| `propagate.rs`  | `Propagate(&Layer) -> ConstraintGraph` walker |
| `proof.rs`      | `TryLocal` — cheap syntactic proofs before the solver call |

Function entry pushes each parameter's `where` clause onto the stack. `if (cond)` pushes `cond` on the then-arm and `!cond` on the else-arm. Every `panic` / `unreachable` emits a `ProofObligation`; the graph carries these to Layer 4.

## Layer 4 — Solver (from scratch, no Z3)

Files under [`layer4_smt/`](../../rings/ring2/elaboration/src/layer4_smt):

| File | Owns |
| :--- | :--- |
| `normalize.rs`  | `Term`, `Atom`, `Prop` + `ToTerm` / `ToProp` (Expression → normal form) |
| `interval.rs`   | `Interval`, `IntervalStore` — abstract integer intervals |
| `backend.rs`    | `Query(assumptions, goal) -> SolverVerdict` — the solver |
| `cache.rs`      | `ProofCache` — hash-keyed memoization of `(assumptions, goal) → verdict` |
| `translate.rs`  | Legacy `TranslateToSmt` (SMT-LIB serializer for debug / future Z3 export) |

**The solver is pure Rust — no `z3` dependency.** It handles the fragment LayerScript refinements actually produce: linear integer arithmetic, boolean combinators, comparisons.

### Algorithm

Given `A₁ ∧ A₂ ∧ … ∧ Aₙ` (assumptions) and `G` (goal), decide `sat(A₁ ∧ … ∧ Aₙ ∧ G)`:

1. **Normalize** every `Expression` into `Prop`. Anything we can't lower (floats, strings, calls, member access, etc.) → verdict is `Unknown`.
2. **Interval propagation.** Walk the conjunctive assertions; every `x op const` atom tightens `x`'s interval. Empty interval anywhere → `Unsat`.
3. **Bounded enumeration.** DFS over per-variable candidate ranges (capped at 128 wide per variable, 4096 total leaves). A satisfying assignment → `Sat` with a model. Exhausting the search without a witness → `Unsat`. Blowing the budget → `Unknown`.

This isn't a real SMT solver, but it's honest: `Unsat` means "erase the check safely," `Sat` includes a concrete counterexample, and `Unknown` means "keep the runtime check."

## Layer 5 — Optimization

Files under [`layer5_optimize/`](../../rings/ring2/elaboration/src/layer5_optimize):

| File | Owns |
| :--- | :--- |
| `dead_branch.rs`    | `RemoveDeadBranches(&Layer) -> Layer` — deletes provably-unreachable arms |
| `safety_erasure.rs` | `EraseProvenSafe(&ConstraintGraph) -> ErasureSet` |
| `constant_fold.rs`  | `Fold(&Layer) -> Layer` — folds constant sub-expressions |
| `dce.rs`            | `Eliminate(&Layer) -> Layer` — drops unobservable layers |

Feeds the rewritten tree to `code_runner`, which sees a smaller, cheaper program.

---

## Pipeline

[`pipeline::RunAll`](../../rings/ring2/elaboration/src/pipeline.rs) is a single call:

```rust
let elab = elaboration::RunAll(&ast);
// elab.Program        — the optimized AST (feed to code_runner)
// elab.Resolved.Errors — semantic diagnostics
// elab.Typed.Errors    — type diagnostics
// elab.Constraints     — every ProofObligation the program produced
// elab.Erasures        — GoalIds proven impossible → checks to skip
```

`layerscript` (the driver) now runs `RunAll` between parsing and interpretation.

## See also
- [Codebase Reference](Codebase%20Reference.md) — file-by-file map
- [Elaboration and Constraints](Elaboration%20and%20Constraints.md) — original constraint model
- [Refined Types](../Type%20System%20and%20Coercions/Refined%20Types.md) — the `where` clauses that feed all this
- [[Complete Gameplan]]
