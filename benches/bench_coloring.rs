//! Vertex-coloring greedy benchmark (CN + DSATUR).
//!
//! Run: `cargo bench --bench bench_coloring`.
//! Results land under `target/criterion/`. Numbers are committed to
//! `.codefuse/tracking/perf/ALGO-CL-001.json`.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{GreedyColoringHeuristic, erdos_renyi_gnp, vertex_coloring_greedy};

fn sparse_er(n: u32) -> rust_igraph::Graph {
    let p = 4.0 / f64::from(n.saturating_sub(1).max(1));
    erdos_renyi_gnp(n, p, false, false, 0x0C01_0001).expect("ER sparse")
}

fn dense_er(n: u32) -> rust_igraph::Graph {
    let p = 0.3;
    erdos_renyi_gnp(n, p, false, false, 0x0C01_0002).expect("ER dense")
}

fn bench_cn_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("coloring_cn/sparse");
    for n in [100u32, 1_000, 5_000] {
        let g = sparse_er(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| {
                vertex_coloring_greedy(g, GreedyColoringHeuristic::ColoredNeighbors).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_dsatur_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("coloring_dsatur/sparse");
    for n in [100u32, 1_000, 5_000] {
        let g = sparse_er(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| vertex_coloring_greedy(g, GreedyColoringHeuristic::DSatur).unwrap());
        });
    }
    group.finish();
}

fn bench_dsatur_dense(c: &mut Criterion) {
    let mut group = c.benchmark_group("coloring_dsatur/dense");
    for n in [100u32, 500, 1_000] {
        let g = dense_er(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| vertex_coloring_greedy(g, GreedyColoringHeuristic::DSatur).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_cn_sparse,
    bench_dsatur_sparse,
    bench_dsatur_dense
);
criterion_main!(benches);
