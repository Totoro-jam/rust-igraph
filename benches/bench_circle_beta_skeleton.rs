//! Circle β-skeleton construction baseline (ALGO-GEO-007).
//!
//! Run: `cargo bench --bench bench_circle_beta_skeleton`.
//! Results land under `target/criterion/`. A snapshot is committed to
//! `.codefuse/tracking/perf/ALGO-GEO-007.json`.
//!
//! `circle_beta_skeleton` runs an O(n²) candidate enumeration with an O(n)
//! union/intersection empty-region test per pair (2-D only), so cost grows
//! quadratically in the point count. Two deterministic regimes exercise the
//! same kernel: a pseudo-random scatter (LCG, no dependency) varying the point
//! count at fixed β, and a fixed scatter varying β across the β ≥ 1
//! union-empty construction and the β < 1 intersection-empty construction.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::circle_beta_skeleton;

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
    let mut group = c.benchmark_group("circle_beta_skeleton/point_count");
    for n in [50usize, 200, 400] {
        let pts = scatter(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &pts, |b, pts| {
            b.iter(|| circle_beta_skeleton(pts, 2.0).expect("circle"));
        });
    }
    group.finish();
}

fn bench_beta(c: &mut Criterion) {
    let mut group = c.benchmark_group("circle_beta_skeleton/beta");
    let pts = scatter(300);
    for beta in [0.5f64, 1.0, 2.0] {
        group.bench_with_input(BenchmarkId::from_parameter(beta), &beta, |b, &beta| {
            b.iter(|| circle_beta_skeleton(&pts, beta).expect("circle"));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_point_count, bench_beta);
criterion_main!(benches);
