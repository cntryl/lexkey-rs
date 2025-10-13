use bytes::Bytes;
use std::cmp::Ordering;
use uuid::Uuid;

/// A lexicographically sortable key.
///
/// Keys are compared by their raw bytes. Use the provided encoders to ensure that numeric and
/// floating-point values sort according to their numeric order when compared lexicographically.
///
/// Example
/// ```
/// use lexkey::LexKey;
/// let a = LexKey::encode_i64(-5);
/// let b = LexKey::encode_i64(7);
/// assert!(a < b); // numeric order preserved via encoding transform
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LexKey {
    bytes: Bytes,
}

impl LexKey {
    /// The 0x00 separator placed between composite parts.
    pub const SEPARATOR: u8 = 0x00;
    /// The 0xFF end marker used by `encode_last`.
    pub const END_MARKER: u8 = 0xFF;

    /// Create an empty key.
    pub fn empty() -> Self {
        Self {
            bytes: Bytes::new(),
        }
    }

    /// Create a key from raw bytes.
    pub fn from_bytes(bytes: impl Into<Bytes>) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }

    /// Get the raw bytes backing this key.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Check if key is empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Encode a UTF-8 string as raw bytes.
    ///
    /// Per spec: strings are copied as-is without a terminator. For composites, use
    /// `encode_composite` to join parts with a `SEPARATOR`.
    ///
    /// ```
    /// use lexkey::LexKey;
    /// let k = LexKey::encode_string("hello");
    /// assert_eq!(k.as_bytes(), b"hello");
    /// ```
    #[inline]
    pub fn encode_string(s: &str) -> Self {
        // Per SPEC: strings are raw bytes with no terminator.
        Self::from_bytes(Bytes::copy_from_slice(s.as_bytes()))
    }

    /// Encode an unsigned integer as 8-byte big-endian.
    ///
    /// ```
    /// use lexkey::LexKey;
    /// assert_eq!(LexKey::encode_u64(123).to_hex_string(), "000000000000007b");
    /// ```
    pub fn encode_u64(n: u64) -> Self {
        Self::from_bytes(Bytes::copy_from_slice(&n.to_be_bytes()))
    }

    /// Append the 8-byte big-endian encoding of `n` into `dst`.
    /// Returns the number of bytes written (always 8).
    pub fn encode_u64_into(dst: &mut Vec<u8>, n: u64) -> usize {
        let b = n.to_be_bytes();
        dst.extend_from_slice(&b);
        8
    }

    /// Encode a signed integer so that lexicographic order matches numeric order.
    ///
    /// Transform: `(n as u64) ^ 0x8000_0000_0000_0000`, then big-endian.
    pub fn encode_i64(n: i64) -> Self {
        let u = (n as u64) ^ 0x8000_0000_0000_0000u64;
        Self::from_bytes(Bytes::copy_from_slice(&u.to_be_bytes()))
    }

    /// Append the transformed 8-byte encoding of an `i64` into `dst` (always 8 bytes).
    pub fn encode_i64_into(dst: &mut Vec<u8>, n: i64) -> usize {
        let u = (n as u64) ^ 0x8000_0000_0000_0000u64;
        let b = u.to_be_bytes();
        dst.extend_from_slice(&b);
        8
    }

    /// Encode a boolean: `false -> 0x00`, `true -> 0x01`.
    pub fn encode_bool(b: bool) -> Self {
        Self::from_bytes(vec![if b { 0x01 } else { 0x00 }])
    }

    /// Append the boolean encoding into `dst` and return 1.
    pub fn encode_bool_into(dst: &mut Vec<u8>, b: bool) -> usize {
        dst.push(if b { 0x01 } else { 0x00 });
        1
    }

    /// Encode an IEEE-754 `f64` using a transform so that lexicographic order matches numeric order.
    ///
    /// NaN values are not supported and will cause a panic. Use a schema-level marker for
    /// optional/absent floating-point values if you need to express missingness.
    /// Negative values are bitwise-not of their IEEE representation; non-negative values are
    /// XOR'd with the sign bit.
    pub fn encode_f64(x: f64) -> Self {
        if x.is_nan() {
            panic!("NaN is not encodable; use a schema-level marker for missing floats");
        }
        let bits: u64 = {
            let b = x.to_bits();
            if x < 0.0 {
                !b
            } else {
                b ^ 0x8000_0000_0000_0000u64
            }
        };

        Self::from_bytes(Bytes::copy_from_slice(&bits.to_be_bytes()))
    }

    /// Append the transformed 8-byte encoding of an `f64` into `dst` (always 8 bytes).
    pub fn encode_f64_into(dst: &mut Vec<u8>, x: f64) -> usize {
        // compute transformed bits and write via single extend
        if x.is_nan() {
            panic!("NaN is not encodable; use a schema-level marker for missing floats");
        }
        let bits: u64 = {
            let b = x.to_bits();
            if x < 0.0 {
                !b
            } else {
                b ^ 0x8000_0000_0000_0000u64
            }
        };
        let b = bits.to_be_bytes();
        dst.extend_from_slice(&b);
        8
    }

    /// Encode a UUID as its 16 raw RFC4122 bytes.
    pub fn encode_uuid(u: &Uuid) -> Self {
        Self::from_bytes(Bytes::copy_from_slice(u.as_bytes()))
    }

    /// Append a UUID’s 16 bytes into `dst` and return 16.
    pub fn encode_uuid_into(dst: &mut Vec<u8>, u: &Uuid) -> usize {
        dst.extend_from_slice(u.as_bytes());
        16
    }

    /// Encode a UTC timestamp represented as UNIX nanoseconds.
    pub fn encode_time_unix_nanos(nanos: i64) -> Self {
        Self::encode_i64(nanos)
    }

    // Note: encode_nil() has been removed. `SEPARATOR` (`0x00`) is used between composite parts.
    // If you need an explicit zero byte in your schema, encode it via an explicit boolean or
    // other schema-level marker. This crate focuses on encode-only primitives and composite
    // construction; generic decoding or split-on-0x00 semantics are intentionally unsupported.

    /// Encode the end sentinel as a single `0xFF` byte.
    pub fn encode_end_marker() -> Self {
        Self::from_bytes(vec![0xFFu8])
    }

    /// Create a composite key from multiple parts.
    ///
    /// Concatenate parts with a single `SEPARATOR` (0x00) between adjacent parts.
    /// No trailing separator is added after the last part.
    ///
    /// Example: representing an optional value in a composite
    /// ```rust
    /// use lexkey::{Encoder, LexKey};
    ///
    /// // Schema decision: prefix optional parts with a presence byte (0x01 = present, 0x00 = absent).
    /// let tenant = "tenant";
    /// let optional_user: Option<&str> = Some("alice");
    /// let mut enc = Encoder::with_capacity(64);
    ///
    /// enc.encode_string_into(tenant);
    /// enc.push_byte(LexKey::SEPARATOR);
    /// match optional_user {
    ///     Some(name) => {
    ///         enc.push_byte(0x01); // presence marker
    ///         enc.encode_string_into(name);
    ///     }
    ///     None => {
    ///         enc.push_byte(0x00); // absence marker
    ///     }
    /// }
    /// let key = enc.freeze();
    /// assert!(!key.is_empty());
    /// ```
    pub fn encode_composite(parts: &[&[u8]]) -> Self {
        let total = crate::encode_len(parts);
        let mut v = Vec::with_capacity(total);
        for (i, part) in parts.iter().enumerate() {
            v.extend_from_slice(part);
            if i + 1 < parts.len() {
                v.push(Self::SEPARATOR);
            }
        }
        Self::from_bytes(v)
    }

    /// Append composite parts into `dst` without a trailing separator and return bytes written.
    pub fn encode_composite_into(dst: &mut Vec<u8>, parts: &[&[u8]]) -> usize {
        let start = dst.len();
        for (i, part) in parts.iter().enumerate() {
            dst.extend_from_slice(part);
            if i + 1 < parts.len() {
                dst.push(Self::SEPARATOR);
            }
        }
        dst.len() - start
    }

    /// Build `encode_first`: `prefix + SEPARATOR` (sorts before keys that extend the same prefix).
    pub fn encode_first(parts: &[&[u8]]) -> Self {
        let mut enc = crate::encoder::Encoder::with_capacity(crate::encode_len(parts) + 1);
        enc.encode_composite_into_buf(parts);
        enc.push_byte(Self::SEPARATOR);
        Self::from_bytes(enc.freeze())
    }

    /// Build `encode_last`: `prefix + END_MARKER` (sorts after keys that extend the same prefix).
    pub fn encode_last(parts: &[&[u8]]) -> Self {
        let mut enc = crate::encoder::Encoder::with_capacity(crate::encode_len(parts) + 1);
        enc.encode_composite_into_buf(parts);
        enc.push_byte(Self::END_MARKER);
        Self::from_bytes(enc.freeze())
    }

    /// Convert to a lowercase hex string, useful for debugging.
    pub fn to_hex_string(&self) -> String {
        hex::encode(&self.bytes)
    }
}

impl PartialOrd for LexKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LexKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

// Use `crate::encode_len` defined at the crate root.

impl From<&[u8]> for LexKey {
    fn from(bytes: &[u8]) -> Self {
        Self {
            bytes: Bytes::copy_from_slice(bytes),
        }
    }
}

impl From<Vec<u8>> for LexKey {
    fn from(bytes: Vec<u8>) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<&str> for LexKey {
    fn from(s: &str) -> Self {
        Self::encode_string(s)
    }
}
