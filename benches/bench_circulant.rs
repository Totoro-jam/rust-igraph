//! Circulant graph constructor benchmarks (ALGO-CN-011).
//!
//! Run: `cargo bench --bench bench_circulant`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-011.json`.
//!
//! Coverage: representative `circulant(n, shifts)` shapes — a basic
//! cycle (shift = 1), squared cycle (shifts = [1, 2]), an even-`n`
//! antipodal shift (shift = n/2), the complete graph specialisation
//! (shifts = 1..=n/2), and two larger sizes to amortise per-call
//! overhead.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::circulant;

struct Shape {
    label: &'static str,
    n: u32,
    shifts: &'static [i64],
    directed: bool,
    /// Total edges emitted (after the various dedup / halving rules)
    /// — used for throughput reporting.
    edge_work: u64,
}

const SHAPES: &[Shape] = &[
    Shape {
        label: "c_5_shift_1",
        n: 5,
        shifts: &[1],
        directed: false,
        edge_work: 5,
    },
    Shape {
        label: "c_8_shift_4_antipodal",
        n: 8,
        shifts: &[4],
        directed: false,
        edge_work: 4, // n/2 with even n undirected: limit halved
    },
    Shape {
        label: "squared_c_10",
        n: 10,
        shifts: &[1, 2],
        directed: false,
        edge_work: 20,
    },
    Shape {
        label: "k_10_full_shifts",
        n: 10,
        shifts: &[1, 2, 3, 4, 5],
        directed: false,
        edge_work: 45,
    },
    Shape {
        label: "n_1000_shift_7",
        n: 1_000,
        shifts: &[7],
        directed: false,
        edge_work: 1_000,
    },
    Shape {
        label: "n_10000_shifts_1_5_17_101",
        n: 10_000,
        shifts: &[1, 5, 17, 101],
        directed: false,
        edge_work: 40_000,
    },
];

fn bench_circulant(c: &mut Criterion) {
    let mut group = c.benchmark_group("circulant");
    for shape in SHAPES {
        group.throughput(Throughput::Elements(shape.edge_work));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &shape,
            |b, shape| {
                b.iter(|| circulant(shape.n, shape.shifts, shape.directed).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_circulant);
criterion_main!(benches);
