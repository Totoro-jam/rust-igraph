//! Configuration-model simple-graph degree-sequence generator benchmarks
//! (ALGO-GN-027).
//!
//! Run: `cargo bench --bench bench_degree_sequence_configuration_simple`.
//! Results land under `target/criterion/`. A snapshot of the baseline lives
//! at `.codefuse/tracking/perf/ALGO-GN-027.json`.
//!
//! The configuration-simple routine has three measurable cost drivers:
//!   * stub bag construction, `O(Σd)` once per attempt.
//!   * two-swap-per-edge incremental Fisher-Yates, `O(Σd)` per attempt.
//!   * `HashSet<u32>` (undirected) or bumped vertex-mark counter (directed)
//!     adjacency check, `O(1)` per stub pair amortised, plus restart-on-
//!     collision: expected restart count grows as `exp(O((Σd/n)²))` so
//!     keep density modest.
//!
//! We bench three cases that exercise these drivers without hitting the
//! `MAX_OUTER_ATTEMPTS = 1024` ceiling:
//!   * `size_sweep_3regular_undirected` — 3-regular sequence at
//!     `n ∈ {100, 300, 600}`. Density Σd/n=3 keeps expected restarts ≈
//!     exp(2.25) ≈ 9.5 per call, so cost is dominated by FY + dedup.
//!   * `moderate_skew_n100` — fixed `n = 100` moderately skewed sequence
//!     with Σd=240 (avg 2.4). Exercises the dedup hash path with mixed
//!     degree multiplicity.
//!   * `directed_balanced_n100_d2` — n=100 directed graph with out/in =
//!     [2]*100, Σ=200. Covers the directed branch end-to-end (bumped
//!     `vertex_done` counter, no `HashSet`).
//!
//! All benches use a hardcoded seed to keep medians comparable across runs.

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::degree_sequence_game_configuration_simple;

fn bench_size_sweep_3regular(c: &mut Criterion) {
    let mut group =
        c.benchmark_group("degree_sequence_configuration_simple/size_sweep_3regular_undirected");
    let degree: u32 = 3;
    for n in [100usize, 300, 600] {
        let degrees: Vec<u32> = vec![degree; n];
        group.throughput(Throughput::Elements(u64::try_from(n).expect("n fits u64")));
        group.bench_with_input(BenchmarkId::from_parameter(n), &degrees, |b, seq| {
            b.iter(|| {
                degree_sequence_game_configuration_simple(seq, None, 0x00FA_57FA_u64).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_moderate_skew(c: &mut Criterion) {
    let mut group = c.benchmark_group("degree_sequence_configuration_simple/moderate_skew_n100");
    // n = 100, Σd = 258, moderately skewed (avg ~2.58, max 6) — kept gentle
    // so expected restart count stays bounded.
    let mut degrees: Vec<u32> = Vec::with_capacity(100);
    degrees.extend([6u32; 4]);
    degrees.extend([3u32; 60]);
    degrees.extend([2u32; 18]);
    degrees.extend([1u32; 18]);
    assert_eq!(degrees.len(), 100);
    assert_eq!(degrees.iter().map(|&d| u64::from(d)).sum::<u64>(), 258);
    group.throughput(Throughput::Elements(
        u64::try_from(degrees.len()).expect("fits u64"),
    ));
    group.bench_function("default_seed", |b| {
        b.iter(|| {
            degree_sequence_game_configuration_simple(&degrees, None, 0xC0FE_BABE_u64).unwrap()
        });
    });
    group.finish();
}

fn bench_directed_balanced(c: &mut Criterion) {
    let mut group =
        c.benchmark_group("degree_sequence_configuration_simple/directed_balanced_n100_d2");
    let n: usize = 100;
    let out_seq: Vec<u32> = vec![2; n];
    let in_seq: Vec<u32> = vec![2; n];
    group.throughput(Throughput::Elements(u64::try_from(n).expect("n fits u64")));
    group.bench_function("default_seed", |b| {
        b.iter(|| {
            degree_sequence_game_configuration_simple(&out_seq, Some(&in_seq), 0xDEAD_BEEF_u64)
                .unwrap()
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_size_sweep_3regular,
    bench_moderate_skew,
    bench_directed_balanced,
);
criterion_main!(benches);
