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
use rust_igraph::{Graph, canonical_permutation};

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

criterion_group!(benches, bench_cycle, bench_path);
criterion_main!(benches);
