# Memory Layout and Packing

By default, LayerScript treats a `struct` as an *abstract* bag of fields. Because the [observability boundary](../Compiler%20Mechanics/Observability%20and%20Trace%20Folding.md) analysis knows which fields ever escape, the compiler is free to reorder them, insert padding, split them across registers, or dissolve them entirely. When you interface with hardware or a wire protocol, that freedom is exactly wrong — so LayerScript gives you precise control.

---

## 1. `packed struct`

A `packed struct` freezes the layout: fields appear in **declaration order**, with **zero padding**, at their natural bit offsets. This is what you reach for when the bytes *are* the interface.

```layerscript
// A 4-byte MMIO control register, laid out exactly.
packed struct UartControl {
    enable:   b1,   // bit 0
    parity:   b1,   // bit 1
    stop_bits: b2,  // bits 2-3
    baud_div: b12,  // bits 4-15
    reserved: b16,  // bits 16-31
}
```

Compare with a plain `struct`, where the compiler owns the layout:

```layerscript
struct Point {
    x: f64,
    y: f64,
}   // compiler may keep x, y purely in xmm registers and never materialize a Point
```

| | `struct` | `packed struct` |
| :--- | :--- | :--- |
| Field order | compiler's choice | as written |
| Padding | inserted freely | none |
| Register dissolution | allowed | disallowed |
| Use for | general data | hardware, protocols, FFI |

---

## 2. Alignment

Alignment is a proof the constraint system can carry, not just a hint. Use the `align(A)` attribute on a type or binding; violations are compile errors under `@strict`.

```layerscript
// 64-byte aligned so a full cache line is never split.
align(64) struct RingBuffer {
    head: u32,
    tail: u32,
    data: [u8; 4096],
}

// The Aligned<T, A> refinement proves an address meets an alignment.
function dma_start(src: Aligned<u8*, 64>, len: usize) { /* ... */ }
```

Because the alignment is expressed as a refinement (`(addr as usize) % A == 0`), the SMT solver can discharge it at compile time and codegen skips the runtime `& (A-1)` check.

---

## 3. Hardware-Specific Layouts

The `at` modifier maps a variable or struct onto a fixed physical address, turning a declaration into a memory-mapped register window.

```layerscript
// Map the UART control block to its MMIO address.
var uart: UartControl* at 0x1000_0000;

function enable_uart() {
    uart.enable = 1 as b1;
    // A device may change these bits behind our back:
    havoc uart;
}
```

Guidelines for device layouts:

- Always use `packed struct` for register blocks — padding would desync every field after it.
- Pair `at`-mapped memory with [`havoc`](Hardware%20and%20Havoc.md) whenever the device (not your code) can mutate it, so trace folding never assumes a stale read.
- Prefer `b<N>` bitfields over hand-rolled masks; the compiler proves the fields tile the word with no overlap or gap.

> [!NOTE]
> Endianness follows the target by default. Annotate a field with `be`/`le` (e.g. `length: u32 be`) when parsing a protocol whose byte order differs from the host.

## See also
- [Hardware and Havoc](Hardware%20and%20Havoc.md)
- [FFI and extern](FFI%20and%20extern.md)
- [Base and Compound Types](../Language%20Specification/Base%20and%20Compound%20Types.md)
