# Built-in Types Reference

This page is the authoritative reference for every type the LayerScript compiler knows about without a single line of user code. These types live in the root `Program` layer's [TypeStorage](../Type%20System%20and%20Coercions/Type%20Storage%20and%20Inheritance.md) and are visible from every descendant layer.

---

## 1. Primitive Types Table

LayerScript has **no fixed-width primitives** like `int` or `float`. Every scalar is parameterized by an explicit bit width `N`, represented internally by the `BitPrecise(kind, N)` variant of the `Type` enum.

| Family | Syntax | Internal | Arithmetic | Bitwise | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Signed integer | `i<N>` | `BitPrecise('i', N)` | ✅ checked | ✅ | Two's complement. `i8`, `i16`, `i32`, `i64`, `i128`. |
| Unsigned integer | `u<N>` | `BitPrecise('u', N)` | ✅ modular | ✅ | `u8` … `u128`. `usize` aliases the pointer width. |
| Floating point | `f<N>` | `BitPrecise('f', N)` | ✅ IEEE-754 | ❌ | `f32`, `f64`, `f80`. |
| Bit vector | `b<N>` | `BitPrecise('b', N)` | ❌ | ✅ | Raw bits, no numeric meaning. `b1` … `b1024`. |
| Unit | `()` | `Unit` | ❌ | ❌ | The empty type; return type of procedures. |
| Boolean | `bool` | alias of `b1` | ❌ | ✅ | `true` / `false`. Used by the SMT path solver. |

> [!NOTE]
> `N` is any positive integer literal the target can lower. Arithmetic operators are **invalid** on `b<N>`; bit vectors only accept `&`, `|`, `^`, `~`, `<<`, `>>`. See [Base and Compound Types](../Language%20Specification/Base%20and%20Compound%20Types.md) for range bounds and binary representations.

### Compound built-ins

| Type | Syntax | Internal | Description |
| :--- | :--- | :--- | :--- |
| Pointer | `T*` | `Pointer(Box<Type>)` | Raw address. Arithmetic bounded by the constraint system. |
| Array | `[T; N]` | `Array(Box<Type>, N)` | Contiguous, fixed-length sequence. |
| Slice | `[T]` | `Slice(Box<Type>)` | Pointer + length pair; length feeds `where` refinements. |

---

## 2. `layertrace` Type

`layertrace` is the built-in handle to the [layertrace runtime](../Execution%20Model/layertrace%20Runtime.md). It is a first-class value of the intrinsic type `LayerTrace` and is always in scope.

```layerscript
function report() {
    var here: LayerTrace = layertrace.current();  // the executing layer
    trace!("kind = {}", here.kind);               // e.g. "Function"
    trace!("doc  = {}", here.get_metadata("doc"));
}
```

| Member | Signature | Returns |
| :--- | :--- | :--- |
| `current()` | `fn() -> Layer` | The currently executing layer. |
| `root()` | `fn() -> Layer` | The `Program` layer. |
| `push(id)` | `fn(str) -> ()` | Opens a nested trace scope. |
| `pop()` | `fn() -> ()` | Closes the innermost trace scope. |
| `lookup_type(name)` | `fn(str) -> TypeInfo` | Resolves a type through the layer hierarchy. |
| `get_metadata(key)` | `fn(str) -> str` | Reads a metadata field of the current layer. |

---

## 3. `Layer` Type

`Layer` is the runtime reflection of the compiler's `Layer` struct (see [Layer System](../Language%20Specification/Layer%20System.md)). User code never constructs one directly — the compiler mints them — but introspection exposes a read-only view:

| Field | Type | Meaning |
| :--- | :--- | :--- |
| `id` | `LayerId` | Stable identity within the tree. |
| `kind` | `LayerKind` | `Program`, `Function`, `Block`, `VariableBinding`, `Loop`, `Panic`, … |
| `type_env` | `TypeStorage` | Types and aliases visible at this layer. |
| `observability` | `ObservabilityFlags` | Whether the layer affects output/hardware. |

---

## 4. Predefined Refinements

The standard library ships a handful of named [refinements](../Type%20System%20and%20Coercions/Refined%20Types.md) so common proofs read cleanly. They are ordinary `where` predicates behind a type alias.

| Alias | Expands to | Use |
| :--- | :--- | :--- |
| `NonZero<T>` | `T where self != 0` | Divisors, capacities. |
| `Positive<T>` | `T where self > 0` | Counts, lengths. |
| `Index<N>` | `usize where self < N` | Bounds-check-free array access. |
| `Aligned<T, A>` | `T* where (self as usize) % A == 0` | Alignment proofs for DMA / SIMD. |

```layerscript
// safe_read proves idx is in-bounds, so no runtime check survives codegen.
function safe_read<T, N: usize>(arr: T*, idx: Index<N>) -> T {
    return arr[idx];
}
```

## See also
- [Base and Compound Types](../Language%20Specification/Base%20and%20Compound%20Types.md)
- [Type Storage and Inheritance](../Type%20System%20and%20Coercions/Type%20Storage%20and%20Inheritance.md)
- [Refined Types](../Type%20System%20and%20Coercions/Refined%20Types.md)
