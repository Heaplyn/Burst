// Demonstrates structured mapping of raw bit vectors.
struct Flags {
    active: b1;
    error: b1;
    reserved: b6;
}

function main() {
    // A raw 8-bit value (e.g., from a hardware register)
    var status: b8 = 0x01;

    // Zero-cost coercion: treat these bits as a structured Flags object.
    var f = status as Flags;

    // Access individual bits as if they were struct fields.
    if (f.active) {
        output_trace('STATUS', 'ACTIVE');
    }
}
