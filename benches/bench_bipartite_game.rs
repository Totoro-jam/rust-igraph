//! Bipartite random-graph generator benchmarks (ALGO-GN-030).
//!
//! Run: `cargo bench --bench bench_bipartite_game`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-GN-030.json`.
//!
//! Coverage:
//!   * `bipartite_game_gnp/undirected`: Batagelj–Brandes geometric skip
//!     across three (n1, n2) sizes with `p = 0.05` (sparse) and
//!     `mode = All`.
//!   * `bipartite_game_gnp/directed_all`: same sweep but directed with
//!     `mode = All` (both ordered cross-pairs sampled independently).
//!   * `bipartite_game_gnm/undirected`: Floyd distinct-sample for an
//!     edge count fixed at 10·max(n1,n2) edges so the sampler stays in
//!     the sparse regime where the algorithm is intended.
//!   * `bipartite_game_gnm/directed_out`: directed sampler with
//!     `mode = Out` so the pair space is exactly n1·n2.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{BipartiteMode, bipartite_game_gnm, bipartite_game_gnp};

const SIZES: &[(u32, u32)] = &[(100, 100), (1_000, 1_000), (5_000, 5_000)];
const SPARSE_P: f64 = 0.05;

fn bench_gnp_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("bipartite_game_gnp/undirected");
    for &(n1, n2) in SIZES {
        let label = format!("{n1}x{n2}");
        group.throughput(Throughput::Elements(u64::from(n1) * u64::from(n2)));
        group.bench_with_input(
            BenchmarkId::from_parameter(&label),
            &(n1, n2),
            |b, &(n1, n2)| {
                b.iter(|| {
                    bipartite_game_gnp(n1, n2, SPARSE_P, false, BipartiteMode::All, 0xB1_AE_F0_03)
                        .unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_gnp_directed_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("bipartite_game_gnp/directed_all");
    for &(n1, n2) in SIZES {
        let label = format!("{n1}x{n2}");
        group.throughput(Throughput::Elements(2 * u64::from(n1) * u64::from(n2)));
        group.bench_with_input(
            BenchmarkId::from_parameter(&label),
            &(n1, n2),
            |b, &(n1, n2)| {
                b.iter(|| {
                    bipartite_game_gnp(n1, n2, SPARSE_P, true, BipartiteMode::All, 0xB1_AE_F0_05)
                        .unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_gnm_undirected(c: &mut Criterion) {
    let mut group = c.benchmark_group("bipartite_game_gnm/undirected");
    for &(n1, n2) in SIZES {
        let label = format!("{n1}x{n2}");
        let m = u64::from(n1.max(n2)) * 10;
        group.throughput(Throughput::Elements(m));
        group.bench_with_input(
            BenchmarkId::from_parameter(&label),
            &(n1, n2, m),
            |b, &(n1, n2, m)| {
                b.iter(|| {
                    bipartite_game_gnm(n1, n2, m, false, BipartiteMode::All, 0xB1_AE_F0_07).unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_gnm_directed_out(c: &mut Criterion) {
    let mut group = c.benchmark_group("bipartite_game_gnm/directed_out");
    for &(n1, n2) in SIZES {
        let label = format!("{n1}x{n2}");
        let m = u64::from(n1.max(n2)) * 10;
        group.throughput(Throughput::Elements(m));
        group.bench_with_input(
            BenchmarkId::from_parameter(&label),
            &(n1, n2, m),
            |b, &(n1, n2, m)| {
                b.iter(|| {
                    bipartite_game_gnm(n1, n2, m, true, BipartiteMode::Out, 0xB1_AE_F0_09).unwrap()
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_gnp_undirected,
    bench_gnp_directed_all,
    bench_gnm_undirected,
    bench_gnm_directed_out
);
criterion_main!(benches);
