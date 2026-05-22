//! `reindex_membership` (ALGO-CM-014) benchmark.
//!
//! Run: `cargo bench --bench bench_reindex_membership`. Numbers land in
//! `.codefuse/tracking/perf/ALGO-CM-014.json`. The helper is a pure
//! single-pass densification, so cells cover (a) the fast `Vec`-lookup
//! branch (max id < n) and (b) the sparse `BTreeMap` fallback (max id
//! >> n), at a couple of input sizes.

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::reindex_membership;

/// Already-dense membership on `n` vertices with `k` clusters,
/// striped: vertex `i` -> cluster `i % k`. Stays in the fast path.
fn striped_dense(n: usize, k: u32) -> Vec<u32> {
    (0..n)
        .map(|i| u32::try_from(i).unwrap_or(u32::MAX) % k)
        .collect()
}

/// First-occurrence-shuffled membership on `n` vertices and `k`
/// clusters, with ids in `0..k` but in an order that makes the
/// densifier do real work. Permutes ids by `(id * 0x9E37) % k`.
fn shuffled_dense(n: usize, k: u32) -> Vec<u32> {
    (0..n)
        .map(|i| u32::try_from(i).unwrap_or(u32::MAX).wrapping_mul(0x9E37) % k)
        .collect()
}

/// Sparse-id membership: ids drawn from a much larger space than `n`
/// — exercises the `BTreeMap` fallback. `k` distinct ids cycled.
fn sparse_ids(n: usize, k: u32) -> Vec<u32> {
    let ids: Vec<u32> = (0..k).map(|i| 1_000_000 + i * 17).collect();
    (0..n).map(|i| ids[i % ids.len()]).collect()
}

fn bench_fast_path_256_8(c: &mut Criterion) {
    let m = striped_dense(256, 8);
    c.bench_function("reindex_membership/fast 256 vertices 8 clusters", |b| {
        b.iter(|| reindex_membership(&m).unwrap());
    });
}

fn bench_fast_path_1024_32(c: &mut Criterion) {
    let m = shuffled_dense(1024, 32);
    c.bench_function("reindex_membership/fast 1024 vertices 32 clusters", |b| {
        b.iter(|| reindex_membership(&m).unwrap());
    });
}

fn bench_fast_path_10000_100(c: &mut Criterion) {
    let m = shuffled_dense(10_000, 100);
    c.bench_function("reindex_membership/fast 10000 vertices 100 clusters", |b| {
        b.iter(|| reindex_membership(&m).unwrap());
    });
}

fn bench_sparse_1024_32(c: &mut Criterion) {
    let m = sparse_ids(1024, 32);
    c.bench_function(
        "reindex_membership/sparse 1024 vertices 32 large-ids",
        |b| {
            b.iter(|| reindex_membership(&m).unwrap());
        },
    );
}

fn bench_sparse_10000_100(c: &mut Criterion) {
    let m = sparse_ids(10_000, 100);
    c.bench_function(
        "reindex_membership/sparse 10000 vertices 100 large-ids",
        |b| {
            b.iter(|| reindex_membership(&m).unwrap());
        },
    );
}

criterion_group!(
    benches,
    bench_fast_path_256_8,
    bench_fast_path_1024_32,
    bench_fast_path_10000_100,
    bench_sparse_1024_32,
    bench_sparse_10000_100,
);
criterion_main!(benches);
