//! Callaway-traits-game benchmarks (ALGO-GN-016).
//!
//! Run: `cargo bench --bench bench_callaway_traits`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-016.json`.
//!
//! Coverage:
//!   * `size_scaling/eps3_diag` — `types=4`, `edges_per_step=3`,
//!     diagonal `p=0.20` pref matrix, undirected, sweep
//!     `n ∈ {500, 5_000}` — measures the cost of two uniform draws +
//!     pref lookup as the population grows.
//!   * `eps_count/n1000_full` — fixed `n=1_000`, `types=2`, `p=1.0`
//!     (every candidate accepted), sweep
//!     `edges_per_step ∈ {1, 4, 16}` — isolates the per-step edge-rate
//!     scaling.
//!   * `directed/n1000_3types` — directed variant with asymmetric pref
//!     matrix on `types=3`, `edges_per_step=3` — exercises the
//!     asymmetric accept path.
//!
//! Throughput is reported in total vertices so wall-clock per element
//! is directly comparable to the other generator benches (e.g.
//! `bench_establishment`).

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::callaway_traits_game;

/// `k × k` pref matrix with `p_in` on the diagonal and `p_off`
/// elsewhere.
fn assortative_pref(k: usize, p_in: f64, p_off: f64) -> Vec<Vec<f64>> {
    (0..k)
        .map(|i| (0..k).map(|j| if i == j { p_in } else { p_off }).collect())
        .collect()
}

fn bench_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("callaway_traits_game/size_scaling/eps3_diag");
    let types = 4u32;
    let eps = 3u32;
    let pref = assortative_pref(types as usize, 0.20, 0.0);
    for n in [500u32, 5_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                callaway_traits_game(n, types, eps, None, &pref, false, 0xCA11_0001_u64).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_eps_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("callaway_traits_game/eps_count/n1000_full");
    let n = 1_000u32;
    let types = 2u32;
    let pref = vec![vec![1.0; types as usize]; types as usize];
    for eps in [1u32, 4, 16] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(eps), &eps, |b, &eps| {
            b.iter(|| {
                callaway_traits_game(n, types, eps, None, &pref, false, 0xCA11_0002_u64).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_directed(c: &mut Criterion) {
    let mut group = c.benchmark_group("callaway_traits_game/directed/n1000_3types");
    let n = 1_000u32;
    let types = 3u32;
    let eps = 3u32;
    // 3x3 directed pref with asymmetric off-diagonal entries.
    let pref: Vec<Vec<f64>> = vec![
        vec![0.10, 0.30, 0.05],
        vec![0.05, 0.10, 0.30],
        vec![0.30, 0.05, 0.10],
    ];
    group.throughput(Throughput::Elements(u64::from(n)));
    group.bench_function("directed_asymmetric", |b| {
        b.iter(|| callaway_traits_game(n, types, eps, None, &pref, true, 0xCA11_0003_u64).unwrap());
    });
    group.finish();
}

criterion_group!(benches, bench_size_scaling, bench_eps_count, bench_directed,);
criterion_main!(benches);
