use lexkey::LexKey;
use uuid::Uuid;

fn main() {
    // Example: Build range query bounds for a prefix scan
    // Use case: Query all items for a specific tenant and user

    let tenant = "acme-corp";
    let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    // Build the common prefix parts
    let parts = &[tenant.as_bytes(), user_id.as_bytes()];

    // Build the prefix key
    let prefix = LexKey::encode_composite(parts);

    // Use encode_first to get the lower bound (prefix + 0x00)
    let lower_bound = LexKey::encode_first(parts);

    // Use encode_last to get the upper bound (prefix + 0xFF)
    let upper_bound = LexKey::encode_last(parts);

    println!("Tenant: {}", tenant);
    println!("User ID: {}", user_id);
    println!();
    println!("Prefix:      {}", prefix.to_hex_string());
    println!("Lower bound: {}", lower_bound.to_hex_string());
    println!("Upper bound: {}", upper_bound.to_hex_string());
    println!();
    println!("Range query: [lower_bound, upper_bound)");
    println!("This will match all keys that start with the prefix.");
}
