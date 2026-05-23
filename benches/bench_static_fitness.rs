//! Static-fitness / static-power-law game benchmarks (ALGO-GN-013).
//!
//! Run: `cargo bench --bench bench_static_fitness`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-013.json`.
//!
//! Coverage:
//!   * `static_fitness_game/size_scaling/uniform_undirected_simple` —
//!     hold mean degree at `m / n ≈ 5`, sweep `n ∈ {500, 5_000}` under
//!     simple (loops=false, multiple=false) sampling. Exercises the
//!     `O(m log n)` cumulative-fitness binary search.
//!   * `static_fitness_game/multi_vs_simple/n2000_uniform` — fixed
//!     `n = 2_000`, `m = 8 n`, contrast trivial sample-and-keep
//!     (`multiple=true`) against rejection-sampled simple
//!     (`multiple=false`). The simple loop pays an extra `HashSet`
//!     lookup per accepted edge.
//!   * `static_fitness_game/directed/n2000` — directed shape with
//!     separate `fitness_in` vs undirected baseline; the directed code
//!     path samples both vertices independently and skips the canonical
//!     `(min, max)` tuple step.
//!   * `static_fitness_game/skew/n2000_simple` — uniform fitness vs
//!     power-law-flavoured fitness `f_i = 1 + 0.5·i`. Skew widens the
//!     hub fan-in so rejection drops fewer samples.
//!   * `static_power_law_game/exponent_sweep/n2000_undirected` —
//!     sweep `γ ∈ {2.1, 2.5, 3.0, 4.0}` (closer to 2 → heavier tails).
//!   * `static_power_law_game/fsc_toggle/n2000_undirected_g25` —
//!     `finite_size_correction` on vs off at `γ = 2.5`.
//!
//! Throughput is reported in vertices so wall-clock per vertex is
//! comparable across generators.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{static_fitness_game, static_power_law_game};

/// Vector of `n` identical fitness scores.
fn uniform_fitness(n: usize, w: f64) -> Vec<f64> {
    vec![w; n]
}

/// Linearly skewed fitness vector `f_i = base + step · i`, rescaled so
/// `Σ f` matches `target_sum` to keep mean degree comparable.
fn skewed_fitness(n: usize, base: f64, step: f64, target_sum: f64) -> Vec<f64> {
    let mut f: Vec<f64> = (0..n).map(|i| base + step * (i as f64)).collect();
    let raw_sum: f64 = f.iter().sum();
    let scale = target_sum / raw_sum;
    for x in &mut f {
        *x *= scale;
    }
    f
}

fn bench_fitness_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("static_fitness_game/size_scaling/uniform_undirected_simple");
    for n in [500usize, 5_000] {
        let fitness = uniform_fitness(n, 1.0);
        let m = (5 * n) as u32;
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| static_fitness_game(m, &fitness, None, false, false, 0x5F11_1001).unwrap());
        });
    }
    group.finish();
}

fn bench_fitness_multi_vs_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("static_fitness_game/multi_vs_simple/n2000_uniform");
    let n = 2_000usize;
    let fitness = uniform_fitness(n, 1.0);
    let m = (8 * n) as u32;
    group.throughput(Throughput::Elements(n as u64));
    group.bench_function("simple_no_loops", |b| {
        b.iter(|| static_fitness_game(m, &fitness, None, false, false, 0x5F11_1002).unwrap());
    });
    group.bench_function("multi_with_loops", |b| {
        b.iter(|| static_fitness_game(m, &fitness, None, true, true, 0x5F11_1002).unwrap());
    });
    group.finish();
}

fn bench_fitness_directed(c: &mut Criterion) {
    let mut group = c.benchmark_group("static_fitness_game/directed/n2000");
    let n = 2_000usize;
    let fitness = uniform_fitness(n, 1.0);
    let m = (8 * n) as u32;
    group.throughput(Throughput::Elements(n as u64));
    group.bench_function("directed_simple", |b| {
        b.iter(|| {
            static_fitness_game(m, &fitness, Some(&fitness), false, false, 0x5F11_1003).unwrap()
        });
    });
    group.bench_function("undirected_simple", |b| {
        b.iter(|| static_fitness_game(m, &fitness, None, false, false, 0x5F11_1003).unwrap());
    });
    group.finish();
}

fn bench_fitness_skew(c: &mut Criterion) {
    let mut group = c.benchmark_group("static_fitness_game/skew/n2000_simple");
    let n = 2_000usize;
    let m = (8 * n) as u32;
    let target_sum = (n as f64) * 1.0;
    let uniform = uniform_fitness(n, 1.0);
    let skewed = skewed_fitness(n, 0.1, 0.05, target_sum);
    group.throughput(Throughput::Elements(n as u64));
    group.bench_function("uniform_f1", |b| {
        b.iter(|| static_fitness_game(m, &uniform, None, false, false, 0x5F11_1004).unwrap());
    });
    group.bench_function("powerlawish_f", |b| {
        b.iter(|| static_fitness_game(m, &skewed, None, false, false, 0x5F11_1004).unwrap());
    });
    group.finish();
}

fn bench_power_law_exponent_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("static_power_law_game/exponent_sweep/n2000_undirected");
    let n = 2_000u32;
    let m = 8_000u32;
    for gamma in [2.1_f64, 2.5, 3.0, 4.0] {
        group.throughput(Throughput::Elements(u64::from(n)));
        let label = format!("g{gamma:.1}");
        group.bench_with_input(BenchmarkId::from_parameter(label), &gamma, |b, &g| {
            b.iter(|| {
                static_power_law_game(n, m, g, None, false, false, true, 0x5F11_2001).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_power_law_fsc_toggle(c: &mut Criterion) {
    let mut group = c.benchmark_group("static_power_law_game/fsc_toggle/n2000_undirected_g25");
    let n = 2_000u32;
    let m = 8_000u32;
    let gamma = 2.5_f64;
    group.throughput(Throughput::Elements(u64::from(n)));
    group.bench_function("fsc_on", |b| {
        b.iter(|| {
            static_power_law_game(n, m, gamma, None, false, false, true, 0x5F11_2002).unwrap()
        });
    });
    group.bench_function("fsc_off", |b| {
        b.iter(|| {
            static_power_law_game(n, m, gamma, None, false, false, false, 0x5F11_2002).unwrap()
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_fitness_size_scaling,
    bench_fitness_multi_vs_simple,
    bench_fitness_directed,
    bench_fitness_skew,
    bench_power_law_exponent_sweep,
    bench_power_law_fsc_toggle,
);
criterion_main!(benches);
