use criterion::{black_box, criterion_group, criterion_main, Criterion};
mod common;
use common::bench_config;
use lexkey::{Encoder, LexKey};
use uuid::Uuid;

fn bench_encoder_string_new(c: &mut Criterion) {
    let s = "a fairly typical string used for benchmarking encoders";
    let len = s.len();
    c.bench_function("encoder_string_new", |b| {
        b.iter(|| {
            let mut enc = Encoder::with_capacity(len);
            let n = enc.encode_string_into(black_box(s));
            black_box(enc.as_slice());
            black_box(n);
        })
    });
}

fn bench_encoder_string_reuse(c: &mut Criterion) {
    let s = "a fairly typical string used for benchmarking encoders";
    let len = s.len();
    c.bench_function("encoder_string_reuse", |b| {
        let mut enc = Encoder::with_capacity(len);
        b.iter(|| {
            enc.clear();
            let n = enc.encode_string_into(black_box(s));
            black_box(enc.as_slice());
            black_box(n);
        })
    });
}

fn bench_encoder_u64_new(c: &mut Criterion) {
    let v = 0x0102_0304_0506_0708u64;
    c.bench_function("encoder_u64_new", |b| {
        b.iter(|| {
            let mut enc = Encoder::with_capacity(8);
            let n = enc.encode_u64_into(black_box(v));
            black_box(enc.as_slice());
            black_box(n);
        })
    });
}

fn bench_encoder_u64_reuse(c: &mut Criterion) {
    let v = 0x0102_0304_0506_0708u64;
    c.bench_function("encoder_u64_reuse", |b| {
        let mut enc = Encoder::with_capacity(8);
        b.iter(|| {
            enc.clear();
            let n = enc.encode_u64_into(black_box(v));
            black_box(enc.as_slice());
            black_box(n);
        })
    });
}

fn bench_encoder_i64_new(c: &mut Criterion) {
    c.bench_function("encoder_i64_new", |b| {
        b.iter(|| {
            let mut enc = Encoder::with_capacity(8);
            let n = enc.encode_i64_into(black_box(-123456789i64));
            black_box(enc.as_slice());
            black_box(n);
        })
    });
}

fn bench_encoder_i64_reuse(c: &mut Criterion) {
    c.bench_function("encoder_i64_reuse", |b| {
        let mut enc = Encoder::with_capacity(8);
        b.iter(|| {
            enc.clear();
            let n = enc.encode_i64_into(black_box(-123456789i64));
            black_box(enc.as_slice());
            black_box(n);
        })
    });
}

fn bench_encoder_f64_new(c: &mut Criterion) {
    c.bench_function("encoder_f64_new", |b| {
        b.iter(|| {
            let mut enc = Encoder::with_capacity(8);
            let n = enc.encode_f64_into(black_box(std::f64::consts::PI));
            black_box(enc.as_slice());
            black_box(n);
        })
    });
}

fn bench_encoder_f64_reuse(c: &mut Criterion) {
    c.bench_function("encoder_f64_reuse", |b| {
        let mut enc = Encoder::with_capacity(8);
        b.iter(|| {
            enc.clear();
            let n = enc.encode_f64_into(black_box(std::f64::consts::PI));
            black_box(enc.as_slice());
            black_box(n);
        })
    });
}

fn bench_encoder_uuid_new(c: &mut Criterion) {
    let u = Uuid::new_v4();
    c.bench_function("encoder_uuid_new", |b| {
        b.iter(|| {
            let mut enc = Encoder::with_capacity(16);
            let n = enc.encode_uuid_into_buf(black_box(&u));
            black_box(enc.as_slice());
            black_box(n);
        })
    });
}

fn bench_encoder_uuid_reuse(c: &mut Criterion) {
    let u = Uuid::new_v4();
    c.bench_function("encoder_uuid_reuse", |b| {
        let mut enc = Encoder::with_capacity(16);
        b.iter(|| {
            enc.clear();
            let n = enc.encode_uuid_into_buf(black_box(&u));
            black_box(enc.as_slice());
            black_box(n);
        })
    });
}

fn bench_encoder_composite_new(c: &mut Criterion) {
    let u = Uuid::new_v4();
    let parts: Vec<&[u8]> = vec![b"tenant", b"row", u.as_bytes()];
    c.bench_function("encoder_composite_new", |b| {
        b.iter(|| {
            let mut enc = Encoder::with_capacity(64);
            let n = enc.encode_composite_into_buf(black_box(&parts));
            black_box(enc.as_slice());
            black_box(n);
        })
    });
}

fn bench_encoder_composite_reuse(c: &mut Criterion) {
    let u = Uuid::new_v4();
    let i64_bytes = LexKey::encode_i64(123).as_bytes().to_vec();
    let parts: Vec<&[u8]> = vec![b"tenant", b"row", u.as_bytes(), &i64_bytes[..]];
    c.bench_function("encoder_composite_reuse", |b| {
        let mut enc = Encoder::with_capacity(128);
        b.iter(|| {
            enc.clear();
            let n = enc.encode_composite_into_buf(black_box(&parts));
            black_box(enc.as_slice());
            black_box(n);
        })
    });
}

criterion_group! {
    name = encoder_benches;
    config = bench_config();
    targets =
        bench_encoder_string_new,
        bench_encoder_string_reuse,
        bench_encoder_u64_new,
        bench_encoder_u64_reuse,
        bench_encoder_i64_new,
        bench_encoder_i64_reuse,
        bench_encoder_f64_new,
        bench_encoder_f64_reuse,
        bench_encoder_uuid_new,
        bench_encoder_uuid_reuse,
        bench_encoder_composite_new,
        bench_encoder_composite_reuse,
}
criterion_main!(encoder_benches);
