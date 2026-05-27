//! Burt's constraint benchmark (ALGO-PR-040).
//!
//! Run: `cargo bench --bench bench_constraint`.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, constraint, create};

fn star_graph(n: u32) -> Graph {
    let edges: Vec<(u32, u32)> = (1..n).map(|i| (0, i)).collect();
    create(&edges, n, false).unwrap()
}

fn path_graph(n: u32) -> Graph {
    let mut g = Graph::with_vertices(n);
    for i in 0..n.saturating_sub(1) {
        g.add_edge(i, i + 1).unwrap();
    }
    g
}

fn complete_graph(n: u32) -> Graph {
    let mut edges = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            edges.push((i, j));
        }
    }
    create(&edges, n, false).unwrap()
}

fn bench_constraint(c: &mut Criterion) {
    let mut group = c.benchmark_group("constraint");

    for &n in &[10u32, 50, 100] {
        let g = star_graph(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::new("star", n), &g, |bench, g| {
            bench.iter(|| constraint(g, None).unwrap());
        });
    }

    for &n in &[10u32, 50, 100] {
        let g = path_graph(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::new("path", n), &g, |bench, g| {
            bench.iter(|| constraint(g, None).unwrap());
        });
    }

    for &n in &[10u32, 30, 50] {
        let g = complete_graph(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::new("complete", n), &g, |bench, g| {
            bench.iter(|| constraint(g, None).unwrap());
        });
    }

    group.finish();
}

criterion_group!(benches, bench_constraint);
criterion_main!(benches);
