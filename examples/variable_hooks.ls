// Demonstrates reactive bare-metal programming with Variable Hooks.
fn main() {
    // A mutable variable with an attached behavior hook.
    var health: f64 = 100.0 {
        on_change: function(new: f64, old: f64) -> f64 {
            // Automatically clamp the value between 0 and 100
            if (new < 0.0) { return 0.0; }
            if (new > 100.0) { return 100.0; }
            return new;
        }
    }

    health = health - 50.0; // health is now 50.0
    print('HEALTH', health as b8);

    health = health - 70.0; // result is -20.0, but hook clamps it to 0.0
    print('HEALTH', health as b8);
}

main();