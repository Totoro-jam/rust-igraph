//! Barabási–Albert PSUMTREE variant random-graph benchmarks
//! (ALGO-GN-020).
//!
//! Run: `cargo bench --bench bench_barabasi_psumtree`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-GN-020.json`.
//!
//! Coverage targets the two cost drivers in the Fenwick-BIT path:
//!   * `simple/directed_classic` — classical (power=1, A=1) BA with
//!     `m = 2` and the per-draw zero-and-refresh dance that defines the
//!     simple variant. Scans n to track the `O(n · m · log n)` slope.
//!   * `simple/undirected_pow15` — same but `power = 1.5`, exercising
//!     the `pow()` call inside the weight refresh; undirected forces
//!     `outpref = true` so every fresh vertex's own out-degree feeds
//!     back into the BIT.
//!   * `multiple/directed_m3` — `PSUMTREE_MULTIPLE` with `m = 3`,
//!     measuring the snapshot-once-per-step path that allows
//!     within-step multi-edges.

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{barabasi_game_psumtree, barabasi_game_psumtree_multiple};

fn bench_psumtree_simple_directed(c: &mut Criterion) {
    let mut group = c.benchmark_group("barabasi_psumtree/simple_directed_classic");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                barabasi_game_psumtree(n, 1.0, 2, None, false, 1.0, true, 0xBA_5E_BA_5E).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_psumtree_simple_undirected_pow15(c: &mut Criterion) {
    let mut group = c.benchmark_group("barabasi_psumtree/simple_undirected_pow15");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                barabasi_game_psumtree(n, 1.5, 2, None, false, 1.0, false, 0xC0FF_EE15).unwrap()
            });
        });
    }
    group.finish();
}

fn bench_psumtree_multiple_directed(c: &mut Criterion) {
    let mut group = c.benchmark_group("barabasi_psumtree/multiple_directed_m3");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                barabasi_game_psumtree_multiple(n, 1.0, 3, None, false, 1.0, true, 0xDEAD_F00D)
                    .unwrap()
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_psumtree_simple_directed,
    bench_psumtree_simple_undirected_pow15,
    bench_psumtree_multiple_directed,
);
criterion_main!(benches);
