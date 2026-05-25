//! Triangular lattice constructor benchmarks (ALGO-CN-023).
//!
//! Run: `cargo bench --bench bench_triangular_lattice`.
//! Results land under `target/criterion/`. A baseline snapshot lives at
//! `.codefuse/tracking/perf/ALGO-CN-023.json`.
//!
//! Coverage: the three shape branches (triangle, rectangle, hexagon)
//! across representative sizes:
//!
//! * `triangle_100`      —  5 050 vertices, 14 850 edges.
//! * `triangle_200`      — 20 100 vertices, 59 700 edges.
//! * `rect_100x100`      — 10 000 vertices, 29 601 edges.
//! * `rect_200x100`      — 20 000 vertices, 59 401 edges.
//! * `hex_30_30_30`      —  2 611 vertices,  7 656 edges.
//!
//! Work per call is `O(|V| + |E|)`. Throughput is reported in edges
//! generated.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::triangular_lattice;

struct Shape {
    label: &'static str,
    dims: &'static [u32],
    n_edges: u64,
}

const SHAPES: &[Shape] = &[
    Shape {
        label: "triangle_100",
        dims: &[100],
        n_edges: 14_850,
    },
    Shape {
        label: "triangle_200",
        dims: &[200],
        n_edges: 59_700,
    },
    Shape {
        label: "rect_100x100",
        dims: &[100, 100],
        n_edges: 29_601,
    },
    Shape {
        label: "rect_200x100",
        dims: &[200, 100],
        n_edges: 59_401,
    },
    Shape {
        label: "hex_30_30_30",
        dims: &[30, 30, 30],
        n_edges: 7_656,
    },
];

fn bench_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("triangular_lattice/undirected");
    for shape in SHAPES {
        group.throughput(Throughput::Elements(shape.n_edges));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| triangular_lattice(shape.dims, false, false).unwrap());
            },
        );
    }
    group.finish();
}

fn bench_directed_mutual(c: &mut Criterion) {
    let mut group = c.benchmark_group("triangular_lattice/directed_mutual");
    for shape in SHAPES {
        // directed_mutual doubles edge count
        group.throughput(Throughput::Elements(shape.n_edges * 2));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| triangular_lattice(shape.dims, true, true).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_undirected, bench_directed_mutual);
criterion_main!(benches);
