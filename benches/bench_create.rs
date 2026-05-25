//! Basic edge-list constructor benchmarks (ALGO-CN-022).
//!
//! Run: `cargo bench --bench bench_create`.
//! A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-CN-022.json`.
//!
//! Coverage:
//! - `create/path_n`: cold builds of an `n`-vertex path, where `n`
//!   sweeps `100..100_000`. Probes the linear-in-|E| add-edges loop
//!   and the `max(edges)+1` scan when `n=0`.
//! - `create/star_n`: cold builds of an `n`-vertex star (one hub).
//!   Same complexity as the path, but the degree distribution is
//!   degenerate (one node holds all the incidence), which exercises
//!   the per-vertex incidence-vector growth path inside `Graph::new`.
//! - `create/dense_n`: cold builds of a complete graph (|E| = n(n-1)/2).
//!   The `n` here is the worst-case quadratic-edges branch.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::create;

fn path_edges(n: u32) -> Vec<(u32, u32)> {
    (1..n).map(|i| (i - 1, i)).collect()
}

fn star_edges(n: u32) -> Vec<(u32, u32)> {
    (1..n).map(|i| (0, i)).collect()
}

fn complete_edges(n: u32) -> Vec<(u32, u32)> {
    let mut e = Vec::with_capacity((n as usize * (n as usize - 1)) / 2);
    for i in 0..n {
        for j in (i + 1)..n {
            e.push((i, j));
        }
    }
    e
}

fn bench_create_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("create/path_n");
    for &n in &[100u32, 1_000, 10_000, 100_000] {
        let edges = path_edges(n);
        group.throughput(Throughput::Elements(edges.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &edges, |b, edges| {
            b.iter(|| create(edges, n, false).unwrap());
        });
    }
    group.finish();
}

fn bench_create_star(c: &mut Criterion) {
    let mut group = c.benchmark_group("create/star_n");
    for &n in &[100u32, 1_000, 10_000, 100_000] {
        let edges = star_edges(n);
        group.throughput(Throughput::Elements(edges.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &edges, |b, edges| {
            b.iter(|| create(edges, n, false).unwrap());
        });
    }
    group.finish();
}

fn bench_create_dense(c: &mut Criterion) {
    let mut group = c.benchmark_group("create/dense_n");
    for &n in &[50u32, 100, 200, 400] {
        let edges = complete_edges(n);
        group.throughput(Throughput::Elements(edges.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &edges, |b, edges| {
            b.iter(|| create(edges, n, false).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_create_path,
    bench_create_star,
    bench_create_dense
);
criterion_main!(benches);
