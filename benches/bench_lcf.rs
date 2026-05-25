//! LCF constructor benchmarks (ALGO-CN-018).
//!
//! Run: `cargo bench --bench bench_lcf`. Results land under
//! `target/criterion/`; a snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-CN-018.json`.
//!
//! The constructor is `O(n + repeats * shifts.len())`. The interesting
//! axes are therefore (1) the vertex count `n` and (2) the chord-pass
//! length. We bench three regimes:
//!
//! * **`famous`** — fixed-size cubic graphs from the standard catalogue
//!   (Franklin, Heawood, Frucht, Truncated tetrahedron, Truncated
//!   octahedron). Tiny inputs, useful as a smoke baseline.
//! * **`cycle_only`** — `lcf(n, &[], 0)` over a sweep of `n` values.
//!   Measures the Hamilton-cycle backbone in isolation (no chord pass).
//! * **`chord_heavy`** — `lcf(n, &[3, -5, 7, -11, 13], n / 5)` so the
//!   chord pass dominates the runtime (~n chords) and exercises the
//!   `BTreeSet` simplify path on large fixtures.
//!
//! Throughput is reported in `n` elements per second so "graphs built per
//! second" comparisons stay clean.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::lcf;

const FAMOUS: &[(&str, u32, &[i64], u32)] = &[
    ("franklin", 12, &[5, -5], 6),
    ("heawood", 14, &[5, -5], 7),
    ("frucht", 12, &[-5, -2, -4, 2, 5, -2, 2, 5, -2, -5, 4, 2], 1),
    ("truncated_tetrahedron", 12, &[2, 6, -2, -6], 3),
    ("truncated_octahedron", 24, &[3, -7, 7, -3], 6),
];

fn bench_famous(c: &mut Criterion) {
    let mut group = c.benchmark_group("lcf/famous");
    for (label, n, shifts, repeats) in FAMOUS {
        group.throughput(Throughput::Elements(u64::from(*n)));
        group.bench_function(BenchmarkId::from_parameter(label), |b| {
            b.iter(|| lcf(*n, shifts, *repeats).unwrap());
        });
    }
    group.finish();
}

fn bench_cycle_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("lcf/cycle_only");
    for n in [64u32, 1_024, 16_384, 131_072] {
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| lcf(n, &[], 0).unwrap());
        });
    }
    group.finish();
}

fn bench_chord_heavy(c: &mut Criterion) {
    let shifts: &[i64] = &[3, -5, 7, -11, 13];
    let mut group = c.benchmark_group("lcf/chord_heavy");
    for n in [64u32, 1_024, 16_384, 131_072] {
        let repeats = (n / u32::try_from(shifts.len()).unwrap()).max(1);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(
            BenchmarkId::from_parameter(n),
            &(n, repeats),
            |b, &(n, r)| {
                b.iter(|| lcf(n, shifts, r).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_famous, bench_cycle_only, bench_chord_heavy);
criterion_main!(benches);
