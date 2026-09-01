// Demonstrates principled error handling & safety in LayerScript.
// Rather than using untyped 'null' (which introduces hidden null-pointer crashes),
// LayerScript uses compile-time SMT refinement proofs and explicit result states.

// 1. Refinement Proof Guarantee:
// Non-zero denominator is enforced at the type level.
function safe_divide(numerator: u32, denominator: u32 where denominator > 0) -> u32 {
    return numerator / denominator;
}

// 2. Explicit Fallback with 'else' refinement syntax:
// When a value can be outside ideal bounds, the fallback is stated directly in the signature.
function clamp_read(sensor_val: u32 where sensor_val <= 100 else 100) -> u32 {
    return sensor_val;
}

// 3. Status Handling (Result / State pattern):
function find_item_index(id: u32) -> u32 {
    if (id == 42) {
        return 1; // Found at index 1
    }
    // Explicit error code / unhandled outcome instead of null
    return 9999;
}

fn main() {
    println('--- 1. Type-Safe Division ---');
    let div_res = safe_divide(100, 4);
    println('100 / 4 =', div_res);

    println('--- 2. Explicit Refinement Fallback ---');
    let valid_reading = clamp_read(45);
    let overflow_reading = clamp_read(250);
    println('Valid sensor reading:', valid_reading);
    println('Clamped sensor reading (with explicit else fallback):', overflow_reading);

    println('--- 3. Explicit Outcome Handling ---');
    let idx = find_item_index(42);
    if (idx == 9999) {
        println('Item not found.');
    } else {
        println('Item found at index:', idx);
    }
}

main();
