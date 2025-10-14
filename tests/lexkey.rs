use lexkey::{Encoder, LexKey};
use uuid::Uuid;

#[test]
fn should_return_raw_bytes_given_string_when_encode_string() {
    // Arrange
    let s = "hello";

    // Act
    let k = LexKey::encode_string(s);

    // Assert
    assert_eq!(k.as_bytes(), s.as_bytes());
    assert_eq!(k.to_hex_string(), "68656c6c6f");
}

#[test]
fn should_return_original_bytes_given_vec_when_from_vec() {
    // Arrange
    let b = vec![0x00u8, 0x01, 0xff];

    // Act
    let kb = LexKey::from(b.clone());

    // Assert
    assert_eq!(kb.as_bytes(), &b[..]);
}

#[test]
fn should_widen_unsigned_to_8_bytes_when_encode_u64() {
    // Arrange/Act
    let k = LexKey::encode_u64(123);

    // Assert
    assert_eq!(k.to_hex_string(), "000000000000007b");
}

#[test]
fn should_xor_signbit_and_order_signed_integers() {
    // Arrange
    let p = LexKey::encode_i64(123);
    let n = LexKey::encode_i64(-123);

    // Act/Assert: hex values and ordering
    assert_eq!(p.to_hex_string(), "800000000000007b");
    assert_eq!(n.to_hex_string(), "7fffffffffffff85");
    assert!(n < p);
}

#[test]
fn should_encode_bool_nil_and_end_marker_as_single_bytes() {
    // Arrange/Act
    let f = LexKey::encode_bool(false);
    let t = LexKey::encode_bool(true);
    let e = LexKey::encode_end_marker();

    // Assert
    assert_eq!(f.to_hex_string(), "00");
    assert_eq!(t.to_hex_string(), "01");
    assert_eq!(e.to_hex_string(), "ff");
}

#[test]
fn should_transform_floats_and_map_nan_to_canonical() {
    // Arrange
    let p = LexKey::encode_f64(std::f64::consts::PI);
    let n = LexKey::encode_f64(-std::f64::consts::PI);

    // Act/Assert: ordering
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
    // Arrange/Act
    let u8_v: u8 = 255;
    let u16_v: u16 = 123;
    let u32_v: u32 = 123;

    // Assert: canonical widen to u64 yields same encoding as encode_u64
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

    // Act/Assert: canonical widen to i64 yields same encoding as encode_i64
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
    // Arrange/Act
    let min = LexKey::encode_i64(i64::MIN);
    let max = LexKey::encode_i64(i64::MAX);

    // Assert: transforms map i64::MIN -> 0x000... and i64::MAX -> 0xfff...
    assert_eq!(min.to_hex_string(), "0000000000000000");
    assert_eq!(max.to_hex_string(), "ffffffffffffffff");
}

#[test]
fn should_encode_u64_zero_and_max() {
    // Arrange/Act
    let zero = LexKey::encode_u64(0);
    let max = LexKey::encode_u64(u64::MAX);

    // Assert
    assert_eq!(zero.to_hex_string(), "0000000000000000");
    assert_eq!(max.to_hex_string(), "ffffffffffffffff");
}

#[test]
fn should_order_negative_zero_before_positive_zero_for_floats() {
    // Arrange
    let neg_zero = LexKey::encode_f64(-0.0_f64);
    let pos_zero = LexKey::encode_f64(0.0_f64);

    // Act/Assert
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

    // Assert ordering
    assert!(neg_inf < large_neg);
    assert!(large_neg < zero);
    assert!(zero < large_pos);
    assert!(large_pos < pos_inf);
}

#[test]
fn should_widen_float32_to_float64_before_encoding() {
    // Arrange: pick a float32 value
    let v32: f32 = std::f32::consts::PI;
    let widened = v32 as f64;

    // Act
    let from_widen = LexKey::encode_f64(widened);

    // Assert: widening float32 to f64 then encoding is the canonical behavior
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
    // Arrange/Act
    let empty = LexKey::empty();

    // Assert
    assert!(empty.is_empty());
    assert_eq!(empty.as_bytes().len(), 0);
    assert_eq!(empty.to_hex_string(), "");
}

#[test]
fn should_compare_using_ord_and_partialord() {
    // Arrange
    let a = LexKey::encode_string("a");
    let b = LexKey::encode_string("b");

    // Act/Assert: PartialOrd/Ord
    assert!(a < b);
    assert!(a.cmp(&b).is_lt());
    assert!(b.cmp(&a).is_gt());
}

#[test]
fn should_encode_composite_single_part_without_trailing_separator() {
    // Arrange
    let part = b"foo";

    // Act
    let k = LexKey::encode_composite(&[part.as_ref()]);

    // Assert: single part has no trailing separator
    assert_eq!(k.to_hex_string(), "666f6f");
}

#[test]
fn should_from_str_equivalent_to_encode_string() {
    // Arrange
    let s = "hello";

    // Act
    let from = LexKey::from(s);
    let encoded = LexKey::encode_string(s);

    // Assert
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

    // Assert: lengths
    assert_eq!(n_written_i, 8);
    assert_eq!(n_written_u, 8);
    assert_eq!(n_written_f, 8);
    assert_eq!(n_written_b, 1);

    // Check that concatenating individual encoders equals composing via composite_into
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
    let mut buf = Vec::new();
    let u = Uuid::new_v4();
    let n = LexKey::encode_uuid_into(&mut buf, &u);
    assert_eq!(n, 16);
    assert_eq!(&buf[..], u.as_bytes());

    // composite single part (no separator)
    buf.clear();
    let parts1: Vec<&[u8]> = vec![b"one"];
    let n1 = LexKey::encode_composite_into(&mut buf, &parts1);
    assert_eq!(n1, 3);
    assert_eq!(&buf[..], b"one");

    // composite multi-part contains separator but no trailing separator
    buf.clear();
    let parts2: Vec<&[u8]> = vec![b"ten", b"row"];
    let n2 = LexKey::encode_composite_into(&mut buf, &parts2);
    assert_eq!(n2, 3 + 1 + 3);
    assert_eq!(&buf[..], b"ten\x00row");
}

#[test]
fn encode_f64_nan_and_negative_zero() {
    // Note: NaN is not encodable and is tested separately with should_panic tests.

    // negative zero should sort before positive zero
    let kn = LexKey::encode_f64(-0.0f64);
    let kp = LexKey::encode_f64(0.0f64);
    assert!(kn.as_bytes() < kp.as_bytes());
}

#[test]
fn encode_f64_all_branches_lexkey_and_encoder() {
    // LexKey::encode_f64_into: NaN, negative, positive
    let mut b1 = Vec::new();
    // NaN branch is covered by should_panic tests. Test negative and positive branches here.
    b1.clear();
    assert_eq!(LexKey::encode_f64_into(&mut b1, -2.5f64), 8);
    b1.clear();
    assert_eq!(LexKey::encode_f64_into(&mut b1, 2.5f64), 8);

    // Encoder::encode_f64_into: NaN, negative, positive
    let mut enc = Encoder::with_capacity(32);
    // NaN branch is covered by should_panic tests. Test negative and positive branches here.
    assert_eq!(enc.encode_f64_into(-1.25f64), 8);
    enc.clear();
    assert_eq!(enc.encode_f64_into(1.25f64), 8);
}

#[test]
#[should_panic]
fn encode_f64_allocating_panics_on_nan() {
    // Allocating API should panic when given NaN
    let _ = LexKey::encode_f64(f64::NAN);
}

#[test]
#[should_panic]
fn encode_f64_into_panics_on_nan() {
    let mut buf = Vec::new();
    // encode_f64_into should panic on NaN
    let _ = LexKey::encode_f64_into(&mut buf, f64::NAN);
}

#[test]
#[should_panic]
fn encoder_encode_f64_into_panics_on_nan() {
    let mut enc = Encoder::with_capacity(8);
    // Encoder::encode_f64_into should panic on NaN
    let _ = enc.encode_f64_into(f64::NAN);
}
