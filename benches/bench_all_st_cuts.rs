//! `all_st_cuts` baseline benchmarks for ALGO-FL-031.
//!
//! Run: `cargo bench --bench bench_all_st_cuts`.
//! Results land under `target/criterion/`; headline numbers recorded in
//! `.codefuse/tracking/perf/ALGO-FL-031.json`.
//!
//! `all_st_cuts` enumerates EVERY (s,t) edge cut of a directed graph via the
//! Provan-Shier paradigm, so its cost is proportional to the number of cuts,
//! which can be exponential. The workloads here are deliberately BOUNDED:
//!   * **Linear path** `0 → 1 → … → n` — exactly `n` cuts (one per edge),
//!     so cost scales linearly. The primary scaling workload (n = 50/100/200).
//!   * **Series of diamonds (ladder)** — `k` diamond blocks chained in
//!     series, each `a→b→d`, `a→c→d`; the cut count is `4^k`, so `k` is kept
//!     tiny (k = 1/2/3 → 4/16/64 cuts). Exercises the pivot/dominator
//!     machinery more heavily per cut.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, all_st_cuts};

/// Simple directed path `0 → 1 → … → n` on `n + 1` vertices. Source 0,
/// target `n`; exactly `n` distinct (s,t) cuts (one per edge).
fn linear_path(n: u32) -> Graph {
    let mut g = Graph::new(n + 1, true).expect("graph init");
    for i in 0..n {
        g.add_edge(i, i + 1).expect("edge in range");
    }
    g
}

/// Series of `k` diamond blocks chained head-to-tail. Block `i` spans the
/// three new vertices after its shared entry node: entry `a`, branches
/// `b`/`c`, and exit `d` (which becomes the next block's entry). Vertices
/// total `3 * k + 1`; source 0, target `3 * k`. The cut count is `4^k`.
fn diamond_series(k: u32) -> Graph {
    let n = 3 * k + 1;
    let mut g = Graph::new(n, true).expect("graph init");
    for i in 0..k {
        let entry = 3 * i;
        let upper = 3 * i + 1;
        let lower = 3 * i + 2;
        let exit = 3 * i + 3;
        g.add_edge(entry, upper).expect("edge in range");
        g.add_edge(entry, lower).expect("edge in range");
        g.add_edge(upper, exit).expect("edge in range");
        g.add_edge(lower, exit).expect("edge in range");
    }
    g
}

fn bench_linear_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_st_cuts/linear_path");
    for n in [50u32, 100, 200] {
        let g = linear_path(n);
        let target = u64::from(n);
        group.throughput(Throughput::Elements(target));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| all_st_cuts(g, 0, n).expect("all_st_cuts"));
        });
    }
    group.finish();
}

fn bench_diamond_series(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_st_cuts/diamond_series");
    for k in [1u32, 2, 3] {
        let g = diamond_series(k);
        let target = 3 * k;
        group.throughput(Throughput::Elements(u64::from(target)));
        group.bench_with_input(BenchmarkId::from_parameter(k), &g, |b, g| {
            b.iter(|| all_st_cuts(g, 0, target).expect("all_st_cuts"));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_linear_path, bench_diamond_series);
criterion_main!(benches);
