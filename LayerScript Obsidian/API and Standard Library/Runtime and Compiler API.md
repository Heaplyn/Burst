# LayerScript: API and Standard Library Specification

This folder details the APIs for both:
1. **The Compiler/Host API** (how Rust-based toolchain plugins interact with `layerscriptc`).
2. **The LayerScript Standard Library API** (how developers interact with constraints, the executor, and hardware boundaries in LayerScript source code).

---

## 1. Compiler Host & Plugin API (Rust Interface)

`layerscriptc` provides a compiler-plugin architecture in Rust allowing developers to inject custom semantic verification passes, target-specific optimizations, or custom SMT theories during the **Elaboration Phase**.

### Defining a Custom Compiler Pass
To write a lint or constraint-solving extension, compile a Rust library matching the `layerscriptc_plugin` interface:

```rust
use layerscriptc_ast::ast::{Expr, Type};
use layerscriptc_elaboration::{ElaborationContext, ConstraintPass};

pub struct CustomRangeVerifier;

impl ConstraintPass for CustomRangeVerifier {
    fn name(&self) -> &'static str {
        "custom_range_verifier"
    }

    fn run(&self, ctx: &mut ElaborationContext) -> Result<(), String> {
        for (node_id, expr) in ctx.ast.expressions() {
            if let Expr::BinaryOp { op, lhs, rhs } = expr {
                // If it is a division, we inject a static check to verify divisor != 0
                if op == "/" {
                    let divisor_type = ctx.type_layout.get(rhs);
                    let non_zero_condition = format!("{} != 0", ctx.ast.symbol_name(rhs));
                    
                    // Register the constraint with the solver
                    ctx.solver.add_constraint(node_id, &non_zero_condition);
                }
            }
        }
        Ok(())
    }
}
```

---

## 2. LayerScript Standard Library API (`core`)

Within LayerScript source files, the standard library exposes direct hooks into the compiler's constraint engine and the BAM scheduler.

### A. The Constraint Engine API (`core::constraint`)

Allows dynamic manipulation of compile-time logic obligations.

```layerscript
namespace core::constraint;

// A type-wrapper representing a verified mathematical proof
pub struct Proof<Prop: expression> {
    b1 verified;
}

// Statically query whether a proposition is solvable
pub fn is_provable(expression expr) -> b1;

// Force compile-time verification. Under strict mode, this fails build if unsat.
// Under relaxed mode, it injects a runtime panic branch.
pub fn verify(expression expr);
```

#### Example Usage in LayerScript
```layerscript
import core::constraint::{verify, Proof};

fn compute_division(i32 numerator, i32 denominator) -> i32 {
    // Assert to the compiler that denominator cannot be zero.
    verify(denominator != 0);
    
    return numerator / denominator; // Compile-time safe from division-by-zero
}
```

---

### B. The POMSET Executor Interface (`core::executor`)

Provides direct control over scheduling order, thread-affinity mapping, and priority heuristics for independent traces.

```layerscript
namespace core::executor;

pub enum ThreadPriority {
    RealTime,
    High,
    Normal,
    Low,
}

// Maps the current execution trace path to a specific CPU core group
pub fn pin_trace(usize core_mask);

// Suggest a scheduling priority hint to the BAM work-stealer
pub fn set_trace_priority(ThreadPriority priority);

// Creates an execution trace barrier that stalls dependent traces until
// the current trace group writes all outputs to volatile space.
pub fn trace_barrier();
```

---

### C. The Hardware Mapping API (`core::hardware`)

Exposes hardware register definition overlays and atomic synchronization primitives.

```layerscript
namespace core::hardware;

// Declares that the memory target is a volatile hardware interface
pub struct VolatileAddress<T> {
    usize address;
}

impl<T> VolatileAddress<T> {
    pub fn read(&self) -> T;
    pub fn write(&self, T value);
}

// Forces a pipeline reload for registers mapped to this memory space
pub fn register_flush();
```
