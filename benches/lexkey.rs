use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lexkey::LexKey;
use uuid::Uuid;

fn bench_encode_string(c: &mut Criterion) {
    let s = "a very long string used for benchmarking purposes that has some repeating patterns";
    c.bench_function("encode_string", |b| {
        b.iter(|| {
            let k = LexKey::encode_string(black_box(s));
            black_box(k);
        })
    });
}

fn bench_encode_i64(c: &mut Criterion) {
    c.bench_function("encode_i64", |b| {
        b.iter(|| {
            let k = LexKey::encode_i64(black_box(123456789i64));
            black_box(k);
        })
    });
}

fn bench_encode_i64_into(c: &mut Criterion) {
    c.bench_function("encode_i64_into", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(8);
            let n = LexKey::encode_i64_into(&mut buf, black_box(123456789i64));
            black_box((buf, n));
        })
    });
}

fn bench_encode_f64(c: &mut Criterion) {
    c.bench_function("encode_f64", |b| {
        b.iter(|| {
            let k = LexKey::encode_f64(black_box(3.141592653589793));
            black_box(k);
        })
    });
}

fn bench_encode_f64_into(c: &mut Criterion) {
    c.bench_function("encode_f64_into", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(8);
            let n = LexKey::encode_f64_into(&mut buf, black_box(3.141592653589793));
            black_box((buf, n));
        })
    });
}

fn bench_encode_composite(c: &mut Criterion) {
    let u = Uuid::new_v4();
    let parts: Vec<&[u8]> = vec![b"tenant", b"row", u.as_bytes()];
    c.bench_function("encode_composite", |b| {
        b.iter(|| {
            let k = LexKey::encode_composite(black_box(&parts));
            black_box(k);
        })
    });
}

fn bench_encode_composite_into(c: &mut Criterion) {
    let u = Uuid::new_v4();
    let parts: Vec<&[u8]> = vec![b"tenant", b"row", u.as_bytes()];
    c.bench_function("encode_composite_into", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(64);
            let n = LexKey::encode_composite_into(&mut buf, black_box(&parts));
            black_box((buf, n));
        })
    });
}

fn bench_encode_i64_into_reuse(c: &mut Criterion) {
    c.bench_function("encode_i64_into_reuse", |b| {
        let mut buf = Vec::with_capacity(8);
        b.iter(|| {
            buf.clear();
            let n = LexKey::encode_i64_into(&mut buf, black_box(123456789i64));
            black_box((&buf, n));
        })
    });
}

fn bench_encode_f64_into_reuse(c: &mut Criterion) {
    c.bench_function("encode_f64_into_reuse", |b| {
        let mut buf = Vec::with_capacity(8);
        b.iter(|| {
            buf.clear();
            let n = LexKey::encode_f64_into(&mut buf, black_box(3.141592653589793));
            black_box((&buf, n));
        })
    });
}

fn bench_encode_composite_into_reuse(c: &mut Criterion) {
    let u = Uuid::new_v4();
    let parts: Vec<&[u8]> = vec![b"tenant", b"row", u.as_bytes()];
    c.bench_function("encode_composite_into_reuse", |b| {
        let mut buf = Vec::with_capacity(64);
        b.iter(|| {
            buf.clear();
            let n = LexKey::encode_composite_into(&mut buf, black_box(&parts));
            black_box((&buf, n));
        })
    });
}

criterion_group!(
    benches,
    bench_encode_string,
    bench_encode_i64,
    bench_encode_i64_into,
    bench_encode_f64,
    bench_encode_f64_into,
    bench_encode_composite,
    bench_encode_composite_into,
    bench_encode_i64_into_reuse,
    bench_encode_f64_into_reuse,
    bench_encode_composite_into_reuse
);
criterion_main!(benches);
