//! Nearest-neighbor-graph construction baseline (ALGO-GEO-005).
//!
//! Run: `cargo bench --bench bench_nearest_neighbor_graph`.
//! Results land under `target/criterion/`. A snapshot is committed to
//! `.codefuse/tracking/perf/ALGO-GEO-005.json`.
//!
//! `nearest_neighbor_graph` runs an O(n²·d) all-pairs scan and, per source
//! point, a partial sort of the in-cutoff candidates to pick the k nearest,
//! so cost grows quadratically in the point count. Two deterministic regimes
//! exercise the same kernel: a pseudo-random scatter (LCG, no dependency)
//! varying the point count at fixed k, and a fixed scatter varying k to show
//! the selection cost.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{DistanceMetric, nearest_neighbor_graph};

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
    let mut group = c.benchmark_group("nearest_neighbor_graph/point_count");
    for n in [50usize, 200, 400] {
        let pts = scatter(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &pts, |b, pts| {
            b.iter(|| {
                nearest_neighbor_graph(pts, DistanceMetric::Euclidean, 3, -1.0, true).expect("nng")
            });
        });
    }
    group.finish();
}

fn bench_k(c: &mut Criterion) {
    let mut group = c.benchmark_group("nearest_neighbor_graph/k");
    let pts = scatter(300);
    for k in [1i64, 5, 20] {
        group.bench_with_input(BenchmarkId::from_parameter(k), &k, |b, &k| {
            b.iter(|| {
                nearest_neighbor_graph(&pts, DistanceMetric::Euclidean, k, -1.0, true).expect("nng")
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_point_count, bench_k);
criterion_main!(benches);
