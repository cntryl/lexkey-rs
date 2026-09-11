use cntryl_stress::{black_box, stress, StressContext};
use lexkey::LexKey;
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

#[stress(tier = 1, metadata(component = "lexkey", row_class = "allocation"))]
fn encode_string(ctx: &mut StressContext) {
    let value =
        "a very long string used for benchmarking purposes that has some repeating patterns";
    ctx.parameter("input_bytes", value.len());
    measure_encode_batch(ctx, "encode_string", || {
        let key = LexKey::encode_string(black_box(value));
        black_box(key.as_bytes());
    });
}

#[stress(tier = 1, metadata(component = "lexkey", row_class = "allocation"))]
fn encode_i64(ctx: &mut StressContext) {
    measure_encode_batch(ctx, "encode_i64", || {
        let key = LexKey::encode_i64(black_box(123_456_789_i64));
        black_box(key.as_bytes());
    });
}

#[stress(tier = 1, metadata(component = "lexkey", row_class = "allocation"))]
fn encode_i64_into(ctx: &mut StressContext) {
    measure_encode_batch(ctx, "encode_i64_into", || {
        let mut buffer = Vec::with_capacity(8);
        let written = LexKey::encode_i64_into(&mut buffer, black_box(123_456_789_i64));
        black_box((&buffer[..], written));
    });
}

#[stress(tier = 1, metadata(component = "lexkey", row_class = "allocation"))]
fn encode_f64(ctx: &mut StressContext) {
    measure_encode_batch(ctx, "encode_f64", || {
        let key = LexKey::encode_f64(black_box(std::f64::consts::PI));
        black_box(key.as_bytes());
    });
}

#[stress(tier = 1, metadata(component = "lexkey", row_class = "allocation"))]
fn encode_f64_into(ctx: &mut StressContext) {
    measure_encode_batch(ctx, "encode_f64_into", || {
        let mut buffer = Vec::with_capacity(8);
        let written = LexKey::encode_f64_into(&mut buffer, black_box(std::f64::consts::PI));
        black_box((&buffer[..], written));
    });
}

#[stress(tier = 1, metadata(component = "lexkey", row_class = "allocation"))]
fn encode_composite(ctx: &mut StressContext) {
    let uuid = Uuid::new_v4();
    let parts: Vec<&[u8]> = vec![b"tenant", b"row", uuid.as_bytes()];
    ctx.parameter("parts", parts.len());
    measure_encode_batch(ctx, "encode_composite", || {
        let key = LexKey::encode_composite(black_box(&parts));
        black_box(key.as_bytes());
    });
}

#[stress(tier = 1, metadata(component = "lexkey", row_class = "allocation"))]
fn encode_composite_into(ctx: &mut StressContext) {
    let uuid = Uuid::new_v4();
    let parts: Vec<&[u8]> = vec![b"tenant", b"row", uuid.as_bytes()];
    let capacity = parts.iter().map(|part| part.len()).sum::<usize>() + parts.len() - 1;
    ctx.parameter("parts", parts.len());
    measure_encode_batch(ctx, "encode_composite_into", || {
        let mut buffer = Vec::with_capacity(capacity);
        let written = LexKey::encode_composite_into(&mut buffer, black_box(&parts));
        black_box((&buffer[..], written));
    });
}

#[stress(
    tier = 1,
    max_allocs_per_op = 0,
    max_bytes_per_op = 0,
    metadata(component = "lexkey", validated_micro = "true")
)]
fn encode_i64_into_reuse(ctx: &mut StressContext) {
    const OPERATIONS: u64 = MICRO_BATCH_OPERATIONS;
    record_encode_batch(ctx, OPERATIONS);
    let mut buffer = Vec::with_capacity(8);
    let mut value = 123_456_789_i64;
    ctx.measure_batch("encode_i64_into_reuse", OPERATIONS, || {
        for _ in 0..OPERATIONS {
            buffer.clear();
            value = value.wrapping_add(0x0101_0101);
            let written = LexKey::encode_i64_into(&mut buffer, black_box(value));
            black_box((&buffer[..], written));
        }
    });
}

#[stress(
    tier = 1,
    max_allocs_per_op = 0,
    max_bytes_per_op = 0,
    metadata(component = "lexkey", validated_micro = "true")
)]
fn encode_f64_into_reuse(ctx: &mut StressContext) {
    const OPERATIONS: u64 = MICRO_BATCH_OPERATIONS;
    record_encode_batch(ctx, OPERATIONS);
    let mut buffer = Vec::with_capacity(8);
    let mut value = std::f64::consts::PI;
    ctx.measure_batch("encode_f64_into_reuse", OPERATIONS, || {
        for _ in 0..OPERATIONS {
            buffer.clear();
            value = f64::from_bits(value.to_bits().wrapping_add(1));
            let written = LexKey::encode_f64_into(&mut buffer, black_box(value));
            black_box((&buffer[..], written));
        }
    });
}

#[stress(
    tier = 1,
    max_allocs_per_op = 0,
    max_bytes_per_op = 0,
    metadata(component = "lexkey")
)]
fn encode_composite_into_reuse(ctx: &mut StressContext) {
    let uuid = Uuid::new_v4();
    let parts: Vec<&[u8]> = vec![b"tenant", b"row", uuid.as_bytes()];
    let capacity = parts.iter().map(|part| part.len()).sum::<usize>() + parts.len() - 1;
    let mut buffer = Vec::with_capacity(capacity);
    ctx.parameter("parts", parts.len());
    measure_encode_batch(ctx, "encode_composite_into_reuse", || {
        buffer.clear();
        let written = LexKey::encode_composite_into(&mut buffer, black_box(&parts));
        black_box((&buffer[..], written));
    });
}

#[stress(tier = 1, metadata(component = "lexkey", row_class = "allocation"))]
fn encode_composite_macro(ctx: &mut StressContext) {
    let uuid = Uuid::new_v4();
    measure_encode_batch(ctx, "encode_composite_macro", || {
        let key = lexkey::encode_composite!("tenant", 42_i64, black_box(true), black_box(&uuid));
        black_box(key.as_bytes());
    });
}

fn measure_composite_scaling(ctx: &mut StressContext, part_count: usize) {
    let piece = [0_u8; 8];
    let parts: Vec<&[u8]> = (0..part_count).map(|_| piece.as_slice()).collect();
    let capacity = part_count * piece.len() + part_count - 1;
    ctx.parameter("parts", part_count);
    ctx.parameter("part_bytes", piece.len());
    measure_encode_batch(ctx, "composite_parts_scaling", || {
        let mut buffer = Vec::with_capacity(capacity);
        let written = LexKey::encode_composite_into(&mut buffer, black_box(&parts));
        black_box((&buffer[..], written));
    });
}

macro_rules! composite_scaling_benchmark {
    ($name:ident, $parts:literal) => {
        #[stress(tier = 1, metadata(component = "lexkey", row_class = "allocation"))]
        fn $name(ctx: &mut StressContext) {
            measure_composite_scaling(ctx, $parts);
        }
    };
}

composite_scaling_benchmark!(composite_parts_scaling_1, 1);
composite_scaling_benchmark!(composite_parts_scaling_2, 2);
composite_scaling_benchmark!(composite_parts_scaling_4, 4);
composite_scaling_benchmark!(composite_parts_scaling_8, 8);
composite_scaling_benchmark!(composite_parts_scaling_16, 16);
composite_scaling_benchmark!(composite_parts_scaling_32, 32);
composite_scaling_benchmark!(composite_parts_scaling_64, 64);
composite_scaling_benchmark!(composite_parts_scaling_128, 128);
composite_scaling_benchmark!(composite_parts_scaling_256, 256);

#[stress(tier = 1, metadata(component = "lexkey", row_class = "allocation"))]
fn prefix_end_via_range_upper(ctx: &mut StressContext) {
    let prefix = b"acme\0kv\0users\0profile\0";
    ctx.parameter("prefix_bytes", prefix.len());
    measure_encode_batch(ctx, "prefix_end_via_range_upper", || {
        let output = LexKey::encode_range_upper(black_box(prefix), None)
            .as_bytes()
            .to_vec();
        black_box(output);
    });
}

#[stress(tier = 1, metadata(component = "lexkey", row_class = "allocation"))]
fn prefix_end_vec(ctx: &mut StressContext) {
    let prefix = b"acme\0kv\0users\0profile\0";
    ctx.parameter("prefix_bytes", prefix.len());
    measure_encode_batch(ctx, "prefix_end_vec", || {
        black_box(LexKey::prefix_end(black_box(prefix)));
    });
}

#[stress(tier = 1, metadata(component = "lexkey", row_class = "allocation"))]
fn prefix_successor(ctx: &mut StressContext) {
    let prefix = b"acme\0kv\0users\0profile\xff\xff";
    ctx.parameter("prefix_bytes", prefix.len());
    measure_encode_batch(ctx, "prefix_successor", || {
        black_box(LexKey::prefix_successor(black_box(prefix)));
    });
}

cntryl_stress::stress_main!();
