use bytes::BufMut;
use bytes::{Bytes, BytesMut};
use uuid::Uuid;

/// A reusable encoder that holds an internal buffer for zero-allocation builds.
///
/// Use `Encoder` to build keys efficiently by reusing the same buffer across operations:
///
/// ```
/// use lexkey::{Encoder, LexKey};
/// let mut enc = Encoder::with_capacity(64);
/// enc.encode_string_into("tenant");
/// enc.push_byte(LexKey::SEPARATOR);
/// enc.encode_i64_into(123);
/// let bytes = enc.freeze();
/// assert!(!bytes.is_empty());
/// ```
pub struct Encoder {
    buf: BytesMut,
}

impl Encoder {
    /// Create a new encoder with an optional capacity hint.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buf: BytesMut::with_capacity(cap),
        }
    }

    /// Clear the internal buffer for reuse.
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    // Fast single-copy writer: reserve, get mutable bytes, copy and advance.
    fn write_bytes(&mut self, src: &[u8]) -> usize {
        let len = src.len();
        if len == 0 {
            return 0;
        }
        self.buf.reserve(len);
        // BufMut::put_slice will copy the bytes into the BytesMut buffer
        self.buf.put_slice(src);
        len
    }

    /// Freeze the internal buffer into an immutable `Bytes`.
    pub fn freeze(self) -> Bytes {
        self.buf.freeze()
    }

    /// Borrow the current buffer as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    /// Append a single byte.
    pub fn push_byte(&mut self, b: u8) {
        // use fast path to avoid temporary slice allocation
        self.buf.reserve(1);
        self.buf.put_u8(b);
    }

    /// Write a string's bytes into the buffer and return the number of bytes written.
    pub fn encode_string_into(&mut self, s: &str) -> usize {
        self.write_bytes(s.as_bytes())
    }

    /// Write the canonical 8-byte big-endian encoding of a `u64`.
    pub fn encode_u64_into(&mut self, n: u64) -> usize {
        // put_u64 writes big-endian
        self.buf.put_u64(n);
        8
    }

    /// Write the transformed 8-byte encoding of an `i64` for lexicographic ordering.
    pub fn encode_i64_into(&mut self, n: i64) -> usize {
        let u = (n as u64) ^ 0x8000_0000_0000_0000u64;
        self.buf.put_u64(u);
        8
    }

    /// Write the transformed 8-byte encoding of an `f64` for lexicographic ordering.
    pub fn encode_f64_into(&mut self, x: f64) -> usize {
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
        self.buf.put_u64(bits);
        8
    }

    /// Write 16 RFC4122 UUID bytes.
    pub fn encode_uuid_into_buf(&mut self, u: &Uuid) -> usize {
        self.write_bytes(u.as_bytes())
    }

    /// Write a composite sequence of parts (no trailing separator) using 0x00 separators.
    pub fn encode_composite_into_buf(&mut self, parts: &[&[u8]]) -> usize {
        // compute total once and reserve to avoid repeated growth checks
        let total: usize = crate::encode_len(parts);
        if total == 0 {
            return 0;
        }
        self.buf.reserve(total);
        let mut written = 0usize;
        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                self.buf.put_slice(part);
                written += part.len();
            }
            if i + 1 < parts.len() {
                self.buf.put_u8(0x00u8);
                written += 1;
            }
        }
        written
    }
}
