//! β-weighted Gabriel graph construction baseline (ALGO-GEO-008).
//!
//! Run: `cargo bench --bench bench_beta_weighted_gabriel_graph`.
//! Results land under `target/criterion/`. A snapshot is committed to
//! `.codefuse/tracking/perf/ALGO-GEO-008.json`.
//!
//! `beta_weighted_gabriel_graph` runs an O(n²) candidate enumeration with an
//! O(n·d) per-pair point scan (no early break — every other point can lower
//! the edge's β-threshold), so cost grows as O(n³·d). Two deterministic
//! regimes exercise the same kernel: a pseudo-random scatter (LCG, no
//! dependency) varying the point count at `max_beta = ∞`, and a fixed scatter
//! varying `max_beta` (a smaller cutoff lets the `beta < max_beta` test skip
//! updates sooner, but the per-pair scan still visits every point).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::beta_weighted_gabriel_graph;

/// Deterministic pseudo-random points in the unit square via a 64-bit LCG.
#[allow(clippy::cast_precision_loss)]
fn scatter(n: usize) -> Vec<Vec<f64>> {
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        // Top 53 bits → [0, 1); the 53-bit value is exact in f64.
        (state >> 11) as f64 / (1u64 << 53) as f64
    };
    (0..n).map(|_| vec![next(), next()]).collect()
}

fn bench_point_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("beta_weighted_gabriel_graph/point_count");
    for n in [50usize, 200, 400] {
        let pts = scatter(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &pts, |b, pts| {
            b.iter(|| beta_weighted_gabriel_graph(pts, f64::INFINITY).expect("weighted"));
        });
    }
    group.finish();
}

fn bench_max_beta(c: &mut Criterion) {
    let mut group = c.benchmark_group("beta_weighted_gabriel_graph/max_beta");
    let pts = scatter(300);
    for cutoff in [1.0f64, 5.0, f64::INFINITY] {
        group.bench_with_input(
            BenchmarkId::from_parameter(cutoff),
            &cutoff,
            |b, &cutoff| {
                b.iter(|| beta_weighted_gabriel_graph(&pts, cutoff).expect("weighted"));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_point_count, bench_max_beta);
criterion_main!(benches);
