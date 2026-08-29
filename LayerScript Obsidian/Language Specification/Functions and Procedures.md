# Functions and Procedures

Every function in LayerScript is a **`Function` layer** in the tree: it owns its parameters, return type, safety flags, and a child `Block` layer for its body. A function that returns nothing is conventionally called a *procedure*, but it is the same construct with return type `()`.

---

## Basic Function

```layerscript
function add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

Parameters are `name: Type`. The arrow `-> T` declares the return type; omit it for `()`.

## Without Return (Procedure)

```layerscript
function print_hello() {
    trace!("Hello, LayerScript!");
}
```

## Parameters, Defaults, and Multiple Returns

Return a tuple to hand back several values; destructure at the call site.

```layerscript
function divmod(a: u32, b: NonZero<u32>) -> (u32, u32) {
    return (a / b, a % b);   // b proven non-zero, so no divide-by-zero guard
}

var (q, r) = divmod(17, 5);
```

Note `b: NonZero<u32>` — a [refined parameter](../Type%20System%20and%20Coercions/Refined%20Types.md). The proof obligation lives in the signature, so callers must establish it and the body may assume it.

## Unsafe Functions

`unsafe` marks a function whose body steps outside what the compiler can prove — raw `asm!`, unchecked pointer math, or ABI tricks. Callers acknowledge the risk by the name being `unsafe`.

```layerscript
unsafe function write_to_hardware(address: u8*, value: u8) {
    asm!("mov [{}], {}", address, value);
}
```

## Extern Functions (FFI)

Declarations with no body import a symbol resolved at link time (see [FFI and extern](../Bare%20Metal%20Interfacing/FFI%20and%20extern.md)).

```layerscript
extern function printf(format: u8*, ...) -> i32;
extern function malloc(size: usize) -> u8*;
```

## Generic Functions

Type parameters go in `<...>`; bounds use `where T: Trait`.

```layerscript
function identity<T>(x: T) -> T {
    return x;
}

function swap<T>(a: T, b: T) -> (T, T) {
    return (b, a);
}

function max<T>(a: T, b: T) -> T where T: Ord {
    if (a > b) { return a; }
    return b;
}
```

Each instantiation is monomorphized during [type checking](../Compiler%20Mechanics/Type%20Checking%20and%20Inference.md), so generics cost nothing at runtime.

## Functions with Refined Types

```layerscript
function safe_read<T, N: usize>(arr: T*, idx: usize where idx < N) -> T {
    return arr[idx];  // idx < N is proven, so codegen emits no bounds check
}
```

## Directives and Optimization Hints

Directives attach to a function layer as metadata the later rings read.

```layerscript
@inline
function hot_path(x: i32) -> i32 { return x * x; }

@cold
function handle_fatal() { panic; }
```

| Directive | Effect |
| :--- | :--- |
| `@inline` | Prefer inlining at every call site. |
| `@cold` | Mark as rarely taken; move off the hot path. |
| `@strict` | Fail compilation if any refinement is undecidable. |
| `@silent` | Downgrade unproven refinements to runtime checks instead of errors. |

## Functions as Layers

Because a function *is* a layer, it can introspect itself through [layertrace](../Execution%20Model/layertrace%20Runtime.md):

```layerscript
function main() {
    var current = layertrace.current();
    trace!("In function: {}", current.kind);   // "Function"
}
```

## See also
- [Refined Types](../Type%20System%20and%20Coercions/Refined%20Types.md)
- [First-Class Types and Generics](First-Class%20Types%20and%20Generics.md)
- [Layer System](Layer%20System.md)
- [FFI and extern](../Bare%20Metal%20Interfacing/FFI%20and%20extern.md)
