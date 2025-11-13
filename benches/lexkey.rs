use criterion::{black_box, criterion_group, criterion_main, Criterion};
mod common;
use common::bench_config;
use lexkey::LexKey;
use uuid::Uuid;

fn bench_encode_string(c: &mut Criterion) {
    let s = "a very long string used for benchmarking purposes that has some repeating patterns";
    c.bench_function("encode_string", |b| {
        b.iter(|| {
            let k = LexKey::encode_string(black_box(s));
            black_box(k.as_bytes());
        })
    });
}

fn bench_encode_i64(c: &mut Criterion) {
    c.bench_function("encode_i64", |b| {
        b.iter(|| {
            let k = LexKey::encode_i64(black_box(123456789i64));
            black_box(k.as_bytes());
        })
    });
}

fn bench_encode_i64_into(c: &mut Criterion) {
    c.bench_function("encode_i64_into", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(8); // exact width
            let n = LexKey::encode_i64_into(&mut buf, black_box(123456789i64));
            black_box((&buf[..], n));
        })
    });
}

fn bench_encode_f64(c: &mut Criterion) {
    c.bench_function("encode_f64", |b| {
        b.iter(|| {
            let k = LexKey::encode_f64(black_box(std::f64::consts::PI));
            black_box(k.as_bytes());
        })
    });
}

fn bench_encode_f64_into(c: &mut Criterion) {
    c.bench_function("encode_f64_into", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(8);
            let n = LexKey::encode_f64_into(&mut buf, black_box(std::f64::consts::PI));
            black_box((&buf[..], n));
        })
    });
}

fn bench_encode_composite(c: &mut Criterion) {
    let u = Uuid::new_v4();
    let parts: Vec<&[u8]> = vec![b"tenant", b"row", u.as_bytes()];

    c.bench_function("encode_composite", |b| {
        b.iter(|| {
            let k = LexKey::encode_composite(black_box(&parts));
            black_box(k.as_bytes());
        })
    });
}

fn bench_encode_composite_into(c: &mut Criterion) {
    let u = Uuid::new_v4();
    let parts: Vec<&[u8]> = vec![b"tenant", b"row", u.as_bytes()];
    let cap = parts.iter().map(|p| p.len()).sum::<usize>() + (parts.len() - 1);

    c.bench_function("encode_composite_into", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(cap);
            let n = LexKey::encode_composite_into(&mut buf, black_box(&parts));
            black_box((&buf[..], n));
        })
    });
}

fn bench_encode_i64_into_reuse(c: &mut Criterion) {
    c.bench_function("encode_i64_into_reuse", |b| {
        let mut buf = Vec::with_capacity(8);
        b.iter(|| {
            buf.clear();
            let n = LexKey::encode_i64_into(&mut buf, black_box(123456789i64));
            black_box((&buf[..], n));
        })
    });
}

fn bench_encode_f64_into_reuse(c: &mut Criterion) {
    c.bench_function("encode_f64_into_reuse", |b| {
        let mut buf = Vec::with_capacity(8);
        b.iter(|| {
            buf.clear();
            let n = LexKey::encode_f64_into(&mut buf, black_box(std::f64::consts::PI));
            black_box((&buf[..], n));
        })
    });
}

fn bench_encode_composite_into_reuse(c: &mut Criterion) {
    let u = Uuid::new_v4();
    let parts: Vec<&[u8]> = vec![b"tenant", b"row", u.as_bytes()];
    let cap = parts.iter().map(|p| p.len()).sum::<usize>() + (parts.len() - 1);

    c.bench_function("encode_composite_into_reuse", |b| {
        let mut buf = Vec::with_capacity(cap);
        b.iter(|| {
            buf.clear();
            let n = LexKey::encode_composite_into(&mut buf, black_box(&parts));
            black_box((&buf[..], n));
        })
    });
}

fn bench_composite_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("composite_parts_scaling");

    // Use exponential growth so the curve is really obvious
    for parts in [1usize, 2, 4, 8, 16, 32, 64, 128, 256].iter().cloned() {
        // Build synthetic parts of realistic size
        // 8-byte slices mimic UUIDs, timestamps, numeric components, etc.
        let piece = [0u8; 8];
        let vec_parts: Vec<&[u8]> = (0..parts).map(|_| &piece[..]).collect();

        // Precompute capacity so we benchmark encoding, not allocation
        let cap = parts * piece.len() + (parts - 1);

        group.bench_with_input(format!("parts={}", parts), &vec_parts, |b, p| {
            b.iter(|| {
                let mut buf = Vec::with_capacity(cap);
                let n = LexKey::encode_composite_into(&mut buf, black_box(p));
                black_box((&buf[..], n));
            })
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = bench_config();
    targets =
        bench_encode_string,
        bench_encode_i64,
        bench_encode_i64_into,
        bench_encode_f64,
        bench_encode_f64_into,
        bench_encode_composite,
        bench_encode_composite_into,
        bench_encode_i64_into_reuse,
        bench_encode_f64_into_reuse,
        bench_encode_composite_into_reuse,
        bench_composite_scaling,
}

criterion_main!(benches);
