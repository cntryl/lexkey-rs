use bytes::Bytes;
use uuid::Uuid;

const SIGN_BIT: u64 = 0x8000_0000_0000_0000;
const SIGN_BIT_8: u8 = 0x80;
const SIGN_BIT_16: u16 = 0x8000;
const SIGN_BIT_32: u32 = 0x8000_0000;

/// A fast, one-way, lexicographically sortable key encoder.
///
/// This encoder produces byte sequences where the natural byte ordering
/// matches the natural ordering of encoded values within the same declared
/// numeric width, plus UUID and composite path-like keys.
///
/// All numeric encodings are monotonic:
/// - unsigned ints: big-endian at declared width
/// - signed ints: sign-bit flip at declared width
/// - floats: IEEE-754 sortable transform at declared width (error on NaN)
///
/// `Encoder` is reusable; call `clear()` between uses.
pub struct Encoder {
    buf: Vec<u8>,
}

impl Encoder {
    /// Create a new encoder with a capacity hint.
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buf: Vec::with_capacity(cap),
        }
    }

    /// Reset the internal buffer so the encoder can be reused.
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// Convert the accumulated buffer into an immutable `Bytes`.
    #[must_use]
    pub fn freeze(self) -> Bytes {
        Bytes::from(self.buf)
    }

    /// Return the accumulated buffer as an owned `Vec<u8>`.
    ///
    /// Use this when the caller ultimately needs an owned vector, for example
    /// storage-engine keys. This avoids freezing into `Bytes` and copying back
    /// out to a `Vec`.
    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        self.buf
    }

    /// Borrow the current buffer contents.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    /// Return current buffer length.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Check if buffer is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Append a single byte.
    #[inline(always)]
    pub fn push_byte(&mut self, b: u8) {
        self.buf.push(b);
    }

    /// Append the composite part separator (`0x00`).
    #[inline(always)]
    pub fn push_separator(&mut self) {
        self.buf.push(crate::LexKey::SEPARATOR);
    }

    /// Append the range end marker (`0xff`).
    #[inline(always)]
    pub fn push_end_marker(&mut self) {
        self.buf.push(crate::LexKey::END_MARKER);
    }

    /// Append a UTF-8 string (raw bytes).
    #[inline(always)]
    pub fn encode_string_into(&mut self, s: &str) -> usize {
        let bytes = s.as_bytes();
        self.buf.extend_from_slice(bytes);
        bytes.len()
    }

    /// Append raw bytes without transformation.
    #[inline(always)]
    pub fn encode_bytes_into(&mut self, bytes: &[u8]) -> usize {
        self.buf.extend_from_slice(bytes);
        bytes.len()
    }

    /// Append the 8-byte big-endian encoding of a `u64`.
    #[inline(always)]
    pub fn encode_u64_into(&mut self, n: u64) -> usize {
        self.buf.extend_from_slice(&n.to_be_bytes());
        8
    }

    /// Append the native 1-byte encoding of a `u8`.
    #[inline(always)]
    pub fn encode_u8_into(&mut self, n: u8) -> usize {
        self.buf.push(n);
        1
    }

    /// Append the native 2-byte big-endian encoding of a `u16`.
    #[inline(always)]
    pub fn encode_u16_into(&mut self, n: u16) -> usize {
        self.buf.extend_from_slice(&n.to_be_bytes());
        2
    }

    /// Append the native 4-byte big-endian encoding of a `u32`.
    #[inline(always)]
    pub fn encode_u32_into(&mut self, n: u32) -> usize {
        self.buf.extend_from_slice(&n.to_be_bytes());
        4
    }

    /// Append the sortable 8-byte encoding of an `i64`.
    ///
    /// Mapping preserves ordering:
    ///   `i64::MIN` → 0x00...
    ///   `i64::MAX` → 0xFF...
    #[inline(always)]
    pub fn encode_i64_into(&mut self, n: i64) -> usize {
        let u = (n as u64) ^ SIGN_BIT;
        self.buf.extend_from_slice(&u.to_be_bytes());
        8
    }

    /// Append the native 1-byte sortable encoding of an `i8`.
    #[inline(always)]
    pub fn encode_i8_into(&mut self, n: i8) -> usize {
        self.buf.push((n as u8) ^ SIGN_BIT_8);
        1
    }

    /// Append the native 2-byte sortable encoding of an `i16`.
    #[inline(always)]
    pub fn encode_i16_into(&mut self, n: i16) -> usize {
        let u = (n as u16) ^ SIGN_BIT_16;
        self.buf.extend_from_slice(&u.to_be_bytes());
        2
    }

    /// Append the native 4-byte sortable encoding of an `i32`.
    #[inline(always)]
    pub fn encode_i32_into(&mut self, n: i32) -> usize {
        let u = (n as u32) ^ SIGN_BIT_32;
        self.buf.extend_from_slice(&u.to_be_bytes());
        4
    }

    /// Append the sortable IEEE-754 encoding of an `f64`.
    ///
    /// - Sign bit set: bitwise NOT
    /// - Sign bit clear: flip the sign bit
    ///
    /// NaN is rejected because it breaks total ordering.
    ///
    /// # Panics
    ///
    /// Panics if `x` is NaN.
    #[inline(always)]
    pub fn encode_f64_into(&mut self, x: f64) -> usize {
        if x.is_nan() {
            panic!("NaN is not encodable in lexkeys");
        }
        let b = x.to_bits();
        let mask = ((b as i64) >> 63) as u64; // all 1s for negative, 0 for positive
        let neg = !b;
        let pos = b ^ SIGN_BIT;
        let enc = (neg & mask) | (pos & !mask);
        self.buf.extend_from_slice(&enc.to_be_bytes());
        8
    }

    /// Append the native 4-byte sortable IEEE-754 encoding of an `f32`.
    ///
    /// # Panics
    ///
    /// Panics if `x` is NaN.
    #[inline(always)]
    pub fn encode_f32_into(&mut self, x: f32) -> usize {
        if x.is_nan() {
            panic!("NaN is not encodable in lexkeys");
        }
        let b = x.to_bits();
        let mask = ((b as i32) >> 31) as u32;
        let neg = !b;
        let pos = b ^ SIGN_BIT_32;
        let enc = (neg & mask) | (pos & !mask);
        self.buf.extend_from_slice(&enc.to_be_bytes());
        4
    }

    /// Append the 16-byte RFC-4122 UUID representation.
    #[inline(always)]
    pub fn encode_uuid_into_buf(&mut self, u: &Uuid) -> usize {
        self.buf.extend_from_slice(u.as_bytes());
        16
    }

    /// Append a composite multi-part key separated by `0x00`.
    ///
    /// Parts are copied as-is. Empty parts are allowed and can produce adjacent
    /// separators because every adjacent part pair is separated. No trailing separator is written.
    #[inline(always)]
    pub fn encode_composite_into_buf(&mut self, parts: &[&[u8]]) -> usize {
        if parts.is_empty() {
            return 0;
        }

        let total = crate::encode_len(parts);
        self.buf.reserve(total);
        crate::encode_parts_into(&mut self.buf, parts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_length_when_encoding_string() {
        let mut enc = Encoder::with_capacity(64);
        let n = enc.encode_string_into("hello");
        assert_eq!(n, 5);
    }

    #[test]
    fn should_encode_raw_bytes_and_markers() {
        let mut enc = Encoder::with_capacity(16);
        assert_eq!(enc.encode_bytes_into(b"realm"), 5);
        enc.push_separator();
        assert_eq!(enc.encode_bytes_into(b"kv"), 2);
        enc.push_end_marker();
        assert_eq!(enc.as_slice(), b"realm\x00kv\xff");
    }

    #[test]
    fn should_return_owned_vec_without_copying_through_bytes() {
        let mut enc = Encoder::with_capacity(16);
        enc.encode_string_into("realm");
        enc.push_separator();
        enc.encode_bytes_into(b"kv");
        enc.push_separator();

        let out = enc.into_vec();

        assert_eq!(out, b"realm\x00kv\x00");
    }

    #[test]
    fn should_return_eight_when_encoding_u64() {
        let mut enc = Encoder::with_capacity(64);
        let n = enc.encode_u64_into(0x0102030405060708);
        assert_eq!(n, 8);
    }

    #[test]
    fn should_return_native_lengths_when_encoding_narrow_unsigned() {
        let mut enc = Encoder::with_capacity(64);
        assert_eq!(enc.encode_u8_into(0x12), 1);
        assert_eq!(enc.encode_u16_into(0x3456), 2);
        assert_eq!(enc.encode_u32_into(0x789abcde), 4);
        assert_eq!(enc.as_slice(), &[0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde]);
    }

    #[test]
    fn should_return_eight_when_encoding_i64() {
        let mut enc = Encoder::with_capacity(64);
        let n = enc.encode_i64_into(-1);
        assert_eq!(n, 8);
    }

    #[test]
    fn should_return_native_lengths_when_encoding_narrow_signed() {
        let mut enc = Encoder::with_capacity(64);
        assert_eq!(enc.encode_i8_into(-1), 1);
        assert_eq!(enc.encode_i16_into(-1), 2);
        assert_eq!(enc.encode_i32_into(-1), 4);
        assert_eq!(enc.as_slice(), &[0x7f, 0x7f, 0xff, 0x7f, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn should_return_eight_when_encoding_f64() {
        let mut enc = Encoder::with_capacity(64);
        let n = enc.encode_f64_into(std::f64::consts::PI);
        assert_eq!(n, 8);
    }

    #[test]
    fn should_return_four_when_encoding_f32() {
        let mut enc = Encoder::with_capacity(64);
        let n = enc.encode_f32_into(std::f32::consts::PI);
        assert_eq!(n, 4);
    }

    #[test]
    fn should_return_sixteen_when_encoding_uuid() {
        let mut enc = Encoder::with_capacity(64);
        let u = Uuid::new_v4();
        let n = enc.encode_uuid_into_buf(&u);
        assert_eq!(n, 16);
    }

    #[test]
    fn should_encode_composite() {
        // Arrange
        let mut enc = Encoder::with_capacity(128);
        let u = Uuid::new_v4();
        let parts: Vec<&[u8]> = vec![b"a".as_ref(), b"".as_ref(), b"b".as_ref(), u.as_bytes()];

        // Act
        let n = enc.encode_composite_into_buf(&parts);

        // Assert
        assert!(n > 0);
        assert!(enc.as_slice().contains(&0x00));
    }

    #[test]
    fn should_yield_bytes_after_push_and_freeze() {
        let mut enc = Encoder::with_capacity(16);
        enc.push_byte(0xFF);
        let out = enc.freeze();
        assert!(!out.is_empty());
    }
}
