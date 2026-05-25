//! Hexagonal lattice constructor benchmarks (ALGO-CN-024).
//!
//! Run: `cargo bench --bench bench_hexagonal_lattice`.
//! Results land under `target/criterion/`. A baseline snapshot lives at
//! `.codefuse/tracking/perf/ALGO-CN-024.json`.
//!
//! Coverage: the three shape branches (triangle, rectangle, hexagon)
//! across representative sizes. Counts captured live from
//! `igraph.Graph.Hexagonal_Lattice`:
//!
//! * `triangle_50`   —  2 701 vertices,  3 975 edges.
//! * `triangle_100`  — 10 401 vertices, 15 450 edges.
//! * `rect_100x100`  — 20 400 vertices, 30 399 edges.
//! * `rect_200x100`  — 40 600 vertices, 60 599 edges.
//! * `hex_30_30_30`  —  5 400 vertices,  8 010 edges.
//!
//! Work per call is `O(|V| + |E|)`. Throughput is reported in edges
//! generated.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::hexagonal_lattice;

struct Shape {
    label: &'static str,
    dims: &'static [u32],
    n_edges: u64,
}

const SHAPES: &[Shape] = &[
    Shape {
        label: "triangle_50",
        dims: &[50],
        n_edges: 3_975,
    },
    Shape {
        label: "triangle_100",
        dims: &[100],
        n_edges: 15_450,
    },
    Shape {
        label: "rect_100x100",
        dims: &[100, 100],
        n_edges: 30_399,
    },
    Shape {
        label: "rect_200x100",
        dims: &[200, 100],
        n_edges: 60_599,
    },
    Shape {
        label: "hex_30_30_30",
        dims: &[30, 30, 30],
        n_edges: 8_010,
    },
];

fn bench_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("hexagonal_lattice/undirected");
    for shape in SHAPES {
        group.throughput(Throughput::Elements(shape.n_edges));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| hexagonal_lattice(shape.dims, false, false).unwrap());
            },
        );
    }
    group.finish();
}

fn bench_directed_mutual(c: &mut Criterion) {
    let mut group = c.benchmark_group("hexagonal_lattice/directed_mutual");
    for shape in SHAPES {
        // directed_mutual doubles edge count.
        group.throughput(Throughput::Elements(shape.n_edges * 2));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| hexagonal_lattice(shape.dims, true, true).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_undirected, bench_directed_mutual);
criterion_main!(benches);
