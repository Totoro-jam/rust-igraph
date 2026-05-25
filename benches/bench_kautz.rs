//! Kautz graph constructor benchmarks (ALGO-CN-013).
//!
//! Run: `cargo bench --bench bench_kautz`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-013.json`.
//!
//! Coverage: representative `K(m, n)` shapes — the tiny canonical
//! cases used in tests, two medium shapes that exercise the
//! index-table + cursor walk, and three larger shapes (high-`m`
//! short-`n` vs. low-`m` long-`n`) so the throughput plot covers both
//! axes that drive the arc count `m·(m+1)·m^n`.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::kautz;

struct Shape {
    label: &'static str,
    m: u32,
    n: u32,
    /// Total arcs emitted (= m · (m+1) · m^n) — used for throughput reporting.
    edge_work: u64,
}

const SHAPES: &[Shape] = &[
    Shape {
        label: "k_2_1",
        m: 2,
        n: 1,
        edge_work: 12,
    },
    Shape {
        label: "k_3_2",
        m: 3,
        n: 2,
        edge_work: 108,
    },
    Shape {
        label: "k_2_8",
        m: 2,
        n: 8,
        edge_work: 1_536,
    },
    Shape {
        label: "k_4_4",
        m: 4,
        n: 4,
        edge_work: 5_120,
    },
    Shape {
        label: "k_2_14",
        m: 2,
        n: 14,
        edge_work: 98_304,
    },
    Shape {
        label: "k_5_5",
        m: 5,
        n: 5,
        edge_work: 93_750,
    },
];

fn bench_kautz(c: &mut Criterion) {
    let mut group = c.benchmark_group("kautz");
    for shape in SHAPES {
        group.throughput(Throughput::Elements(shape.edge_work));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| kautz(shape.m, shape.n).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_kautz);
criterion_main!(benches);
