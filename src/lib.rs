//! Lexkey: build lexicographically sortable byte keys for numbers, strings, UUIDs and composites.
//!
//! This crate provides two complementary APIs:
//! - `LexKey`: ergonomic, allocating constructors that return an immutable key (`Bytes`).
//! - `Encoder`: a reusable buffer for zero-allocation hot paths; write multiple values into one buffer.
//!
//! Ordering is by raw byte lexicographic comparison. Numeric and float encoders transform values
//! so that lexicographic order matches numeric order. Note: NaN values are not encodable by this
//! crate's encoders and will cause a panic; represent missing or invalid floats with a schema-level
//! presence/marker value instead.
//!
//! Quick start
//!
//! ```
//! use lexkey::{LexKey, Encoder};
//! use uuid::Uuid;
//!
//! // Allocating convenience APIs
//! let k = LexKey::encode_i64(42);
//! assert!(k.as_bytes() < LexKey::encode_i64(100).as_bytes());
//!
//! // Encode a composite of parts separated by 0x00
//! let user_id = Uuid::nil();
//! let comp = LexKey::encode_composite(&[b"tenant", b"user", user_id.as_bytes()]);
//! assert!(comp.as_bytes().windows(1).any(|w| w == [0x00]));
//!
//! // Zero-allocation hot path using Encoder reuse
//! let mut enc = Encoder::with_capacity(64);
//! enc.encode_string_into("tenant");
//! enc.push_byte(LexKey::SEPARATOR);
//! enc.encode_i64_into(123);
//! let bytes = enc.freeze();
//! assert!(!bytes.is_empty());
//! ```
//!
//! See `LexKey` and `Encoder` for detailed APIs and more examples.
pub mod encoder;
pub mod lexkey;

// Re-export commonly used types at the crate root for convenient imports in tests and consumers
pub use encoder::Encoder;
pub use lexkey::LexKey;

// Shared crate-level helpers/constants used by both `encoder` and `lexkey` modules.
// Keep these `pub(crate)` so they are available across the crate but not exported in the public API.

/// Compute the encoded length of composite `parts`, including separators between parts
/// (no trailing separator after the last part).
pub(crate) fn encode_len(parts: &[&[u8]]) -> usize {
    parts.iter().map(|p| p.len()).sum::<usize>() + if parts.len() > 1 { parts.len() - 1 } else { 0 }
}
