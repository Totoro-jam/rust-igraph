//! K-regular random-graph benchmarks (ALGO-GN-008).
//!
//! Run: `cargo bench --bench bench_k_regular`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-008.json`.
//!
//! Coverage:
//!   * `n_scaling/simple` — fixed k=6, sweep n ∈ {50, 200, 1000} on the
//!     undirected simple sampler (fast-heur with restart cap).
//!   * `k_sweep/simple` — fixed n=200, sweep k ∈ {4, 16, 64} to capture
//!     stub-pairing cost vs duplicate-rejection cost.
//!   * `multigraph/configuration` — fixed n=500, k=10 undirected, runs
//!     the cheaper configuration-model path (no rejection sampling).
//!   * `directed/simple` — fixed n=200, k=8 directed (separate
//!     in-stub/out-stub bags).
//!
//! Throughput is reported in total vertices, so wall-clock per element
//! is directly comparable to the other generator benches.

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::k_regular_game;

fn bench_n_scaling_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("k_regular_game/n_scaling/simple");
    let k = 6u32;
    for n in [50u32, 200, 1000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| k_regular_game(n, k, false, false, 0x1514_5001).unwrap());
        });
    }
    group.finish();
}

fn bench_k_sweep_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("k_regular_game/k_sweep/simple");
    let n = 200u32;
    for k in [4u32, 16, 64] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(k), &k, |b, &k| {
            b.iter(|| k_regular_game(n, k, false, false, 0x1514_5002).unwrap());
        });
    }
    group.finish();
}

fn bench_multigraph(c: &mut Criterion) {
    let mut group = c.benchmark_group("k_regular_game/multigraph/configuration");
    let n = 500u32;
    let k = 10u32;
    group.throughput(Throughput::Elements(u64::from(n)));
    group.bench_function("n500_k10_undirected", |b| {
        b.iter(|| k_regular_game(n, k, false, true, 0x1514_5003).unwrap());
    });
    group.finish();
}

fn bench_directed_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("k_regular_game/directed/simple");
    let n = 200u32;
    let k = 8u32;
    group.throughput(Throughput::Elements(u64::from(n)));
    group.bench_function("n200_k8_directed", |b| {
        b.iter(|| k_regular_game(n, k, true, false, 0x1514_5004).unwrap());
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_n_scaling_simple,
    bench_k_sweep_simple,
    bench_multigraph,
    bench_directed_simple,
);
criterion_main!(benches);
