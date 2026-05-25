//! Prüfer-decoder benchmarks (ALGO-CN-016).
//!
//! Run: `cargo bench --bench bench_prufer`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-CN-016.json`.
//!
//! `igraph_from_prufer` is `O(n)` so the interesting axis is just the
//! sequence length. We sweep three regimes and three topologies to make
//! sure no path through the inner loop is pathological:
//!
//! * **`star`** — `prufer[..] == [0; n-2]`. Every leaf's parent is the
//!   same vertex 0; the inner while-loop fires once per `i` and never
//!   cascades.
//! * **`path`** — `prufer[i] = i + 1`. The decoder cascades all the way
//!   through the chain at `i = 0` (worst case for the inner walk).
//! * **`random`** — fixed-seed deterministic pseudo-random vertices in
//!   `[0, n)`; represents a typical mixed-degree tree.
//!
//! Throughput is reported in `n` elements per second (so cross-shape
//! numbers are directly comparable as "trees decoded per second" in
//! vertex-count terms).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::from_prufer;

fn make_star(n: u32) -> Vec<u32> {
    vec![0u32; (n - 2) as usize]
}

fn make_path(n: u32) -> Vec<u32> {
    (1..(n - 1)).collect()
}

fn make_random(n: u32) -> Vec<u32> {
    // Small linear-congruential generator: deterministic, no extra dep.
    let mut state: u64 = 0xDEAD_BEEF_CAFE_F00D;
    let len = (n - 2) as usize;
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        // SplitMix64-ish step (constants from the original SplitMix64
        // paper; reused throughout the bench corpus for reproducibility).
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        // Modulo keeps z in [0, n) so the truncation back to u32 is
        // exact; we still gate it on `try_from` to satisfy clippy.
        let r = z % u64::from(n);
        out.push(u32::try_from(r).expect("modulo result fits u32"));
    }
    out
}

fn bench_star(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_prufer/star");
    for n in [64u32, 1_024, 16_384, 131_072] {
        let seq = make_star(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &seq, |b, seq| {
            b.iter(|| from_prufer(seq).unwrap());
        });
    }
    group.finish();
}

fn bench_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_prufer/path");
    for n in [64u32, 1_024, 16_384, 131_072] {
        let seq = make_path(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &seq, |b, seq| {
            b.iter(|| from_prufer(seq).unwrap());
        });
    }
    group.finish();
}

fn bench_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_prufer/random");
    for n in [64u32, 1_024, 16_384, 131_072] {
        let seq = make_random(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &seq, |b, seq| {
            b.iter(|| from_prufer(seq).unwrap());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_star, bench_path, bench_random);
criterion_main!(benches);
