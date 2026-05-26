//! Recent-degree preferential attachment with vertex aging benchmarks
//! (ALGO-GN-032).
//!
//! Run: `cargo bench --bench bench_recent_degree_aging`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-GN-032.json`.
//!
//! Coverage targets the three cost drivers in the recent-degree-aging
//! path:
//!   * `no_aging` — `pa_exp=1`, `aging_exp=0` (aging term collapses to
//!     1 constant; the sliding window still fires).
//!   * `strong_aging` — `pa_exp=1`, `aging_exp=-1` (every age boundary
//!     forces a real `pow(age, -1)` evaluation; the age sweep plus
//!     window expiry both amortize work).
//!   * `outpref_undirected` — `pa_exp=1`, `aging_exp=-0.5`,
//!     `outpref=true`, undirected (each fresh vertex's own out-degree
//!     feeds back into the BIT, doubling refresh work).

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::recent_degree_aging_game;

fn bench_no_aging(c: &mut Criterion) {
    let mut group = c.benchmark_group("recent_degree_aging/no_aging");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                recent_degree_aging_game(
                    n,
                    2,
                    None,
                    false,
                    1.0,
                    0.0,
                    10,
                    5,
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

fn bench_strong_aging(c: &mut Criterion) {
    let mut group = c.benchmark_group("recent_degree_aging/strong_aging");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                recent_degree_aging_game(
                    n,
                    2,
                    None,
                    false,
                    1.0,
                    -1.0,
                    10,
                    8,
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

fn bench_outpref_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("recent_degree_aging/outpref_undirected");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                recent_degree_aging_game(
                    n,
                    2,
                    None,
                    true,
                    1.0,
                    -0.5,
                    8,
                    10,
                    0.5,
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
    bench_no_aging,
    bench_strong_aging,
    bench_outpref_undirected,
);
criterion_main!(benches);
