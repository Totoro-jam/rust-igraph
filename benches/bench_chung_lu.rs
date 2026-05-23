//! Chung–Lu expected-degree-model benchmarks (ALGO-GN-012).
//!
//! Run: `cargo bench --bench bench_chung_lu`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-012.json`.
//!
//! Coverage:
//!   * `size_scaling/uniform_undirected` — uniform per-vertex weight
//!     `w = 10.0` (≈ mean degree 10) sweep `n ∈ {500, 5_000}` —
//!     measures the linear `O(|V| + |E|)` cost of the Miller–Hagberg
//!     geometric-skip sampler under a constant mean-degree regime.
//!   * `variant_sweep/uniform_n2000` — fixed `n = 2_000`, uniform
//!     `w = 8.0`, undirected, sweep over the three connection-probability
//!     formulas (`Original`, `Maxent`, `Nr`). The three variants share
//!     the same sampler skeleton but differ in a per-edge `q → p`
//!     transform; this group isolates that transform's marginal cost.
//!   * `weight_skew/n2000_maxent` — fixed `n = 2_000`, undirected,
//!     `Maxent`; compare uniform `w = 8.0` vs a power-law-style weight
//!     vector with a heavy tail (`w_i = 1.0 + 0.5·i` so a handful of
//!     hubs dominate). Miller–Hagberg's cost is sensitive to the
//!     descending-sort tail: heavy-tailed vectors traverse the early
//!     indices much faster.
//!   * `directed/n2000` — directed graph, `n = 2_000`, `in_weights =
//!     out_weights` (uniform), undirected → directed contrast. The
//!     directed code path samples the full `n × n` slot space (no
//!     `j ≥ i` early-exit) so the per-edge cost roughly doubles.
//!   * `density_sweep/maxent_n2000` — fixed `n = 2_000`, undirected,
//!     `Maxent`, sweep uniform `w ∈ {1.0, 5.0, 20.0, 100.0}` — drives
//!     mean degree from sparse (≈1) to dense (≈100); shows how the
//!     realised edge count dominates total time.
//!
//! Throughput is reported in total vertices so wall-clock per vertex is
//! directly comparable to the other generator benches in this crate.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{ChungLuVariant, chung_lu_game};

/// Vector of `n` identical weights, useful as a sane "expected mean
/// degree = w" knob.
fn uniform_weights(n: usize, w: f64) -> Vec<f64> {
    vec![w; n]
}

/// Power-law-flavoured weight vector: `w_i = base + step · i`. We then
/// rescale so the total weight matches `target_sum`, so the mean degree
/// is comparable to a uniform-`w` baseline.
fn skewed_weights(n: usize, base: f64, step: f64, target_sum: f64) -> Vec<f64> {
    let mut w: Vec<f64> = (0..n).map(|i| base + step * (i as f64)).collect();
    let raw_sum: f64 = w.iter().sum();
    let scale = target_sum / raw_sum;
    for x in &mut w {
        *x *= scale;
    }
    w
}

fn bench_size_scaling_uniform(c: &mut Criterion) {
    let mut group = c.benchmark_group("chung_lu_game/size_scaling/uniform_undirected");
    let w = 10.0_f64;
    for n in [500usize, 5_000] {
        let weights = uniform_weights(n, w);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                chung_lu_game(&weights, None, false, ChungLuVariant::Maxent, 0x5C10_1001).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_variant_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("chung_lu_game/variant_sweep/uniform_n2000");
    let n = 2_000usize;
    let weights = uniform_weights(n, 8.0);
    for (label, variant) in [
        ("original", ChungLuVariant::Original),
        ("maxent", ChungLuVariant::Maxent),
        ("nr", ChungLuVariant::Nr),
    ] {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(label), &variant, |b, &v| {
            b.iter(|| chung_lu_game(&weights, None, false, v, 0x5C10_1002).unwrap());
        });
    }
    group.finish();
}

fn bench_weight_skew(c: &mut Criterion) {
    let mut group = c.benchmark_group("chung_lu_game/weight_skew/n2000_maxent");
    let n = 2_000usize;
    let target_sum = 8.0 * (n as f64);

    let uniform = uniform_weights(n, 8.0);
    let skewed = skewed_weights(n, 1.0, 0.5, target_sum);

    group.throughput(Throughput::Elements(n as u64));
    group.bench_function("uniform_w8", |b| {
        b.iter(|| {
            chung_lu_game(&uniform, None, false, ChungLuVariant::Maxent, 0x5C10_1003).unwrap()
        });
    });
    group.bench_function("powerlawish_basescale", |b| {
        b.iter(|| {
            chung_lu_game(&skewed, None, false, ChungLuVariant::Maxent, 0x5C10_1003).unwrap()
        });
    });
    group.finish();
}

fn bench_directed(c: &mut Criterion) {
    let mut group = c.benchmark_group("chung_lu_game/directed/n2000");
    let n = 2_000usize;
    let weights = uniform_weights(n, 8.0);
    group.throughput(Throughput::Elements(n as u64));
    group.bench_function("directed_no_loops", |b| {
        b.iter(|| {
            chung_lu_game(
                &weights,
                Some(&weights),
                false,
                ChungLuVariant::Maxent,
                0x5C10_1004,
            )
            .unwrap()
        });
    });
    group.bench_function("undirected_no_loops", |b| {
        b.iter(|| {
            chung_lu_game(&weights, None, false, ChungLuVariant::Maxent, 0x5C10_1004).unwrap()
        });
    });
    group.finish();
}

fn bench_density_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("chung_lu_game/density_sweep/maxent_n2000");
    let n = 2_000usize;
    for w in [1.0_f64, 5.0, 20.0, 100.0] {
        let weights = uniform_weights(n, w);
        let label = format!("w{w:.0}");
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(label), &w, |b, _| {
            b.iter(|| {
                chung_lu_game(&weights, None, false, ChungLuVariant::Maxent, 0x5C10_1005).unwrap()
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_size_scaling_uniform,
    bench_variant_sweep,
    bench_weight_skew,
    bench_directed,
    bench_density_sweep,
);
criterion_main!(benches);
