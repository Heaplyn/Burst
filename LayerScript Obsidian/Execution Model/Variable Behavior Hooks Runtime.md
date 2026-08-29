# Variable Behavior Hooks Runtime

[Variable hooks](../Language%20Specification/Variables%20and%20Hooks.md) let a binding react to reads and writes. This page specifies their **runtime semantics** — the exact order in which hooks fire, how conflicts resolve, what happens on failure, and what it all costs after optimization.

---

## 1. Hook Execution Order

Each access to a hooked variable expands into a fixed sequence. The hooks fire *around* the raw memory operation, never replacing it.

**On write** (`x = expr;`):

```
1. evaluate expr                -> new
2. snapshot current value       -> old
3. run on_change(new, old)      -> value'   (may clamp/transform)
4. store value'                 (the actual memory write)
5. run on_assign(value')                    (post-commit notification)
```

**On read** (`... x ...`):

```
1. run on_read()                -> value    (may compute/trace)
2. yield value to the expression
```

Key rule: `on_change` runs **before** the store and its return value is what gets stored; `on_assign` runs **after** and cannot change the stored value. This is why clamping goes in `on_change`.

---

## 2. Hook Prioritization

A variable may accumulate more than one hook of the same kind — declared locally and/or inherited from an enclosing type or layer. They run **inner-most first, outer-most last**, so a local hook sees the value before ancestor hooks do.

```
value flows:  local on_change -> parent on_change -> ... -> store
```

Rules:
- Within a single binding, hooks fire in **declaration order**.
- For `on_change`, each hook's return feeds the next hook's `new` — they compose like a pipeline.
- A hook may read `layertrace.current()` to learn which layer it is guarding, useful for shared/inherited hooks.

> [!NOTE]
> Inherited hooks come from the [layer hierarchy](../Language%20Specification/Layer%20System.md); shadowing a variable in an inner scope also shadows its hooks.

---

## 3. Error Handling in Hooks

A hook is an ordinary function layer, so it may `panic` or hit `unreachable`.

- **`panic` inside `on_change`** aborts the write — memory is left holding `old`, preserving the invariant that a rejected value never lands.
- **`panic` inside `on_read`** aborts the enclosing expression; no partial value is produced.
- **`unreachable`** is a proof obligation: the [SMT solver](../Compiler%20Mechanics/Elaboration%20and%20Constraints.md) must show the hook can never reach it, or compilation fails under `@strict`.

Because hooks can reject writes, they are a natural place to enforce refinements that cannot be proven statically (e.g. values arriving from FFI).

---

## 4. Performance Implications

Hooks are designed to be **zero-cost when provably unnecessary**:

- If [observability analysis](../Compiler%20Mechanics/Observability%20and%20Trace%20Folding.md) shows a hook has no observable effect (pure, result unused, no hardware/output), the whole hook trace is folded away.
- A constant-valued `on_change` (e.g. a clamp the solver proves always returns `new`) is elided — the store proceeds directly.
- Hooks that survive are **inlined** at each access site; there is no dispatch table and no indirect call.
- Frequent reads with a pure `on_read` may be hoisted out of loops when the solver proves the computed value is loop-invariant.

The practical guidance: write hooks freely for correctness. What you cannot observe, you do not pay for.

## See also
- [Variables and Hooks](../Language%20Specification/Variables%20and%20Hooks.md)
- [Observability and Trace Folding](../Compiler%20Mechanics/Observability%20and%20Trace%20Folding.md)
- [Type Checking and Inference](../Compiler%20Mechanics/Type%20Checking%20and%20Inference.md)
