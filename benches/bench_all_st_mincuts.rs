//! `all_st_mincuts` baseline benchmarks for ALGO-FL-032.
//!
//! Run: `cargo bench --bench bench_all_st_mincuts`.
//! Results land under `target/criterion/`; headline numbers recorded in
//! `.codefuse/tracking/perf/ALGO-FL-032.json`.
//!
//! `all_st_mincuts` enumerates every *minimum* (s,t) edge cut of a directed
//! graph: it first computes a max flow, then runs the Provan-Shier search over
//! the (contracted, topologically relabelled) residual graph. The cost is
//! proportional to the number of minimum cuts, which can be exponential, so the
//! workloads are deliberately BOUNDED:
//!   * **Linear path** `0 → 1 → … → n` — value 1, exactly `n` minimum cuts
//!     (one per edge), so cost scales linearly. Primary scaling driver
//!     (n = 50/100/200); stresses the residual contraction + relabel pipeline
//!     on a long chain.
//!   * **Parallel bundle** — source and sink joined by `k` vertex-disjoint
//!     length-2 paths (`0 → mid_i → t`); max-flow value `k`, and each path can
//!     be cut at EITHER of its two edges independently, giving `2^k` minimum
//!     cuts. `k` is kept tiny (k = 4/6/8 → 16/64/256 cuts) to exercise the
//!     enumeration over a wide, flat residual.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, all_st_mincuts};

/// Simple directed path `0 → 1 → … → n` on `n + 1` vertices. Source 0,
/// target `n`; min-cut value 1 with exactly `n` distinct minimum cuts.
fn linear_path(n: u32) -> Graph {
    let mut g = Graph::new(n + 1, true).expect("graph init");
    for i in 0..n {
        g.add_edge(i, i + 1).expect("edge in range");
    }
    g
}

/// Source 0 and sink `k + 1` joined by `k` vertex-disjoint length-2 paths
/// `0 → mid_i → t` (one middle per path). Max-flow value `k`; each path can be
/// cut at either edge independently, so there are `2^k` minimum cuts.
fn parallel_bundle(k: u32) -> Graph {
    let sink = k + 1;
    let mut g = Graph::new(k + 2, true).expect("graph init");
    for i in 0..k {
        let mid = i + 1;
        g.add_edge(0, mid).expect("edge in range");
        g.add_edge(mid, sink).expect("edge in range");
    }
    g
}

fn bench_linear_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_st_mincuts/linear_path");
    for n in [50u32, 100, 200] {
        let g = linear_path(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| all_st_mincuts(g, 0, n, None).expect("all_st_mincuts"));
        });
    }
    group.finish();
}

fn bench_parallel_bundle(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_st_mincuts/parallel_bundle");
    for k in [4u32, 6, 8] {
        let g = parallel_bundle(k);
        let sink = k + 1;
        group.throughput(Throughput::Elements(u64::from(k)));
        group.bench_with_input(BenchmarkId::from_parameter(k), &g, |b, g| {
            b.iter(|| all_st_mincuts(g, 0, sink, None).expect("all_st_mincuts"));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_linear_path, bench_parallel_bundle);
criterion_main!(benches);
