// A simple function that accepts an integer between 0 and 100.
fn process_score(u32 score: where score <= 100) {
    // The compiler is guaranteed score <= 100, so it compiles without checking.
    println('SCORE',score);
}

function main() {
    var x = 30;
    // Statically proven safe, compiles cleanly.
    process_score(85);

    // Will cause a compile error in @strict mode or a runtime panic in relaxed mode
    process_score(150);
}
main();
