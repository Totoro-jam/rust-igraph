//! VF2 subgraph-isomorphism baseline benchmarks (ALGO-ISO-002).
//!
//! Run: `cargo bench --bench bench_subiso`.
//! Results land under `target/criterion/`. A snapshot is committed to
//! `.codefuse/tracking/perf/ALGO-ISO-002.json`.
//!
//! Workloads keep the backtracking bounded: a single-edge pattern embeds into
//! ring(n) in exactly 2n ways (counting path), and finding the first embedding
//! of a short path into ring(n) terminates quickly (first-match path).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, count_subisomorphisms_vf2, ring_graph, subisomorphic_vf2};

fn ring(n: u32) -> Graph {
    ring_graph(n, false, false, true).expect("ring")
}

/// Single undirected edge — the smallest non-trivial pattern.
fn edge_pattern() -> Graph {
    let mut g = Graph::new(2, false).expect("graph");
    g.add_edge(0, 1).expect("edge");
    g
}

/// Path on three vertices (two edges): 0-1-2.
fn path3_pattern() -> Graph {
    let mut g = Graph::new(3, false).expect("graph");
    g.add_edge(0, 1).expect("edge");
    g.add_edge(1, 2).expect("edge");
    g
}

/// Counting embeddings of a single edge into ring(n) exercises the full
/// backtracking enumeration; the answer is 2n, so work scales predictably.
fn bench_subiso_count_edge_into_ring(c: &mut Criterion) {
    let mut group = c.benchmark_group("subiso/count_edge_into_ring");
    let pattern = edge_pattern();
    for n in [50u32, 100, 200] {
        let target = ring(n);
        group.throughput(Throughput::Elements(u64::from(target.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &target, |b, target| {
            b.iter(|| {
                count_subisomorphisms_vf2(target, &pattern, None, None, None, None).expect("count")
            });
        });
    }
    group.finish();
}

/// Finding the first embedding of a short path into ring(n) terminates at the
/// first complete mapping — the common "does this motif occur" query.
fn bench_subiso_first_path_into_ring(c: &mut Criterion) {
    let mut group = c.benchmark_group("subiso/first_path_into_ring");
    let pattern = path3_pattern();
    for n in [100u32, 1_000, 5_000] {
        let target = ring(n);
        group.throughput(Throughput::Elements(u64::from(target.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &target, |b, target| {
            b.iter(|| subisomorphic_vf2(target, &pattern, None, None, None, None).expect("subiso"));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_subiso_count_edge_into_ring,
    bench_subiso_first_path_into_ring
);
criterion_main!(benches);
