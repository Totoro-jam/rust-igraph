//! `canonical_permutation` baseline benchmarks for ALGO-ISO-003.
//!
//! Run: `cargo bench --bench bench_canonical`.
//! Results land under `target/criterion/`; headline numbers recorded in
//! `.codefuse/tracking/perf/ALGO-ISO-003.json`.
//!
//! The hand-rolled individualization-refinement engine explores the full I-R
//! tree, so cost is driven by graph symmetry (number of leaves) more than by
//! raw size. Two workload families bracket the behaviour:
//!   * **cycle** `C_n` — dihedral symmetry `2n`; refinement collapses slowly
//!     so the tree branches at the first cell. Stresses the symmetric case.
//!   * **path** `P_n` — almost rigid (only the end-swap automorphism);
//!     degree refinement discretizes fast, so the tree is shallow. Stresses
//!     the common low-symmetry case.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{
    Graph, automorphism_group, canonical_permutation, count_automorphisms, isomorphic,
    isomorphic_bliss, subisomorphic,
};

fn cycle(n: u32) -> Graph {
    let mut g = Graph::new(n, false).expect("graph init");
    for i in 0..n {
        g.add_edge(i, (i + 1) % n).expect("edge in range");
    }
    g
}

fn path(n: u32) -> Graph {
    let mut g = Graph::new(n, false).expect("graph init");
    for i in 0..n - 1 {
        g.add_edge(i, i + 1).expect("edge in range");
    }
    g
}

fn bench_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("canonical_permutation/cycle");
    for n in [8u32, 16, 32] {
        let g = cycle(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| canonical_permutation(g, None).expect("canonical_permutation"));
        });
    }
    group.finish();
}

fn bench_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("canonical_permutation/path");
    for n in [16u32, 32, 64] {
        let g = path(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| canonical_permutation(g, None).expect("canonical_permutation"));
        });
    }
    group.finish();
}

fn bench_count_automorphisms(c: &mut Criterion) {
    let mut group = c.benchmark_group("count_automorphisms/cycle");
    for n in [8u32, 16, 32] {
        let g = cycle(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| count_automorphisms(g, None).expect("count_automorphisms"));
        });
    }
    group.finish();
}

fn bench_automorphism_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("automorphism_group/cycle");
    for n in [8u32, 16, 32] {
        let g = cycle(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| automorphism_group(g, None).expect("automorphism_group"));
        });
    }
    group.finish();
}

fn bench_isomorphic_bliss(c: &mut Criterion) {
    // Self-isomorphism of a cycle: both sides are canonicalized, so cost is
    // two I-R searches plus a certificate compare — the symmetric worst case.
    let mut group = c.benchmark_group("isomorphic_bliss/cycle");
    for n in [8u32, 16, 32] {
        let g = cycle(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| isomorphic_bliss(g, g, None, None).expect("isomorphic_bliss"));
        });
    }
    group.finish();
}

fn bench_isomorphic(c: &mut Criterion) {
    // Generic dispatcher on simple graphs: cycle vs itself routes through the
    // BLISS canonical-form test (after the vcount/ecount guard).
    let mut group = c.benchmark_group("isomorphic/cycle");
    for n in [8u32, 16, 32] {
        let g = cycle(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| isomorphic(g, g).expect("isomorphic"));
        });
    }
    group.finish();
}

fn bench_subisomorphic(c: &mut Criterion) {
    // Generic subgraph dispatcher: embed a triangle pattern into a cycle
    // target via the VF2 backend.
    let mut triangle = Graph::new(3, false).expect("graph init");
    for (u, v) in [(0u32, 1u32), (1, 2), (2, 0)] {
        triangle.add_edge(u, v).expect("triangle edge");
    }
    let mut group = c.benchmark_group("subisomorphic/cycle_target");
    for n in [8u32, 16, 32] {
        let g = cycle(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| subisomorphic(g, &triangle).expect("subisomorphic"));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_cycle,
    bench_path,
    bench_count_automorphisms,
    bench_automorphism_group,
    bench_isomorphic_bliss,
    bench_isomorphic,
    bench_subisomorphic
);
criterion_main!(benches);
