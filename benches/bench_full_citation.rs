//! Full citation graph (complete DAG / `K_n`) constructor benchmarks
//! (ALGO-CN-025).
//!
//! Run: `cargo bench --bench bench_full_citation`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-025.json`.
//!
//! Coverage: small / medium / large `n` for both `directed` settings.
//! `igraph_full_citation` emits `n·(n-1)/2` edges via the canonical
//! citation walk `for i in 1..n { for j in 0..i { push (i, j) } }`, so
//! the cost is dominated by `Graph::add_edges` populating the adjacency
//! arrays.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::full_citation;

struct Shape {
    label: &'static str,
    n: u32,
    directed: bool,
    /// Total edges emitted — used for throughput reporting.
    edge_work: u64,
}

const SHAPES: &[Shape] = &[
    // Small (n = 8): bench startup overhead + the tight inner loop.
    Shape {
        label: "n8_undirected",
        n: 8,
        directed: false,
        edge_work: 28,
    },
    Shape {
        label: "n8_directed",
        n: 8,
        directed: true,
        edge_work: 28,
    },
    // Medium (n = 64): dense enough to expose adjacency allocation cost.
    Shape {
        label: "n64_undirected",
        n: 64,
        directed: false,
        edge_work: 2_016,
    },
    Shape {
        label: "n64_directed",
        n: 64,
        directed: true,
        edge_work: 2_016,
    },
    // Large (n = 512): O(n²) edge buffer + adjacency populate dominates.
    Shape {
        label: "n512_undirected",
        n: 512,
        directed: false,
        edge_work: 130_816,
    },
    Shape {
        label: "n512_directed",
        n: 512,
        directed: true,
        edge_work: 130_816,
    },
];

fn bench_full_citation(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_citation");
    for shape in SHAPES {
        group.throughput(Throughput::Elements(shape.edge_work));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| full_citation(shape.n, shape.directed).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_full_citation);
criterion_main!(benches);
