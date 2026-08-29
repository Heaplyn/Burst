# POMSET and Task Scheduler

## Partially Ordered Multiset (POMSET)
layerscript doesn't just run code line-by-line. it models the program as a graph of traces where some things *must* happen before others, and some things can happen in parallel.

### Execution Constraints
every `Layer` can hold dependencies that the scheduler uses to build the graph:
```rust
pub enum Constraint {
    /// telling the scheduler that Layer A must run before Layer B
    POMSET {
        Before: LayerId,
        After: LayerId,
    },
    // ...
}
```

## Task Scheduler
the Ring 2/3 driver uses these constraints to feed a **Work-Stealing Scheduler**. 
1. **Graph Construction**: elaboration builds the full dependency tree.
2. **Trace Folding**: any layer that doesn't affect the observability boundary is folded or erased.
3. **Parallel Execution**: traces with no dependencies between them are scheduled on separate hardware threads.

## NLL Lifetime Dissolution
because the program is a pomset, we can dissolve variable lifetimes as soon as their last observable effect is finished, even if the "block" hasn't technically ended.
