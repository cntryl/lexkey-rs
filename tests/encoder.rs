use lexkey::{Encoder, LexKey};
use uuid::Uuid;

// Arrange/Act/Assert style tests, behavior-first names, single-Act per test.

#[test]
fn encoder_encode_string_returns_length() {
    // Arrange
    let mut enc = Encoder::with_capacity(64);

    // Act
    let n = enc.encode_string_into("hello");

    // Assert
    assert_eq!(n, 5);
}

#[test]
fn encoder_encode_u64_returns_eight() {
    // Arrange
    let mut enc = Encoder::with_capacity(64);

    // Act
    let n = enc.encode_u64_into(0x0102030405060708u64);

    // Assert
    assert_eq!(n, 8);
}

#[test]
fn encoder_encode_i64_returns_eight() {
    // Arrange
    let mut enc = Encoder::with_capacity(64);

    // Act
    let n = enc.encode_i64_into(-1i64);

    // Assert
    assert_eq!(n, 8);
}

#[test]
fn encoder_encode_f64_returns_eight() {
    // Arrange
    let mut enc = Encoder::with_capacity(64);

    // Act
    let n = enc.encode_f64_into(3.14);

    // Assert
    assert_eq!(n, 8);
}

#[test]
fn encoder_encode_uuid_returns_sixteen() {
    // Arrange
    let mut enc = Encoder::with_capacity(64);
    let u = Uuid::new_v4();

    // Act
    let n = enc.encode_uuid_into_buf(&u);

    // Assert
    assert_eq!(n, 16);
}

#[test]
fn encoder_encode_composite_returns_positive_length() {
    // Arrange
    let mut enc = Encoder::with_capacity(128);
    let u = Uuid::new_v4();
    let parts: Vec<&[u8]> = vec![b"a", b"b", u.as_bytes()];

    // Act
    let n = enc.encode_composite_into_buf(&parts);

    // Assert
    assert!(n > 0);
}

#[test]
fn encoder_push_byte_and_freeze_yields_non_empty_bytes() {
    // Arrange
    let mut enc = Encoder::with_capacity(16);

    // Act
    enc.push_byte(0xFF);
    let out = enc.freeze();

    // Assert
    assert!(!out.is_empty());
}

#[test]
fn lexkey_encode_i64_into_appends_eight_bytes() {
    // Arrange
    let mut buf = Vec::with_capacity(64);

    // Act
    let n = LexKey::encode_i64_into(&mut buf, 42i64);

    // Assert
    assert_eq!(n, 8);
    assert_eq!(buf.len(), 8);
}

#[test]
fn lexkey_clear_and_reuse_vec_for_encoding() {
    // Arrange
    let mut buf = Vec::with_capacity(64);
    let first = LexKey::encode_i64_into(&mut buf, 1i64);
    assert_eq!(first, 8);

    // Act (clear and reuse)
    buf.clear();
    let second = LexKey::encode_f64_into(&mut buf, 2.71828);

    // Assert
    assert_eq!(second, 8);
    assert_eq!(buf.len(), 8);
}
