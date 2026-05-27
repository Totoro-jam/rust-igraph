//! Simple layout benchmark (ALGO-LO-001).
//!
//! Run: `cargo bench --bench bench_layout_simple`.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{
    Graph, layout_circle, layout_grid, layout_grid_3d, layout_random, layout_random_3d,
    layout_sphere, layout_star,
};

fn bench_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout_simple");

    for &n in &[100u32, 500, 1000] {
        let g = Graph::with_vertices(n);
        group.throughput(Throughput::Elements(u64::from(n)));

        group.bench_with_input(BenchmarkId::new("random", n), &g, |bench, g| {
            bench.iter(|| layout_random(g, 42));
        });

        group.bench_with_input(BenchmarkId::new("random_3d", n), &g, |bench, g| {
            bench.iter(|| layout_random_3d(g, 42));
        });

        group.bench_with_input(BenchmarkId::new("circle", n), &g, |bench, g| {
            bench.iter(|| layout_circle(g, None));
        });

        group.bench_with_input(BenchmarkId::new("star", n), &g, |bench, g| {
            bench.iter(|| layout_star(g, 0, None).unwrap());
        });

        group.bench_with_input(BenchmarkId::new("grid", n), &g, |bench, g| {
            bench.iter(|| layout_grid(g, 0));
        });

        group.bench_with_input(BenchmarkId::new("grid_3d", n), &g, |bench, g| {
            bench.iter(|| layout_grid_3d(g, 0, 0));
        });

        group.bench_with_input(BenchmarkId::new("sphere", n), &g, |bench, g| {
            bench.iter(|| layout_sphere(g));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_layout);
criterion_main!(benches);
