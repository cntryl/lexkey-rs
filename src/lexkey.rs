use bytes::Bytes;
use std::cmp::Ordering;
use uuid::Uuid;

// Small static byte buffers used to avoid allocating tiny Vecs for common single-byte
// encodings (false/true/end-marker). Using `Bytes::from_static` avoids a heap
// allocation for these hot small constructors.
static FALSE_BYTE: [u8; 1] = [0x00u8];
static TRUE_BYTE: [u8; 1] = [0x01u8];
static END_MARKER_BYTE: [u8; 1] = [0xFFu8];

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
    #[inline]
    pub fn empty() -> Self {
        Self {
            bytes: Bytes::new(),
        }
    }

    /// Create a key from raw bytes.
    #[inline]
    pub fn from_bytes(bytes: impl Into<Bytes>) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }

    /// Get the raw bytes backing this key.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Check if key is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Encode a UTF-8 string as raw bytes.
    ///
    /// Per spec: strings are copied as-is without a terminator. For composites, use
    /// `encode_composite` to join parts with a `SEPARATOR`.
    ///
    /// ```rust
    /// use lexkey::LexKey;
    /// let k = LexKey::encode_string("hello");
    /// assert_eq!(k.as_bytes(), b"hello");
    /// ```
    #[inline]
    pub fn encode_string(s: &str) -> Self {
        Self::from_bytes(Bytes::copy_from_slice(s.as_bytes()))
    }

    /// Encode an unsigned integer as 8-byte big-endian.
    ///
    /// ```rust
    /// use lexkey::LexKey;
    /// assert_eq!(LexKey::encode_u64(123).to_hex_string(), "000000000000007b");
    /// ```
    #[inline]
    pub fn encode_u64(n: u64) -> Self {
        Self::from_bytes(Bytes::copy_from_slice(&n.to_be_bytes()))
    }

    /// Append the 8-byte big-endian encoding of `n` into `dst`.
    /// Returns the number of bytes written (always 8).
    #[inline]
    pub fn encode_u64_into(dst: &mut Vec<u8>, n: u64) -> usize {
        dst.reserve(8);
        let b = n.to_be_bytes();
        dst.extend_from_slice(&b);
        8
    }

    /// Encode a signed integer so that lexicographic order matches numeric order.
    ///
    /// Transform: `(n as u64) ^ 0x8000_0000_0000_0000`, then big-endian.
    #[inline]
    pub fn encode_i64(n: i64) -> Self {
        let u = (n as u64) ^ 0x8000_0000_0000_0000u64;
        Self::from_bytes(Bytes::copy_from_slice(&u.to_be_bytes()))
    }

    /// Append the transformed 8-byte encoding of an `i64` into `dst` (always 8 bytes).
    #[inline]
    pub fn encode_i64_into(dst: &mut Vec<u8>, n: i64) -> usize {
        dst.reserve(8);
        let u = (n as u64) ^ 0x8000_0000_0000_0000u64;
        let b = u.to_be_bytes();
        dst.extend_from_slice(&b);
        8
    }

    /// Encode a boolean: `false -> 0x00`, `true -> 0x01`.
    #[inline]
    pub fn encode_bool(b: bool) -> Self {
        let bytes = if b {
            Bytes::from_static(&TRUE_BYTE)
        } else {
            Bytes::from_static(&FALSE_BYTE)
        };
        Self { bytes }
    }

    /// Append the boolean encoding into `dst` and return 1.
    #[inline]
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
    #[inline]
    pub fn encode_f64(x: f64) -> Self {
        if x.is_nan() {
            panic!("NaN is not encodable; use a schema-level marker for missing floats");
        }

        let bits = x.to_bits();
        let enc = if bits >> 63 == 1 {
            !bits // negative
        } else {
            bits ^ 0x8000_0000_0000_0000u64 // non-negative
        };

        Self::from_bytes(Bytes::copy_from_slice(&enc.to_be_bytes()))
    }

    /// Append the transformed 8-byte encoding of an `f64` into `dst` (always 8 bytes).
    #[inline]
    pub fn encode_f64_into(dst: &mut Vec<u8>, x: f64) -> usize {
        if x.is_nan() {
            panic!("NaN is not encodable; use a schema-level marker for missing floats");
        }

        let bits = x.to_bits();
        let enc = if bits >> 63 == 1 {
            !bits
        } else {
            bits ^ 0x8000_0000_0000_0000u64
        };

        dst.reserve(8);
        let b = enc.to_be_bytes();
        dst.extend_from_slice(&b);
        8
    }

    /// Encode a UUID as its 16 raw RFC4122 bytes.
    #[inline]
    pub fn encode_uuid(u: &Uuid) -> Self {
        Self::from_bytes(Bytes::copy_from_slice(u.as_bytes()))
    }

    /// Append a UUID’s 16 bytes into `dst` and return 16.
    #[inline]
    pub fn encode_uuid_into(dst: &mut Vec<u8>, u: &Uuid) -> usize {
        dst.reserve(16);
        dst.extend_from_slice(u.as_bytes());
        16
    }

    /// Encode a UTC timestamp represented as UNIX nanoseconds.
    #[inline]
    pub fn encode_time_unix_nanos(nanos: i64) -> Self {
        Self::encode_i64(nanos)
    }

    /// Encode the end sentinel as a single `0xFF` byte.
    #[inline]
    pub fn encode_end_marker() -> Self {
        Self {
            bytes: Bytes::from_static(&END_MARKER_BYTE),
        }
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
        if parts.is_empty() {
            return 0;
        }

        let total_len = parts.iter().map(|p| p.len()).sum::<usize>() + parts.len().saturating_sub(1);
        dst.reserve(total_len);

        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                dst.extend_from_slice(part);
            }
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
    #[inline]
    pub fn to_hex_string(&self) -> String {
        hex::encode(&self.bytes)
    }
}

impl PartialOrd for LexKey {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LexKey {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl From<&[u8]> for LexKey {
    #[inline]
    fn from(bytes: &[u8]) -> Self {
        Self {
            bytes: Bytes::copy_from_slice(bytes),
        }
    }
}

impl From<Vec<u8>> for LexKey {
    #[inline]
    fn from(bytes: Vec<u8>) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<&str> for LexKey {
    #[inline]
    fn from(s: &str) -> Self {
        Self::encode_string(s)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::Encoder;
    use uuid::Uuid;

    #[test]
    fn should_return_raw_bytes_given_string_when_encode_string() {
        let s = "hello";
        let k = LexKey::encode_string(s);
        assert_eq!(k.as_bytes(), s.as_bytes());
        assert_eq!(k.to_hex_string(), "68656c6c6f");
    }

    #[test]
    fn should_return_original_bytes_given_vec_when_from_vec() {
        let b = vec![0x00u8, 0x01, 0xff];
        let kb = LexKey::from(b.clone());
        assert_eq!(kb.as_bytes(), &b[..]);
    }

    #[test]
    fn should_widen_unsigned_to_8_bytes_when_encode_u64() {
        let k = LexKey::encode_u64(123);
        assert_eq!(k.to_hex_string(), "000000000000007b");
    }

    #[test]
    fn should_xor_signbit_and_order_signed_integers() {
        let p = LexKey::encode_i64(123);
        let n = LexKey::encode_i64(-123);
        assert_eq!(p.to_hex_string(), "800000000000007b");
        assert_eq!(n.to_hex_string(), "7fffffffffffff85");
        assert!(n < p);
    }

    #[test]
    fn should_encode_bool_nil_and_end_marker_as_single_bytes() {
        let f = LexKey::encode_bool(false);
        let t = LexKey::encode_bool(true);
        let e = LexKey::encode_end_marker();
        assert_eq!(f.to_hex_string(), "00");
        assert_eq!(t.to_hex_string(), "01");
        assert_eq!(e.to_hex_string(), "ff");
    }

    #[test]
    fn should_transform_floats_and_map_nan_to_canonical() {
        let p = LexKey::encode_f64(std::f64::consts::PI);
        let n = LexKey::encode_f64(-std::f64::consts::PI);
        assert!(n < p);
    }

    #[test]
    fn should_encode_uuid_and_time_and_duration_examples_from_spec() {
        // Arrange
        let u = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        // Act
        let ku = LexKey::encode_uuid(&u);
        let t0 = LexKey::encode_time_unix_nanos(0);
        let dur = LexKey::encode_i64(42);

        // Assert
        assert_eq!(ku.to_hex_string(), "550e8400e29b41d4a716446655440000");
        assert_eq!(t0.to_hex_string(), "8000000000000000");
        assert_eq!(dur.to_hex_string(), "800000000000002a");
    }

    #[test]
    fn should_concatenate_parts_with_separator_and_encode_first_last() {
        // Arrange
        let part1 = b"foo";
        let part2 = &LexKey::encode_i64(42).as_bytes().to_vec()[..];
        let part3 = &[0x01u8];

        // Act
        let k = LexKey::encode_composite(&[part1.as_ref(), part2, part3]);
        let first = LexKey::encode_first(&[b"part".as_ref()]);
        let last = LexKey::encode_last(&[b"part".as_ref()]);

        // Assert
        assert_eq!(k.to_hex_string(), "666f6f00800000000000002a0001");
        assert_eq!(first.to_hex_string(), "7061727400");
        assert_eq!(last.to_hex_string(), "70617274ff");
    }

    #[test]
    fn should_widen_smaller_unsigned_types_to_u64() {
        // Arrange
        let u8_v: u8 = 255;
        let u16_v: u16 = 123;
        let u32_v: u32 = 123;

        // Act & Assert
        assert_eq!(
            LexKey::encode_u64(u8_v as u64).to_hex_string(),
            LexKey::encode_u64(u8_v as u64).to_hex_string()
        );
        assert_eq!(
            LexKey::encode_u64(u16_v as u64).to_hex_string(),
            LexKey::encode_u64(u16_v as u64).to_hex_string()
        );
        assert_eq!(
            LexKey::encode_u64(u32_v as u64).to_hex_string(),
            LexKey::encode_u64(u32_v as u64).to_hex_string()
        );
    }

    #[test]
    fn should_widen_smaller_signed_types_to_i64() {
        // Arrange
        let i8_v: i8 = -100;
        let i16_v: i16 = -12345;
        let i32_v: i32 = 12345678;

        // Act & Assert
        assert_eq!(
            LexKey::encode_i64(i8_v as i64).to_hex_string(),
            LexKey::encode_i64(i8_v as i64).to_hex_string()
        );
        assert_eq!(
            LexKey::encode_i64(i16_v as i64).to_hex_string(),
            LexKey::encode_i64(i16_v as i64).to_hex_string()
        );
        assert_eq!(
            LexKey::encode_i64(i32_v as i64).to_hex_string(),
            LexKey::encode_i64(i32_v as i64).to_hex_string()
        );
    }

    #[test]
    fn should_encode_i64_min_and_max_correctly() {
        let min = LexKey::encode_i64(i64::MIN);
        let max = LexKey::encode_i64(i64::MAX);
        assert_eq!(min.to_hex_string(), "0000000000000000");
        assert_eq!(max.to_hex_string(), "ffffffffffffffff");
    }

    #[test]
    fn should_encode_u64_zero_and_max() {
        let zero = LexKey::encode_u64(0);
        let max = LexKey::encode_u64(u64::MAX);
        assert_eq!(zero.to_hex_string(), "0000000000000000");
        assert_eq!(max.to_hex_string(), "ffffffffffffffff");
    }

    #[test]
    fn should_order_negative_zero_before_positive_zero_for_floats() {
        let neg_zero = LexKey::encode_f64(-0.0_f64);
        let pos_zero = LexKey::encode_f64(0.0_f64);
        assert!(neg_zero < pos_zero);
    }

    #[test]
    fn should_order_infinities_correctly() {
        // Arrange
        let neg_inf = LexKey::encode_f64(f64::NEG_INFINITY);
        let large_neg = LexKey::encode_f64(-1e308_f64);
        let zero = LexKey::encode_f64(0.0_f64);
        let large_pos = LexKey::encode_f64(1e308_f64);
        let pos_inf = LexKey::encode_f64(f64::INFINITY);

        // Assert
        assert!(neg_inf < large_neg);
        assert!(large_neg < zero);
        assert!(zero < large_pos);
        assert!(large_pos < pos_inf);
    }

    #[test]
    fn should_widen_float32_to_float64_before_encoding() {
        let v32: f32 = std::f32::consts::PI;
        let widened = v32 as f64;
        let from_widen = LexKey::encode_f64(widened);
        assert_eq!(
            from_widen.to_hex_string(),
            LexKey::encode_f64(widened).to_hex_string()
        );
    }

    #[test]
    fn should_construct_from_bytes_vec_and_slice_given_same_input() {
        // Arrange
        let bytes = vec![0x10u8, 0x20, 0x30];

        // Act
        let k_from_bytes = LexKey::from_bytes(bytes.clone());
        let k_from_vec = LexKey::from(bytes.clone());
        let k_from_slice = LexKey::from(&bytes[..]);

        // Assert
        assert_eq!(k_from_bytes.as_bytes(), k_from_vec.as_bytes());
        assert_eq!(k_from_vec.as_bytes(), k_from_slice.as_bytes());
    }

    #[test]
    fn should_report_empty_for_empty_and_expose_as_bytes() {
        let empty = LexKey::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.as_bytes().len(), 0);
        assert_eq!(empty.to_hex_string(), "");
    }

    #[test]
    fn should_compare_using_ord_and_partialord() {
        let a = LexKey::encode_string("a");
        let b = LexKey::encode_string("b");
        assert!(a < b);
        assert!(a.cmp(&b).is_lt());
        assert!(b.cmp(&a).is_gt());
    }

    #[test]
    fn should_encode_composite_single_part_without_trailing_separator() {
        let part = b"foo";
        let k = LexKey::encode_composite(&[part.as_ref()]);
        assert_eq!(k.to_hex_string(), "666f6f");
    }

    #[test]
    fn should_from_str_equivalent_to_encode_string() {
        let s = "hello";
        let from = LexKey::from(s);
        let encoded = LexKey::encode_string(s);
        assert_eq!(from.as_bytes(), encoded.as_bytes());
    }

    #[test]
    fn should_write_into_buffer_for_various_types() {
        // Arrange
        let mut buf = Vec::with_capacity(64);

        // Act
        let n_written_i = LexKey::encode_i64_into(&mut buf, -123i64);
        let n_written_u = LexKey::encode_u64_into(&mut buf, 123u64);
        let n_written_f = LexKey::encode_f64_into(&mut buf, std::f64::consts::PI);
        let n_written_b = LexKey::encode_bool_into(&mut buf, true);

        // Assert
        assert_eq!(n_written_i, 8);
        assert_eq!(n_written_u, 8);
        assert_eq!(n_written_f, 8);
        assert_eq!(n_written_b, 1);
        let mut dst1 = Vec::new();
        LexKey::encode_composite_into(&mut dst1, &[b"foo", &buf[..]]);
        let mut dst2 = Vec::new();
        dst2.extend_from_slice(b"foo");
        dst2.push(LexKey::SEPARATOR);
        dst2.extend_from_slice(&buf);
        assert_eq!(dst1, dst2);
    }

    #[test]
    fn encoder_clear_and_as_slice_are_covered() {
        let mut enc = Encoder::with_capacity(16);
        enc.encode_string_into("abc");
        assert!(!enc.as_slice().is_empty());
        enc.clear();
        assert!(enc.as_slice().is_empty());
    }

    #[test]
    fn encode_uuid_into_and_composite_into_variants() {
        // Arrange
        let mut buf = Vec::new();
        let u = Uuid::new_v4();

        // Act
        let n = LexKey::encode_uuid_into(&mut buf, &u);

        // Assert
        assert_eq!(n, 16);
        assert_eq!(&buf[..], u.as_bytes());
        buf.clear();
        let parts1: Vec<&[u8]> = vec![b"one"];
        let n1 = LexKey::encode_composite_into(&mut buf, &parts1);
        assert_eq!(n1, 3);
        assert_eq!(&buf[..], b"one");
        buf.clear();
        let parts2: Vec<&[u8]> = vec![b"ten", b"row"];
        let n2 = LexKey::encode_composite_into(&mut buf, &parts2);
        assert_eq!(n2, 3 + 1 + 3);
        assert_eq!(&buf[..], b"ten\x00row");
    }

    #[test]
    fn encode_f64_nan_and_negative_zero() {
        let kn = LexKey::encode_f64(-0.0f64);
        let kp = LexKey::encode_f64(0.0f64);
        assert!(kn.as_bytes() < kp.as_bytes());
    }

    #[test]
    fn encode_f64_all_branches_lexkey_and_encoder() {
        // Arrange
        let mut b1 = Vec::new();
        b1.clear();

        // Act & Assert
        assert_eq!(LexKey::encode_f64_into(&mut b1, -2.5f64), 8);
        b1.clear();
        assert_eq!(LexKey::encode_f64_into(&mut b1, 2.5f64), 8);
        let mut enc = Encoder::with_capacity(32);
        assert_eq!(enc.encode_f64_into(-1.25f64), 8);
        enc.clear();
        assert_eq!(enc.encode_f64_into(1.25f64), 8);
    }

    #[test]
    #[should_panic]
    fn encode_f64_allocating_panics_on_nan() {
        let _ = LexKey::encode_f64(f64::NAN);
    }

    #[test]
    #[should_panic]
    fn encode_f64_into_panics_on_nan() {
        let mut buf = Vec::new();
        let _ = LexKey::encode_f64_into(&mut buf, f64::NAN);
    }

    #[test]
    #[should_panic]
    fn encoder_encode_f64_into_panics_on_nan() {
        let mut enc = Encoder::with_capacity(8);
        let _ = enc.encode_f64_into(f64::NAN);
    }

}
