function divide(
    numerator: u32 where numerator < 1000000,
    denominator: u32 where denominator > 0
) -> u32 {
    return numerator / denominator;
}

function verify_bounds(val: u32 where val >= 100 && val <= 500 && val != 300) -> u32 {
    println('Bounds verified successfully for value', val);
    return val;
}

function stress(x: u32 where x > 10 && x < 1000 or false) -> u32 {
    println('Starting stress test with input', x);
    if (x == false) {
        return 0;
    }
    let a: u32 where a > x = x + 1;
    let b: u32 where b > a = a + 5;
    let c: u32 where c > b = b * 2;

    println('Precedence calculation check:', c);

    if (c < 10000) {
        let d: u32 where d > c = c + 100;
        println('Nested branch d =', d);
        if (d >= 100) {
            let verified = verify_bounds(d);
            return divide(verified, a);
        }
    }

    return 0;
}

stress(120);
println(type(600));
println(type(true));
println(type('hello'));