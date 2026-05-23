//! Wilson loop-erased random walk tree generator benchmarks
//! (ALGO-GN-004).
//!
//! Run: `cargo bench --bench bench_tree_game`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-GN-004.json`.
//!
//! Coverage: undirected and directed at three vertex-count scales. The
//! algorithm itself does the same work in both modes; the directed
//! group exists so that any storage-layer cost (canonicalisation,
//! adjacency-list direction tagging) shows up if it ever drifts.

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::tree_game_lerw;

fn bench_tree_lerw_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_game_lerw/undirected");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| tree_game_lerw(n, false, 0xC0FF_EE00).unwrap());
        });
    }
    group.finish();
}

fn bench_tree_lerw_directed(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_game_lerw/directed");
    for n in [100u32, 1_000, 10_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| tree_game_lerw(n, true, 0xDEAD_BEEF).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_tree_lerw_undirected,
    bench_tree_lerw_directed
);
criterion_main!(benches);
