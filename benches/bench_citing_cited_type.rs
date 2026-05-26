//! Citing-cited-type-game benchmarks (ALGO-GN-029).
//!
//! Run: `cargo bench --bench bench_citing_cited_type`. Results land
//! under `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-029.json`.
//!
//! Coverage:
//!   * `size_scaling/eps3_uniform` — `types=4` round-robin, `pref=ones`
//!     (4x4), `edges_per_step=3`, directed, sweep `n ∈ {500, 5_000}` —
//!     measures cost of `types` parallel Fenwick BITs as the BIT size
//!     grows linearly with `n`.
//!   * `eps_count/n1000_skewed` — fixed `n=1_000`, `types=3` with a
//!     skewed 3x3 pref (heavily diagonal: each citing type prefers its
//!     own cited type 10:1:0.05), sweep `edges_per_step ∈ {1, 4, 16}`
//!     — isolates per-step citation cost dominated by the `O(log n)`
//!     `search_bounded` calls.
//!   * `types_scaling/n1000_eps2` — fixed `n=1_000`, `eps=2`, uniform
//!     identity pref, sweep `types ∈ {2, 4, 8, 16}` — measures the
//!     per-type BIT overhead (each step touches one BIT for sampling
//!     and all BITs for the per-step append).
//!   * `undirected/n1000_uniform` — undirected variant with uniform
//!     pref over `types=2` (every accepted candidate is real, no
//!     fallback path) at `eps=4`.
//!
//! Throughput is reported in total vertices so wall-clock per element
//! is directly comparable to the other generator benches (e.g.
//! `bench_cited_type`).

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::citing_cited_type_game;

fn pref_uniform(t: usize) -> Vec<Vec<f64>> {
    vec![vec![1.0; t]; t]
}

fn bench_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("citing_cited_type_game/size_scaling/eps3_uniform");
    let num_types = 4u32;
    let eps = 3u32;
    let pref_rows = pref_uniform(num_types as usize);
    let pref_views: Vec<&[f64]> = pref_rows.iter().map(Vec::as_slice).collect();
    for n in [500u32, 5_000] {
        let types: Vec<u32> = (0..n).map(|v| v % num_types).collect();
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                citing_cited_type_game(n, &types, &pref_views, eps, true, 0xC17C_0001_u64).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_eps_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("citing_cited_type_game/eps_count/n1000_skewed");
    let n = 1_000u32;
    let num_types = 3u32;
    let pref_rows: Vec<Vec<f64>> = vec![
        vec![10.0, 1.0, 0.05],
        vec![1.0, 10.0, 0.05],
        vec![0.05, 1.0, 10.0],
    ];
    let pref_views: Vec<&[f64]> = pref_rows.iter().map(Vec::as_slice).collect();
    let types: Vec<u32> = (0..n).map(|v| v % num_types).collect();
    for eps in [1u32, 4, 16] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(eps), &eps, |b, &eps| {
            b.iter(|| {
                citing_cited_type_game(n, &types, &pref_views, eps, true, 0xC17C_0002_u64).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_types_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("citing_cited_type_game/types_scaling/n1000_eps2");
    let n = 1_000u32;
    let eps = 2u32;
    for t in [2u32, 4, 8, 16] {
        let pref_rows = pref_uniform(t as usize);
        let pref_views: Vec<&[f64]> = pref_rows.iter().map(Vec::as_slice).collect();
        let types: Vec<u32> = (0..n).map(|v| v % t).collect();
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(t), &t, |b, _| {
            b.iter(|| {
                citing_cited_type_game(n, &types, &pref_views, eps, true, 0xC17C_0003_u64).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("citing_cited_type_game/undirected/n1000_uniform");
    let n = 1_000u32;
    let num_types = 2u32;
    let eps = 4u32;
    let pref_rows = pref_uniform(num_types as usize);
    let pref_views: Vec<&[f64]> = pref_rows.iter().map(Vec::as_slice).collect();
    let types: Vec<u32> = (0..n).map(|v| v % num_types).collect();
    group.throughput(Throughput::Elements(u64::from(n)));
    group.bench_function("undirected_uniform", |b| {
        b.iter(|| {
            citing_cited_type_game(n, &types, &pref_views, eps, false, 0xC17C_0004_u64).unwrap()
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_size_scaling,
    bench_eps_count,
    bench_types_scaling,
    bench_undirected,
);
criterion_main!(benches);
