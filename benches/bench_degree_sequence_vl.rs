//! Viger–Latapy degree-sequence generator benchmarks (ALGO-GN-025).
//!
//! Run: `cargo bench --bench bench_degree_sequence_vl`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-025.json`.
//!
//! The VL routine has three measurable cost drivers we want a signal on:
//!   * realisation: Hakimi-style greedy connected build, `O(Σd · log Σd)`.
//!   * MCMC: edge-switch chain runs `5·2·|E|` proposals on average, each
//!     `O(1)` if the swap is accepted (windowed snapshot/restore otherwise).
//!   * connectivity guard: every `max(16, 2|E|)` proposals we re-check weak
//!     connectivity in `O(|V| + |E|)` and roll back on disconnect.
//!
//! We bench two cases that exercise these jointly without becoming flaky:
//!   * `size_sweep_3regular_undirected` — 3-regular sequence at
//!     `n ∈ {200, 600, 1_200}`. Even-Σd 3-regular always satisfies Hakimi,
//!     so the realisation step is the same shape for every `n` and the
//!     reported time tracks MCMC + connectivity-guard cost. Throughput is
//!     reported per vertex so 3·n scaling is visible at a glance.
//!   * `powerlaw_n200_skewed` — fixed `n = 200` skewed sequence with a
//!     small high-degree tail. Exercises the rejection-heavy regime of the
//!     edge-switch chain where many proposals collide on the few high-
//!     degree stubs, isolating the swap-rejection branch.
//!
//! All benches use a hardcoded seed to keep medians comparable across runs.

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::degree_sequence_game_vl;

fn bench_size_sweep_3regular(c: &mut Criterion) {
    let mut group = c.benchmark_group("degree_sequence_vl/size_sweep_3regular_undirected");
    let degree: u32 = 3;
    for n in [200usize, 600, 1_200] {
        let degrees: Vec<u32> = vec![degree; n];
        group.throughput(Throughput::Elements(u64::try_from(n).expect("n fits u64")));
        group.bench_with_input(BenchmarkId::from_parameter(n), &degrees, |b, seq| {
            b.iter(|| degree_sequence_game_vl(seq, 0x00_5E_E9_DEu64).unwrap());
        });
    }
    group.finish();
}

fn bench_skewed_powerlaw(c: &mut Criterion) {
    let mut group = c.benchmark_group("degree_sequence_vl/powerlaw_n200_skewed");
    // n = 200, sum = 600 (even, connected-graphical).
    // top-heavy sequence: 4 vertices of degree 10 + 196 of degree ~2.86.
    // Build a concrete sequence that is graphical and connected-graphical.
    let mut degrees: Vec<u32> = Vec::with_capacity(200);
    degrees.extend([10u32; 4]);
    // remaining 196 vertices, total remaining = 560 ≈ 2.857 each ⇒ pattern 3,3,2,3,3,2,…
    // exact split: 168·3 + 28·2 = 504 + 56 = 560. ✓
    degrees.extend([3u32; 168]);
    degrees.extend([2u32; 28]);
    assert_eq!(degrees.len(), 200);
    assert_eq!(degrees.iter().map(|&d| u64::from(d)).sum::<u64>(), 600);
    group.throughput(Throughput::Elements(
        u64::try_from(degrees.len()).expect("fits u64"),
    ));
    group.bench_function("default_seed", |b| {
        b.iter(|| degree_sequence_game_vl(&degrees, 0xC0FE_BABE_u64).unwrap());
    });
    group.finish();
}

criterion_group!(benches, bench_size_sweep_3regular, bench_skewed_powerlaw,);
criterion_main!(benches);
