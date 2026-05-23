//! Growing random graph benchmarks (ALGO-GN-003).
//!
//! Run: `cargo bench --bench bench_growing_random`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-GN-003.json`.
//!
//! Coverage:
//!   - Citation mode `m = 2`: dominant path (one RNG draw per edge,
//!     fixed source).
//!   - Free mode `m = 2`: two RNG draws per edge, exercises the
//!     symmetric uniform-uniform kernel.
//!   - Citation mode `m = 5`: same shape, denser bag — measures the
//!     constant-factor cost of repeated emits on a single step.

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::growing_random_game;

fn bench_growing_citation_m2(c: &mut Criterion) {
    let mut group = c.benchmark_group("growing_random/citation_m2");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| growing_random_game(n, 2, true, true, 0xC1A0).unwrap());
        });
    }
    group.finish();
}

fn bench_growing_free_m2(c: &mut Criterion) {
    let mut group = c.benchmark_group("growing_random/free_m2");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| growing_random_game(n, 2, true, false, 0xBA_BA_BA_BA).unwrap());
        });
    }
    group.finish();
}

fn bench_growing_citation_m5(c: &mut Criterion) {
    let mut group = c.benchmark_group("growing_random/citation_m5");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| growing_random_game(n, 5, true, true, 0xDEAD_BEEF).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_growing_citation_m2,
    bench_growing_free_m2,
    bench_growing_citation_m5,
);
criterion_main!(benches);
