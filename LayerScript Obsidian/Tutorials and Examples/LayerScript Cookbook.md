# LayerScript Cookbook: Advanced Examples

This cookbook provides non-trivial, low-level examples demonstrating the unique features of the LayerScript programming language: refinement types, zero-cost checks, inline hardware interrupts, and custom numeric systems.

---

## 🗺️ Cookbook Contents
1. [Bounded N-Dimensional Math Vectors](#1-bounded-n-dimensional-math-vectors)
2. [Zero-Copy Circular Ring Buffer](#2-zero-copy-circular-ring-buffer)
3. [Memory-Mapped I/O UART Driver](#3-memory-mapped-io-uart-driver)
4. [Custom Modulo prime-field Crypto Ring](#4-custom-modulo-prime-field-crypto-ring)

---

## 1. Bounded N-Dimensional Math Vectors

This example demonstrates using type functions (instead of standard generic syntax) to define fixed-size math vectors whose index access is proven safe at compile time.

```layerscript
// A type function that returns a vector type of size N
function VectorType(T: type, N: usize) -> type {
    return [T; N];
}

// Bounded index dot product. Index is guaranteed <= N at compile time.
function dot_product<N: usize>(
    lhs: VectorType(f64, N)*, 
    rhs: VectorType(f64, N)*
) -> f64 {
    var mut sum: f64 = 0.0;
    
    // The compiler can statically prove that i never exceeds N-1.
    // As a result, all lookups on lhs and rhs have bounds checking completely elided.
    for (var mut i: usize = 0; i < N; i++) {
        sum += lhs[i] * rhs[i];
    }
    
    return sum;
}

function main() {
    // Instantiate vectors
    var v1: VectorType(f64, 3) = [1.0, 2.0, 3.0] as VectorType(f64, 3);
    var v2: VectorType(f64, 3) = [4.0, 5.0, 6.0] as VectorType(f64, 3);
    
    var result: f64 = dot_product(3, &v1, &v2);
    output_trace('DOT_PROD_RESULT', result);
}
```

---

## 2. Zero-Copy Circular Ring Buffer

A classic high-performance ring buffer. Because the index calculations are mathematically proven to fall within the allocated array bounds, `layerscriptc` elides all array index checks.

```layerscript
packed struct RingBuffer {
    [u8; 1024] data;
    u32 read_ptr;
    u32 write_ptr;
}

// Write a byte to the ring buffer.
function ring_write(buf: RingBuffer*, val: u8) {
    // SMT solver asserts: (write_ptr % 1024) is strictly < 1024.
    var index: u32 = buf.write_ptr % 1024 as u32;
    
    // Bounds check on index is statically proven safe -> elided.
    buf.data[index] = val;
    buf.write_ptr += 1 as u32;
}

// Read a byte from the ring buffer.
function ring_read(buf: RingBuffer*) -> u8 {
    // SMT solver asserts: (read_ptr % 1024) is strictly < 1024.
    var index: u32 = buf.read_ptr % 1024 as u32;
    
    var val: u8 = buf.data[index];
    buf.read_ptr += 1 as u32;
    return val;
}
```

---

## 3. Memory-Mapped I/O UART Driver

An embedded driver communicating with a 16550 UART port at address `0x3F8`. Demonstrates the `at` memory mapping layout, `interrupt` boundaries, and fine-grained register `havoc` state.

```layerscript
// Map a UART register block layout directly to memory base address 0x3F8
packed struct UARTPort {
    b8 data_register;          // Offset 0
    b8 interrupt_enable;       // Offset 1
    b8 line_control;           // Offset 3
    b8 line_status;            // Offset 5
}
UARTPort uart: at 0x3F8;

function init_uart() {
    // Disable interrupts
    uart.interrupt_enable = 0x00 as b8;
    
    // Enable DLAB (Divisor Latch Access Bit)
    uart.line_control = 0x80 as b8;
    
    // Write baud rate divisor (115200 baud)
    uart.data_register = 0x01 as b8; // divisor low byte
    
    // Re-lock divisor, set format 8 bits, no parity, 1 stop bit
    uart.line_control = 0x03 as b8;
}

function uart_write(byte_to_send: b8) {
    // Spin until UART transmit buffer is empty (bit 5 of line_status is set)
    var mut is_empty: b1 = false;
    while (!is_empty) {
        // Read line status register
        var status: b8 = uart.line_status;
        is_empty = (status & 0x20_b8) != 0x00_b8;
    }
    
    // Write data byte
    uart.data_register = byte_to_send;
    
    // Notify compiler that writing to data_register triggers a hardware state mutation.
    // The data_register has been "havoc'd", forcing a reload on subsequent reads,
    // but the configuration in line_control and interrupt_enable remains cached.
    interrupt 'out dx, al', &uart {
        havoc uart.data_register;
    }
}
```

---

## 4. Custom Modulo Prime-Field Crypto Ring

In cryptography, operations are executed modulo a prime. This example implements a prime-field addition operator on a custom type, and demonstrates how LayerScript folds loop operations.

```layerscript
// 256-bit big-integer block
packed struct u256 {
    u64 part0;
    u64 part1;
    u64 part2;
    u64 part3;
}

// Bounded prime element type
enum Secp256k1Field {
    // Constrain Secp256k1 field elements to be strictly less than the field prime:
    // P = 2^256 - 2^32 - 977
    Element(u256 value: where value < 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F)
}

trait Add<T> {
    type Output;
    function add(&self, &T rhs) -> Output;
}

impl Add<Secp256k1Field> for Secp256k1Field {
    type Output = Secp256k1Field;
    
    function add(&self, &Secp256k1Field rhs) -> Secp256k1Field {
        var raw_sum: u256 = self.value + rhs.value; // Hardware addition
        
        var prime = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F;
        
        // Modulo math returns a validated Secp256k1Field Element.
        // The SMT solver can statically prove that (raw_sum % prime)
        // is always less than the prime, satisfying the GADT enum constraint.
        return Secp256k1Field::Element(raw_sum % prime);
    }
}

// Demonstrating Trace Loop Folding:
// Compiling a sequence of additions into a single step
function multiply_field_element(element: Secp256k1Field, scalar: usize) -> Secp256k1Field {
    var mut result: Secp256k1Field = Secp256k1Field::Element(0 as u256);
    
    // Instead of executing this loop 'scalar' times,
    // layerscriptc recognizes the loop represents a linear recurrence:
    // result_n = element * scalar (mod prime).
    // The compiler automatically reduces this O(N) loop to an O(1) modulo multiplication.
    for (var mut i: usize = 0; i < scalar; i++) {
        result = result + element;
    }
    
    return result;
}
```
