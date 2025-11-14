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
//! Encode a composite of parts separated by 0x00
//! let user_id = Uuid::nil();
//! let comp = LexKey::encode_composite(&[b"tenant", b"user", user_id.as_bytes()]);
//! assert!(comp.as_bytes().windows(1).any(|w| w == [0x00]));
//!
//! // Encode a composite from mixed types using the macro
//! let comp2 = encode_composite!("tenant", 42i64, true);
//! assert!(comp2.as_bytes().windows(1).any(|w| w == [0x00]));
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

/// Trait for types that can be encoded into a lexkey.
pub trait Encodable {
    /// Returns the number of bytes this value will encode to.
    fn encoded_len(&self) -> usize;
    /// Encodes this value into the given buffer, returning the number of bytes written.
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize;
}

/// Macro to encode a composite key from mixed types.
///
/// This macro pre-calculates the total encoded size, allocates a buffer once,
/// and encodes all parts with separators. It is the primary, zero-overhead way
/// to construct composite LexKeys from mixed types.
///
/// # Examples
/// ```
/// use lexkey::encode_composite;
/// let key = encode_composite!("tenant", 42i64, true);
/// ```
///
/// For empty composites, use `encode_composite!()` which returns an empty key.
#[macro_export]
macro_rules! encode_composite {
    ($first:expr $(, $rest:expr)* $(,)?) => {
        {
            // Calculate total length: sum of encoded lengths + separators between parts
            let mut total_len = $first.encoded_len() $(+ $rest.encoded_len())*;
            let num_parts: usize = 1 $(+ { let _ = $rest; 1 })*;
            total_len += num_parts - 1; // separators: always num_parts - 1 for n >= 1

            // Allocate exact capacity and encode directly
            let mut buf = ::std::vec::Vec::with_capacity(total_len);
            $first.encode_into(&mut buf);
            $(
                buf.push($crate::LexKey::SEPARATOR);
                $rest.encode_into(&mut buf);
            )*
            $crate::LexKey::from_bytes(buf)
        }
    };
    () => {
        $crate::LexKey::empty()
    };
}

// Shared crate-level helpers/constants used by both `encoder` and `lexkey` modules.
// Keep these `pub(crate)` so they are available across the crate but not exported in the public API.

/// Compute the encoded length of composite `parts`, including separators between parts
/// (no trailing separator after the last part).
pub(crate) fn encode_len(parts: &[&[u8]]) -> usize {
    parts.iter().map(|p| p.len()).sum::<usize>() + if parts.len() > 1 { parts.len() - 1 } else { 0 }
}
