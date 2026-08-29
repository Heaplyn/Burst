# Type Checking and Inference

Type checking runs after the [parser](Parser%20and%20Lexer.md) has built the layer tree and before [elaboration](Elaboration%20and%20Constraints.md) hands constraints to the SMT solver. Its job is to assign a `Type` to every layer, verify that operations are well-typed, and resolve names against the layered [TypeStorage](../Type%20System%20and%20Coercions/Type%20Storage%20and%20Inheritance.md). Refinement *values* (`where` clauses) are checked later by Z3; type checking only proves the *shapes* line up.

```mermaid
graph LR
    P[Layer Tree] --> TC[Type Checker]
    TC --> R[Resolved Types on every Layer]
    R --> E[Ring 2: Elaboration / SMT]
```

---

## 1. Type Storage Traversal

Every layer owns a `TypeStorage`. To resolve a name the checker walks **outward and upward**: current layer first, then each parent to the root, then the built-in primitives seeded in the `Program` layer.

```
lookup("Point"):
  1. this.TypeStorage.DefinedTypes / TypeAliases
  2. parent.TypeStorage  (repeat to root)
  3. built-in primitives (i8..i128, u8..u128, f32..f128, b1..bN)
  -> first hit wins; miss => "unknown type" error at the layer's SourceLoc
```

Because lookup stops at the first hit, an inner definition **shadows** an outer one — see below.

---

## 2. Inheritance Resolution

Child layers inherit every type visible in their ancestors. When a child redefines a name, the child's definition shadows the parent's *within that subtree only*; sibling and ancestor layers are unaffected.

```layerscript
type Id = u64;              // visible everywhere below the Program layer

function parse() {
    type Id = u32;          // shadows the outer Id inside `parse` only
    var x: Id = 0;          // x : u32
}

var g: Id = 0;              // g : u64  (outer definition still holds)
```

Resolution is purely lexical — it follows the layer tree, never the call graph — so a type's meaning is decidable at the definition site.

---

## 3. Hook Type Validation

Variable [hooks](../Language%20Specification/Variables%20and%20Hooks.md) are checked against the type of the variable they guard:

- `on_change(new, old)` — both parameters must be the variable's type; the return type must coerce back to it (the returned value becomes the stored value).
- `on_read()` — return type must coerce to the variable's type.
- A hook body is itself a layer, so it is type-checked in a scope nested under the binding.

```layerscript
var health: f64 = 100.0 {
    on_change: function(new: f64, old: f64) -> f64 {   // ✅ f64 in, f64 out
        if (new < 0.0) { return 0.0; }
        return new;
    }
}
```

A signature mismatch (wrong parameter type, or a return that cannot coerce to the variable's type) is a hard error here, long before the [hooks runtime](../Execution%20Model/Variable%20Behavior%20Hooks%20Runtime.md) ever runs.

---

## 4. Generic Instantiation

Generics are checked in two passes:

1. **Definition pass** — the generic body is checked against its type parameters abstractly. Anything that must hold for *all* `T` (e.g. that `T` supports `+`) is recorded as a bound.
2. **Instantiation pass** — at each call/use site, the checker substitutes concrete arguments, re-checks the bounds, and mints a specialized copy (monomorphization) that later rings can optimize.

```layerscript
function max<T>(a: T, b: T) -> T where T: Ord {
    if (a > b) { return a; }
    return b;
}

var m = max<i32>(3, 9);     // instantiates max@i32; Ord<i32> bound discharged
```

Const generics (`N: usize`) participate too: their values flow into refinement predicates, so `Index<N>` and `[T; N]` are checked with the concrete `N` in scope.

## See also
- [Type Storage and Inheritance](../Type%20System%20and%20Coercions/Type%20Storage%20and%20Inheritance.md)
- [Elaboration and Constraints](Elaboration%20and%20Constraints.md)
- [First-Class Types and Generics](../Language%20Specification/First-Class%20Types%20and%20Generics.md)
