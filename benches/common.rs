use criterion::Criterion;
use std::time::Duration;

pub fn bench_config() -> Criterion {
    Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(1))
        .warm_up_time(Duration::from_millis(250))
}
