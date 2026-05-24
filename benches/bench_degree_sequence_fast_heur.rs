//! Fast-heuristic-simple degree-sequence generator benchmarks (ALGO-GN-026).
//!
//! Run: `cargo bench --bench bench_degree_sequence_fast_heur`. Results land
//! under `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-026.json`.
//!
//! The fast-heuristic routine has two measurable cost drivers we want a
//! signal on:
//!   * stub bag construction + Fisher-Yates shuffle, `O(Σd)` per attempt.
//!   * pair walk + per-vertex sorted-Vec adjacency dedup, `O(Σd · log Δ)`
//!     amortised per attempt (where `Δ` is the maximum degree). On collision
//!     we bump residuals and retry; on infeasibility we restart from scratch.
//!
//! We bench three cases that exercise these jointly without becoming flaky:
//!   * `size_sweep_3regular_undirected` — 3-regular sequence at
//!     `n ∈ {200, 600, 1_200}`. Always graphical, so each run is one
//!     successful attempt; cost tracks shuffle + dedup.
//!   * `powerlaw_n200_skewed` — fixed `n = 200` skewed sequence with a
//!     small high-degree tail. Exercises the collision-and-retry regime
//!     where many pairs land on the same high-degree stub.
//!   * `directed_balanced_n200_d4` — n=200 directed graph with out/in =
//!     [4]*200, Σ=800. Covers the directed branch end-to-end (separate
//!     out/in stub bags, no self-loop / no multi-arc dedup).
//!
//! All benches use a hardcoded seed to keep medians comparable across runs.

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::degree_sequence_game_fast_heur_simple;

fn bench_size_sweep_3regular(c: &mut Criterion) {
    let mut group = c.benchmark_group("degree_sequence_fast_heur/size_sweep_3regular_undirected");
    let degree: u32 = 3;
    for n in [200usize, 600, 1_200] {
        let degrees: Vec<u32> = vec![degree; n];
        group.throughput(Throughput::Elements(u64::try_from(n).expect("n fits u64")));
        group.bench_with_input(BenchmarkId::from_parameter(n), &degrees, |b, seq| {
            b.iter(|| degree_sequence_game_fast_heur_simple(seq, None, 0x00FA_57FA_u64).unwrap());
        });
    }
    group.finish();
}

fn bench_skewed_powerlaw(c: &mut Criterion) {
    let mut group = c.benchmark_group("degree_sequence_fast_heur/powerlaw_n200_skewed");
    // n = 200, Σd = 600, top-heavy sequence.
    let mut degrees: Vec<u32> = Vec::with_capacity(200);
    degrees.extend([10u32; 4]);
    degrees.extend([3u32; 168]);
    degrees.extend([2u32; 28]);
    assert_eq!(degrees.len(), 200);
    assert_eq!(degrees.iter().map(|&d| u64::from(d)).sum::<u64>(), 600);
    group.throughput(Throughput::Elements(
        u64::try_from(degrees.len()).expect("fits u64"),
    ));
    group.bench_function("default_seed", |b| {
        b.iter(|| degree_sequence_game_fast_heur_simple(&degrees, None, 0xC0FE_BABE_u64).unwrap());
    });
    group.finish();
}

fn bench_directed_balanced(c: &mut Criterion) {
    let mut group = c.benchmark_group("degree_sequence_fast_heur/directed_balanced_n200_d4");
    let n: usize = 200;
    let out_seq: Vec<u32> = vec![4; n];
    let in_seq: Vec<u32> = vec![4; n];
    group.throughput(Throughput::Elements(u64::try_from(n).expect("n fits u64")));
    group.bench_function("default_seed", |b| {
        b.iter(|| {
            degree_sequence_game_fast_heur_simple(&out_seq, Some(&in_seq), 0xDEAD_BEEF_u64).unwrap()
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_size_sweep_3regular,
    bench_skewed_powerlaw,
    bench_directed_balanced,
);
criterion_main!(benches);
