# Traits and Operator Overloading

LayerScript does not use traditional class inheritance. Instead, it relies on **traits** to define shared behavior interfaces, and supports **operator overloading** via trait implementation.

---

## 1. Defining and Implementing Traits

A **trait** is a contract defining a set of functions that a type must implement.

```layerscript
pub trait Serializable {
    function serialize(buffer: b8*, max_len: u32) -> u32;
}

struct Packet {
    id: u32,
    payload: b64,
}

impl Serializable for Packet {
    function serialize(buffer: b8*, max_len: u32) -> u32 {
        // Serialization logic...
        return 12; // bytes written
    }
}
```

---

## 2. Operator Overloading

Operator overloading is achieved by implementing standard library traits associated with specific symbols (e.g., `+`, `-`, `*`, `/`).

### Arithmetic Operator Example (`Add`)
To support the `+` operator for a custom struct, implement the standard `Add` trait:

```layerscript
pub trait Add<Rhs = Self> {
    type Output;
    function add(self, rhs: Rhs) -> Self.Output;
}

struct Vector2D {
    x: f64,
    y: f64,
}

impl Add for Vector2D {
    type Output = Vector2D;

    function add(self, rhs: Vector2D) -> Vector2D {
        return Vector2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        };
    }
}
```

Now, you can add two `Vector2D` instances directly using `+`:
```layerscript
var v1 = Vector2D { x: 1.0, y: 2.0 };
var v2 = Vector2D { x: 3.0, y: 4.0 };
var v3 = v1 + v2; // Desugars to: v1.add(v2)
```

---

## 3. Operator Coercion Machinery

LayerScript is a bare-metal language designed for zero-overhead hardware mapping. It uses the `as` operator for coercions, which operates under strict compiler-proven rules.

### Bit-Coercions
When casting between raw bit vectors (e.g., `b8`, `b16`) and structured types (structs), the cast compiles to a **zero-overhead bit-reinterpret cast** (similar to `reinterpret_cast` in C++ or `transmute` in Rust) as long as:
1. The memory size of both types is identical.
2. The cast is proven safe by refinement constraints.

```layerscript
struct StatusFlags {
    active: b1,
    error: b1,
    unused: b6,
}

function main() {
    var raw_byte: b8 = 0x03;
    
    // Zero-overhead coercion: treats the memory byte directly as StatusFlags
    var flags = raw_byte as StatusFlags;
    
    if (flags.active) {
        // ...
    }
}
```
Because the structures are packed, this coercion has no runtime instructions; the compiler simply indexes the fields at compile-time as offsets into the original register byte.
