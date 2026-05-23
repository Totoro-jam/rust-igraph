//! Stochastic Block Model benchmarks (ALGO-GN-010).
//!
//! Run: `cargo bench --bench bench_sbm`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-010.json`.
//!
//! Coverage:
//!   * `size_scaling/balanced_k4` — `k=4` equal blocks with in-block
//!     `p=0.05` and off-block `p=0.005`, sweep `total_size ∈ {500,
//!     5_000}` — assortative regime; exercises both diagonal-Tri and
//!     off-diagonal-Rect samplers as `n` grows.
//!   * `sparsity/off_diagonal_k4_n2000` — fixed `k=4, n=2_000`, in-block
//!     `p=0.05`, sweep off-block `p ∈ {0.0, 0.001, 0.005, 0.05}` —
//!     isolates how between-block density drives the rectangular-pair
//!     geometric-skip cost.
//!   * `density_sweep/k4_n2000` — fixed `k=4, n=2_000`, off-block
//!     `p=0.005`, sweep in-block `p ∈ {0.01, 0.05, 0.1, 0.3}` — how
//!     within-block density grows the triangular sampler cost.
//!   * `directed/k4_n2000_asym` — directed graph, `k=4, n=2_000`, with a
//!     mildly asymmetric pref matrix; exercises the directed code path
//!     (both `Rect` and `RectNoDiag` decoders).
//!   * `multigraph/k4_n2000_dense` — `k=4, n=2_000`, multigraph mode,
//!     in-block expected-multiplicity `0.2`, off-block `0.02` — the
//!     `Pascal`-distributed fast path with no de-duplication.
//!
//! Throughput is reported in total vertices so wall-clock per element
//! is directly comparable to the other generator benches.

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::sbm_game;

/// `k × k` pref matrix with `p_in` on the diagonal and `p_off`
/// elsewhere.
fn assortative_pref(k: usize, p_in: f64, p_off: f64) -> Vec<Vec<f64>> {
    (0..k)
        .map(|i| (0..k).map(|j| if i == j { p_in } else { p_off }).collect())
        .collect()
}

/// Balanced block-size vector summing to `total`. The last block
/// absorbs any rounding remainder so the sum is exact.
fn balanced_sizes(k: usize, total: u32) -> Vec<u32> {
    let base = total / k as u32;
    let rem = total - base * k as u32;
    let mut sizes = vec![base; k];
    *sizes.last_mut().expect("k > 0") += rem;
    sizes
}

fn bench_size_scaling_balanced(c: &mut Criterion) {
    let mut group = c.benchmark_group("sbm_game/size_scaling/balanced_k4");
    let k = 4usize;
    let p_in = 0.05_f64;
    let p_off = 0.005_f64;
    let pref = assortative_pref(k, p_in, p_off);
    for total in [500u32, 5_000] {
        let sizes = balanced_sizes(k, total);
        group.throughput(Throughput::Elements(u64::from(total)));
        group.bench_with_input(BenchmarkId::from_parameter(total), &total, |b, _| {
            b.iter(|| sbm_game(&pref, &sizes, false, false, false, 0x5B30_1001).unwrap());
        });
    }
    group.finish();
}

fn bench_sparsity_off_diagonal(c: &mut Criterion) {
    let mut group = c.benchmark_group("sbm_game/sparsity/off_diagonal_k4_n2000");
    let k = 4usize;
    let total = 2_000u32;
    let sizes = balanced_sizes(k, total);
    let p_in = 0.05_f64;
    for p_off in [0.0_f64, 0.001, 0.005, 0.05] {
        let pref = assortative_pref(k, p_in, p_off);
        let label = format!("poff{p_off:.3}");
        group.throughput(Throughput::Elements(u64::from(total)));
        group.bench_with_input(BenchmarkId::from_parameter(label), &p_off, |b, _| {
            b.iter(|| sbm_game(&pref, &sizes, false, false, false, 0x5B30_1002).unwrap());
        });
    }
    group.finish();
}

fn bench_density_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("sbm_game/density_sweep/k4_n2000");
    let k = 4usize;
    let total = 2_000u32;
    let sizes = balanced_sizes(k, total);
    let p_off = 0.005_f64;
    for p_in in [0.01_f64, 0.05, 0.1, 0.3] {
        let pref = assortative_pref(k, p_in, p_off);
        let label = format!("pin{p_in:.2}");
        group.throughput(Throughput::Elements(u64::from(total)));
        group.bench_with_input(BenchmarkId::from_parameter(label), &p_in, |b, _| {
            b.iter(|| sbm_game(&pref, &sizes, false, false, false, 0x5B30_1003).unwrap());
        });
    }
    group.finish();
}

fn bench_directed(c: &mut Criterion) {
    let mut group = c.benchmark_group("sbm_game/directed/k4_n2000_asym");
    let k = 4usize;
    let total = 2_000u32;
    let sizes = balanced_sizes(k, total);
    // Asymmetric pref: upper triangle slightly heavier than lower.
    let pref: Vec<Vec<f64>> = (0..k)
        .map(|i| {
            (0..k)
                .map(|j| match i.cmp(&j) {
                    std::cmp::Ordering::Equal => 0.05,
                    std::cmp::Ordering::Less => 0.01,
                    std::cmp::Ordering::Greater => 0.002,
                })
                .collect()
        })
        .collect();
    group.throughput(Throughput::Elements(u64::from(total)));
    group.bench_function("directed_no_loops", |b| {
        b.iter(|| sbm_game(&pref, &sizes, true, false, false, 0x5B30_1004).unwrap());
    });
    group.finish();
}

fn bench_multigraph(c: &mut Criterion) {
    let mut group = c.benchmark_group("sbm_game/multigraph/k4_n2000_dense");
    let k = 4usize;
    let total = 2_000u32;
    let sizes = balanced_sizes(k, total);
    let pref = assortative_pref(k, 0.2, 0.02);
    group.throughput(Throughput::Elements(u64::from(total)));
    group.bench_function("undirected_loops_multi", |b| {
        b.iter(|| sbm_game(&pref, &sizes, false, true, true, 0x5B30_1005).unwrap());
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_size_scaling_balanced,
    bench_sparsity_off_diagonal,
    bench_density_sweep,
    bench_directed,
    bench_multigraph,
);
criterion_main!(benches);
