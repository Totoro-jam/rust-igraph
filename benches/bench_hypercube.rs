//! Hypercube constructor benchmarks (ALGO-CN-007).
//!
//! Run: `cargo bench --bench bench_hypercube`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-007.json`.
//!
//! Coverage: undirected and directed variants across three representative
//! dimensions:
//!
//! * `n = 6`  —   64 vertices, 192 edges (tiny).
//! * `n = 10` — 1024 vertices, 5120 edges (medium).
//! * `n = 14` — 16384 vertices, 114688 edges (large).
//!
//! Total work is `O(|V| · n) = O(|E|)` per call: each vertex `v` toggles
//! every bit `i ∈ [0, n)` to emit the canonical edge `(v, v ^ (1 << i))`
//! when `v < (v ^ (1 << i))`. Throughput is therefore expressed in
//! edges; both variants do the same enumeration work.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::hypercube;

const CONFIGS: &[(&str, u32, u64)] = &[("n6", 6, 192), ("n10", 10, 5120), ("n14", 14, 114_688)];

fn bench_hypercube_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("hypercube/undirected");
    for (label, n, n_edges) in CONFIGS {
        group.throughput(Throughput::Elements(*n_edges));
        let n_v = *n;
        group.bench_with_input(BenchmarkId::from_parameter(label), &n_v, |b, &n| {
            b.iter(|| hypercube(n, false).unwrap());
        });
    }
    group.finish();
}

fn bench_hypercube_directed(c: &mut Criterion) {
    let mut group = c.benchmark_group("hypercube/directed");
    for (label, n, n_edges) in CONFIGS {
        group.throughput(Throughput::Elements(*n_edges));
        let n_v = *n;
        group.bench_with_input(BenchmarkId::from_parameter(label), &n_v, |b, &n| {
            b.iter(|| hypercube(n, true).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_hypercube_undirected,
    bench_hypercube_directed
);
criterion_main!(benches);
