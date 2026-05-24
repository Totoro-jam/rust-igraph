//! Ring / path / cycle deterministic constructor benchmarks
//! (ALGO-CN-001).
//!
//! Run: `cargo bench --bench bench_ring`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-001.json`.
//!
//! Coverage: undirected path + undirected cycle + directed-mutual cycle
//! across three vertex-count scales (1e2, 1e4, 1e6). The model is
//! `O(|V|)` everywhere, so timings should scale linearly and dominate
//! purely by `Graph::add_edges` cost.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::ring_graph;

fn bench_ring_path_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_graph/path_undirected");
    for n in [100u32, 10_000, 1_000_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| ring_graph(n, false, false, false).unwrap());
        });
    }
    group.finish();
}

fn bench_ring_cycle_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_graph/cycle_undirected");
    for n in [100u32, 10_000, 1_000_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| ring_graph(n, false, false, true).unwrap());
        });
    }
    group.finish();
}

fn bench_ring_cycle_directed_mutual(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_graph/cycle_directed_mutual");
    for n in [100u32, 10_000, 1_000_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| ring_graph(n, true, true, true).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_ring_path_undirected,
    bench_ring_cycle_undirected,
    bench_ring_cycle_directed_mutual
);
criterion_main!(benches);
