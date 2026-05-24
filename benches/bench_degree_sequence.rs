//! Configuration-model degree-sequence generator benchmarks
//! (ALGO-GN-024).
//!
//! Run: `cargo bench --bench bench_degree_sequence`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-GN-024.json`.
//!
//! Coverage targets the two cost drivers in
//! `degree_sequence_game_configuration`:
//!   * `size_sweep_uniform_d4_undirected` — uniform degree `d = 4` at
//!     vertex counts `n ∈ {1_000, 10_000, 100_000}`. Stub-bag construction
//!     is `Θ(n + Σd_i)` and pairing is `Θ(|E|) = Θ(n·d/2)`, so wall
//!     clock scales linearly in `n` at fixed `d`. Per-edge throughput is
//!     a tight proxy for the inner `swap_remove` loop's overhead.
//!   * `directed_vs_undirected_n10000_d4` — paired runs at `n = 10_000,
//!     d = 4`. The directed branch maintains two stub bags and draws two
//!     PRNG indices per edge; undirected uses a single bag and two draws
//!     per edge. The ratio reveals the relative cost of bag-management
//!     vs. PRNG draws.

#![allow(clippy::cast_precision_loss)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::degree_sequence_game_configuration;

fn bench_size_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("degree_sequence/size_sweep_uniform_d4_undirected");
    let degree: u32 = 4;
    for n in [1_000usize, 10_000, 100_000] {
        let out_degrees: Vec<u32> = vec![degree; n];
        group.throughput(Throughput::Elements(u64::try_from(n).expect("n fits u64")));
        group.bench_with_input(BenchmarkId::from_parameter(n), &out_degrees, |b, seq| {
            b.iter(|| degree_sequence_game_configuration(seq, None, 0x00DE_5EE9_u64).unwrap());
        });
    }
    group.finish();
}

fn bench_directed_vs_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("degree_sequence/directed_vs_undirected_n10000_d4");
    let n: usize = 10_000;
    let degree: u32 = 4;
    let out_degrees: Vec<u32> = vec![degree; n];
    let in_degrees: Vec<u32> = vec![degree; n];
    group.throughput(Throughput::Elements(u64::try_from(n).expect("n fits u64")));
    group.bench_function("undirected", |b| {
        b.iter(|| degree_sequence_game_configuration(&out_degrees, None, 0xC0FE_BABE_u64).unwrap());
    });
    group.bench_function("directed", |b| {
        b.iter(|| {
            degree_sequence_game_configuration(&out_degrees, Some(&in_degrees), 0xC0FE_BABE_u64)
                .unwrap()
        });
    });
    group.finish();
}

criterion_group!(benches, bench_size_sweep, bench_directed_vs_undirected,);
criterion_main!(benches);
