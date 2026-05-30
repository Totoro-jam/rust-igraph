//! Gabriel-graph construction baseline (ALGO-GEO-003).
//!
//! Run: `cargo bench --bench bench_gabriel_graph`.
//! Results land under `target/criterion/`. A snapshot is committed to
//! `.codefuse/tracking/perf/ALGO-GEO-003.json`.
//!
//! `gabriel_graph` enumerates all O(n²) candidate pairs and runs an O(n)
//! empty-ball test per pair (O(n³) for fixed 2-D), so cost grows cubically in
//! the point count. Two deterministic regimes exercise the same kernel: a
//! pseudo-random scatter (LCG, no dependency) and a square integer lattice
//! (maximally co-circular, stressing the closed-ball boundary branch).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::gabriel_graph;

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

/// A `k × k` integer lattice (k² points): every cell's opposite corners are
/// co-circular, exercising the closed-ball boundary on every candidate.
fn lattice(k: usize) -> Vec<Vec<f64>> {
    let mut pts = Vec::with_capacity(k * k);
    for i in 0..k {
        for j in 0..k {
            #[allow(clippy::cast_precision_loss)]
            pts.push(vec![i as f64, j as f64]);
        }
    }
    pts
}

fn bench_scatter(c: &mut Criterion) {
    let mut group = c.benchmark_group("gabriel_graph/scatter");
    for n in [25usize, 100, 200] {
        let pts = scatter(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &pts, |b, pts| {
            b.iter(|| gabriel_graph(pts).expect("gabriel_graph"));
        });
    }
    group.finish();
}

fn bench_lattice(c: &mut Criterion) {
    let mut group = c.benchmark_group("gabriel_graph/lattice");
    for k in [5usize, 10, 14] {
        let pts = lattice(k);
        let n = pts.len();
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &pts, |b, pts| {
            b.iter(|| gabriel_graph(pts).expect("gabriel_graph"));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_scatter, bench_lattice);
criterion_main!(benches);
