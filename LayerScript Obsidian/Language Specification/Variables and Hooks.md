# Variables and Hooks

A variable in LayerScript is a **`VariableBinding` layer**. It carries a type annotation, a mutability flag, and an optional block of lifecycle *hooks*. This page covers the surface syntax; for the exact firing order and cost, see the [Variable Behavior Hooks Runtime](../Execution%20Model/Variable%20Behavior%20Hooks%20Runtime.md).

## Mutability rules

LayerScript keeps mutability simple:

- **`var`** — mutable. Reassign freely.
- **`let`** — immutable. Fixed once initialized.

```layerscript
var x = 10;      // reassignable
let y = 20;      // fixed forever

x = x + 5;       // ✅ allowed
y = y + 5;       // ❌ compiler error: y is immutable
```

Types are inferred from the initializer but may be written explicitly, which is required when the value is bit-precise or refined:

```layerscript
var age: u32 = 25;
var delta: i8 = -1;
let mask: b16 = 0xFFFF;
```

## Variable Behavior Hooks

Hooks attach reactive logic to a binding. They live in a `{ ... }` block after the initializer and may be placed on both `var` and `let` (a `let` hook runs once, at initialization).

### Design Principle: Observability vs Hidden Control Flow
Hooks are designed for **reactivity, telemetry, tracing, and boundary validation**, not for hiding business logic or silent data corruption:
- State transitions (like game mechanics, damage calculations, and state machines) should be modeled explicitly in functions and types.
- Hooks should be used to observe changes, emit telemetry, trigger hardware interrupts, or reject invalid data at FFI boundaries.

### `on_change`

Runs **before** the store; its return value is what gets written. Ideal for sanitization at hardware/FFI boundaries or validating invariants:

```layerscript
var raw_packet_len: u32 = 0 {
    on_change: function(new: u32, old: u32) -> u32 {
        if (new > 1500) { panic; } // Reject MTU overflow at FFI boundary
        return new;
    }
}
```

### `on_read`

Runs when the value is accessed. Useful for lazy computation or tracing:

```layerscript
var counter: u64 = 0 {
    on_read: function() -> u64 {
        trace!("counter read at {}", layertrace.current().kind);
        return counter;
    }
}
```

### `on_assign`

Runs **after** the store commits. It cannot change the stored value — use it for notifications, hardware interrupts, and side effects:

```layerscript
var dirty: bool = false {
    on_assign: function(value: bool) {
        if (value) { schedule_repaint(); }
    }
}
```

Multiple hooks may coexist; they compose inner-to-outer as described in the [runtime page](../Execution%20Model/Variable%20Behavior%20Hooks%20Runtime.md).

## Hooks as safety enforcement

Because `on_change` may `panic`, a hook can reject a value the compiler could not prove safe statically — for example, data arriving across an [FFI boundary](../Bare%20Metal%20Interfacing/FFI%20and%20extern.md):

```layerscript
var fd: i32 = -1 {
    on_change: function(new: i32, old: i32) -> i32 {
        if (new < 0) { panic; }   // never store an invalid descriptor
        return new;
    }
}
```

## Implementation

The parser (Ring 1) records the choice on the `VariableBinding` layer:

- `var` sets `IsMutable: true`.
- `let` sets `IsMutable: false`.

Hook blocks become child layers of the binding, so they are [type-checked](../Compiler%20Mechanics/Type%20Checking%20and%20Inference.md) against the variable's type before they can ever run.

## See also
- [Variable Behavior Hooks Runtime](../Execution%20Model/Variable%20Behavior%20Hooks%20Runtime.md)
- [Layer System](Layer%20System.md)
- [Control Flow and Statements](Control%20Flow%20and%20Statements.md)
