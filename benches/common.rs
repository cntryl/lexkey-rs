use criterion::Criterion;
use std::time::Duration;

pub fn bench_config() -> Criterion {
    // Fast path if FAST_BENCH env var is set, otherwise use defaults
    if std::env::var("BENCH_FULL").is_ok() {
        Criterion::default()
    } else {
        Criterion::default()
            .sample_size(10)
            .measurement_time(Duration::from_secs(1))
            .warm_up_time(Duration::from_millis(250))
    }
}
