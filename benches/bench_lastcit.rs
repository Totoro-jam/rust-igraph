//! Last-citation-game benchmarks (ALGO-GN-018).
//!
//! Run: `cargo bench --bench bench_lastcit`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-018.json`.
//!
//! Coverage:
//!   * `size_scaling/eps3_decay` — `agebins=4`,
//!     `preference=[8,4,2,1,0.5]`, `edges_per_node=3`, directed, sweep
//!     `n ∈ {500, 5_000}` — measures cost of the psumtree update +
//!     binary-lifting search as the tree grows.
//!   * `eps_count/n1000_decay` — fixed `n=1_000`, `agebins=3`,
//!     `preference=[16,4,1,0.25]`, sweep
//!     `edges_per_node ∈ {1, 4, 16}` — isolates per-step citation cost.
//!   * `agebins_count/n1000_eps2` — `n=1_000`, `eps=2`, sweep
//!     `agebins ∈ {1, 4, 16}` — exercises the age-sweep work per step
//!     (more bins ⇒ more bin-boundary crossings per i).
//!   * `undirected/n1000_uniform` — undirected variant with uniform
//!     preference at `eps=4`, `agebins=2`.
//!
//! Throughput is reported in total vertices so wall-clock per element
//! is directly comparable to the other generator benches.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::lastcit_game;

fn bench_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("lastcit_game/size_scaling/eps3_decay");
    let agebins = 4u32;
    let eps = 3u32;
    let pref = vec![8.0, 4.0, 2.0, 1.0, 0.5];
    for n in [500u32, 5_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| lastcit_game(n, eps, agebins, &pref, true, 0x1A57_C170_u64).unwrap());
        });
    }
    group.finish();
}

fn bench_eps_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("lastcit_game/eps_count/n1000_decay");
    let n = 1_000u32;
    let agebins = 3u32;
    let pref = vec![16.0, 4.0, 1.0, 0.25];
    for eps in [1u32, 4, 16] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(eps), &eps, |b, &eps| {
            b.iter(|| lastcit_game(n, eps, agebins, &pref, true, 0x1A57_C171_u64).unwrap());
        });
    }
    group.finish();
}

fn bench_agebins_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("lastcit_game/agebins_count/n1000_eps2");
    let n = 1_000u32;
    let eps = 2u32;
    for agebins in [1u32, 4, 16] {
        let pref: Vec<f64> = (0..=agebins as usize)
            .map(|i| 1.0 / (i as f64 + 1.0))
            .collect();
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(
            BenchmarkId::from_parameter(agebins),
            &agebins,
            |b, &agebins| {
                b.iter(|| lastcit_game(n, eps, agebins, &pref, true, 0x1A57_C172_u64).unwrap());
            },
        );
    }
    group.finish();
}

fn bench_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("lastcit_game/undirected/n1000_uniform");
    let n = 1_000u32;
    let eps = 4u32;
    let agebins = 2u32;
    let pref = vec![1.0; (agebins as usize) + 1];
    group.throughput(Throughput::Elements(u64::from(n)));
    group.bench_function("undirected_uniform", |b| {
        b.iter(|| lastcit_game(n, eps, agebins, &pref, false, 0x1A57_C173_u64).unwrap());
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_size_scaling,
    bench_eps_count,
    bench_agebins_count,
    bench_undirected,
);
criterion_main!(benches);
