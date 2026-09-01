function divide(numerator: u32 where numerator < 1000000, denominator: u32 where denominator > 0) -> u32 {
    return numerator / denominator;
}

function verify_range(val: u32 where val >= 10 && val <= 1000 else 0) -> bool {
    if (val == 0) {
        return false;
    }
    return true;
}

function compute_complex_metric(a: u32, b: u32 where a + b < 5000, c: u32 where c > a && c > b) -> u32 {
    let sum = a + b;
    let diff = c - sum;
    if (diff > 100) {
        return diff / 2;
    } else {
        return sum * 2;
    }
}

function run_pipeline(x: u32 where x > 20 && x < 500) -> u32 {
    println('--- Pipeline Start ---');
    println('Input x:', x);
    println('Type of x is:', type(x));

    let is_valid = verify_range(x);
    println('Is input valid:', is_valid);
    println('Type of is_valid:', type(is_valid));

    let step1: u32 where step1 > x = x + 5;
    let step2: u32 where step2 > step1 && step2 < 1000 = step1 * 2;
    let step3: u32 where step3 > step2 = step2 + 100;

    println('Step values:', step1, step2, step3);

    if (step3 < 500) {
        let metric = compute_complex_metric(step1, step2, step3);
        println('Metric calculated:', metric);
        let result = divide(metric, step1);
        return result;
    } else {
        let fallback = step2 / 2;
        println('Fallback path taken. Value:', fallback);
        return fallback;
    }
}

// Call the pipeline with various inputs
let res1 = run_pipeline(100);
println('Pipeline(100) result:', res1);

let res2 = run_pipeline(25);
println('Pipeline(25) result:', res2);

let Thing = verify_range(2);
println(Thing);

// Direct checks of built-ins and boolean literals
println('--- Builtins & Literals Verification ---');
let t_bool = type(true);
let t_int = type(12345);
let t_str = type('LayerScript');
let t_unit = type(print());

println('Type of true:', t_bool);
println('Type of 12345:', t_int);
println('Type of String:', t_str);
println('Type of Unit:', t_unit);