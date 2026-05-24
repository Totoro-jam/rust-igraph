//! Random dot-product graph benchmarks (ALGO-GN-022).
//!
//! Run: `cargo bench --bench bench_dot_product`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-022.json`.
//!
//! Coverage targets the three cost drivers in `dot_product_game`:
//!   * `dim_sweep_undirected` — sweep latent-vector dimension `d ∈
//!     {1, 4, 16, 64}` at fixed `n = 400`, undirected. Each pair pays
//!     `O(d)` for the inner product; this group isolates that linear
//!     factor.
//!   * `size_sweep_undirected_d8` — sweep vertex count `n ∈ {100, 400,
//!     1_600}` at fixed `d = 8`, undirected. Pair count is `n(n−1)/2`,
//!     so wall-clock per-vertex grows linearly.
//!   * `directed_vs_undirected_n400_d8` — directed vs undirected at
//!     fixed `n = 400`, `d = 8`. The directed code path inspects all
//!     `n(n−1)` ordered pairs (≈ 2× the undirected work).
//!
//! Latent vectors are drawn from a deterministic `SplitMix64`-style
//! reduction of the bench seed so each call sees the same input; values
//! sit in `[0, 1/√d]` so the expected pair probability hovers near
//! `1/3` (well inside the Bernoulli regime, exercising the RNG draw).

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::dot_product_game;

/// Build a deterministic latent-position matrix of `n` vectors of
/// dimension `d`, with entries in `[0, 1/√d]` so that pair dot products
/// are bounded by `1.0` (well inside the Bernoulli regime).
fn make_vecs(n: usize, d: usize, seed: u64) -> Vec<Vec<f64>> {
    // Cheap deterministic mix — same SplitMix64 step constants used by
    // the project's core RNG. We don't want to allocate a real PRNG in
    // a hot bench helper.
    let mut state = seed.wrapping_add(1);
    let mut step = || -> f64 {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        let bits = (z ^ (z >> 31)) >> 11; // 53-bit mantissa
        #[allow(clippy::cast_precision_loss)]
        let u01 = (bits as f64) * (1.0_f64 / 9_007_199_254_740_992.0_f64);
        u01
    };
    let bound = 1.0_f64 / (d as f64).sqrt();
    (0..n)
        .map(|_| (0..d).map(|_| step() * bound).collect())
        .collect()
}

fn bench_dim_sweep_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("dot_product/dim_sweep_undirected");
    let n = 400usize;
    for d in [1usize, 4, 16, 64] {
        let vecs = make_vecs(n, d, 0xDEAD_BEEF_u64.wrapping_add(d as u64));
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(d), &d, |b, _| {
            b.iter(|| dot_product_game(&vecs, false, 0xC0FF_EE21).unwrap());
        });
    }
    group.finish();
}

fn bench_size_sweep_undirected_d8(c: &mut Criterion) {
    let mut group = c.benchmark_group("dot_product/size_sweep_undirected_d8");
    let d = 8usize;
    for n in [100usize, 400, 1_600] {
        let vecs = make_vecs(n, d, 0x1234_5678_u64.wrapping_add(n as u64));
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| dot_product_game(&vecs, false, 0xCAFE_F00D).unwrap());
        });
    }
    group.finish();
}

fn bench_directed_vs_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("dot_product/directed_vs_undirected_n400_d8");
    let n = 400usize;
    let d = 8usize;
    let vecs = make_vecs(n, d, 0xABCD_EF01);
    group.throughput(Throughput::Elements(n as u64));
    group.bench_function("undirected", |b| {
        b.iter(|| dot_product_game(&vecs, false, 0x0FFE_BEAD).unwrap());
    });
    group.bench_function("directed", |b| {
        b.iter(|| dot_product_game(&vecs, true, 0x0FFE_BEAD).unwrap());
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_dim_sweep_undirected,
    bench_size_sweep_undirected_d8,
    bench_directed_vs_undirected,
);
criterion_main!(benches);
