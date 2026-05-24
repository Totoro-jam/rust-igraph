//! Regular tree (Bethe lattice) constructor benchmarks (ALGO-CN-006).
//!
//! Run: `cargo bench --bench bench_regular_tree`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-006.json`.
//!
//! Coverage: all three `TreeMode` variants (Out / In / Undirected) across
//! three representative `(h, k)` configurations:
//!
//! * `(h=3, k=3)`  —   22 vertices (small Bethe lattice).
//! * `(h=4, k=4)`  —  365 vertices (medium).
//! * `(h=5, k=5)`  — 6831 vertices (large).
//!
//! Total work is `O(|V|)` per call. Since `regular_tree` is a thin
//! wrapper that builds a length-`h` branching vector and delegates to
//! `symmetric_tree`, throughput tracks closely with the `symmetric_tree`
//! benchmark.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{TreeMode, regular_tree};

const CONFIGS: &[(&str, u32, u32, u64)] = &[
    ("h3_k3", 3, 3, 22),
    ("h4_k4", 4, 4, 365),
    ("h5_k5", 5, 5, 6831),
];

fn bench_regular_tree_out(c: &mut Criterion) {
    let mut group = c.benchmark_group("regular_tree/out");
    for (label, h, k, n_elements) in CONFIGS {
        group.throughput(Throughput::Elements(*n_elements));
        let h_v = *h;
        let k_v = *k;
        group.bench_with_input(
            BenchmarkId::from_parameter(label),
            &(h_v, k_v),
            |b, &(h, k)| {
                b.iter(|| regular_tree(h, k, TreeMode::Out).unwrap());
            },
        );
    }
    group.finish();
}

fn bench_regular_tree_in(c: &mut Criterion) {
    let mut group = c.benchmark_group("regular_tree/in");
    for (label, h, k, n_elements) in CONFIGS {
        group.throughput(Throughput::Elements(*n_elements));
        let h_v = *h;
        let k_v = *k;
        group.bench_with_input(
            BenchmarkId::from_parameter(label),
            &(h_v, k_v),
            |b, &(h, k)| {
                b.iter(|| regular_tree(h, k, TreeMode::In).unwrap());
            },
        );
    }
    group.finish();
}

fn bench_regular_tree_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("regular_tree/undirected");
    for (label, h, k, n_elements) in CONFIGS {
        group.throughput(Throughput::Elements(*n_elements));
        let h_v = *h;
        let k_v = *k;
        group.bench_with_input(
            BenchmarkId::from_parameter(label),
            &(h_v, k_v),
            |b, &(h, k)| {
                b.iter(|| regular_tree(h, k, TreeMode::Undirected).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_regular_tree_out,
    bench_regular_tree_in,
    bench_regular_tree_undirected
);
criterion_main!(benches);
