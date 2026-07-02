//! Lexkey: build lexicographically sortable byte keys for numbers, strings, UUIDs and composites.
//!
//! This crate provides two complementary APIs:
//! - `LexKey`: ergonomic, allocating constructors that return an immutable key (`Bytes`).
//! - `Encoder`: a reusable buffer for zero-allocation hot paths; write multiple values into one buffer.
//!
//! Ordering is by raw byte lexicographic comparison. Numeric and float encoders transform values
//! so that lexicographic order matches numeric order within the same declared width. Narrow
//! `Encodable` numeric types keep their native width (`u8` is 1 byte, `u16` is 2, `u32`/`f32`
//! are 4). Use explicit 64-bit values when a schema needs cross-width canonicalization. Note:
//! NaN values are not encodable by this crate's encoders and will cause a panic; represent missing
//! or invalid floats with a schema-level presence/marker value instead.
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
//! // Encode range bounds for primary keys
//! let lower = LexKey::encode_range_lower(b"partition", Some(b"start"));
//! let upper = LexKey::encode_range_upper(b"partition", Some(b"end"));
//! let (full_lower, full_upper) = LexKey::encode_range_bounds(b"partition");
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
/// to construct composite `LexKeys` from mixed types.
///
/// # Examples
/// ```
/// use lexkey::encode_composite;
/// let key = encode_composite!("tenant", 42i64, true);
/// assert!(!key.is_empty());
/// ```
///
/// For empty composites, use `encode_composite!()` which returns an empty key.
#[macro_export]
macro_rules! encode_composite {
    ($first:expr $(, $rest:expr)* $(,)?) => {
        {
            let parts = (&$first $(, &$rest)*,);
            $crate::__private::encode_composite_tuple(&parts)
        }
    };
    () => {
        $crate::LexKey::empty()
    };
}

/// Macro to encode range bounds for a partition from mixed types.
///
/// This macro encodes the partition as a composite key, then returns the full range bounds
/// for that partition as a tuple `(lower, upper)`.
///
/// # Examples
/// ```
/// use lexkey::encode_range_bounds;
/// let (lower, upper) = encode_range_bounds!("tenant", 42i64);
/// assert!(lower.as_bytes() < upper.as_bytes());
/// ```
///
#[macro_export]
macro_rules! encode_range_bounds {
    ($first:expr $(, $rest:expr)* $(,)?) => {
        {
            // Encode the partition composite
            let partition_key = $crate::encode_composite!($first $(, $rest)*);
            $crate::LexKey::encode_range_bounds(partition_key.as_bytes())
        }
    };
    () => {
        $crate::LexKey::encode_range_bounds(&[])
    };
}

// Shared crate-level helpers/constants used by both `encoder` and `lexkey` modules.
// Keep these `pub(crate)` so they are available across the crate but not exported in the public API.

/// Compute the encoded length of composite `parts`, including separators between parts
/// (no trailing separator after the last part).
#[inline]
pub(crate) fn encode_len(parts: &[&[u8]]) -> usize {
    match parts {
        [] => 0,
        [a] => a.len(),
        [a, b] => a.len() + b.len() + 1,
        [a, b, c] => a.len() + b.len() + c.len() + 2,
        [a, b, c, d] => a.len() + b.len() + c.len() + d.len() + 3,
        _ => parts.iter().map(|p| p.len()).sum::<usize>() + parts.len() - 1,
    }
}

/// Append already-encoded composite `parts` into `dst`, separated by 0x00.
#[inline]
pub(crate) fn encode_parts_into(dst: &mut Vec<u8>, parts: &[&[u8]]) -> usize {
    let start = dst.len();

    match parts {
        [] => {}
        [a] => dst.extend_from_slice(a),
        [a, b] => {
            dst.extend_from_slice(a);
            dst.push(LexKey::SEPARATOR);
            dst.extend_from_slice(b);
        }
        [a, b, c] => {
            dst.extend_from_slice(a);
            dst.push(LexKey::SEPARATOR);
            dst.extend_from_slice(b);
            dst.push(LexKey::SEPARATOR);
            dst.extend_from_slice(c);
        }
        [a, b, c, d] => {
            dst.extend_from_slice(a);
            dst.push(LexKey::SEPARATOR);
            dst.extend_from_slice(b);
            dst.push(LexKey::SEPARATOR);
            dst.extend_from_slice(c);
            dst.push(LexKey::SEPARATOR);
            dst.extend_from_slice(d);
        }
        _ => {
            for (i, part) in parts.iter().enumerate() {
                dst.extend_from_slice(part);
                if i + 1 < parts.len() {
                    dst.push(LexKey::SEPARATOR);
                }
            }
        }
    }

    dst.len() - start
}

#[doc(hidden)]
pub mod __private {
    use super::{Encodable, LexKey};

    pub trait EncodableTuple {
        fn encoded_len(&self) -> usize;
        fn encode_into(&self, dst: &mut Vec<u8>) -> usize;
    }

    #[inline]
    pub fn encode_composite_tuple<T: EncodableTuple>(parts: &T) -> LexKey {
        let mut buf = Vec::with_capacity(parts.encoded_len());
        parts.encode_into(&mut buf);
        LexKey::from_bytes(buf)
    }

    macro_rules! impl_encodable_tuple {
        ($($ty:ident:$value:ident),+) => {
            impl<$($ty),+> EncodableTuple for ($($ty,)+)
            where
                $($ty: Encodable,)+
            {
                #[inline]
                fn encoded_len(&self) -> usize {
                    let ($($value,)+) = self;
                    0 $(+ $value.encoded_len())+ + impl_encodable_tuple!(@separators $($ty),+)
                }

                #[inline]
                fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
                    let start = dst.len();
                    let ($($value,)+) = self;
                    impl_encodable_tuple!(@encode dst; $($value),+);
                    dst.len() - start
                }
            }
        };
        (@separators $single:ident) => {
            0
        };
        (@separators $first:ident, $($rest:ident),+) => {
            impl_encodable_tuple!(@count $($rest),+)
        };
        (@count $single:ident) => {
            1
        };
        (@count $first:ident, $($rest:ident),+) => {
            1 + impl_encodable_tuple!(@count $($rest),+)
        };
        (@encode $dst:ident; $first:ident) => {
            $first.encode_into($dst);
        };
        (@encode $dst:ident; $first:ident, $($rest:ident),+) => {
            $first.encode_into($dst);
            $(
                $dst.push(LexKey::SEPARATOR);
                $rest.encode_into($dst);
            )+
        };
    }

    impl_encodable_tuple!(A:a);
    impl_encodable_tuple!(A:a, B:b);
    impl_encodable_tuple!(A:a, B:b, C:c);
    impl_encodable_tuple!(A:a, B:b, C:c, D:d);
    impl_encodable_tuple!(A:a, B:b, C:c, D:d, E:e);
    impl_encodable_tuple!(A:a, B:b, C:c, D:d, E:e, F:f);
    impl_encodable_tuple!(A:a, B:b, C:c, D:d, E:e, F:f, G:g);
    impl_encodable_tuple!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h);
    impl_encodable_tuple!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i);
    impl_encodable_tuple!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i, J:j);
    impl_encodable_tuple!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i, J:j, K:k);
    impl_encodable_tuple!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i, J:j, K:k, L:l);
    impl_encodable_tuple!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i, J:j, K:k, L:l, M:m);
    impl_encodable_tuple!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i, J:j, K:k, L:l, M:m, N:n);
    impl_encodable_tuple!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i, J:j, K:k, L:l, M:m, N:n, O:o);
    impl_encodable_tuple!(A:a, B:b, C:c, D:d, E:e, F:f, G:g, H:h, I:i, J:j, K:k, L:l, M:m, N:n, O:o, P:p);
}
