//! `community_to_membership` (ALGO-CM-013) benchmark.
//!
//! Run: `cargo bench --bench bench_community_to_membership`. Numbers
//! land in `.codefuse/tracking/perf/ALGO-CM-013.json`. The helper is a
//! pure-function dendrogram cut, so cells exercise it across a range
//! of dendrogram sizes and cut depths to map out scaling.

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::community_to_membership;

/// Build a left-leaning chain dendrogram on `n` leaves:
/// `(0,1)->n, (n,2)->n+1, (n+1,3)->n+2, ...` — `n-1` merges total.
fn chain_dendrogram(n: u32) -> Vec<[u32; 2]> {
    let mut merges = Vec::with_capacity((n as usize).saturating_sub(1));
    if n < 2 {
        return merges;
    }
    merges.push([0, 1]);
    let mut left: u32 = n;
    for next_leaf in 2..n {
        merges.push([left, next_leaf]);
        left += 1;
    }
    merges
}

/// Build a balanced binary dendrogram on `n` leaves (power-of-two
/// expected). Each round pairs adjacent live nodes.
fn balanced_dendrogram(n: u32) -> Vec<[u32; 2]> {
    let mut merges = Vec::with_capacity((n as usize).saturating_sub(1));
    let mut live: Vec<u32> = (0..n).collect();
    let mut next_id = n;
    while live.len() >= 2 {
        let mut new_live = Vec::with_capacity(live.len() / 2 + 1);
        let mut i = 0;
        while i + 1 < live.len() {
            merges.push([live[i], live[i + 1]]);
            new_live.push(next_id);
            next_id += 1;
            i += 2;
        }
        if i < live.len() {
            new_live.push(live[i]);
        }
        live = new_live;
    }
    merges
}

fn bench_chain_64_full(c: &mut Criterion) {
    let merges = chain_dendrogram(64);
    let steps = u32::try_from(merges.len()).expect("steps fits u32");
    c.bench_function("community_to_membership/chain 64 full collapse", |b| {
        b.iter(|| community_to_membership(&merges, 64, steps).unwrap());
    });
}

fn bench_balanced_256_full(c: &mut Criterion) {
    let merges = balanced_dendrogram(256);
    let steps = u32::try_from(merges.len()).expect("steps fits u32");
    c.bench_function("community_to_membership/balanced 256 full collapse", |b| {
        b.iter(|| community_to_membership(&merges, 256, steps).unwrap());
    });
}

fn bench_balanced_1024_full(c: &mut Criterion) {
    let merges = balanced_dendrogram(1024);
    let steps = u32::try_from(merges.len()).expect("steps fits u32");
    c.bench_function("community_to_membership/balanced 1024 full collapse", |b| {
        b.iter(|| community_to_membership(&merges, 1024, steps).unwrap());
    });
}

fn bench_balanced_1024_half(c: &mut Criterion) {
    let merges = balanced_dendrogram(1024);
    let steps = u32::try_from(merges.len() / 2).expect("steps fits u32");
    c.bench_function("community_to_membership/balanced 1024 half cut", |b| {
        b.iter(|| community_to_membership(&merges, 1024, steps).unwrap());
    });
}

fn bench_balanced_1024_zero(c: &mut Criterion) {
    let merges = balanced_dendrogram(1024);
    c.bench_function("community_to_membership/balanced 1024 zero steps", |b| {
        b.iter(|| community_to_membership(&merges, 1024, 0).unwrap());
    });
}

criterion_group!(
    benches,
    bench_chain_64_full,
    bench_balanced_256_full,
    bench_balanced_1024_full,
    bench_balanced_1024_half,
    bench_balanced_1024_zero,
);
criterion_main!(benches);
