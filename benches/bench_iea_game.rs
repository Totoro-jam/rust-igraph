//! Independent Edge Allocation multigraph benchmarks (ALGO-GN-031).
//!
//! Run: `cargo bench --bench bench_iea_game`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-GN-031.json`.
//!
//! Coverage: directed/loops, directed/no-loops, undirected/no-loops at
//! three edge-count scales. Throughput is reported in edges (the inner
//! cost is one — or two, for directed-with-loops — PRNG draws per edge
//! plus a single push into the staging buffer).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::iea_game;

const N: u32 = 1_000;

fn bench_iea_directed_loops(c: &mut Criterion) {
    let mut group = c.benchmark_group("iea_game/directed_loops");
    for m in [1_000u64, 10_000, 100_000] {
        group.throughput(Throughput::Elements(m));
        group.bench_with_input(BenchmarkId::from_parameter(m), &m, |b, &m| {
            b.iter(|| iea_game(N, m, true, true, 0x1EA_D031).unwrap());
        });
    }
    group.finish();
}

fn bench_iea_directed_no_loops(c: &mut Criterion) {
    let mut group = c.benchmark_group("iea_game/directed_no_loops");
    for m in [1_000u64, 10_000, 100_000] {
        group.throughput(Throughput::Elements(m));
        group.bench_with_input(BenchmarkId::from_parameter(m), &m, |b, &m| {
            b.iter(|| iea_game(N, m, true, false, 0x1EA_C031).unwrap());
        });
    }
    group.finish();
}

fn bench_iea_undirected_no_loops(c: &mut Criterion) {
    let mut group = c.benchmark_group("iea_game/undirected_no_loops");
    for m in [1_000u64, 10_000, 100_000] {
        group.throughput(Throughput::Elements(m));
        group.bench_with_input(BenchmarkId::from_parameter(m), &m, |b, &m| {
            b.iter(|| iea_game(N, m, false, false, 0x1EA_E031).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_iea_directed_loops,
    bench_iea_directed_no_loops,
    bench_iea_undirected_no_loops
);
criterion_main!(benches);
