//! Symmetric tree deterministic constructor benchmarks (ALGO-CN-005).
//!
//! Run: `cargo bench --bench bench_symmetric_tree`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-005.json`.
//!
//! Coverage: all three `TreeMode` variants (Out / In / Undirected) across
//! three representative branching configurations:
//!
//! * `[3, 3, 3]`        —  40 vertices (depth 3, small).
//! * `[4, 4, 4, 4]`     — 341 vertices (depth 4, medium).
//! * `[5, 5, 5, 5, 5]`  — 3906 vertices (depth 5, large).
//!
//! Total work is `O(|V|)` per call and dominated by `Graph::add_edges`
//! reallocations, mirroring the `kary_tree` benchmark.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{TreeMode, symmetric_tree};

const BRANCH_CONFIGS: &[(&str, &[u32], u64)] = &[
    ("depth3_3x3x3", &[3u32, 3, 3], 40),
    ("depth4_4x4", &[4u32, 4, 4, 4], 341),
    ("depth5_5x5", &[5u32, 5, 5, 5, 5], 3906),
];

fn bench_symmetric_tree_out(c: &mut Criterion) {
    let mut group = c.benchmark_group("symmetric_tree/out");
    for (label, branches, n_elements) in BRANCH_CONFIGS {
        group.throughput(Throughput::Elements(*n_elements));
        group.bench_with_input(BenchmarkId::from_parameter(label), branches, |b, br| {
            b.iter(|| symmetric_tree(br, TreeMode::Out).unwrap());
        });
    }
    group.finish();
}

fn bench_symmetric_tree_in(c: &mut Criterion) {
    let mut group = c.benchmark_group("symmetric_tree/in");
    for (label, branches, n_elements) in BRANCH_CONFIGS {
        group.throughput(Throughput::Elements(*n_elements));
        group.bench_with_input(BenchmarkId::from_parameter(label), branches, |b, br| {
            b.iter(|| symmetric_tree(br, TreeMode::In).unwrap());
        });
    }
    group.finish();
}

fn bench_symmetric_tree_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("symmetric_tree/undirected");
    for (label, branches, n_elements) in BRANCH_CONFIGS {
        group.throughput(Throughput::Elements(*n_elements));
        group.bench_with_input(BenchmarkId::from_parameter(label), branches, |b, br| {
            b.iter(|| symmetric_tree(br, TreeMode::Undirected).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_symmetric_tree_out,
    bench_symmetric_tree_in,
    bench_symmetric_tree_undirected
);
criterion_main!(benches);
