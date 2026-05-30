//! Power-law fit baseline benchmarks (ALGO-PR-019).
//!
//! Run: `cargo bench --bench bench_power_law_fit`.
//! Results land under `target/criterion/`. A snapshot is committed to
//! `.codefuse/tracking/perf/ALGO-PR-019.json`.
//!
//! Two regimes exercise the two cost models of `power_law_fit`:
//! * continuous — closed-form MLE per candidate `xmin`, so cost tracks the
//!   xmin scan over the (sorted) distinct values;
//! * discrete — each candidate `xmin` runs a golden-section search whose
//!   objective evaluates a Hurwitz-zeta (Euler–Maclaurin) term, so it is the
//!   markedly heavier path.
//!
//! Data is generated deterministically (seeded LCG → inverse-CDF Pareto) so
//! the benchmark is reproducible without any RNG dependency.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::power_law_fit;

/// Deterministic Pareto(xmin=1, exponent) sample via inverse transform on a
/// 64-bit LCG. Returns `n` continuous values >= 1.
fn pareto_continuous(n: usize, exponent: f64, seed: u64) -> Vec<f64> {
    let mut state = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        // Numerical Recipes LCG; take the high bits as a uniform in (0, 1).
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        // The top 53 bits fit f64's mantissa exactly; the cast is deliberate.
        #[allow(clippy::cast_precision_loss)]
        let u = ((state >> 11) as f64 + 1.0) / ((1u64 << 53) as f64 + 1.0);
        // Inverse CDF of Pareto with xmin = 1: x = (1 - u)^(-1/(a-1)).
        out.push((1.0 - u).powf(-1.0 / (exponent - 1.0)));
    }
    out
}

/// Discrete counterpart: floor the continuous draw to an integer >= 1.
fn pareto_discrete(n: usize, exponent: f64, seed: u64) -> Vec<f64> {
    pareto_continuous(n, exponent, seed)
        .into_iter()
        .map(|x| x.floor().max(1.0))
        .collect()
}

fn bench_continuous(c: &mut Criterion) {
    let mut group = c.benchmark_group("power_law_fit/continuous_auto");
    for n in [200usize, 1_000, 5_000] {
        let data = pareto_continuous(n, 2.5, 0x5eed_1234);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &data, |b, data| {
            b.iter(|| power_law_fit(data, -1.0, false).expect("power_law_fit"));
        });
    }
    group.finish();
}

fn bench_discrete(c: &mut Criterion) {
    let mut group = c.benchmark_group("power_law_fit/discrete_auto");
    for n in [200usize, 1_000, 5_000] {
        let data = pareto_discrete(n, 2.5, 0x5eed_abcd);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &data, |b, data| {
            b.iter(|| power_law_fit(data, -1.0, false).expect("power_law_fit"));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_continuous, bench_discrete);
criterion_main!(benches);
