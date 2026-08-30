struct CPU {
    b64 rax;
    b64 rbx;
}

fn run_operation(CPU* cpu) {
    cpu.rax = 0xAA_b64;
    cpu.rbx = 0x55_b64;

    // Inline assembly block
    interrupt 'nop', cpu {
        // We invalidate rax, forcing a reload on next access
        havoc cpu.rax;
        // rbx is not havoc'd, so its cache remains valid
    }
}
