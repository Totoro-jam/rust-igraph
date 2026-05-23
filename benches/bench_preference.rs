//! Preference-game benchmarks (ALGO-GN-014).
//!
//! Run: `cargo bench --bench bench_preference`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-014.json`.
//!
//! Coverage:
//!   * `size_scaling/k4_diag` — `k=4` types with diagonal `p=0.05`,
//!     uniform `type_dist`, undirected, no loops, sweep
//!     `n ∈ {500, 5_000}` — exercises the per-pair Batagelj–Brandes
//!     skip on each `(i, i)` block.
//!   * `type_count/n1000_diag` — fixed `n=1000` with diagonal `p=0.05`,
//!     sweep `k ∈ {2, 4, 8}` — measures how block-pair count drives
//!     fixed setup cost.
//!   * `fixed_sizes_vs_random/n1000_k4` — toggle `fixed_sizes` between
//!     deterministic equal-split and stochastic categorical assignment;
//!     same pref matrix.
//!   * `asymmetric/n1000_2x3` — directed asymmetric variant with mild
//!     off-diagonal density; loops on/off pair.
//!
//! Throughput is reported in total vertices so wall-clock per element
//! is directly comparable to the other generator benches.

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{asymmetric_preference_game, preference_game};

/// `k × k` pref matrix with `p_in` on the diagonal and `p_off`
/// elsewhere.
fn assortative_pref(k: usize, p_in: f64, p_off: f64) -> Vec<Vec<f64>> {
    (0..k)
        .map(|i| (0..k).map(|j| if i == j { p_in } else { p_off }).collect())
        .collect()
}

fn bench_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("preference_game/size_scaling/k4_diag");
    let k = 4usize;
    let pref = assortative_pref(k, 0.05, 0.0);
    for n in [500u32, 5_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                preference_game(
                    n,
                    k as u32,
                    None,
                    false,
                    &pref,
                    false,
                    false,
                    0xFE14_0001_u64,
                )
                .unwrap()
            });
        });
    }
    group.finish();
}

fn bench_type_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("preference_game/type_count/n1000_diag");
    let n = 1_000u32;
    for k in [2usize, 4, 8] {
        let pref = assortative_pref(k, 0.05, 0.0);
        let label = format!("k{k}");
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(label), &k, |b, _| {
            b.iter(|| {
                preference_game(
                    n,
                    k as u32,
                    None,
                    false,
                    &pref,
                    false,
                    false,
                    0xFE14_0002_u64,
                )
                .unwrap()
            });
        });
    }
    group.finish();
}

fn bench_fixed_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("preference_game/fixed_sizes_vs_random/n1000_k4");
    let n = 1_000u32;
    let k = 4usize;
    let pref = assortative_pref(k, 0.05, 0.005);
    group.throughput(Throughput::Elements(u64::from(n)));
    group.bench_function("random_dist", |b| {
        b.iter(|| {
            preference_game(
                n,
                k as u32,
                None,
                false,
                &pref,
                false,
                false,
                0xFE14_0003_u64,
            )
            .unwrap()
        });
    });
    group.bench_function("fixed_sizes_equal", |b| {
        b.iter(|| {
            preference_game(
                n,
                k as u32,
                None,
                true,
                &pref,
                false,
                false,
                0xFE14_0004_u64,
            )
            .unwrap()
        });
    });
    group.finish();
}

fn bench_asymmetric(c: &mut Criterion) {
    let mut group = c.benchmark_group("asymmetric_preference_game/n1000_2x3");
    let n = 1_000u32;
    let no_out = 2u32;
    let no_in = 3u32;
    // 2x3 pref with a clear hotspot on (out=0, in=1).
    let pref: Vec<Vec<f64>> = vec![vec![0.01, 0.05, 0.005], vec![0.005, 0.005, 0.05]];
    group.throughput(Throughput::Elements(u64::from(n)));
    group.bench_function("no_loops", |b| {
        b.iter(|| {
            asymmetric_preference_game(n, no_out, no_in, None, &pref, false, 0xFE14_0005_u64)
                .unwrap()
        });
    });
    group.bench_function("with_loops", |b| {
        b.iter(|| {
            asymmetric_preference_game(n, no_out, no_in, None, &pref, true, 0xFE14_0006_u64)
                .unwrap()
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_size_scaling,
    bench_type_count,
    bench_fixed_sizes,
    bench_asymmetric,
);
criterion_main!(benches);
