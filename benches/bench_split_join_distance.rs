//! `split_join_distance` (ALGO-CM-016) benchmark.
//!
//! Run: `cargo bench --bench bench_split_join_distance`. Numbers land in
//! `.codefuse/tracking/perf/ALGO-CM-016.json`. The asymmetric pair shares
//! the same confusion-matrix build as `compare_communities(_,_,SplitJoin)`
//! — this bench measures `(d12, d21)` extraction in isolation so we can
//! compare directly against R-igraph's `split_join_distance()` baseline.

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::split_join_distance;

/// Two partitions of `n` vertices with `k` clusters each. `comm1` is a
/// pure stripe by `i % k`. `comm2` is a perturbed copy: every `step`-th
/// vertex gets shifted by 1 cluster (mod k).
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

fn bench_small(c: &mut Criterion) {
    let (a, b) = striped_pair(N_SMALL, K_SMALL, 7);
    c.bench_function(
        &format!("split_join_distance/small n={N_SMALL} k={K_SMALL}"),
        |bench| bench.iter(|| split_join_distance(&a, &b).unwrap()),
    );
}

fn bench_medium(c: &mut Criterion) {
    let (a, b) = striped_pair(N_MEDIUM, K_MEDIUM, 13);
    c.bench_function(
        &format!("split_join_distance/medium n={N_MEDIUM} k={K_MEDIUM}"),
        |bench| bench.iter(|| split_join_distance(&a, &b).unwrap()),
    );
}

fn bench_subpartition(c: &mut Criterion) {
    // Coarsening pair: b = a / 2. Every a-cluster fits in one b-cluster,
    // so d12 = 0 — exercises the "one side dominates" path through the
    // row/col max-sum accumulator.
    let a: Vec<u32> = (0..N_MEDIUM)
        .map(|i| u32::try_from(i).unwrap_or(u32::MAX) % K_MEDIUM)
        .collect();
    let b: Vec<u32> = a.iter().map(|&c| c / 2).collect();
    c.bench_function(
        &format!("split_join_distance/subpartition n={N_MEDIUM} k={K_MEDIUM}"),
        |bench| bench.iter(|| split_join_distance(&a, &b).unwrap()),
    );
}

criterion_group!(benches, bench_small, bench_medium, bench_subpartition);
criterion_main!(benches);
