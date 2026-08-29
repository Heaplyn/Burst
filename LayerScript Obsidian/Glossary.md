# LayerScript Glossary

This glossary defines core terminology and concepts unique to the **LayerScript** language and compiler.

---

### Layer
The fundamental structural unit in LayerScript. Instead of separating statements, expressions, and blocks into flat lists, LayerScript structures the entire program as a nested hierarchy of **layers**. Every function, loop, conditional branch, and variable binding is represented internally as a layer.

### POMSET (Partially Ordered Multiset)
The underlying execution model of LayerScript. Unlike sequential programming languages that enforce a strict line-by-line order of instructions, LayerScript compiles code into a Directed Acyclic Graph (DAG) representing operations. This DAG defines a partial ordering constraint: operations that are not dependent on each other are unordered and can execute concurrently or out-of-order safely.

### Observability Boundary
The critical interface where the program interacts with the physical computer hardware, registers, memory-mapped I/O, or output buffers (e.g., traces). If an operation or variable exists entirely inside the program and does not affect the observability boundary, the compiler is permitted to optimize it away or fold it completely.

### Refinement Type
A type constrained by a logical predicate (a refinement check) using the `where` keyword. For example, `u32 where value < 10`. The compiler attempts to prove that these predicates hold true at compile-time using an SMT solver.

### SMT Solver (Satisfiability Modulo Theories)
An automated theorem prover (specifically Z3) integrated directly into the LayerScript compiler's elaboration phase. It evaluates mathematical path constraints and refinement predicates to statically verify program safety.

### Proof Erasure
The process where the compiler removes safety checks from the final compiled binary. If the SMT solver mathematically proves that a refinement type constraint (like an array index bounds check) is guaranteed to hold true, the compiler **erases** the runtime check, leaving behind zero-overhead assembly instructions.

### Havoc
An instruction (`havoc target;`) that invalidates the compiler's cache or assumptions for a specific register or variable target. This forces the compiler to re-fetch the value directly from hardware memory on its next access, preventing unsafe compiler caching of volatile registers.

### layertrace
The runtime debugging and observation system of LayerScript. It records execution traces and variable states in real-time, allowing developers to inspect operations and trace state transitions across different layers.
