//! Hamming constructor benchmarks (ALGO-CN-008).
//!
//! Run: `cargo bench --bench bench_hamming`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-008.json`.
//!
//! Coverage: undirected and directed variants across three representative
//! `(n, q)` shapes:
//!
//! * `(n=3, q=3)` —    27 vertices,    81 edges (small).
//! * `(n=4, q=4)` —   256 vertices,  1536 edges (medium).
//! * `(n=5, q=4)` —  1024 vertices,  7680 edges (large).
//!
//! Work per call is `O(n · q^(n+1))` — for each of the `q^n` vertices we
//! iterate `n` digit positions and emit up to `q − 1` outgoing edges per
//! digit. Throughput is reported in edges (`q^n · (q−1) · n / 2`).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::hamming;

const CONFIGS: &[(&str, u32, u32, u64)] = &[
    ("n3_q3", 3, 3, 81),
    ("n4_q4", 4, 4, 1536),
    ("n5_q4", 5, 4, 7680),
];

fn bench_hamming_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("hamming/undirected");
    for (label, n, q, n_edges) in CONFIGS {
        group.throughput(Throughput::Elements(*n_edges));
        let nv = *n;
        let qv = *q;
        group.bench_with_input(
            BenchmarkId::from_parameter(label),
            &(nv, qv),
            |b, &(n, q)| {
                b.iter(|| hamming(n, q, false).unwrap());
            },
        );
    }
    group.finish();
}

fn bench_hamming_directed(c: &mut Criterion) {
    let mut group = c.benchmark_group("hamming/directed");
    for (label, n, q, n_edges) in CONFIGS {
        group.throughput(Throughput::Elements(*n_edges));
        let nv = *n;
        let qv = *q;
        group.bench_with_input(
            BenchmarkId::from_parameter(label),
            &(nv, qv),
            |b, &(n, q)| {
                b.iter(|| hamming(n, q, true).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_hamming_undirected, bench_hamming_directed);
criterion_main!(benches);
