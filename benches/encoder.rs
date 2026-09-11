use cntryl_stress::{black_box, stress, StressContext};
use lexkey::{Encoder, LexKey};
use uuid::Uuid;

cntryl_stress::stress_allocator!();

const MICRO_BATCH_OPERATIONS: u64 = 256;

fn record_encode_batch(ctx: &mut StressContext, operations: u64) {
    ctx.parameter("batch_operations", operations);
    ctx.parameter("logical_unit", "encode");
    ctx.parameter("encodes_per_logical_operation", 1);
}

fn measure_encode_batch(ctx: &mut StressContext, name: &str, mut operation: impl FnMut()) {
    record_encode_batch(ctx, MICRO_BATCH_OPERATIONS);
    ctx.measure_batch(name, MICRO_BATCH_OPERATIONS, || {
        for _ in 0..MICRO_BATCH_OPERATIONS {
            operation();
        }
    });
}

#[stress(tier = 1, metadata(component = "encoder", row_class = "allocation"))]
fn encoder_string_new(ctx: &mut StressContext) {
    let value = "a fairly typical string used for benchmarking encoders";
    ctx.parameter("input_bytes", value.len());
    measure_encode_batch(ctx, "encoder_string_new", || {
        let mut encoder = Encoder::with_capacity(value.len());
        let written = encoder.encode_string_into(black_box(value));
        black_box((encoder.as_slice(), written));
    });
}

#[stress(
    tier = 1,
    max_allocs_per_op = 0,
    max_bytes_per_op = 0,
    metadata(component = "encoder")
)]
fn encoder_string_reuse(ctx: &mut StressContext) {
    const OPERATIONS: u64 = MICRO_BATCH_OPERATIONS;
    record_encode_batch(ctx, OPERATIONS);
    let value = "a fairly typical string used for benchmarking encoders";
    let mut encoder = Encoder::with_capacity(value.len());
    ctx.parameter("input_bytes", value.len());
    ctx.measure_batch("encoder_string_reuse", OPERATIONS, || {
        for _ in 0..OPERATIONS {
            encoder.clear();
            let written = encoder.encode_string_into(black_box(value));
            black_box((encoder.as_slice(), written));
        }
    });
}

#[stress(tier = 1, metadata(component = "encoder", row_class = "allocation"))]
fn encoder_u64_new(ctx: &mut StressContext) {
    measure_encode_batch(ctx, "encoder_u64_new", || {
        let mut encoder = Encoder::with_capacity(8);
        let written = encoder.encode_u64_into(black_box(0x0102_0304_0506_0708_u64));
        black_box((encoder.as_slice(), written));
    });
}

#[stress(
    tier = 1,
    max_allocs_per_op = 0,
    max_bytes_per_op = 0,
    metadata(component = "encoder", validated_micro = "true")
)]
fn encoder_u64_reuse(ctx: &mut StressContext) {
    const OPERATIONS: u64 = MICRO_BATCH_OPERATIONS;
    record_encode_batch(ctx, OPERATIONS);
    let mut encoder = Encoder::with_capacity(8);
    let mut value = 0x0102_0304_0506_0708_u64;
    ctx.measure_batch("encoder_u64_reuse", OPERATIONS, || {
        for _ in 0..OPERATIONS {
            encoder.clear();
            value = value.rotate_left(7).wrapping_add(1);
            let written = encoder.encode_u64_into(black_box(value));
            black_box((encoder.as_slice(), written));
        }
    });
}

#[stress(tier = 1, metadata(component = "encoder", row_class = "allocation"))]
fn encoder_i64_new(ctx: &mut StressContext) {
    measure_encode_batch(ctx, "encoder_i64_new", || {
        let mut encoder = Encoder::with_capacity(8);
        let written = encoder.encode_i64_into(black_box(-123_456_789_i64));
        black_box((encoder.as_slice(), written));
    });
}

#[stress(
    tier = 1,
    max_allocs_per_op = 0,
    max_bytes_per_op = 0,
    metadata(component = "encoder", validated_micro = "true")
)]
fn encoder_i64_reuse(ctx: &mut StressContext) {
    const OPERATIONS: u64 = MICRO_BATCH_OPERATIONS;
    record_encode_batch(ctx, OPERATIONS);
    let mut encoder = Encoder::with_capacity(8);
    let mut value = -123_456_789_i64;
    ctx.measure_batch("encoder_i64_reuse", OPERATIONS, || {
        for _ in 0..OPERATIONS {
            encoder.clear();
            value = value.wrapping_add(0x0101_0101);
            let written = encoder.encode_i64_into(black_box(value));
            black_box((encoder.as_slice(), written));
        }
    });
}

#[stress(tier = 1, metadata(component = "encoder", row_class = "allocation"))]
fn encoder_f64_new(ctx: &mut StressContext) {
    measure_encode_batch(ctx, "encoder_f64_new", || {
        let mut encoder = Encoder::with_capacity(8);
        let written = encoder.encode_f64_into(black_box(std::f64::consts::PI));
        black_box((encoder.as_slice(), written));
    });
}

#[stress(
    tier = 1,
    max_allocs_per_op = 0,
    max_bytes_per_op = 0,
    metadata(component = "encoder", validated_micro = "true")
)]
fn encoder_f64_reuse(ctx: &mut StressContext) {
    const OPERATIONS: u64 = MICRO_BATCH_OPERATIONS;
    record_encode_batch(ctx, OPERATIONS);
    let mut encoder = Encoder::with_capacity(8);
    let mut value = std::f64::consts::PI;
    ctx.measure_batch("encoder_f64_reuse", OPERATIONS, || {
        for _ in 0..OPERATIONS {
            encoder.clear();
            value = f64::from_bits(value.to_bits().wrapping_add(1));
            let written = encoder.encode_f64_into(black_box(value));
            black_box((encoder.as_slice(), written));
        }
    });
}

#[stress(tier = 1, metadata(component = "encoder", row_class = "allocation"))]
fn encoder_uuid_new(ctx: &mut StressContext) {
    let uuid = Uuid::new_v4();
    measure_encode_batch(ctx, "encoder_uuid_new", || {
        let mut encoder = Encoder::with_capacity(16);
        let written = encoder.encode_uuid_into_buf(black_box(&uuid));
        black_box((encoder.as_slice(), written));
    });
}

#[stress(
    tier = 1,
    max_allocs_per_op = 0,
    max_bytes_per_op = 0,
    metadata(component = "encoder", validated_micro = "true")
)]
fn encoder_uuid_reuse(ctx: &mut StressContext) {
    const OPERATIONS: u64 = MICRO_BATCH_OPERATIONS;
    record_encode_batch(ctx, OPERATIONS);
    let mut uuid_bytes = *Uuid::new_v4().as_bytes();
    let mut encoder = Encoder::with_capacity(16);
    ctx.measure_batch("encoder_uuid_reuse", OPERATIONS, || {
        for _ in 0..OPERATIONS {
            encoder.clear();
            uuid_bytes[15] = uuid_bytes[15].wrapping_add(1);
            let uuid = Uuid::from_bytes(uuid_bytes);
            let written = encoder.encode_uuid_into_buf(black_box(&uuid));
            black_box((encoder.as_slice(), written));
        }
    });
}

#[stress(tier = 1, metadata(component = "encoder", row_class = "allocation"))]
fn encoder_composite_new(ctx: &mut StressContext) {
    let uuid = Uuid::new_v4();
    let parts: Vec<&[u8]> = vec![b"tenant", b"row", uuid.as_bytes()];
    ctx.parameter("parts", parts.len());
    measure_encode_batch(ctx, "encoder_composite_new", || {
        let mut encoder = Encoder::with_capacity(64);
        let written = encoder.encode_composite_into_buf(black_box(&parts));
        black_box((encoder.as_slice(), written));
    });
}

#[stress(
    tier = 1,
    max_allocs_per_op = 0,
    max_bytes_per_op = 0,
    metadata(component = "encoder")
)]
fn encoder_composite_reuse(ctx: &mut StressContext) {
    let uuid = Uuid::new_v4();
    let i64_bytes = LexKey::encode_i64(123).as_bytes().to_vec();
    let parts: Vec<&[u8]> = vec![b"tenant", b"row", uuid.as_bytes(), &i64_bytes];
    let mut encoder = Encoder::with_capacity(128);
    ctx.parameter("parts", parts.len());
    measure_encode_batch(ctx, "encoder_composite_reuse", || {
        encoder.clear();
        let written = encoder.encode_composite_into_buf(black_box(&parts));
        black_box((encoder.as_slice(), written));
    });
}

#[stress(tier = 1, metadata(component = "encoder", row_class = "allocation"))]
fn encoder_fitz_domain_prefix_freeze_to_vec(ctx: &mut StressContext) {
    let realm = "acme";
    let domain = b"kv";
    measure_encode_batch(ctx, "encoder_fitz_domain_prefix_freeze_to_vec", || {
        let mut encoder = Encoder::with_capacity(realm.len() + domain.len() + 2);
        encoder.encode_string_into(black_box(realm));
        encoder.push_byte(LexKey::SEPARATOR);
        encoder.encode_composite_into_buf(&[black_box(domain.as_slice())]);
        encoder.push_byte(LexKey::SEPARATOR);
        black_box(encoder.freeze().to_vec());
    });
}

#[stress(tier = 1, metadata(component = "encoder", row_class = "allocation"))]
fn encoder_fitz_domain_prefix_into_vec(ctx: &mut StressContext) {
    let realm = "acme";
    let domain = b"kv";
    measure_encode_batch(ctx, "encoder_fitz_domain_prefix_into_vec", || {
        let mut encoder = Encoder::with_capacity(realm.len() + domain.len() + 2);
        encoder.encode_string_into(black_box(realm));
        encoder.push_separator();
        encoder.encode_bytes_into(black_box(domain.as_slice()));
        encoder.push_separator();
        black_box(encoder.into_vec());
    });
}

cntryl_stress::stress_main!();
