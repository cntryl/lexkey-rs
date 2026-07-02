use bytes::Bytes;
use std::cmp::Ordering;
use uuid::Uuid;

use crate::Encodable;

const SIGN_BIT: u64 = 0x8000_0000_0000_0000;
const SIGN_BIT_8: u8 = 0x80;
const SIGN_BIT_16: u16 = 0x8000;
const SIGN_BIT_32: u32 = 0x8000_0000;

// Small static byte buffers used to avoid allocating tiny Vecs for common single-byte
// encodings (false/true/end-marker). Using `Bytes::from_static` avoids a heap
// allocation for these hot small constructors.
static END_MARKER_BYTE: [u8; 1] = [0xFFu8];
static BOOL_FALSE: [u8; 1] = [0x00u8];
static BOOL_TRUE: [u8; 1] = [0x01u8];

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
    #[must_use]
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
    #[inline(always)]
    #[must_use]
    pub fn encode_string(s: &str) -> Self {
        Self::from_bytes(Bytes::copy_from_slice(s.as_bytes()))
    }

    /* -------------------------------------------------------
     *  FIXED-WIDTH ENCODERS — optimized using stack arrays
     * ----------------------------------------------------- */

    /// Encode an unsigned integer as 8-byte big-endian.
    ///
    /// ```rust
    /// use lexkey::LexKey;
    /// assert_eq!(LexKey::encode_u64(123).to_hex_string(), "000000000000007b");
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn encode_u64(n: u64) -> Self {
        let bytes = n.to_be_bytes();
        Self::from_bytes(Bytes::copy_from_slice(&bytes))
    }

    /// Encode an unsigned 8-bit integer as 1 byte.
    #[inline(always)]
    #[must_use]
    pub fn encode_u8(n: u8) -> Self {
        Self::from_bytes(Bytes::copy_from_slice(&[n]))
    }

    /// Encode an unsigned 16-bit integer as 2-byte big-endian.
    #[inline(always)]
    #[must_use]
    pub fn encode_u16(n: u16) -> Self {
        Self::from_bytes(Bytes::copy_from_slice(&n.to_be_bytes()))
    }

    /// Encode an unsigned 32-bit integer as 4-byte big-endian.
    #[inline(always)]
    #[must_use]
    pub fn encode_u32(n: u32) -> Self {
        Self::from_bytes(Bytes::copy_from_slice(&n.to_be_bytes()))
    }

    /// Append the 8-byte big-endian encoding of `n` into `dst`.
    /// Returns the number of bytes written (always 8).
    #[inline(always)]
    pub fn encode_u64_into(dst: &mut Vec<u8>, n: u64) -> usize {
        dst.extend_from_slice(&n.to_be_bytes());
        8
    }

    /// Append the 1-byte encoding of `n` into `dst`.
    #[inline(always)]
    pub fn encode_u8_into(dst: &mut Vec<u8>, n: u8) -> usize {
        dst.push(n);
        1
    }

    /// Append the 2-byte big-endian encoding of `n` into `dst`.
    #[inline(always)]
    pub fn encode_u16_into(dst: &mut Vec<u8>, n: u16) -> usize {
        dst.extend_from_slice(&n.to_be_bytes());
        2
    }

    /// Append the 4-byte big-endian encoding of `n` into `dst`.
    #[inline(always)]
    pub fn encode_u32_into(dst: &mut Vec<u8>, n: u32) -> usize {
        dst.extend_from_slice(&n.to_be_bytes());
        4
    }

    /// Encode a signed integer so that lexicographic order matches numeric order.
    ///
    /// Transform: `(n as u64) ^ 0x8000_0000_0000_0000`, then big-endian.
    #[inline(always)]
    #[must_use]
    pub fn encode_i64(n: i64) -> Self {
        let transformed = (n as u64) ^ SIGN_BIT;
        let bytes = transformed.to_be_bytes();
        Self::from_bytes(Bytes::copy_from_slice(&bytes))
    }

    /// Encode a signed 8-bit integer so lexicographic order matches numeric order.
    #[inline(always)]
    #[must_use]
    pub fn encode_i8(n: i8) -> Self {
        Self::from_bytes(Bytes::copy_from_slice(&[((n as u8) ^ SIGN_BIT_8)]))
    }

    /// Encode a signed 16-bit integer so lexicographic order matches numeric order.
    #[inline(always)]
    #[must_use]
    pub fn encode_i16(n: i16) -> Self {
        let transformed = (n as u16) ^ SIGN_BIT_16;
        Self::from_bytes(Bytes::copy_from_slice(&transformed.to_be_bytes()))
    }

    /// Encode a signed 32-bit integer so lexicographic order matches numeric order.
    #[inline(always)]
    #[must_use]
    pub fn encode_i32(n: i32) -> Self {
        let transformed = (n as u32) ^ SIGN_BIT_32;
        Self::from_bytes(Bytes::copy_from_slice(&transformed.to_be_bytes()))
    }

    /// Append the transformed 8-byte encoding of an `i64` into `dst` (always 8 bytes).
    #[inline(always)]
    pub fn encode_i64_into(dst: &mut Vec<u8>, n: i64) -> usize {
        let t = (n as u64) ^ SIGN_BIT;
        dst.extend_from_slice(&t.to_be_bytes());
        8
    }

    /// Append the transformed 1-byte encoding of an `i8` into `dst`.
    #[inline(always)]
    pub fn encode_i8_into(dst: &mut Vec<u8>, n: i8) -> usize {
        dst.push((n as u8) ^ SIGN_BIT_8);
        1
    }

    /// Append the transformed 2-byte big-endian encoding of an `i16` into `dst`.
    #[inline(always)]
    pub fn encode_i16_into(dst: &mut Vec<u8>, n: i16) -> usize {
        let t = (n as u16) ^ SIGN_BIT_16;
        dst.extend_from_slice(&t.to_be_bytes());
        2
    }

    /// Append the transformed 4-byte big-endian encoding of an `i32` into `dst`.
    #[inline(always)]
    pub fn encode_i32_into(dst: &mut Vec<u8>, n: i32) -> usize {
        let t = (n as u32) ^ SIGN_BIT_32;
        dst.extend_from_slice(&t.to_be_bytes());
        4
    }

    /// Encode a boolean: `false -> 0x00`, `true -> 0x01`.
    #[inline(always)]
    #[must_use]
    pub fn encode_bool(b: bool) -> Self {
        if b {
            Self {
                bytes: Bytes::from_static(&BOOL_TRUE),
            }
        } else {
            Self {
                bytes: Bytes::from_static(&BOOL_FALSE),
            }
        }
    }

    /// Append the boolean encoding into `dst` and return 1.
    #[inline(always)]
    pub fn encode_bool_into(dst: &mut Vec<u8>, b: bool) -> usize {
        dst.push(u8::from(b));
        1
    }

    /// Encode an IEEE-754 `f64` using a transform so that lexicographic order matches numeric order.
    ///
    /// NaN values are not supported and will cause a panic. Use a schema-level marker for
    /// optional/absent floating-point values if you need to express missingness.
    /// Negative values are bitwise-not of their IEEE representation; non-negative values are
    /// XOR'd with the sign bit.
    ///
    /// # Panics
    ///
    /// Panics if `x` is NaN.
    #[inline(always)]
    #[must_use]
    pub fn encode_f64(x: f64) -> Self {
        if x.is_nan() {
            panic!("NaN is not encodable");
        }

        let b = x.to_bits();
        let sign_mask = ((b as i64) >> 63) as u64; // all 1s for negative, 0 for positive

        // branchless:
        // negative → !b
        // positive → b ^ signbit
        let neg = !b;
        let pos = b ^ SIGN_BIT;
        let transformed = (neg & sign_mask) | (pos & !sign_mask);

        let bytes = transformed.to_be_bytes();
        Self::from_bytes(Bytes::copy_from_slice(&bytes))
    }

    /// Encode an IEEE-754 `f32` using the same sortable transform at 4-byte width.
    ///
    /// # Panics
    ///
    /// Panics if `x` is NaN.
    #[inline(always)]
    #[must_use]
    pub fn encode_f32(x: f32) -> Self {
        if x.is_nan() {
            panic!("NaN is not encodable");
        }

        let b = x.to_bits();
        let mask = ((b as i32) >> 31) as u32;
        let neg = !b;
        let pos = b ^ SIGN_BIT_32;
        let transformed = (neg & mask) | (pos & !mask);

        Self::from_bytes(Bytes::copy_from_slice(&transformed.to_be_bytes()))
    }

    /// Append the transformed 8-byte encoding of an `f64` into `dst` (always 8 bytes).
    ///
    /// # Panics
    ///
    /// Panics if `x` is NaN.
    #[inline(always)]
    pub fn encode_f64_into(dst: &mut Vec<u8>, x: f64) -> usize {
        if x.is_nan() {
            panic!("NaN not encodable");
        }

        let b = x.to_bits();
        let mask = ((b as i64) >> 63) as u64;
        let neg = !b;
        let pos = b ^ SIGN_BIT;
        let transformed = (neg & mask) | (pos & !mask);

        dst.extend_from_slice(&transformed.to_be_bytes());
        8
    }

    /// Append the transformed 4-byte encoding of an `f32` into `dst`.
    ///
    /// # Panics
    ///
    /// Panics if `x` is NaN.
    #[inline(always)]
    pub fn encode_f32_into(dst: &mut Vec<u8>, x: f32) -> usize {
        if x.is_nan() {
            panic!("NaN not encodable");
        }

        let b = x.to_bits();
        let mask = ((b as i32) >> 31) as u32;
        let neg = !b;
        let pos = b ^ SIGN_BIT_32;
        let transformed = (neg & mask) | (pos & !mask);

        dst.extend_from_slice(&transformed.to_be_bytes());
        4
    }

    /// Encode a UUID as its 16 raw RFC4122 bytes.
    #[inline(always)]
    #[must_use]
    pub fn encode_uuid(u: &Uuid) -> Self {
        Self::from_bytes(Bytes::copy_from_slice(u.as_bytes()))
    }

    /// Append a UUID's 16 bytes into `dst` and return 16.
    #[inline(always)]
    pub fn encode_uuid_into(dst: &mut Vec<u8>, u: &Uuid) -> usize {
        dst.extend_from_slice(u.as_bytes());
        16
    }

    /// Encode a UTC timestamp represented as UNIX nanoseconds.
    #[inline(always)]
    #[must_use]
    pub fn encode_time_unix_nanos(nanos: i64) -> Self {
        Self::encode_i64(nanos)
    }

    /// Encode the end sentinel as a single `0xFF` byte.
    #[inline(always)]
    #[must_use]
    pub fn encode_end_marker() -> Self {
        Self {
            bytes: Bytes::from_static(&END_MARKER_BYTE),
        }
    }

    /// Return `prefix || 0xff` as an owned vector.
    ///
    /// This is the structured partition upper bound used by
    /// `encode_range_upper(prefix, None)`. It is correct when all keys below
    /// the partition continue with a separator or marker byte less than
    /// `END_MARKER`. For arbitrary raw-byte prefix scans, use
    /// `prefix_successor` or `prefix_scan_bounds`.
    #[inline]
    #[must_use]
    pub fn prefix_end(prefix: &[u8]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(prefix.len() + 1);
        buf.extend_from_slice(prefix);
        buf.push(Self::END_MARKER);
        buf
    }

    /// Alias for `prefix_end`, named for APIs that expose range terminology.
    #[inline]
    #[must_use]
    pub fn range_upper_vec(prefix: &[u8]) -> Vec<u8> {
        Self::prefix_end(prefix)
    }

    /// Return the smallest byte string greater than all keys with `prefix`.
    ///
    /// This is the safe exclusive upper bound for arbitrary raw-byte prefix
    /// scans. It increments the last non-`0xff` byte and truncates any trailing
    /// `0xff` bytes. If no finite upper bound exists (empty prefix or all bytes
    /// are `0xff`), returns `None` and callers should use an unbounded scan end.
    #[inline]
    #[must_use]
    pub fn prefix_successor(prefix: &[u8]) -> Option<Vec<u8>> {
        let mut upper = prefix.to_vec();
        for index in (0..upper.len()).rev() {
            if upper[index] != u8::MAX {
                upper[index] += 1;
                upper.truncate(index + 1);
                return Some(upper);
            }
        }
        None
    }

    /// Return raw-byte prefix scan bounds as `(lower, upper)`.
    ///
    /// `lower` is the prefix itself. `upper` is the exclusive successor from
    /// `prefix_successor`, or `None` when the prefix has no finite successor.
    #[inline]
    #[must_use]
    pub fn prefix_scan_bounds(prefix: &[u8]) -> (Vec<u8>, Option<Vec<u8>>) {
        (prefix.to_vec(), Self::prefix_successor(prefix))
    }

    /// Return `(prefix || 0x00, prefix || 0xff)` as owned structured range bounds.
    #[inline]
    #[must_use]
    pub fn prefix_range_bounds(prefix: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let cap = prefix.len() + 1;
        let mut lower = Vec::with_capacity(cap);
        lower.extend_from_slice(prefix);
        lower.push(Self::SEPARATOR);

        let mut upper = Vec::with_capacity(cap);
        upper.extend_from_slice(prefix);
        upper.push(Self::END_MARKER);

        (lower, upper)
    }

    /// Alias for `prefix_range_bounds`, named for APIs that expose range terminology.
    #[inline]
    #[must_use]
    pub fn range_bounds_vec(prefix: &[u8]) -> (Vec<u8>, Vec<u8>) {
        Self::prefix_range_bounds(prefix)
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
    #[inline]
    fn composite_capacity(parts: &[&[u8]]) -> usize {
        crate::encode_len(parts)
    }

    #[inline]
    #[must_use]
    pub fn encode_composite(parts: &[&[u8]]) -> Self {
        if parts.is_empty() {
            return Self::empty();
        }

        let cap = Self::composite_capacity(parts);
        let mut buf = Vec::with_capacity(cap);
        crate::encode_parts_into(&mut buf, parts);
        Self::from_bytes(buf)
    }

    /// Append composite parts into `dst` without a trailing separator and return bytes written.
    #[inline]
    pub fn encode_composite_into(dst: &mut Vec<u8>, parts: &[&[u8]]) -> usize {
        if parts.is_empty() {
            return 0;
        }

        let additional = Self::composite_capacity(parts);
        dst.reserve(additional);
        crate::encode_parts_into(dst, parts)
    }

    /// Build `encode_first`: `prefix + SEPARATOR` (sorts before keys that extend the same prefix).
    #[inline]
    #[must_use]
    pub fn encode_first(parts: &[&[u8]]) -> Self {
        let mut buf = Vec::with_capacity(Self::composite_capacity(parts) + 1);
        crate::encode_parts_into(&mut buf, parts);
        buf.push(Self::SEPARATOR);
        Self::from_bytes(buf)
    }

    /// Build `encode_last`: `prefix + END_MARKER`.
    ///
    /// This is the structured upper bound for keys that extend the prefix via
    /// the normal composite separator path (`prefix || 0x00 || child`). For
    /// arbitrary raw-byte prefix scans, use `prefix_successor`.
    #[inline]
    #[must_use]
    pub fn encode_last(parts: &[&[u8]]) -> Self {
        let mut buf = Vec::with_capacity(Self::composite_capacity(parts) + 1);
        crate::encode_parts_into(&mut buf, parts);
        buf.push(Self::END_MARKER);
        Self::from_bytes(buf)
    }

    /// Encode a range lower bound key for primary keys.
    ///
    /// For a partition `P` and optional row lower bound `L`:
    /// - If `L` is provided: `P || 00 || L`
    /// - If `L` is None: `P || 00`
    ///
    /// This sorts at or before the first key in the range.
    #[inline]
    #[must_use]
    pub fn encode_range_lower(partition: &[u8], row_lower: Option<&[u8]>) -> Self {
        if let Some(row) = row_lower {
            let mut buf = Vec::with_capacity(partition.len() + 1 + row.len());
            buf.extend_from_slice(partition);
            buf.push(Self::SEPARATOR);
            buf.extend_from_slice(row);
            Self::from_bytes(buf)
        } else {
            let mut buf = Vec::with_capacity(partition.len() + 1);
            buf.extend_from_slice(partition);
            buf.push(Self::SEPARATOR);
            Self::from_bytes(buf)
        }
    }

    /// Encode a range upper bound key for primary keys.
    ///
    /// For a partition `P` and optional row upper bound `U`:
    /// - If `U` is provided: `P || 00 || U || ff`
    /// - If `U` is None: `P || ff`
    ///
    /// This sorts after the last key in the range.
    #[inline]
    #[must_use]
    pub fn encode_range_upper(partition: &[u8], row_upper: Option<&[u8]>) -> Self {
        if let Some(row) = row_upper {
            let mut buf = Vec::with_capacity(partition.len() + 1 + row.len() + 1);
            buf.extend_from_slice(partition);
            buf.push(Self::SEPARATOR);
            buf.extend_from_slice(row);
            buf.push(Self::END_MARKER);
            Self::from_bytes(buf)
        } else {
            let mut buf = Vec::with_capacity(partition.len() + 1);
            buf.extend_from_slice(partition);
            buf.push(Self::END_MARKER);
            Self::from_bytes(buf)
        }
    }

    /// Encode the full range bounds for a partition.
    ///
    /// Returns a tuple `(lower, upper)` where:
    /// - `lower`: `partition || 00` (inclusive start)
    /// - `upper`: `partition || ff` (exclusive end)
    ///
    /// This covers all rows in the partition.
    ///
    /// For mixed-type partitions, use the `encode_range_bounds!` macro.
    #[inline]
    #[must_use]
    pub fn encode_range_bounds(partition: &[u8]) -> (Self, Self) {
        let cap = partition.len() + 1;
        let mut lower = Vec::with_capacity(cap);
        lower.extend_from_slice(partition);
        lower.push(Self::SEPARATOR);

        let mut upper = Vec::with_capacity(cap);
        upper.extend_from_slice(partition);
        upper.push(Self::END_MARKER);

        (Self::from_bytes(lower), Self::from_bytes(upper))
    }

    #[inline]
    pub fn encode_composite_encodables(parts: &[&dyn Encodable]) -> Self {
        if parts.is_empty() {
            return Self::empty();
        }
        let total_len =
            parts.iter().map(|p| p.encoded_len()).sum::<usize>() + parts.len().saturating_sub(1);
        let mut buf = Vec::with_capacity(total_len);
        for (i, part) in parts.iter().enumerate() {
            part.encode_into(&mut buf);
            if i + 1 < parts.len() {
                buf.push(Self::SEPARATOR);
            }
        }
        Self::from_bytes(buf)
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

impl<T: Encodable + ?Sized> Encodable for &T {
    #[inline]
    fn encoded_len(&self) -> usize {
        (**self).encoded_len()
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        (**self).encode_into(dst)
    }
}

impl Encodable for &str {
    #[inline]
    fn encoded_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        dst.extend_from_slice(self.as_bytes());
        self.len()
    }
}

impl Encodable for &[u8] {
    #[inline]
    fn encoded_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        dst.extend_from_slice(self);
        self.len()
    }
}

impl<const N: usize> Encodable for &[u8; N] {
    #[inline]
    fn encoded_len(&self) -> usize {
        N
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        dst.extend_from_slice(&self[..]);
        N
    }
}

impl Encodable for LexKey {
    #[inline]
    fn encoded_len(&self) -> usize {
        self.bytes.len()
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        dst.extend_from_slice(&self.bytes);
        self.bytes.len()
    }
}

impl Encodable for Bytes {
    #[inline]
    fn encoded_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        dst.extend_from_slice(self);
        self.len()
    }
}

impl Encodable for i64 {
    #[inline]
    fn encoded_len(&self) -> usize {
        8
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        LexKey::encode_i64_into(dst, *self)
    }
}

impl Encodable for u64 {
    #[inline]
    fn encoded_len(&self) -> usize {
        8
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        LexKey::encode_u64_into(dst, *self)
    }
}

impl Encodable for f64 {
    #[inline]
    fn encoded_len(&self) -> usize {
        8
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        LexKey::encode_f64_into(dst, *self)
    }
}

impl Encodable for bool {
    #[inline]
    fn encoded_len(&self) -> usize {
        1
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        LexKey::encode_bool_into(dst, *self)
    }
}

impl Encodable for Uuid {
    #[inline]
    fn encoded_len(&self) -> usize {
        16
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        LexKey::encode_uuid_into(dst, self)
    }
}

impl Encodable for u8 {
    #[inline]
    fn encoded_len(&self) -> usize {
        1
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        LexKey::encode_u8_into(dst, *self)
    }
}

impl Encodable for u16 {
    #[inline]
    fn encoded_len(&self) -> usize {
        2
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        LexKey::encode_u16_into(dst, *self)
    }
}

impl Encodable for u32 {
    #[inline]
    fn encoded_len(&self) -> usize {
        4
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        LexKey::encode_u32_into(dst, *self)
    }
}

impl Encodable for i8 {
    #[inline]
    fn encoded_len(&self) -> usize {
        1
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        LexKey::encode_i8_into(dst, *self)
    }
}

impl Encodable for i16 {
    #[inline]
    fn encoded_len(&self) -> usize {
        2
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        LexKey::encode_i16_into(dst, *self)
    }
}

impl Encodable for i32 {
    #[inline]
    fn encoded_len(&self) -> usize {
        4
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        LexKey::encode_i32_into(dst, *self)
    }
}

impl Encodable for f32 {
    #[inline]
    fn encoded_len(&self) -> usize {
        4
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        LexKey::encode_f32_into(dst, *self)
    }
}

impl Encodable for String {
    #[inline]
    fn encoded_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        let bytes = self.as_bytes();
        dst.extend_from_slice(bytes);
        bytes.len()
    }
}

impl Encodable for Vec<u8> {
    #[inline]
    fn encoded_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
        dst.extend_from_slice(self);
        self.len()
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
    fn should_encode_u64_as_8_bytes() {
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
    fn should_transform_floats_and_reject_nan() {
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
    fn should_encode_narrow_unsigned_types_at_native_width() {
        // Arrange
        let u8_v: u8 = 255;
        let u16_v: u16 = 123;
        let u32_v: u32 = 123;

        // Act & Assert
        assert_eq!(LexKey::encode_u8(u8_v).to_hex_string(), "ff");
        assert_eq!(LexKey::encode_u16(u16_v).to_hex_string(), "007b");
        assert_eq!(LexKey::encode_u32(u32_v).to_hex_string(), "0000007b");
        assert!(LexKey::encode_u8(1).as_bytes() < LexKey::encode_u8(2).as_bytes());
    }

    #[test]
    fn should_encode_narrow_signed_types_at_native_width() {
        // Arrange
        let i8_v: i8 = -100;
        let i16_v: i16 = -12345;
        let i32_v: i32 = 12345678;

        // Act & Assert
        assert_eq!(LexKey::encode_i8(i8_v).to_hex_string(), "1c");
        assert_eq!(LexKey::encode_i16(i16_v).to_hex_string(), "4fc7");
        assert_eq!(LexKey::encode_i32(i32_v).to_hex_string(), "80bc614e");
        assert!(LexKey::encode_i8(-1).as_bytes() < LexKey::encode_i8(0).as_bytes());
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
    fn should_encode_float32_at_native_width() {
        let v32: f32 = std::f32::consts::PI;
        assert_eq!(LexKey::encode_f32(v32).to_hex_string(), "c0490fdb");
        assert!(LexKey::encode_f32(-1.0).as_bytes() < LexKey::encode_f32(1.0).as_bytes());
    }

    #[test]
    fn should_order_negative_zero_before_positive_zero_for_float32() {
        let neg_zero = LexKey::encode_f32(-0.0_f32);
        let pos_zero = LexKey::encode_f32(0.0_f32);
        assert!(neg_zero < pos_zero);
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
    fn should_encode_smaller_types_via_encodable_trait() {
        // Arrange
        let u8_val: u8 = 42;
        let u16_val: u16 = 1234;
        let u32_val: u32 = 56789;
        let i8_val: i8 = -10;
        let i16_val: i16 = -1234;
        let i32_val: i32 = -56789;
        let f32_val: f32 = std::f32::consts::PI;
        let string_val = "hello world".to_string();
        let vec_val = vec![1, 2, 3, 4, 5];

        // Act & Assert
        assert_eq!(u8_val.encoded_len(), 1);
        assert_eq!(u16_val.encoded_len(), 2);
        assert_eq!(u32_val.encoded_len(), 4);
        assert_eq!(i8_val.encoded_len(), 1);
        assert_eq!(i16_val.encoded_len(), 2);
        assert_eq!(i32_val.encoded_len(), 4);
        assert_eq!(f32_val.encoded_len(), 4);
        assert_eq!(string_val.encoded_len(), string_val.len());
        assert_eq!(vec_val.encoded_len(), vec_val.len());

        let mut buf = Vec::new();
        let written = u8_val.encode_into(&mut buf);
        assert_eq!(written, 1);
        assert_eq!(buf.len(), 1);

        // Check that encoding smaller types matches native-width constructors.
        let mut buf_u8 = Vec::new();
        u8_val.encode_into(&mut buf_u8);
        let mut direct_u8 = Vec::new();
        LexKey::encode_u8_into(&mut direct_u8, u8_val);
        assert_eq!(buf_u8, direct_u8);

        // Check String and Vec<u8>
        let mut buf_str = Vec::new();
        let written_str = string_val.encode_into(&mut buf_str);
        assert_eq!(written_str, string_val.len());
        assert_eq!(buf_str, string_val.as_bytes());

        let mut buf_vec = Vec::new();
        let written_vec = vec_val.encode_into(&mut buf_vec);
        assert_eq!(written_vec, vec_val.len());
        assert_eq!(buf_vec, vec_val);
    }

    #[test]
    fn should_clear_encoder_and_expose_as_slice() {
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

    #[test]
    #[should_panic]
    fn encode_f32_allocating_panics_on_nan() {
        let _ = LexKey::encode_f32(f32::NAN);
    }

    #[test]
    #[should_panic]
    fn encode_f32_into_panics_on_nan() {
        let mut buf = Vec::new();
        let _ = LexKey::encode_f32_into(&mut buf, f32::NAN);
    }

    #[test]
    #[should_panic]
    fn encoder_encode_f32_into_panics_on_nan() {
        let mut enc = Encoder::with_capacity(4);
        let _ = enc.encode_f32_into(f32::NAN);
    }

    #[test]
    fn should_encode_composite_from_mixed_types() {
        use crate::encode_composite;
        let key = encode_composite!("hello", 42i64, true);
        let expected = LexKey::encode_composite(&[
            b"hello",
            LexKey::encode_i64(42).as_bytes(),
            LexKey::encode_bool(true).as_bytes(),
        ]);
        assert_eq!(key, expected);
    }

    #[test]
    fn encode_composite_macro_should_not_move_owned_values() {
        use crate::encode_composite;

        let tenant = String::from("tenant");
        let suffix = vec![0x01, 0x02, 0x03];

        let key = encode_composite!(tenant, suffix);

        assert_eq!(tenant, "tenant");
        assert_eq!(suffix, vec![0x01, 0x02, 0x03]);
        assert_eq!(key.as_bytes(), b"tenant\x00\x01\x02\x03");
    }

    #[test]
    fn encode_composite_macro_should_evaluate_each_expression_once() {
        use crate::{encode_composite, Encodable};
        use std::cell::Cell;

        struct Counted {
            byte: u8,
        }

        impl Encodable for Counted {
            fn encoded_len(&self) -> usize {
                1
            }

            fn encode_into(&self, dst: &mut Vec<u8>) -> usize {
                dst.push(self.byte);
                1
            }
        }

        fn counted(evaluations: &Cell<usize>, byte: u8) -> Counted {
            evaluations.set(evaluations.get() + 1);
            Counted { byte }
        }

        let evaluations = Cell::new(0);
        let key = encode_composite!(counted(&evaluations, b'a'), counted(&evaluations, b'b'));

        assert_eq!(evaluations.get(), 2);
        assert_eq!(key.as_bytes(), b"a\x00b");
    }

    #[test]
    fn should_encode_common_raw_parts_via_encodable_trait() {
        use crate::encode_composite;

        let bytes: &[u8] = b"bytes";
        let array = b"array";
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let nested = LexKey::encode_i64(7);
        let owned_bytes = Bytes::copy_from_slice(b"owned");

        let key = encode_composite!(bytes, array, uuid, nested.clone(), owned_bytes.clone());
        let expected = LexKey::encode_composite(&[
            bytes,
            &array[..],
            uuid.as_bytes(),
            nested.as_bytes(),
            &owned_bytes,
        ]);

        assert_eq!(key, expected);
    }

    #[test]
    fn should_encode_range_lower_with_row() {
        let partition = b"part";
        let row_lower = b"start";
        let key = LexKey::encode_range_lower(partition, Some(row_lower));
        let expected = LexKey::encode_composite(&[partition, row_lower]);
        assert_eq!(key, expected);
        assert_eq!(key.to_hex_string(), "70617274007374617274");
    }

    #[test]
    fn should_encode_range_lower_without_row() {
        let partition = b"part";
        let key = LexKey::encode_range_lower(partition, None);
        assert_eq!(key.to_hex_string(), "7061727400");
    }

    #[test]
    fn should_encode_range_upper_with_row() {
        let partition = b"part";
        let row_upper = b"end";
        let key = LexKey::encode_range_upper(partition, Some(row_upper));
        assert_eq!(key.to_hex_string(), "7061727400656e64ff");
    }

    #[test]
    fn should_encode_range_upper_without_row() {
        let partition = b"part";
        let key = LexKey::encode_range_upper(partition, None);
        assert_eq!(key.to_hex_string(), "70617274ff");
    }

    #[test]
    fn should_encode_range_bounds() {
        let partition = b"part";
        let (lower, upper) = LexKey::encode_range_bounds(partition);
        assert_eq!(lower.to_hex_string(), "7061727400");
        assert_eq!(upper.to_hex_string(), "70617274ff");
    }

    #[test]
    fn should_build_prefix_end_as_owned_vec() {
        let prefix = b"acme\0kv\0";
        let upper = LexKey::prefix_end(prefix);
        let expected = LexKey::encode_range_upper(prefix, None).as_bytes().to_vec();

        assert_eq!(upper, b"acme\0kv\0\xff".to_vec());
        assert_eq!(upper, expected);
        assert!(b"acme\0kv\0users".as_slice() < upper.as_slice());
    }

    #[test]
    fn should_build_range_upper_vec_alias() {
        let prefix = b"acme\0qu\0";
        assert_eq!(LexKey::range_upper_vec(prefix), LexKey::prefix_end(prefix));
    }

    #[test]
    fn should_build_prefix_successor_for_raw_prefix_scan() {
        assert_eq!(LexKey::prefix_successor(b"abc"), Some(b"abd".to_vec()));
        assert_eq!(LexKey::prefix_successor(b"ab\xff"), Some(b"ac".to_vec()));
        assert_eq!(LexKey::prefix_successor(b"\xff"), None);
        assert_eq!(LexKey::prefix_successor(b""), None);
    }

    #[test]
    fn should_build_prefix_scan_bounds_for_arbitrary_bytes() {
        let prefix = b"acme\0kv\0";
        let (lower, upper) = LexKey::prefix_scan_bounds(prefix);
        let upper = upper.expect("prefix should have a finite successor");
        let child = b"acme\0kv\0\xff\x00";
        let structured_upper = LexKey::prefix_end(prefix);

        assert_eq!(lower, prefix.to_vec());
        assert_eq!(upper, b"acme\0kv\x01".to_vec());
        assert!(child.as_slice() < upper.as_slice());
        assert!(child.as_slice() > structured_upper.as_slice());
    }

    #[test]
    fn should_build_prefix_range_bounds_as_owned_vecs() {
        let prefix = b"acme\0";
        let (lower, upper) = LexKey::prefix_range_bounds(prefix);
        let (expected_lower, expected_upper) = LexKey::encode_range_bounds(prefix);

        assert_eq!(lower, b"acme\0\0".to_vec());
        assert_eq!(upper, b"acme\0\xff".to_vec());
        assert_eq!(lower, expected_lower.as_bytes());
        assert_eq!(upper, expected_upper.as_bytes());
    }

    #[test]
    fn should_build_range_bounds_vec_alias() {
        let prefix = b"acme\0st\0";
        assert_eq!(
            LexKey::range_bounds_vec(prefix),
            LexKey::prefix_range_bounds(prefix)
        );
    }

    #[test]
    fn should_encode_range_bounds_macro() {
        use crate::encode_range_bounds;
        let (lower, upper) = encode_range_bounds!("tenant", 42i64);
        let expected_partition =
            LexKey::encode_composite(&[b"tenant", LexKey::encode_i64(42).as_bytes()]);
        let (expected_lower, expected_upper) =
            LexKey::encode_range_bounds(expected_partition.as_bytes());
        assert_eq!(lower, expected_lower);
        assert_eq!(upper, expected_upper);
    }
}
