//! Barabási–Albert (BAG variant) random graph benchmarks (ALGO-GN-002).
//!
//! Run: `cargo bench --bench bench_barabasi`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-GN-002.json`.
//!
//! Coverage:
//!   - Directed sparse: `m = 2`, `outpref = false`, scans n. Exercises
//!     the dominant code path (constant-m, half-size bag).
//!   - Directed denser: `m = 5`, exercises the bag-growth rate when
//!     each step pushes more neighbours back.
//!   - Undirected (forced `outpref = true`): same n grid as the sparse
//!     directed case but with the doubled-bag write cost on every step
//!     — measures the upper bound on bag-allocator pressure.

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::barabasi_game_bag;

fn bench_ba_directed_m2(c: &mut Criterion) {
    let mut group = c.benchmark_group("barabasi_bag/directed_m2");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| barabasi_game_bag(n, 2, false, true, 0xBA_BA_BA_BA).unwrap());
        });
    }
    group.finish();
}

fn bench_ba_directed_m5(c: &mut Criterion) {
    let mut group = c.benchmark_group("barabasi_bag/directed_m5");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| barabasi_game_bag(n, 5, false, true, 0xDEAD_BEEF).unwrap());
        });
    }
    group.finish();
}

fn bench_ba_undirected_m2(c: &mut Criterion) {
    let mut group = c.benchmark_group("barabasi_bag/undirected_m2_outpref_forced");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| barabasi_game_bag(n, 2, false, false, 0xCAFE_F00D).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_ba_directed_m2,
    bench_ba_directed_m5,
    bench_ba_undirected_m2,
);
criterion_main!(benches);
