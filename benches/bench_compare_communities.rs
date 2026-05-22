//! `compare_communities` (ALGO-CM-015) benchmark.
//!
//! Run: `cargo bench --bench bench_compare_communities`. Numbers land in
//! `.codefuse/tracking/perf/ALGO-CM-015.json`. The five methods share the
//! same confusion-matrix build, so the per-method cells essentially
//! measure the post-build arithmetic (entropy/log2 for VI/NMI, max-row /
//! max-col for `SplitJoin`, pair-counting for Rand/`AdjustedRand`).

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{CommunityComparison, compare_communities};

/// Two partitions of `n` vertices with `k` clusters each. `comm1` is a
/// pure stripe by `i % k`. `comm2` is a perturbed copy: every `step`-th
/// vertex gets shifted by 1 cluster (mod k). For small `step` the two
/// partitions diverge fast; for large `step` they are nearly identical.
fn striped_pair(n: usize, k: u32, step: usize) -> (Vec<u32>, Vec<u32>) {
    let comm1: Vec<u32> = (0..n)
        .map(|i| u32::try_from(i).unwrap_or(u32::MAX) % k)
        .collect();
    let comm2: Vec<u32> = (0..n)
        .map(|i| {
            let base = u32::try_from(i).unwrap_or(u32::MAX) % k;
            if step != 0 && i % step == 0 {
                (base + 1) % k
            } else {
                base
            }
        })
        .collect();
    (comm1, comm2)
}

const N_SMALL: usize = 256;
const K_SMALL: u32 = 8;
const N_MEDIUM: usize = 10_000;
const K_MEDIUM: u32 = 100;

fn run_method(c: &mut Criterion, label: &str, method: CommunityComparison) {
    let (a_small, b_small) = striped_pair(N_SMALL, K_SMALL, 7);
    c.bench_function(
        &format!("compare_communities/{label} small n={N_SMALL} k={K_SMALL}"),
        |b| b.iter(|| compare_communities(&a_small, &b_small, method).unwrap()),
    );

    let (a_med, b_med) = striped_pair(N_MEDIUM, K_MEDIUM, 13);
    c.bench_function(
        &format!("compare_communities/{label} medium n={N_MEDIUM} k={K_MEDIUM}"),
        |b| b.iter(|| compare_communities(&a_med, &b_med, method).unwrap()),
    );
}

fn bench_vi(c: &mut Criterion) {
    run_method(c, "vi", CommunityComparison::VariationOfInformation);
}

fn bench_nmi(c: &mut Criterion) {
    run_method(c, "nmi", CommunityComparison::NormalizedMutualInformation);
}

fn bench_split_join(c: &mut Criterion) {
    run_method(c, "split_join", CommunityComparison::SplitJoin);
}

fn bench_rand(c: &mut Criterion) {
    run_method(c, "rand", CommunityComparison::Rand);
}

fn bench_adjusted_rand(c: &mut Criterion) {
    run_method(c, "adjusted_rand", CommunityComparison::AdjustedRand);
}

criterion_group!(
    benches,
    bench_vi,
    bench_nmi,
    bench_split_join,
    bench_rand,
    bench_adjusted_rand,
);
criterion_main!(benches);
