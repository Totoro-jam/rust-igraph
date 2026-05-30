//! Relative-neighborhood-graph construction baseline (ALGO-GEO-004).
//!
//! Run: `cargo bench --bench bench_relative_neighborhood_graph`.
//! Results land under `target/criterion/`. A snapshot is committed to
//! `.codefuse/tracking/perf/ALGO-GEO-004.json`.
//!
//! `relative_neighborhood_graph` enumerates all O(n²) candidate pairs and
//! runs an O(n) empty-lune test per pair (O(n³) for fixed 2-D), so cost
//! grows cubically in the point count. Two deterministic regimes exercise
//! the same kernel: a pseudo-random scatter (LCG, no dependency) and a
//! triangular lattice (every edge co-equidistant, stressing the open-lune
//! boundary branch).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::relative_neighborhood_graph;

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

/// A `k × k` triangular lattice (k² points): every edge has a third lattice
/// point exactly equidistant, exercising the open-lune boundary branch.
fn triangular_lattice(k: usize) -> Vec<Vec<f64>> {
    let h = 3.0_f64.sqrt() / 2.0;
    let mut pts = Vec::with_capacity(k * k);
    for i in 0..k {
        for j in 0..k {
            #[allow(clippy::cast_precision_loss)]
            let y = i as f64 * h;
            #[allow(clippy::cast_precision_loss)]
            let x = j as f64 + if i % 2 == 0 { 0.0 } else { 0.5 };
            pts.push(vec![x, y]);
        }
    }
    pts
}

fn bench_scatter(c: &mut Criterion) {
    let mut group = c.benchmark_group("relative_neighborhood_graph/scatter");
    for n in [25usize, 100, 200] {
        let pts = scatter(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &pts, |b, pts| {
            b.iter(|| relative_neighborhood_graph(pts).expect("rng"));
        });
    }
    group.finish();
}

fn bench_lattice(c: &mut Criterion) {
    let mut group = c.benchmark_group("relative_neighborhood_graph/lattice");
    for k in [5usize, 10, 14] {
        let pts = triangular_lattice(k);
        let n = pts.len();
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &pts, |b, pts| {
            b.iter(|| relative_neighborhood_graph(pts).expect("rng"));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_scatter, bench_lattice);
criterion_main!(benches);
