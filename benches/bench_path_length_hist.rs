//! Path length histogram benchmark (ALGO-SP-012).
//!
//! Run: `cargo bench --bench bench_path_length_hist`.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, path_length_hist};

fn path_graph(n: u32) -> Graph {
    let mut g = Graph::with_vertices(n);
    for i in 0..n.saturating_sub(1) {
        g.add_edge(i, i + 1).unwrap();
    }
    g
}

fn cycle_graph(n: u32) -> Graph {
    let mut g = Graph::with_vertices(n);
    for i in 0..n {
        g.add_edge(i, (i + 1) % n).unwrap();
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
    rust_igraph::create(&edges, n, false).unwrap()
}

fn bench_path_length_hist(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_length_hist");

    for &n in &[10u32, 50, 100] {
        let g = path_graph(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::new("path", n), &g, |bench, g| {
            bench.iter(|| path_length_hist(g, false).unwrap());
        });
    }

    for &n in &[10u32, 50, 100] {
        let g = cycle_graph(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::new("cycle", n), &g, |bench, g| {
            bench.iter(|| path_length_hist(g, false).unwrap());
        });
    }

    for &n in &[10u32, 30, 50] {
        let g = complete_graph(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::new("complete", n), &g, |bench, g| {
            bench.iter(|| path_length_hist(g, false).unwrap());
        });
    }

    group.finish();
}

criterion_group!(benches, bench_path_length_hist);
criterion_main!(benches);
