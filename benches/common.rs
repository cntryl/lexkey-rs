use criterion::Criterion;
use std::time::Duration;

const SAMPLE_SIZE: usize = 200;
const MEASUREMENT_TIME: Duration = Duration::from_secs(10);
const WARM_UP_TIME: Duration = Duration::from_secs(5);
const BOOTSTRAP_RESAMPLES: usize = 200_000;
const CONFIDENCE_LEVEL: f64 = 0.99;
const SIGNIFICANCE_LEVEL: f64 = 0.01;
const NOISE_THRESHOLD: f64 = 0.01;

#[must_use]
pub fn bench_config() -> Criterion {
    Criterion::default()
        .sample_size(SAMPLE_SIZE)
        .measurement_time(MEASUREMENT_TIME)
        .warm_up_time(WARM_UP_TIME)
        .nresamples(BOOTSTRAP_RESAMPLES)
        .confidence_level(CONFIDENCE_LEVEL)
        .significance_level(SIGNIFICANCE_LEVEL)
        .noise_threshold(NOISE_THRESHOLD)
}
