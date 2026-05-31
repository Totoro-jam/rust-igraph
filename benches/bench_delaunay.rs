//! Delaunay triangulation baseline (ALGO-GEO-009).
//!
//! Run: `cargo bench --bench bench_delaunay`.
//! Results land under `target/criterion/`. A snapshot is committed to
//! `.codefuse/tracking/perf/ALGO-GEO-009.json`.
//!
//! `delaunay_graph` runs Bowyer-Watson incremental insertion in O(n²)
//! average. Two regimes: vary point count on a pseudo-random scatter,
//! and test a near-collinear configuration (worst case for super-triangle
//! sizing) at fixed n.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::delaunay_graph;

#[allow(clippy::cast_precision_loss)]
fn scatter(n: usize) -> Vec<Vec<f64>> {
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 11) as f64 / (1u64 << 53) as f64
    };
    (0..n).map(|_| vec![next(), next()]).collect()
}

fn bench_point_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("delaunay/point_count");
    for n in [20usize, 100, 500] {
        let pts = scatter(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &pts, |b, pts| {
            b.iter(|| delaunay_graph(pts).expect("delaunay"));
        });
    }
    group.finish();
}

#[allow(clippy::cast_precision_loss)]
fn bench_near_collinear(c: &mut Criterion) {
    let mut group = c.benchmark_group("delaunay/near_collinear");
    for n in [10usize, 50, 200] {
        let pts: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                let t = i as f64;
                vec![t, t * 1000.0 + (i % 3) as f64 * 0.001]
            })
            .collect();
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &pts, |b, pts| {
            b.iter(|| delaunay_graph(pts).expect("delaunay"));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_point_count, bench_near_collinear);
criterion_main!(benches);
