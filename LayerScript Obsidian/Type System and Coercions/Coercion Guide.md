# LayerScript: Coercion & Type System Guide

This folder contains the formal definitions and specifications of LayerScript's type system, focusing on type-level coercions, semantic boundaries, and physical representations.

---

## 1. Implicit Coercion vs. Explicit Coercion

LayerScript maintains a strict separation between a value's **physical representation** (e.g., bits, size) and its **semantic interpretation** (e.g., units, meaning).

### Implicit Coercion (Automatic Type-Level Promotion)
An implicit coercion is permitted only when two types share a unified semantic domain, and the conversion is mathematically trivial, cheap, and safe (no loss of precision or unexpected units behavior).

```layerscript
// fraction to float: Both are numeric values in the scalar domain.
// Conversion is implicit because a float is a superset of representable fractions.
fraction -> float

float x = 0.5_frac; // Automatically rewritten as float::from_fraction(0.5_frac)
```

Rules for Implicit Coercions in `layerscriptc`:
- **Widening Numeric Conversions**: e.g., `u32 -> u64`, `i8 -> i16`.
- **Dimensional Homogeneity**: `Meters -> Feet` could be declared implicit because they represent lengths, *provided* the conversion factor is statically defined.
- **Strict No-Loss Guarantee**: Any conversion that can truncate data (e.g., `u64 -> u32` or `float -> i32`) is disallowed implicitly.

### Explicit Coercion (Forced Type Reinterpretation)
An explicit coercion requires the programmer to explicitly state their intent, using the `as` cast operator or explicit constructors. This is required whenever there is a change in semantic meaning.

```layerscript
// Jump expects meters.
fn jump(Meters distance);

// compile-error: The literal 500 has no semantic units.
jump(500); 

// OK: Explicit quantity declaration.
jump(inches(500)); 

// OK: Explicit casting.
usize buffer_index = raw_address as usize;
```

---

## 2. Representation Compatibility vs. Semantic Compatibility

Two types can be physically identical in memory (e.g., 16 bits of storage) but belong to entirely separate semantic domains. LayerScript prevents these from interacting implicitly.

| Type | Physical Bit Size | Semantic Meaning | Addition Permitted? |
| :--- | :--- | :--- | :--- |
| `b16` | 16 bits | A bitfield of 16 individual boolean flags | No (requires bitwise operators) |
| `u16` | 16 bits | An unsigned 16-bit scalar integer | Yes (modular arithmetic) |

Example of structural layout vs. type safety:
```layerscript
b16 system_flags = 0b0000_1100_1010_1111;
u16 numerical_value = 100;

// Compile error: Even though both occupy exactly 16 bits in CPU registers,
// they are semantically incompatible.
u16 result = system_flags + numerical_value; 

// Correct: Explicit bit-cast to numeric type
u16 result = (system_flags as u16) + numerical_value;
```

---

## 3. Coercion in Operator Signatures

Operators in LayerScript are not compiler-magic primitives. They are trait-driven methods that utilize implicit coercion parameters to prove compatibility at compilation time.

```layerscript
fn operator+(
    Type T,
    T N,
    (T -> u<q>) coerce,
    i(coerce(N)) rhs,
    i(coerce(N)) lhs,
    !( (rhs[N-1] == lhs[N-1]) && ((rhs as u<q> + lhs as u<q>)[N-1] != lhs[N-1]) ) h
) -> i(coerce(N)) {
    // Under the hood, addition is performed on the coerced unsigned representation
    // to verify the signed overflow bit state (bit N-1).
}
```

In this signature:
1. `(T -> u<q>) coerce`: A compiler-resolved implicit coercion function that converts the signed integer type to its corresponding unsigned type of width `q`.
2. `h`: The logic certificate proving that the sum does not trigger an integer overflow.
