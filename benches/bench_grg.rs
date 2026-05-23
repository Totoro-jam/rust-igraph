//! Geometric random graph generator benchmarks (ALGO-GN-005).
//!
//! Run: `cargo bench --bench bench_grg`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-GN-005.json`.
//!
//! Coverage: plane and torus at three vertex-count scales, with a
//! radius chosen so the expected average degree stays near `10`. That
//! keeps the edge count in the sparse regime where the x-sweep
//! algorithm is asymptotically efficient (`O(n + |E|)` after the
//! `O(n log n)` sort) — and matches typical spatial-network usage.

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::grg_game;

/// Radius for an expected average degree of ~10:
/// `E[deg] ≈ (n-1)·π·r²` (interior approx) ⇒ `r = sqrt(10 / (π·(n-1)))`.
fn radius_for_n(n: u32) -> f64 {
    let n = f64::from(n);
    (10.0 / (std::f64::consts::PI * (n - 1.0))).sqrt()
}

fn bench_grg_plane(c: &mut Criterion) {
    let mut group = c.benchmark_group("grg_game/plane");
    for n in [100u32, 1_000, 10_000] {
        let r = radius_for_n(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| grg_game(n, r, false, 0xC0FF_EE05).unwrap());
        });
    }
    group.finish();
}

fn bench_grg_torus(c: &mut Criterion) {
    let mut group = c.benchmark_group("grg_game/torus");
    for n in [100u32, 1_000, 10_000] {
        let r = radius_for_n(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| grg_game(n, r, true, 0xDEAD_BE05).unwrap());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_grg_plane, bench_grg_torus);
criterion_main!(benches);
