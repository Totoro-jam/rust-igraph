//! Correlated Erdős–Rényi graph benchmarks (ALGO-GN-023).
//!
//! Run: `cargo bench --bench bench_correlated`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-023.json`.
//!
//! Coverage targets the three cost drivers in `correlated_game`:
//!   * `corr_sweep_n800_p2_undirected` — sweep `corr ∈ {0.0, 0.25, 0.5,
//!     0.75, 1.0}` at fixed `n = 800, p = 0.2`, undirected. Higher
//!     `corr` ⇒ smaller `p_del` and `p_add` ⇒ fewer geometric-skip
//!     draws on the delete/add streams; the 3-way merge dominates.
//!   * `size_sweep_corr5_p2_undirected` — sweep vertex count `n ∈
//!     {200, 800, 3_200}` at fixed `corr = 0.5, p = 0.2`, undirected.
//!     Candidate-slot count is `C(n, 2)` so wall-clock scales near-
//!     quadratically with `n`.
//!   * `directed_vs_undirected_n800_corr5_p2` — directed vs undirected
//!     at fixed `n = 800, corr = 0.5, p = 0.2`. Directed walks `n(n−1)`
//!     ordered slots (~2× undirected work) and uses the
//!     diagonal-hole `D_CODE` encoding.
//!
//! Also covers the convenience wrapper `correlated_pair_game` since it
//! shares the same hot loop but pays an extra `ER(n, p)` sample upfront.

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, correlated_game, correlated_pair_game, erdos_renyi_gnp};

/// Sample an undirected ER(n, p) graph deterministically from `seed`.
fn er_graph(n: u32, p: f64, directed: bool, seed: u64) -> Graph {
    erdos_renyi_gnp(n, p, directed, false, seed).expect("ER bench helper sample failed")
}

fn bench_corr_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("correlated/corr_sweep_n800_p2_undirected");
    let n = 800u32;
    let p = 0.2_f64;
    let old = er_graph(n, p, false, 0x00C0_FFEE_u64);
    for corr_milli in [0u32, 250, 500, 750, 1000] {
        let corr = f64::from(corr_milli) / 1000.0;
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(corr_milli), &corr, |b, &c| {
            b.iter(|| correlated_game(&old, c, p, None, 0x1234_5678_u64).unwrap());
        });
    }
    group.finish();
}

fn bench_size_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("correlated/size_sweep_corr5_p2_undirected");
    let p = 0.2_f64;
    let corr = 0.5_f64;
    for n in [200u32, 800, 3_200] {
        let old = er_graph(n, p, false, 0xBEEF_u64.wrapping_add(u64::from(n)));
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| correlated_game(&old, corr, p, None, 0xCAFE_F00D_u64).unwrap());
        });
    }
    group.finish();
}

fn bench_directed_vs_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("correlated/directed_vs_undirected_n800_corr5_p2");
    let n = 800u32;
    let p = 0.2_f64;
    let corr = 0.5_f64;
    let old_undir = er_graph(n, p, false, 0x9999_u64);
    let old_dir = er_graph(n, p, true, 0x9999_u64);
    group.throughput(Throughput::Elements(u64::from(n)));
    group.bench_function("undirected", |b| {
        b.iter(|| correlated_game(&old_undir, corr, p, None, 0x0FFE_BEAD_u64).unwrap());
    });
    group.bench_function("directed", |b| {
        b.iter(|| correlated_game(&old_dir, corr, p, None, 0x0FFE_BEAD_u64).unwrap());
    });
    group.finish();
}

fn bench_pair_game(c: &mut Criterion) {
    let mut group = c.benchmark_group("correlated/pair_game_size_sweep_corr5_p2_undirected");
    let p = 0.2_f64;
    let corr = 0.5_f64;
    for n in [200u32, 800, 3_200] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| correlated_pair_game(n, corr, p, false, None, 0xABCD_EF01_u64).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_corr_sweep,
    bench_size_sweep,
    bench_directed_vs_undirected,
    bench_pair_game,
);
criterion_main!(benches);
