//! Dominator-tree baseline benchmarks for ALGO-FL-030.
//!
//! Run: `cargo bench --bench bench_dominator_tree`.
//! Results land under `target/criterion/`; headline numbers recorded in
//! `.codefuse/tracking/perf/ALGO-FL-030.json`.
//!
//! The Lengauer-Tarjan algorithm is O(|V| + |E|·α(|E|,|V|)) — near-linear
//! with inverse Ackermann factor from the union-find path compression
//! inside `EVAL`. The benches cover three regimes:
//!   * **13-vertex classical** — the canonical igraph C unit-test
//!     flowgraph (`igraph_dominator_tree.c:28-56`); a stress-free baseline
//!     for absolute per-call overhead.
//!   * **Binary tree `T(2, depth)`** — chain-of-stars rooted at 0;
//!     every non-root vertex has `idom = parent`, so the algorithm walks
//!     the DFS tree exactly once with no bucket activity. Measures
//!     near-linear scaling on sparse acyclic input.
//!   * **Random reducible flowgraph** — DAG plus a single back-edge per
//!     non-root vertex onto a *deterministic* earlier vertex (`v / 2`),
//!     exercising the bucket-driven semi-dominator computation with
//!     measurable `EVAL`/`LINK`/`COMPRESS` traffic.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{DominatorMode, Graph, dominator_tree};

/// Canonical 13-vertex flowgraph from
/// `references/igraph/tests/unit/igraph_dominator_tree.c:28-56`.
fn classical_13v() -> Graph {
    let mut g = Graph::new(13, true).expect("graph init");
    let edges = [
        (0u32, 1u32),
        (0, 7),
        (0, 10),
        (1, 2),
        (1, 5),
        (2, 3),
        (3, 4),
        (4, 3),
        (4, 0),
        (5, 3),
        (5, 6),
        (6, 3),
        (7, 8),
        (7, 10),
        (7, 11),
        (8, 9),
        (9, 4),
        (9, 8),
        (10, 11),
        (11, 12),
        (12, 9),
    ];
    for (u, v) in edges {
        g.add_edge(u, v).expect("edge in range");
    }
    g
}

/// Complete binary tree of depth `d`, directed root→leaf (`n = 2^d - 1`).
/// idom is trivial — every non-root vertex's idom is its parent.
fn binary_tree(depth: u32) -> Graph {
    let n: u32 = (1u32 << depth) - 1;
    let mut g = Graph::new(n, true).expect("graph init");
    for parent in 0..(n / 2) {
        let left = 2 * parent + 1;
        let right = 2 * parent + 2;
        if left < n {
            g.add_edge(parent, left).expect("edge in range");
        }
        if right < n {
            g.add_edge(parent, right).expect("edge in range");
        }
    }
    g
}

/// Deterministic reducible flowgraph on `n` vertices. Forward chain
/// `i → i+1` plus per-vertex back-edge `i → i/2` (so vertex 1's back-edge
/// goes to 0, vertex 2's to 1, vertex 3's to 1, vertex 4's to 2, ...). All
/// reachable from root 0; the back-edges create non-trivial dominator
/// fan-in so the LT inner loop does meaningful `EVAL`/`COMPRESS` work.
fn reducible_flowgraph(n: u32) -> Graph {
    let mut g = Graph::new(n, true).expect("graph init");
    for i in 0..(n - 1) {
        g.add_edge(i, i + 1).expect("edge in range");
    }
    for i in 1..n {
        g.add_edge(i, i / 2).expect("edge in range");
    }
    g
}

fn bench_classical_13v(c: &mut Criterion) {
    let g = classical_13v();
    c.bench_function("dominator_tree/13v classical (C reference)", |b| {
        b.iter(|| dominator_tree(&g, 0, DominatorMode::Out).expect("dominator_tree"));
    });
}

fn bench_binary_tree(c: &mut Criterion) {
    let mut group = c.benchmark_group("dominator_tree/binary_tree");
    for depth in [6u32, 8, 10, 12] {
        let g = binary_tree(depth);
        let edges = u64::from(u32::try_from(g.ecount()).expect("ecount fits u32"));
        group.throughput(Throughput::Elements(edges));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("depth_{depth}")),
            &g,
            |b, g| {
                b.iter(|| dominator_tree(g, 0, DominatorMode::Out).expect("dominator_tree"));
            },
        );
    }
    group.finish();
}

fn bench_reducible(c: &mut Criterion) {
    let mut group = c.benchmark_group("dominator_tree/reducible");
    for n in [64u32, 256, 1024, 4096] {
        let g = reducible_flowgraph(n);
        let edges = u64::from(u32::try_from(g.ecount()).expect("ecount fits u32"));
        group.throughput(Throughput::Elements(edges));
        group.bench_with_input(BenchmarkId::from_parameter(format!("n_{n}")), &g, |b, g| {
            b.iter(|| dominator_tree(g, 0, DominatorMode::Out).expect("dominator_tree"));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_classical_13v,
    bench_binary_tree,
    bench_reducible
);
criterion_main!(benches);
