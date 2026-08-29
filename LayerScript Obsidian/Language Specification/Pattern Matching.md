# Pattern Matching

Pattern matching in LayerScript is powered by the `match` expression. Beyond typical branch selection, it features deep, direct integration with the compiler's **SMT path-solver (Z3)**. This allows the compiler to prove safety constraints and erase unreachable paths.

---

## 1. Syntax and Basic Matching

You can match on integers, enums, or structures using the `match` keyword. Every match must be **exhaustive** (either cover all possible cases or include a wildcard `_` fallback).

```layerscript
enum State {
    Idle,
    Running,
    Error(u32),
}

function process(state: State) {
    match state {
        State.Idle => output_trace('IDLE', 0 as b8),
        State.Running => output_trace('RUNN', 1 as b8),
        State.Error(code) => output_trace('ERR_CODE', code as b8),
        _ => panic, // Wildcard fallback
    }
}
```

---

## 2. SMT Path-Solver Interaction

When compiling a match statement, the LayerScript compiler converts each match branch and its guard expressions into mathematical paths. These paths are submitted to the **Z3 SMT Solver** during the elaboration phase.

### Unreachable Path Erasure (Zero-Overhead Branches)
If the SMT solver can mathematically prove that a match branch can *never* be entered under the current program constraints, the compiler completely deletes the branch from the generated binary.

```layerscript
function get_index(idx: u32 where idx < 5) -> u32 {
    match idx {
        0 => return 100,
        1 => return 200,
        2 => return 300,
        3 => return 400,
        4 => return 500,
        _ => {
            // The compiler proves this branch is mathematically unreachable
            // because of the constraint `where idx < 5`.
            // Consequently, this entire block is erased.
            panic; 
        }
    }
}
```

### Path-Dependent Refinements
Within a match arm, the compiler adds the matched pattern as an active constraint to the scope environment. This allows refinement checks inside that branch to pass compile-time verification without requiring explicit checks:

```layerscript
function process_score(score: u32) {
    match score {
        val where val <= 100 => {
            // Inside this block, the compiler registers the constraint `score <= 100`.
            // Any call here requiring `score <= 100` compiles cleanly.
            submit_score(val); 
        }
        _ => {
            output_trace('FAIL', 0 as b8);
        }
    }
}

function submit_score(s: u32 where s <= 100) {
    // ...
}
```
If `submit_score` were called outside the `val where val <= 100` arm, the SMT solver would reject the compilation because it could not statically prove the constraint.
