//! Chordality benchmark (MCS + `is_chordal`).
//!
//! Run: `cargo bench --bench bench_chordality`.
//! Results land under `target/criterion/`. Numbers are committed to
//! `.codefuse/tracking/perf/ALGO-CL-002.json`.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{erdos_renyi_gnp, is_chordal, maximum_cardinality_search};

fn sparse_er(n: u32) -> rust_igraph::Graph {
    let p = 4.0 / f64::from(n.saturating_sub(1).max(1));
    erdos_renyi_gnp(n, p, false, false, 0x0C02_0001).expect("ER sparse")
}

fn dense_er(n: u32) -> rust_igraph::Graph {
    let p = 0.3;
    erdos_renyi_gnp(n, p, false, false, 0x0C02_0002).expect("ER dense")
}

fn bench_mcs_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("chordality_mcs/sparse");
    for n in [100u32, 1_000, 5_000] {
        let g = sparse_er(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| maximum_cardinality_search(g).unwrap());
        });
    }
    group.finish();
}

fn bench_is_chordal_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("chordality_is_chordal/sparse");
    for n in [100u32, 1_000, 5_000] {
        let g = sparse_er(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| is_chordal(g, None).unwrap());
        });
    }
    group.finish();
}

fn bench_is_chordal_dense(c: &mut Criterion) {
    let mut group = c.benchmark_group("chordality_is_chordal/dense");
    for n in [100u32, 500, 1_000] {
        let g = dense_er(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| is_chordal(g, None).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_mcs_sparse,
    bench_is_chordal_sparse,
    bench_is_chordal_dense
);
criterion_main!(benches);
