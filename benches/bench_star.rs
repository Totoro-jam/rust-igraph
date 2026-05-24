//! Star deterministic constructor benchmarks (ALGO-CN-002).
//!
//! Run: `cargo bench --bench bench_star`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-002.json`.
//!
//! Coverage: all four `StarMode` variants (Out / In / Mutual /
//! Undirected) across three vertex-count scales (1e2, 1e4, 1e6). The
//! model is `O(|V|)` everywhere — a single sweep over leaves emits
//! `n - 1` (or `2(n - 1)` for `Mutual`) edges, so timings should
//! scale linearly and are dominated by `Graph::add_edges` cost.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{StarMode, star_graph};

fn bench_star_out(c: &mut Criterion) {
    let mut group = c.benchmark_group("star_graph/out");
    for n in [100u32, 10_000, 1_000_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| star_graph(n, StarMode::Out, 0).unwrap());
        });
    }
    group.finish();
}

fn bench_star_in(c: &mut Criterion) {
    let mut group = c.benchmark_group("star_graph/in");
    for n in [100u32, 10_000, 1_000_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| star_graph(n, StarMode::In, 0).unwrap());
        });
    }
    group.finish();
}

fn bench_star_mutual(c: &mut Criterion) {
    let mut group = c.benchmark_group("star_graph/mutual");
    for n in [100u32, 10_000, 1_000_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| star_graph(n, StarMode::Mutual, 0).unwrap());
        });
    }
    group.finish();
}

fn bench_star_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("star_graph/undirected");
    for n in [100u32, 10_000, 1_000_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| star_graph(n, StarMode::Undirected, 0).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_star_out,
    bench_star_in,
    bench_star_mutual,
    bench_star_undirected
);
criterion_main!(benches);
