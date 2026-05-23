//! Watts-Strogatz 1-D small-world benchmarks (ALGO-GN-009).
//!
//! Run: `cargo bench --bench bench_watts`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-009.json`.
//!
//! Coverage:
//!   * `size_scaling/p0_ring` — fixed `nei=4, p=0`, sweep
//!     `size ∈ {100, 1_000, 10_000}` — pure ring construction, no rewire
//!     work; isolates the `size * nei` edge-push cost.
//!   * `size_scaling/p_half_simple` — fixed `nei=4, p=0.5`, sweep
//!     `size ∈ {100, 1_000, 10_000}` — the canonical small-world regime
//!     against the duplicate-rejection `HashSet` path.
//!   * `p_sweep/size1000_nei4` — fixed `size=1_000, nei=4`, sweep
//!     `p ∈ {0.01, 0.1, 0.5, 1.0}` — how rewire intensity grows the
//!     duplicate-rejection cost.
//!   * `multigraph/fast_path` — `size=10_000, nei=4, p=0.5` with
//!     `loops=true, multiple=true` — the cheaper geometric-skip path
//!     with no rejection.
//!
//! Throughput is reported in total vertices so wall-clock per element
//! is directly comparable to the other generator benches.

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::watts_strogatz_game;

fn bench_size_scaling_p0(c: &mut Criterion) {
    let mut group = c.benchmark_group("watts_strogatz_game/size_scaling/p0_ring");
    let nei = 4u32;
    for size in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(size)));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| watts_strogatz_game(size, nei, 0.0, false, false, 0x1515_5001).unwrap());
        });
    }
    group.finish();
}

fn bench_size_scaling_p_half(c: &mut Criterion) {
    let mut group = c.benchmark_group("watts_strogatz_game/size_scaling/p_half_simple");
    let nei = 4u32;
    for size in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(size)));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| watts_strogatz_game(size, nei, 0.5, false, false, 0x1515_5002).unwrap());
        });
    }
    group.finish();
}

fn bench_p_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("watts_strogatz_game/p_sweep/size1000_nei4");
    let size = 1_000u32;
    let nei = 4u32;
    for p in [0.01_f64, 0.1, 0.5, 1.0] {
        group.throughput(Throughput::Elements(u64::from(size)));
        // criterion benchmark id label needs to be string-friendly.
        let label = format!("p{p:.2}");
        group.bench_with_input(BenchmarkId::from_parameter(label), &p, |b, &p| {
            b.iter(|| watts_strogatz_game(size, nei, p, false, false, 0x1515_5003).unwrap());
        });
    }
    group.finish();
}

fn bench_multigraph_fast_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("watts_strogatz_game/multigraph/fast_path");
    let size = 10_000u32;
    let nei = 4u32;
    group.throughput(Throughput::Elements(u64::from(size)));
    group.bench_function("size10000_nei4_p_half_loops_multi", |b| {
        b.iter(|| watts_strogatz_game(size, nei, 0.5, true, true, 0x1515_5004).unwrap());
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_size_scaling_p0,
    bench_size_scaling_p_half,
    bench_p_sweep,
    bench_multigraph_fast_path,
);
criterion_main!(benches);
