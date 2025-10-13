use lexkey::{Encoder, LexKey};

fn main() {
    // Example: build a composite with an optional user identifier using a presence marker.
    let tenant = "tenant";
    let optional_user: Option<&str> = Some("alice");

    let mut enc = Encoder::with_capacity(128);
    enc.encode_string_into(tenant);
    enc.push_byte(LexKey::SEPARATOR);

    match optional_user {
        Some(name) => {
            enc.push_byte(0x01); // presence marker
            enc.encode_string_into(name);
        }
        None => {
            enc.push_byte(0x00); // absence marker
        }
    }

    let bytes = enc.freeze();
    println!("hex: {}", hex::encode(&bytes));
}
