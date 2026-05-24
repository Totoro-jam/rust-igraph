//! Barabási–Albert with vertex aging random-graph benchmarks
//! (ALGO-GN-021).
//!
//! Run: `cargo bench --bench bench_barabasi_aging`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-GN-021.json`.
//!
//! Coverage targets the three cost drivers in the aging path:
//!   * `classical_no_aging` — `pa_exp=1`, `aging_exp=0` (aging term
//!     collapses to a constant; weight refresh reduces to classical
//!     PA but the age-sweep loop still fires every `binwidth` steps).
//!   * `aging_heavy` — `pa_exp=1`, `aging_exp=-1` (every age boundary
//!     forces a real `pow(age, -1)` evaluation; the age sweep
//!     amortizes `O((n / aging_bins) · log n)` per run).
//!   * `outpref_undirected` — `pa_exp=1`, `aging_exp=-0.5`,
//!     `outpref=true`, undirected (each fresh vertex's own out-degree
//!     feeds back into the BIT, doubling refresh work).

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::barabasi_aging_game;

fn bench_aging_classical_no_aging(c: &mut Criterion) {
    let mut group = c.benchmark_group("barabasi_aging/classical_no_aging");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                barabasi_aging_game(
                    n,
                    2,
                    None,
                    false,
                    1.0,
                    0.0,
                    10,
                    1.0,
                    1.0,
                    1.0,
                    1.0,
                    true,
                    0xAA_BB_CC_DD,
                )
                .unwrap()
            });
        });
    }
    group.finish();
}

fn bench_aging_heavy(c: &mut Criterion) {
    let mut group = c.benchmark_group("barabasi_aging/aging_heavy");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                barabasi_aging_game(
                    n,
                    2,
                    None,
                    false,
                    1.0,
                    -1.0,
                    10,
                    1.0,
                    1.0,
                    1.0,
                    1.0,
                    true,
                    0xBE_EF_FA_CE,
                )
                .unwrap()
            });
        });
    }
    group.finish();
}

fn bench_aging_outpref_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("barabasi_aging/outpref_undirected");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                barabasi_aging_game(
                    n,
                    2,
                    None,
                    true,
                    1.0,
                    -0.5,
                    8,
                    0.5,
                    1.0,
                    1.0,
                    1.0,
                    false,
                    0xC0FF_EE21,
                )
                .unwrap()
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_aging_classical_no_aging,
    bench_aging_heavy,
    bench_aging_outpref_undirected,
);
criterion_main!(benches);
