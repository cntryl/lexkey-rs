use lexkey::{Encoder, LexKey};
use std::time::Instant;

fn main() {
    // Example: Zero-allocation hot path using Encoder reuse
    // This pattern is ideal for tight loops where allocation overhead matters

    let mut encoder = Encoder::with_capacity(64);
    let iterations = 1_000_000;

    let start = Instant::now();

    for i in 0..iterations {
        // Clear the buffer for reuse (doesn't deallocate)
        encoder.clear();

        // Build a composite key: "user" + separator + i64
        encoder.encode_string_into("user");
        encoder.push_byte(LexKey::SEPARATOR);
        encoder.encode_i64_into(i);

        // In real code, you'd use the key here (e.g., database write)
        let _key = encoder.as_slice();

        // The buffer is automatically reused on the next iteration
    }

    let duration = start.elapsed();

    println!("Encoded {} composite keys", iterations);
    println!("Total time: {:?}", duration);
    println!(
        "Average: {:.2} ns/key",
        duration.as_nanos() as f64 / iterations as f64
    );
    println!();
    println!("Key reuse eliminates allocation overhead in tight loops.");
}
