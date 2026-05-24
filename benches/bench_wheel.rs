//! Wheel deterministic constructor benchmarks (ALGO-CN-003).
//!
//! Run: `cargo bench --bench bench_wheel`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-003.json`.
//!
//! Coverage: all four `WheelMode` variants (Out / In / Mutual /
//! Undirected) across three vertex-count scales (1e2, 1e4, 1e6). The
//! wheel emits `n - 1` spoke edges followed by `n - 1` rim edges (twice
//! both for `Mutual`), so total work is `O(|V|)` per call and dominated
//! by `Graph::add_edges` reallocations.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{WheelMode, wheel_graph};

fn bench_wheel_out(c: &mut Criterion) {
    let mut group = c.benchmark_group("wheel_graph/out");
    for n in [100u32, 10_000, 1_000_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| wheel_graph(n, WheelMode::Out, 0).unwrap());
        });
    }
    group.finish();
}

fn bench_wheel_in(c: &mut Criterion) {
    let mut group = c.benchmark_group("wheel_graph/in");
    for n in [100u32, 10_000, 1_000_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| wheel_graph(n, WheelMode::In, 0).unwrap());
        });
    }
    group.finish();
}

fn bench_wheel_mutual(c: &mut Criterion) {
    let mut group = c.benchmark_group("wheel_graph/mutual");
    for n in [100u32, 10_000, 1_000_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| wheel_graph(n, WheelMode::Mutual, 0).unwrap());
        });
    }
    group.finish();
}

fn bench_wheel_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("wheel_graph/undirected");
    for n in [100u32, 10_000, 1_000_000] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| wheel_graph(n, WheelMode::Undirected, 0).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_wheel_out,
    bench_wheel_in,
    bench_wheel_mutual,
    bench_wheel_undirected
);
criterion_main!(benches);
