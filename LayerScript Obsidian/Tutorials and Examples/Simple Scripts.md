# LayerScript Language: Simple Scripts Guide

This document introduces basic code constructs in LayerScript through simple, annotated example scripts. These examples cover stdout tracing, refinement boundaries, and basic inline hardware control.

---

## 🗺️ Simple Script Directory
1. [`hello_world.layerscript`](#1-hello_worldlayerscript---hello-world-output-trace)
2. [`refinement.layerscript`](#2-refinementlayerscript---compile-time-math-boundaries)
3. [`register_havoc.layerscript`](#3-register_havoclayerscript---basic-register-mutations)

---

## 1. `hello_world.layerscript` — Hello World Output Trace
In LayerScript, raw screen printing is modeled as an **external trace side-effect** across the observability boundary. We use `output_trace` to write output.

```layerscript
function main() {
    // Write a hello world trace event to the observability buffer.
    // Strings are automatically translated into byte sequences.
    output_trace('HELLO_WORLD', 0 as b8);
}
```

### Key Takeaway:
Since the only anchor of a LayerScript program is **observability**, the compiler preserves the `output_trace` syscall while completely optimization-folding or dissolving any intermediate variables that do not contribute to it.

---

## 2. `refinement.layerscript` — Compile-Time Math Boundaries
This script demonstrates refinement types using the `where` keyword. It shows how the compiler uses these boundaries to elide checks.

```layerscript
// Accepts a score that is mathematically constrained to be <= 100.
function process_score(score: u32 where score <= 100) {
    // Because score <= 100 is statically proven by the type signature,
    // the compiler generates code for this function with zero bounds checking.
    output_trace('SCORE', score as b8);
}

function main() {
    // Case A: Proven safe.
    // The number 85 is statically known to be <= 100. Compiles without issue.
    process_score(85);

    // Case B: Unprovable/Error.
    // If you uncomment the line below:
    // process_score(150);
    // Under @strict mode: compilation will fail at build time.
    // Under relaxed mode: compile succeeds but inserts a runtime panic check.
}
```

### Key Takeaway:
Refinements move constraints directly into the type signature. If the SMT solver can verify the constraints at the call site, the compiler generates naked machine code with no runtime overhead.

---

## 3. `register_havoc.layerscript` — Basic Register Mutations
This script demonstrates hardware state tracking using inline `interrupt` assembly blocks and the `havoc` cache invalidator.

```layerscript
struct CPUState {
    b64 rax;
    b64 rbx;
}

function mutate_cpu_registers(cpu: CPUState*) {
    // Cache values in registers
    cpu.rax = 0xAA_b64;
    cpu.rbx = 0x55_b64;

    // Inline interrupt boundary execution
    interrupt 'nop', cpu {
        // The instruction might change the value inside RAX.
        // We 'havoc' RAX to tell the compiler to discard any cached value.
        havoc cpu.rax;
        
        // Since RBX is not havoc'd, the compiler assumes its cached value
        // of 0x55_b64 remains valid and avoids reloading it.
    }
}
```

### Key Takeaway:
Instead of treating inline assembly as a total optimization barrier (which spills all CPU registers to memory), `havoc` lets you invalidate specific register lines. The rest of the CPU state remains cached, generating faster assembler output.
