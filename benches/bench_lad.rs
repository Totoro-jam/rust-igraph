//! LAD subgraph-isomorphism baseline benchmarks (ALGO-ISO-020).
//!
//! Run: `cargo bench --bench bench_lad`.
//! Results land under `target/criterion/`. A snapshot is committed to
//! `.codefuse/tracking/perf/ALGO-ISO-020.json`.
//!
//! Workloads exercise the LAD CSP engine's two regimes: enumerating every
//! embedding of a single edge into ring(n) (exactly 2n maps, so the
//! `AllDifferent` propagation + forward-checking run to completion), and the
//! first-match decision for a short path into ring(n) (LAD stops at the first
//! complete assignment).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, get_subisomorphisms_lad, ring_graph, subisomorphic_lad};

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

/// Enumerating every embedding of a single edge into ring(n) drives the full
/// LAD search (2n maps); work scales predictably with target size.
fn bench_lad_enumerate_edge_into_ring(c: &mut Criterion) {
    let mut group = c.benchmark_group("lad/enumerate_edge_into_ring");
    let pattern = edge_pattern();
    for n in [50u32, 100, 200] {
        let target = ring(n);
        group.throughput(Throughput::Elements(u64::from(target.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &target, |b, target| {
            b.iter(|| get_subisomorphisms_lad(&pattern, target, None, false).expect("enumerate"));
        });
    }
    group.finish();
}

/// First-match decision: does a short path embed into ring(n)? LAD returns at
/// the first complete assignment — the common "does this motif occur" query.
fn bench_lad_first_path_into_ring(c: &mut Criterion) {
    let mut group = c.benchmark_group("lad/first_path_into_ring");
    let pattern = path3_pattern();
    for n in [100u32, 1_000, 5_000] {
        let target = ring(n);
        group.throughput(Throughput::Elements(u64::from(target.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &target, |b, target| {
            b.iter(|| subisomorphic_lad(&pattern, target, None, false).expect("lad"));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_lad_enumerate_edge_into_ring,
    bench_lad_first_path_into_ring
);
criterion_main!(benches);
