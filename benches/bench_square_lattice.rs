//! Square lattice constructor benchmarks (ALGO-CN-009).
//!
//! Run: `cargo bench --bench bench_square_lattice`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-009.json`.
//!
//! Coverage: undirected non-periodic, undirected torus, and directed
//! mutual variants across four representative shapes:
//!
//! * `100x100` (2-D)    —   10 000 vertices, 19 800 edges (non-periodic).
//! * `200x200` (2-D)    —   40 000 vertices, 79 600 edges (non-periodic).
//! * `100x100 torus`    —   10 000 vertices, 20 000 edges (4-regular).
//! * `30x30x30` (3-D)   —   27 000 vertices, 78 300 edges (non-periodic).
//!
//! Work per call is `O(|V| · d) = O(|E|)`. Throughput is reported in
//! edges generated.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::square_lattice;

struct Shape {
    label: &'static str,
    dim: &'static [u32],
    periodic: Option<&'static [bool]>,
    n_edges: u64,
}

const NON_PERIODIC: &[Shape] = &[
    Shape {
        label: "2d_100x100",
        dim: &[100, 100],
        periodic: None,
        n_edges: 19_800,
    },
    Shape {
        label: "2d_200x200",
        dim: &[200, 200],
        periodic: None,
        n_edges: 79_600,
    },
    Shape {
        label: "3d_30x30x30",
        dim: &[30, 30, 30],
        periodic: None,
        n_edges: 78_300,
    },
];

const TORUS_2D: Shape = Shape {
    label: "2d_100x100_torus",
    dim: &[100, 100],
    periodic: Some(&[true, true]),
    n_edges: 20_000,
};

fn bench_square_lattice_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("square_lattice/undirected");
    for shape in NON_PERIODIC {
        group.throughput(Throughput::Elements(shape.n_edges));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| square_lattice(shape.dim, 1, false, false, shape.periodic).unwrap());
            },
        );
    }
    // Also include the 2-D torus as an undirected variant.
    group.throughput(Throughput::Elements(TORUS_2D.n_edges));
    group.bench_with_input(
        BenchmarkId::from_parameter(TORUS_2D.label),
        &&TORUS_2D,
        |b, shape| {
            b.iter(|| square_lattice(shape.dim, 1, false, false, shape.periodic).unwrap());
        },
    );
    group.finish();
}

fn bench_square_lattice_directed_mutual(c: &mut Criterion) {
    let mut group = c.benchmark_group("square_lattice/directed_mutual");
    for shape in NON_PERIODIC {
        // directed_mutual doubles edge count
        group.throughput(Throughput::Elements(shape.n_edges * 2));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| square_lattice(shape.dim, 1, true, true, shape.periodic).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_square_lattice_undirected,
    bench_square_lattice_directed_mutual
);
criterion_main!(benches);
