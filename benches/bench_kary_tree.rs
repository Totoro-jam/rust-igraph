//! k-ary tree deterministic constructor benchmarks (ALGO-CN-004).
//!
//! Run: `cargo bench --bench bench_kary_tree`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-004.json`.
//!
//! Coverage: all three `TreeMode` variants (Out / In / Undirected) across
//! three vertex-count scales (1e2, 1e4, 1e6) with binary children. The
//! generator emits exactly `n - 1` edges in BFS order, so total work is
//! `O(|V|)` per call and dominated by `Graph::add_edges` reallocations.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{TreeMode, kary_tree};

fn bench_kary_tree_out(c: &mut Criterion) {
    let mut group = c.benchmark_group("kary_tree/out");
    for n in [100u32, 10_000, 1_000_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| kary_tree(n, 2, TreeMode::Out).unwrap());
        });
    }
    group.finish();
}

fn bench_kary_tree_in(c: &mut Criterion) {
    let mut group = c.benchmark_group("kary_tree/in");
    for n in [100u32, 10_000, 1_000_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| kary_tree(n, 2, TreeMode::In).unwrap());
        });
    }
    group.finish();
}

fn bench_kary_tree_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("kary_tree/undirected");
    for n in [100u32, 10_000, 1_000_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| kary_tree(n, 2, TreeMode::Undirected).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_kary_tree_out,
    bench_kary_tree_in,
    bench_kary_tree_undirected
);
criterion_main!(benches);
