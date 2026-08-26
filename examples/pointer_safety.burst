// Demonstrates zero-overhead pointer safety using refinements.
fn read_at(ptr: u32*, index: u32 where index < 10) -> u32 {
    // The compiler proves index < 10 at the call site.
    // This allows it to skip the runtime bounds check here.
    return ptr[index];
}

fn main() {
    var data: [u32; 10];

    // Statically proven safe: 5 is always < 10.
    var val = read_at(&data as u32*, 5);

    output_trace('VAL', val as b8);
}
