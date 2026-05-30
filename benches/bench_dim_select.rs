//! Profile-likelihood dimensionality selection baseline (ALGO-EM-001).
//!
//! Run: `cargo bench --bench bench_dim_select`.
//! Results land under `target/criterion/`. A snapshot is committed to
//! `.codefuse/tracking/perf/ALGO-EM-001.json`.
//!
//! `dim_select` is a single O(n) running-sum scan over the (already ordered)
//! importance values, so cost grows linearly in the input length. Two regimes
//! exercise the same loop: a clean ascending ramp (elbow at the midpoint) and a
//! two-block step (a sharp gap), both deterministic and dependency-free.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::dim_select;

fn bench_ramp(c: &mut Criterion) {
    let mut group = c.benchmark_group("dim_select/ramp");
    for n in [100usize, 1_000, 10_000] {
        #[allow(clippy::cast_precision_loss)]
        let sv: Vec<f64> = (1..=n).map(|i| i as f64).collect();
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &sv, |b, sv| {
            b.iter(|| dim_select(sv).expect("dim_select"));
        });
    }
    group.finish();
}

fn bench_two_block(c: &mut Criterion) {
    let mut group = c.benchmark_group("dim_select/two_block");
    for n in [100usize, 1_000, 10_000] {
        // First half clustered high, second half clustered low: a sharp elbow.
        #[allow(clippy::cast_precision_loss)]
        let sv: Vec<f64> = (0..n)
            .map(|i| if i < n / 2 { 1000.0 - i as f64 } else { 1.0 })
            .collect();
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &sv, |b, sv| {
            b.iter(|| dim_select(sv).expect("dim_select"));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_ramp, bench_two_block);
criterion_main!(benches);
