//! Gomory-Hu cut tree baseline benchmarks for ALGO-FL-020.
//!
//! Run: `cargo bench --bench bench_gomory_hu_tree`.
//! Results land under `target/criterion/`; headline numbers recorded in
//! `.codefuse/tracking/perf/ALGO-FL-020.json`.
//!
//! `gomory_hu_tree` invokes `st_mincut` (FL-018) exactly `vcount - 1`
//! times via Gusfield's algorithm. The benches cover three regimes:
//!   * **Six-vertex weighted** — the canonical C unit-test fixture; a
//!     stress-free baseline (5 max-flow calls).
//!   * **Complete graphs `K_n`** — every pair has max-flow = (n-1);
//!     measures the worst-case `(n-1) × Dinic(K_n)` total cost.
//!   * **Cycle graphs `C_n`** — every pair has max-flow = 2; Dinic
//!     terminates in one BFS-level pass per call, so this measures
//!     per-call constant overhead at scale.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, gomory_hu_tree};

/// Canonical 6-vertex weighted reference from
/// `references/igraph/tests/unit/igraph_gomory_hu_tree.c:178-191`.
fn six_vertex_weighted() -> (Graph, Vec<f64>) {
    let mut g = Graph::new(6, false).expect("graph init");
    let edges = [
        (0u32, 1u32),
        (0, 2),
        (1, 2),
        (1, 3),
        (1, 4),
        (2, 4),
        (3, 4),
        (3, 5),
        (4, 5),
    ];
    let caps = vec![1.0, 7.0, 1.0, 3.0, 2.0, 4.0, 1.0, 6.0, 2.0];
    for (u, v) in edges {
        g.add_edge(u, v).expect("edge in range");
    }
    (g, caps)
}

/// `K_n` — undirected complete graph, unit capacities.
fn complete_graph(n: u32) -> Graph {
    let mut g = Graph::new(n, false).expect("graph init");
    for u in 0..n {
        for v in (u + 1)..n {
            g.add_edge(u, v).expect("edge in range");
        }
    }
    g
}

/// `C_n` — undirected cycle, unit capacities.
fn cycle_graph(n: u32) -> Graph {
    let mut g = Graph::new(n, false).expect("graph init");
    for i in 0..n {
        g.add_edge(i, (i + 1) % n).expect("edge in range");
    }
    g
}

fn bench_six_vertex_weighted(c: &mut Criterion) {
    let (g, caps) = six_vertex_weighted();
    c.bench_function("gomory_hu_tree/6v weighted (C reference)", |b| {
        b.iter(|| gomory_hu_tree(&g, Some(&caps)).expect("gomory_hu_tree"));
    });
}

fn bench_complete(c: &mut Criterion) {
    let mut group = c.benchmark_group("gomory_hu_tree/complete");
    for n in [8u32, 16, 32, 64] {
        let g = complete_graph(n);
        let edges = u64::from(u32::try_from(g.ecount()).expect("ecount fits u32"));
        group.throughput(Throughput::Elements(edges));
        group.bench_with_input(BenchmarkId::from_parameter(format!("K_{n}")), &g, |b, g| {
            b.iter(|| gomory_hu_tree(g, None).expect("gomory_hu_tree"));
        });
    }
    group.finish();
}

fn bench_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("gomory_hu_tree/cycle");
    for n in [16u32, 64, 256, 1024] {
        let g = cycle_graph(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(format!("C_{n}")), &g, |b, g| {
            b.iter(|| gomory_hu_tree(g, None).expect("gomory_hu_tree"));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_six_vertex_weighted,
    bench_complete,
    bench_cycle
);
criterion_main!(benches);
