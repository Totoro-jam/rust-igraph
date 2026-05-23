//! Establishment-game benchmarks (ALGO-GN-015).
//!
//! Run: `cargo bench --bench bench_establishment`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-015.json`.
//!
//! Coverage:
//!   * `size_scaling/k4_diag` — `types=4`, `k=4`, diagonal `p=0.20`
//!     pref matrix, undirected, sweep `n ∈ {500, 5_000}` —
//!     measures the cost of Floyd-distinct sampling + binary-search
//!     type lookup as new vertices accumulate.
//!   * `k_count/n1000_full` — fixed `n=1_000`, `types=2`, `p=1.0`
//!     so every candidate edge is accepted, sweep `k ∈ {1, 4, 16}` —
//!     isolates the per-vertex edge-rate scaling.
//!   * `directed/n1000_3types` — directed variant with asymmetric
//!     pref matrix on `types=3`, `k=4`, exercises the asymmetric
//!     accept path.
//!
//! Throughput is reported in total vertices so wall-clock per element
//! is directly comparable to the other generator benches.

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::establishment_game;

/// `k × k` pref matrix with `p_in` on the diagonal and `p_off`
/// elsewhere.
fn assortative_pref(k: usize, p_in: f64, p_off: f64) -> Vec<Vec<f64>> {
    (0..k)
        .map(|i| (0..k).map(|j| if i == j { p_in } else { p_off }).collect())
        .collect()
}

fn bench_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("establishment_game/size_scaling/k4_diag");
    let types = 4u32;
    let k = 4u32;
    let pref = assortative_pref(types as usize, 0.20, 0.0);
    for n in [500u32, 5_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                establishment_game(n, types, k, None, &pref, false, 0xE5AB_0001_u64).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_k_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("establishment_game/k_count/n1000_full");
    let n = 1_000u32;
    let types = 2u32;
    let pref = vec![vec![1.0; types as usize]; types as usize];
    for k in [1u32, 4, 16] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(k), &k, |b, &k| {
            b.iter(|| {
                establishment_game(n, types, k, None, &pref, false, 0xE5AB_0002_u64).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_directed(c: &mut Criterion) {
    let mut group = c.benchmark_group("establishment_game/directed/n1000_3types");
    let n = 1_000u32;
    let types = 3u32;
    let k = 4u32;
    // 3x3 directed pref with asymmetric off-diagonal entries.
    let pref: Vec<Vec<f64>> = vec![
        vec![0.10, 0.30, 0.05],
        vec![0.05, 0.10, 0.30],
        vec![0.30, 0.05, 0.10],
    ];
    group.throughput(Throughput::Elements(u64::from(n)));
    group.bench_function("directed_asymmetric", |b| {
        b.iter(|| establishment_game(n, types, k, None, &pref, true, 0xE5AB_0003_u64).unwrap());
    });
    group.finish();
}

criterion_group!(benches, bench_size_scaling, bench_k_count, bench_directed,);
criterion_main!(benches);
