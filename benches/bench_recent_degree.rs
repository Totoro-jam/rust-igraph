//! Recent-degree-game benchmarks (ALGO-GN-019).
//!
//! Run: `cargo bench --bench bench_recent_degree`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-019.json`.
//!
//! Coverage:
//!   * `size_scaling/pow1_window10_m3` — `power=1`, `time_window=10`,
//!     `m=3`, `zero_appeal=1`, directed, sweep `n ∈ {500, 5_000}` —
//!     measures the psumtree set/search amortized over a steady-state
//!     window.
//!   * `m_count/n1000_pow15` — fixed `n=1_000`, `power=1.5`,
//!     `time_window=20`, `zero_appeal=1`, sweep `m ∈ {1, 4, 16}` —
//!     isolates per-step citation cost.
//!   * `window_count/n1000_m2` — `n=1_000`, `m=2`, sweep
//!     `time_window ∈ {2, 20, 200}` — exercises BIT-tree expiry vs
//!     refresh trade-off (short window ⇒ near-constant tree size; long
//!     window ⇒ growing live set up to `min(n, window·m)`).
//!   * `undirected/n1000_outpref` — undirected variant with
//!     `outpref=true`, `power=1.0`, `time_window=15`, `m=2`,
//!     `zero_appeal=0.5` — exercises the source-weight refresh branch.
//!
//! Throughput is reported in total vertices so wall-clock per element
//! is directly comparable to the other generator benches.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::recent_degree_game;

fn bench_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("recent_degree_game/size_scaling/pow1_window10_m3");
    let power = 1.0;
    let time_window = 10u32;
    let m = 3u32;
    let zero_appeal = 1.0;
    for n in [500u32, 5_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                recent_degree_game(
                    n,
                    power,
                    time_window,
                    m,
                    None,
                    false,
                    zero_appeal,
                    true,
                    0x2D57_C170_u64,
                )
                .unwrap()
            });
        });
    }
    group.finish();
}

fn bench_m_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("recent_degree_game/m_count/n1000_pow15");
    let n = 1_000u32;
    let power = 1.5;
    let time_window = 20u32;
    let zero_appeal = 1.0;
    for m in [1u32, 4, 16] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(m), &m, |b, &m| {
            b.iter(|| {
                recent_degree_game(
                    n,
                    power,
                    time_window,
                    m,
                    None,
                    false,
                    zero_appeal,
                    true,
                    0x2D57_C171_u64,
                )
                .unwrap()
            });
        });
    }
    group.finish();
}

fn bench_window_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("recent_degree_game/window_count/n1000_m2");
    let n = 1_000u32;
    let power = 1.0;
    let m = 2u32;
    let zero_appeal = 1.0;
    for time_window in [2u32, 20, 200] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(
            BenchmarkId::from_parameter(time_window),
            &time_window,
            |b, &time_window| {
                b.iter(|| {
                    recent_degree_game(
                        n,
                        power,
                        time_window,
                        m,
                        None,
                        false,
                        zero_appeal,
                        true,
                        0x2D57_C172_u64,
                    )
                    .unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("recent_degree_game/undirected/n1000_outpref");
    let n = 1_000u32;
    let power = 1.0;
    let time_window = 15u32;
    let m = 2u32;
    let zero_appeal = 0.5;
    group.throughput(Throughput::Elements(u64::from(n)));
    group.bench_function("undirected_outpref", |b| {
        b.iter(|| {
            recent_degree_game(
                n,
                power,
                time_window,
                m,
                None,
                true,
                zero_appeal,
                false,
                0x2D57_C173_u64,
            )
            .unwrap()
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_size_scaling,
    bench_m_count,
    bench_window_count,
    bench_undirected,
);
criterion_main!(benches);
