//! VF2 graph-isomorphism baseline benchmarks (ALGO-ISO-001).
//!
//! Run: `cargo bench --bench bench_vf2`.
//! Results land under `target/criterion/`. A snapshot is committed to
//! `.codefuse/tracking/perf/ALGO-ISO-001.json`.
//!
//! Workloads are chosen to keep VF2's backtracking bounded: the undirected
//! ring(n) has exactly 2n automorphisms, and a sparse ER graph compared to
//! itself terminates at the first complete mapping. Highly symmetric dense
//! graphs (e.g. `K_n` with n! automorphisms) are deliberately avoided.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, count_isomorphisms_vf2, erdos_renyi_gnp, isomorphic_vf2, ring_graph};

fn ring(n: u32) -> Graph {
    ring_graph(n, false, false, true).expect("ring")
}

/// Sparse ER graph at average degree ≈ 4 — a realistic adjacency workload
/// for the "is this self-isomorphic" first-mapping path.
fn synthetic(n: u32) -> Graph {
    let p = 4.0 / f64::from(n.saturating_sub(1).max(1));
    erdos_renyi_gnp(n, p, false, false, 0x0BF5_BE0C_1502).expect("ER synthetic")
}

/// Counting automorphisms of a ring exercises the full backtracking search;
/// the answer is 2n so the work scales predictably.
fn bench_vf2_count_ring(c: &mut Criterion) {
    let mut group = c.benchmark_group("vf2/count_automorphisms_ring");
    for n in [50u32, 100, 200] {
        let g = ring(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| count_isomorphisms_vf2(g, g, None, None, None, None).expect("count"));
        });
    }
    group.finish();
}

/// Finding a single isomorphism (self-comparison) terminates at the first
/// complete mapping — the common "are these the same graph" query.
fn bench_vf2_isomorphic_synthetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("vf2/isomorphic_synthetic");
    for n in [100u32, 1_000, 5_000] {
        let g = synthetic(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| isomorphic_vf2(g, g, None, None, None, None).expect("isomorphic"));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_vf2_count_ring,
    bench_vf2_isomorphic_synthetic
);
criterion_main!(benches);
