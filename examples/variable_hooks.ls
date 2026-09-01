// Demonstrates principled variable observability and state management.
// Business logic (damage calculation and bounds) is explicit in functions,
// while Variable Hooks handle telemetry and external trace side-effects.

function calculate_damage(current_health: u32, incoming_damage: u32) -> u32 {
    if (incoming_damage >= current_health) {
        println('[COMBAT] Fatal blow dealt! Health reduced to 0');
        return 0;
    }
    return current_health - incoming_damage;
}

function check_status(health: u32) -> bool {
    if (health == 0) {
        println('[STATUS] Character has perished.');
        return false;
    }
    println('[STATUS] Character is alive with health:', health);
    return true;
}

fn main() {
    var health: u32 = 100;
    println('Initial Health:', health);

    // Explicit damage calculation rather than hidden silent mutation hooks
    health = calculate_damage(health, 40);
    check_status(health);

    health = calculate_damage(health, 70);
    check_status(health);
}

main();