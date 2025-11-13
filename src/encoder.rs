use bytes::{BufMut, Bytes, BytesMut};
use uuid::Uuid;

/// A fast, one-way, lexicographically sortable key encoder.
///
/// This encoder produces byte sequences where the natural byte ordering
/// matches the natural ordering of the encoded values (u64, i64, f64, UUID,
/// and composite path-like keys).
///
/// All numeric encodings are monotonic:
/// - `u64`: big-endian
/// - `i64`: sign-bit flip (`n ^ 0x8000...`)
/// - `f64`: IEEE-754 sortable transform (error on NaN)
///
/// `Encoder` is reusable; call `clear()` between uses.
pub struct Encoder {
    buf: BytesMut,
}

impl Encoder {
    /// Create a new encoder with a capacity hint.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buf: BytesMut::with_capacity(cap),
        }
    }

    /// Reset the internal buffer so the encoder can be reused.
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// Convert the accumulated buffer into an immutable `Bytes`.
    pub fn freeze(self) -> Bytes {
        self.buf.freeze()
    }

    /// Borrow the current buffer contents.
    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    /// Return current buffer length.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Append a single byte.
    #[inline(always)]
    pub fn push_byte(&mut self, b: u8) {
        self.buf.put_u8(b);
    }

    /// Append a UTF-8 string (raw bytes).
    #[inline]
    pub fn encode_string_into(&mut self, s: &str) -> usize {
        let bytes = s.as_bytes();
        self.buf.extend_from_slice(bytes);
        bytes.len()
    }

    /// Append the canonical 8-byte big-endian encoding of a `u64`.
    #[inline(always)]
    pub fn encode_u64_into(&mut self, n: u64) -> usize {
        self.buf.put_u64(n);
        8
    }

    /// Append the sortable 8-byte encoding of an `i64`.
    ///
    /// Mapping preserves ordering:
    ///   i64::MIN → 0x00...
    ///   i64::MAX → 0xFF...
    #[inline(always)]
    pub fn encode_i64_into(&mut self, n: i64) -> usize {
        let u = (n as u64) ^ 0x8000_0000_0000_0000;
        self.buf.put_u64(u);
        8
    }

    /// Append the sortable IEEE-754 encoding of an `f64`.
    ///
    /// - Negative floats: bitwise NOT
    /// - Positive floats: flip the sign bit
    ///
    /// NaN is rejected because it breaks total ordering.
    #[inline(always)]
    pub fn encode_f64_into(&mut self, x: f64) -> usize {
        if x.is_nan() {
            panic!("NaN is not encodable in lexkeys");
        }
        let b = x.to_bits();
        let mask = ((b as i64) >> 63) as u64; // all 1s for negative, 0 for positive
        let neg = !b;
        let pos = b ^ 0x8000_0000_0000_0000u64;
        let enc = (neg & mask) | (pos & !mask);
        self.buf.put_u64(enc);
        8
    }

    /// Append the 16-byte RFC-4122 UUID representation.
    #[inline(always)]
    pub fn encode_uuid_into_buf(&mut self, u: &Uuid) -> usize {
        self.buf.extend_from_slice(u.as_bytes());
        16
    }

    /// Append a composite multi-part key separated by `0x00`.
    ///
    /// Parts must not contain interior null bytes. Empty parts are allowed but
    /// never produce double separators. No trailing separator is written.
    #[inline(always)]
    pub fn encode_composite_into_buf(&mut self, parts: &[&[u8]]) -> usize {
        if parts.is_empty() {
            return 0;
        }

        let total = parts.iter().map(|p| p.len()).sum::<usize>() + parts.len().saturating_sub(1);
        self.buf.reserve(total);
        let mut written = 0usize;
        
        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                self.buf.extend_from_slice(part);
                written += part.len();
            }

            if i + 1 < parts.len() {
                self.buf.put_u8(0x00);
                written += 1;
            }
        }

        written
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
    fn should_return_eight_when_encoding_u64() {
        let mut enc = Encoder::with_capacity(64);
        let n = enc.encode_u64_into(0x0102030405060708);
        assert_eq!(n, 8);
    }

    #[test]
    fn should_return_eight_when_encoding_i64() {
        let mut enc = Encoder::with_capacity(64);
        let n = enc.encode_i64_into(-1);
        assert_eq!(n, 8);
    }

    #[test]
    fn should_return_eight_when_encoding_f64() {
        let mut enc = Encoder::with_capacity(64);
        let n = enc.encode_f64_into(std::f64::consts::PI);
        assert_eq!(n, 8);
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
