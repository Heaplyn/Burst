function divide(
    numerator: u32 where numerator < 1000000,
    denominator: u32 where denominator > 0
) -> u32 {
    return numerator / denominator;
}

function stress(x: u32 where x > 10 && x < 1000) -> u32 {
    println("Test lmao");
    let a: u32 where a > x = x + 1;

    let b: u32 where b > a = a + 1;

    let c: u32 where c > b = b * 2;

    if (c < 10000) {
        let d: u32 where d > c = c + 100;
        return divide(d, a);
    }

    return 0;
}
stress(20);