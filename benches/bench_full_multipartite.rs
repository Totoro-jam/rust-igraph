//! Full multipartite graph constructor benchmarks (ALGO-CN-026).
//!
//! Run: `cargo bench --bench bench_full_multipartite`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-026.json`.
//!
//! Coverage: small bipartite, medium balanced tripartite, large
//! balanced multipartite — across the three relevant mode combos
//! (undirected ALL, directed OUT, directed ALL). `igraph_full_multipartite`
//! emits `Σ_{i<j} n_i·n_j` undirected edges (`2x` that for directed ALL),
//! so total work is dominated by the inner emission loop and the
//! subsequent `Graph::add_edges` populating the adjacency arrays.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{MultipartiteMode, full_multipartite};

struct Shape {
    label: &'static str,
    partitions: &'static [u32],
    directed: bool,
    mode: MultipartiteMode,
    /// Total emitted edges — used for throughput reporting.
    edge_work: u64,
}

const SHAPES: &[Shape] = &[
    // Small K_{2,3} bipartite, both directed and undirected.
    Shape {
        label: "k23_undirected_all",
        partitions: &[2, 3],
        directed: false,
        mode: MultipartiteMode::All,
        edge_work: 6,
    },
    Shape {
        label: "k23_directed_out",
        partitions: &[2, 3],
        directed: true,
        mode: MultipartiteMode::Out,
        edge_work: 6,
    },
    // Medium tripartite K_{8,8,8} balanced.
    Shape {
        label: "tripartite_8_8_8_undirected_all",
        partitions: &[8, 8, 8],
        directed: false,
        mode: MultipartiteMode::All,
        edge_work: 192, // 3 * 8*8
    },
    Shape {
        label: "tripartite_8_8_8_directed_all",
        partitions: &[8, 8, 8],
        directed: true,
        mode: MultipartiteMode::All,
        edge_work: 384,
    },
    // Larger balanced four-partite K_{32,32,32,32}.
    Shape {
        label: "four_partitions_32_undirected_all",
        partitions: &[32, 32, 32, 32],
        directed: false,
        mode: MultipartiteMode::All,
        edge_work: 6_144, // C(4,2) * 32*32
    },
    Shape {
        label: "four_partitions_32_directed_out",
        partitions: &[32, 32, 32, 32],
        directed: true,
        mode: MultipartiteMode::Out,
        edge_work: 6_144,
    },
    // Heavy-dense balanced six-partite K_{16}^6.
    Shape {
        label: "six_partitions_16_undirected_all",
        partitions: &[16, 16, 16, 16, 16, 16],
        directed: false,
        mode: MultipartiteMode::All,
        edge_work: 3_840, // C(6,2) * 16*16
    },
];

fn bench_full_multipartite(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_multipartite");
    for shape in SHAPES {
        group.throughput(Throughput::Elements(shape.edge_work));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| full_multipartite(shape.partitions, shape.directed, shape.mode).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_full_multipartite);
criterion_main!(benches);
