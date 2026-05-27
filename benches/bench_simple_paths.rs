//! All simple paths benchmark (ALGO-SP-031).
//!
//! Run: `cargo bench --bench bench_simple_paths`.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, SimplePathMode, all_simple_paths, create};

fn path_graph(n: u32) -> Graph {
    let mut g = Graph::with_vertices(n);
    for i in 0..n.saturating_sub(1) {
        g.add_edge(i, i + 1).unwrap();
    }
    g
}

fn grid_graph(side: u32) -> Graph {
    let n = side * side;
    let mut edges = Vec::new();
    for r in 0..side {
        for c in 0..side {
            let v = r * side + c;
            if c + 1 < side {
                edges.push((v, v + 1));
            }
            if r + 1 < side {
                edges.push((v, v + side));
            }
        }
    }
    create(&edges, n, false).unwrap()
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

fn bench_simple_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_paths");

    for &n in &[10u32, 20, 50] {
        let g = path_graph(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::new("path", n), &g, |bench, g| {
            bench.iter(|| all_simple_paths(g, 0, None, SimplePathMode::Out, 0, -1, -1).unwrap());
        });
    }

    for &side in &[3u32, 4, 5] {
        let g = grid_graph(side);
        let n = side * side;
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::new("grid", side), &g, |bench, g| {
            bench.iter(|| all_simple_paths(g, 0, None, SimplePathMode::Out, 0, -1, 1000).unwrap());
        });
    }

    for &n in &[6u32, 8, 10] {
        let g = complete_graph(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::new("complete", n), &g, |bench, g| {
            bench.iter(|| all_simple_paths(g, 0, None, SimplePathMode::Out, 0, -1, 1000).unwrap());
        });
    }

    group.finish();
}

criterion_group!(benches, bench_simple_paths);
criterion_main!(benches);
