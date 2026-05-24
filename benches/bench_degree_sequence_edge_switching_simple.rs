//! Edge-switching MCMC simple-graph degree-sequence generator benchmarks
//! (ALGO-GN-028).
//!
//! Run: `cargo bench --bench bench_degree_sequence_edge_switching_simple`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-GN-028.json`.
//!
//! Cost model: two phases.
//!   * Seed build (Havel-Hakimi INDEX / Kleitman-Wang INDEX) is
//!     `Θ(n²)` worst case (residual sort per hub), but n is typically
//!     small compared to the MCMC budget so the rewire phase dominates.
//!   * MCMC body runs `10 · |E|` trials. Each trial does two RNG
//!     draws, an O(1) `HashSet<u32>` adjacency lookup, and on success
//!     a constant number of insert/remove operations. Net per-trial
//!     cost is amortised O(1), giving the whole routine `O(Σd)`
//!     expected wall-clock — linear in the number of edges,
//!     independent of density.
//!
//! Key contrast with siblings: GN-026 `FAST_HEUR_SIMPLE` is faster but
//! gives no MCMC mixing guarantee; GN-027 `CONFIGURATION_SIMPLE` is
//! uniform but degrades exponentially with density; GN-028
//! `EDGE_SWITCHING_SIMPLE` stays linear in `|E|` for any graphical input.
//! These benches exercise three regimes that exhibit that scaling:
//!   * `size_sweep_3regular_undirected` — 3-regular sequence at
//!     `n ∈ {100, 300, 600}` to chart the linear |E| trend.
//!   * `dense_5regular_n100` — fixed n=100 dense regime (`Σd/n = 5`)
//!     that `CONFIGURATION_SIMPLE` rejects often but `EDGE_SWITCHING_SIMPLE`
//!     handles in stride.
//!   * `directed_balanced_n100_d2` — n=100 directed (out=in=2 each)
//!     to cover the directed rewire branch end-to-end.
//!
//! All benches use a hardcoded seed to keep medians comparable across runs.

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::degree_sequence_game_edge_switching_simple;

fn bench_size_sweep_3regular(c: &mut Criterion) {
    let mut group =
        c.benchmark_group("degree_sequence_edge_switching_simple/size_sweep_3regular_undirected");
    let degree: u32 = 3;
    for n in [100usize, 300, 600] {
        let degrees: Vec<u32> = vec![degree; n];
        group.throughput(Throughput::Elements(u64::try_from(n).expect("n fits u64")));
        group.bench_with_input(BenchmarkId::from_parameter(n), &degrees, |b, seq| {
            b.iter(|| {
                degree_sequence_game_edge_switching_simple(seq, None, 0x00ED_5E5A_u64).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_dense_5regular(c: &mut Criterion) {
    let mut group = c.benchmark_group("degree_sequence_edge_switching_simple/dense_5regular_n100");
    // n = 100, Σd = 500, density Σd/n = 5 — a regime where
    // CONFIGURATION_SIMPLE rejects most attempts but EDGE_SWITCHING_SIMPLE
    // remains linear in |E|.
    let degrees: Vec<u32> = vec![5u32; 100];
    assert_eq!(degrees.iter().map(|&d| u64::from(d)).sum::<u64>(), 500);
    group.throughput(Throughput::Elements(
        u64::try_from(degrees.len()).expect("fits u64"),
    ));
    group.bench_function("default_seed", |b| {
        b.iter(|| {
            degree_sequence_game_edge_switching_simple(&degrees, None, 0xC0FE_5A5A_u64).unwrap()
        });
    });
    group.finish();
}

fn bench_directed_balanced(c: &mut Criterion) {
    let mut group =
        c.benchmark_group("degree_sequence_edge_switching_simple/directed_balanced_n100_d2");
    let n: usize = 100;
    let out_seq: Vec<u32> = vec![2; n];
    let in_seq: Vec<u32> = vec![2; n];
    group.throughput(Throughput::Elements(u64::try_from(n).expect("n fits u64")));
    group.bench_function("default_seed", |b| {
        b.iter(|| {
            degree_sequence_game_edge_switching_simple(&out_seq, Some(&in_seq), 0xDEAD_5E5A_u64)
                .unwrap()
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_size_sweep_3regular,
    bench_dense_5regular,
    bench_directed_balanced,
);
criterion_main!(benches);
