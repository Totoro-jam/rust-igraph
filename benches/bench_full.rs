//! Full graph (complete graph) constructor benchmarks (ALGO-CN-014).
//!
//! Run: `cargo bench --bench bench_full`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-014.json`.
//!
//! Coverage: every `(directed, loops)` combination at small, medium, and
//! large `n` so the throughput plot captures both axes that drive the
//! edge count (`O(n²)` in every variant). The 4-way match dispatch is
//! cheap, so the cost is dominated by `Graph::add_edges` populating the
//! adjacency, which is what the bench actually exercises.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::full_graph;

struct Shape {
    label: &'static str,
    n: u32,
    directed: bool,
    loops: bool,
    /// Total edges emitted — used for throughput reporting.
    edge_work: u64,
}

const SHAPES: &[Shape] = &[
    // Small (n = 8): bench startup overhead and the tight inner loop.
    Shape {
        label: "n8_ud_noloops",
        n: 8,
        directed: false,
        loops: false,
        edge_work: 28, // 8·7/2
    },
    Shape {
        label: "n8_d_loops",
        n: 8,
        directed: true,
        loops: true,
        edge_work: 64, // 8²
    },
    // Medium (n = 64): dense enough to expose adjacency allocation cost.
    Shape {
        label: "n64_ud_noloops",
        n: 64,
        directed: false,
        loops: false,
        edge_work: 2_016, // 64·63/2
    },
    Shape {
        label: "n64_d_noloops",
        n: 64,
        directed: true,
        loops: false,
        edge_work: 4_032, // 64·63
    },
    Shape {
        label: "n64_ud_loops",
        n: 64,
        directed: false,
        loops: true,
        edge_work: 2_080, // 64·65/2
    },
    Shape {
        label: "n64_d_loops",
        n: 64,
        directed: true,
        loops: true,
        edge_work: 4_096, // 64²
    },
    // Large (n = 512): O(n²) edge buffer + adjacency populate dominates.
    Shape {
        label: "n512_ud_noloops",
        n: 512,
        directed: false,
        loops: false,
        edge_work: 130_816, // 512·511/2
    },
    Shape {
        label: "n512_d_loops",
        n: 512,
        directed: true,
        loops: true,
        edge_work: 262_144, // 512²
    },
];

fn bench_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_graph");
    for shape in SHAPES {
        group.throughput(Throughput::Elements(shape.edge_work));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| full_graph(shape.n, shape.directed, shape.loops).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_full);
criterion_main!(benches);
