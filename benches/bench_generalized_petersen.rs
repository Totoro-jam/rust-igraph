//! Generalized Petersen graph constructor benchmarks (ALGO-CN-010).
//!
//! Run: `cargo bench --bench bench_generalized_petersen`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-010.json`.
//!
//! Coverage: a small spread of G(n, k) shapes, from the eponymous
//! Petersen graph up through n = 10 000. Work per call is exactly
//! `3n` edges; throughput is reported in edges generated.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::generalized_petersen;

struct Shape {
    label: &'static str,
    n: u32,
    k: u32,
}

const SHAPES: &[Shape] = &[
    Shape {
        label: "petersen_5_2",
        n: 5,
        k: 2,
    },
    Shape {
        label: "mobius_kantor_8_3",
        n: 8,
        k: 3,
    },
    Shape {
        label: "desargues_10_3",
        n: 10,
        k: 3,
    },
    Shape {
        label: "nauru_12_5",
        n: 12,
        k: 5,
    },
    Shape {
        label: "gpg_n1000_k7",
        n: 1_000,
        k: 7,
    },
    Shape {
        label: "gpg_n10000_k101",
        n: 10_000,
        k: 101,
    },
];

fn bench_generalized_petersen(c: &mut Criterion) {
    let mut group = c.benchmark_group("generalized_petersen");
    for shape in SHAPES {
        group.throughput(Throughput::Elements(u64::from(shape.n) * 3));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| generalized_petersen(shape.n, shape.k).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_generalized_petersen);
criterion_main!(benches);
