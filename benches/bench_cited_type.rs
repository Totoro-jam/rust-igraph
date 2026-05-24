//! Cited-type-game benchmarks (ALGO-GN-017).
//!
//! Run: `cargo bench --bench bench_cited_type`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-017.json`.
//!
//! Coverage:
//!   * `size_scaling/eps3_uniform` — `types=4` round-robin, `pref=ones`,
//!     `edges_per_step=3`, directed, sweep `n ∈ {500, 5_000}` —
//!     measures cost of the inverse-transform binary search as the
//!     `cumsum` vector grows.
//!   * `eps_count/n1000_skewed` — fixed `n=1_000`, `types=3` with
//!     heavily-skewed `pref=[10.0, 1.0, 0.05]`, sweep
//!     `edges_per_step ∈ {1, 4, 16}` — isolates per-step citation cost.
//!   * `undirected/n1000_uniform` — undirected variant with uniform
//!     pref over `types=2` (every accepted candidate is real, no
//!     fallback path) at `eps=4`.
//!
//! Throughput is reported in total vertices so wall-clock per element
//! is directly comparable to the other generator benches (e.g.
//! `bench_callaway_traits`).

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::cited_type_game;

fn bench_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("cited_type_game/size_scaling/eps3_uniform");
    let num_types = 4u32;
    let eps = 3u32;
    let pref = vec![1.0; num_types as usize];
    for n in [500u32, 5_000] {
        let types: Vec<u32> = (0..n).map(|v| v % num_types).collect();
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| cited_type_game(n, &types, &pref, eps, true, 0xC17E_0001_u64).unwrap());
        });
    }
    group.finish();
}

fn bench_eps_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("cited_type_game/eps_count/n1000_skewed");
    let n = 1_000u32;
    let num_types = 3u32;
    let pref = vec![10.0, 1.0, 0.05];
    let types: Vec<u32> = (0..n).map(|v| v % num_types).collect();
    for eps in [1u32, 4, 16] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(eps), &eps, |b, &eps| {
            b.iter(|| cited_type_game(n, &types, &pref, eps, true, 0xC17E_0002_u64).unwrap());
        });
    }
    group.finish();
}

fn bench_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("cited_type_game/undirected/n1000_uniform");
    let n = 1_000u32;
    let num_types = 2u32;
    let eps = 4u32;
    let pref = vec![1.0, 1.0];
    let types: Vec<u32> = (0..n).map(|v| v % num_types).collect();
    group.throughput(Throughput::Elements(u64::from(n)));
    group.bench_function("undirected_uniform", |b| {
        b.iter(|| cited_type_game(n, &types, &pref, eps, false, 0xC17E_0003_u64).unwrap());
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_size_scaling,
    bench_eps_count,
    bench_undirected,
);
criterion_main!(benches);
