//! De Bruijn graph constructor benchmarks (ALGO-CN-012).
//!
//! Run: `cargo bench --bench bench_de_bruijn`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-012.json`.
//!
//! Coverage: representative `B(m, n)` shapes — the tiny canonical
//! cases used in tests, two medium binary alphabets that exercise the
//! arc emission loop, and three larger shapes (high-`m` short-`n` vs.
//! low-`m` long-`n`) so the throughput plot covers both axes that
//! drive `m^(n+1)`.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::de_bruijn;

struct Shape {
    label: &'static str,
    m: u32,
    n: u32,
    /// Total arcs emitted (= m^(n+1)) — used for throughput reporting.
    edge_work: u64,
}

const SHAPES: &[Shape] = &[
    Shape {
        label: "b_2_2",
        m: 2,
        n: 2,
        edge_work: 8,
    },
    Shape {
        label: "b_3_2",
        m: 3,
        n: 2,
        edge_work: 27,
    },
    Shape {
        label: "b_2_8",
        m: 2,
        n: 8,
        edge_work: 512,
    },
    Shape {
        label: "b_4_4",
        m: 4,
        n: 4,
        edge_work: 1_024,
    },
    Shape {
        label: "b_2_16",
        m: 2,
        n: 16,
        edge_work: 131_072,
    },
    Shape {
        label: "b_5_6",
        m: 5,
        n: 6,
        edge_work: 78_125,
    },
];

fn bench_de_bruijn(c: &mut Criterion) {
    let mut group = c.benchmark_group("de_bruijn");
    for shape in SHAPES {
        group.throughput(Throughput::Elements(shape.edge_work));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| de_bruijn(shape.m, shape.n).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_de_bruijn);
criterion_main!(benches);
