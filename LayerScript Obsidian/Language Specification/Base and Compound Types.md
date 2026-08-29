# LayerScript Language: Base and Compound Types Specification

This document defines the physical memory representation, mathematical properties, and compiler behaviors of LayerScript's base primitives and compound types.

---

## 🗺️ Specification Directory
1. [Bit Vectors (`b<N>`)](#1-bit-vectors-bn)
2. [Unsigned and Signed Integers (`u<N>` / `i<N>`)](#2-unsigned-and-signed-integers-un--in)
3. [Floating-Point Types (`f<N>`)](#3-floating-point-types-fn)
4. [Pointers (`T*`)](#4-pointers-t)
5. [Contiguous Arrays (`[T; N]`)](#5-contiguous-arrays-t-n)

---

## 1. Bit Vectors (`b<N>`)

Bit vectors represent raw, uninterpreted hardware register bits. They are parameterized by bit width `N`.

* **Arithmetic Restriction**: Arithmetic operators (`+`, `-`, `*`, `/`, `%`) are **strictly prohibited** on bit vectors. They carry no numeric interpretation.
* **Supported Operators**: Only bitwise logical operations and shifts are permitted:
  - Bitwise Not: `~`
  - Bitwise And/Or/Xor: `&`, `|`, `^`
  - Bitwise Shift Left/Right: `<<`, `>>`
* **Boolean Equivalence**: `b1` is the native boolean type. The values `1 as b1` and `0 as b1` alias to the keywords `true` and `false`.
* **Casting**: Can be explicitly cast (`as`) to numeric integer types when arithmetic is required:
  ```layerscript
  var raw_port_bits: b16 = read_port();
  var numeric_value: u16 = raw_port_bits as u16;
  ```

---

## 2. Unsigned and Signed Integers (`u<N>` / `i<N>`)

Integers represent scalar numeric values of width `N`.

### A. Unsigned Integers (`u<N>`)
Unsigned scalars operate under modular arithmetic.
* **Range**: \([0, 2^N - 1]\)
* **Overflow**: Arithmetic overflow wraps around modulo \(2^N\). Unsigned addition/subtraction does not trigger compiler constraints or panics by default.

### B. Signed Integers (`i<N>`)
Signed scalars use two's complement binary representation.
* **Range**: \([-2^{N-1}, 2^{N-1} - 1]\)
* **Overflow**: Signed overflow is undefined behavior. In LayerScript, this is protected by compile-time SMT constraints:
  - Under strict mode, the compiler rejects operations if it cannot prove that overflow is mathematically impossible.
  - Under relaxed mode, a runtime overflow check (`jo` flag check) is injected.

---

## 3. Floating-Point Types (`f<N>`)

Floating-point types map to standard IEEE-754 hardware formats based on width `N`.

* **`f32`**: IEEE-754 Single Precision (32 bits: 1 sign, 8 exponent, 23 fraction)
* **`f64`**: IEEE-754 Double Precision (64 bits: 1 sign, 11 exponent, 52 fraction)
* **`f80`**: x87 Double Extended Precision (80 bits: 1 sign, 15 exponent, 64 fraction)
* **NaN and Infinities**: Standard NaN, positive infinity, and negative infinity are supported. Comparisons involving NaN always yield `false` (`0 as b1`).

---

## 4. Pointers (`T*`)

Pointers represent raw physical memory addresses pointing to a type `T`.

* **Address Width**: The physical size of a pointer matches the target architecture's native address width (e.g. 64 bits on x86-64).
* **Pointer Arithmetic**:
  - You can add or subtract an integer from a pointer: `ptr + offset`.
  - The offset is scaled by the size of the target type `T` (e.g., `ptr + 1` increments the address by `sizeof(T)` bytes).
  - Pointer arithmetic is audited by the SMT solver; the resulting address must be proven to point to a valid memory location allocated to that block.
* **Opaque Pointers**: `b8*` acts as the opaque/generic byte pointer (equivalent to `void*` in C).
* **Null Pointer**: Represented by the integer constant `0` explicitly cast: `0 as T*`.

---

## 5. Contiguous Arrays (`[T; N]`)

Arrays represent fixed-size, contiguously allocated sequences of elements of type `T`.

* **Sizing Rules**: The array length `N` must be a compile-time constant or a type function parameter.
* **Contiguity**: Elements are packed directly next to each other in memory with no padding, unless padding is explicitly declared inside a custom struct element.
* **Multidimensional Arrays**: Declared by nesting array definitions in row-major order:
  ```layerscript
  // An array of 3 elements, where each element is an array of 4 signed 32-bit integers.
  // Layout in memory is 12 contiguous i32 values.
  var matrix: [[i32; 4]; 3];
  ```
* **Pointer Decay**: An array reference implicitly decays (coerces) to a raw pointer to its first element when passed into a function accepting a pointer:
  ```layerscript
  function read_buffer(ptr: u8*);
  
  function main() {
      var buffer: [u8; 128];
      read_buffer(buffer); // Decays to u8* pointing to buffer[0]
  }
  ```
* **Bounds Safety**: Array indexing (`array[idx]`) requires the compiler to prove `idx < N`. If the proof is established, the index lookup compiles with zero runtime bounds checking overhead.
