//! `edge_connectivity` baseline benchmarks for ALGO-FL-016.
//!
//! Run: `cargo bench --bench bench_edge_connectivity`.
//! Results land under `target/criterion/`. Headline numbers are
//! recorded in `.codefuse/tracking/perf/ALGO-FL-016.json`.
//!
//! `edge_connectivity` is the global adhesion: it dispatches to the
//! cheap short-circuits (connectedness, min-degree=1) when
//! `checks=true`, and otherwise runs a fixed-vertex iteration calling
//! `st_edge_connectivity(0, v)` for each `v != 0` (both directions for
//! directed graphs). Unlike FL-015 there is no complete-graph
//! shortcut (multigraphs can have lambda > n-1), so even `K_n` must
//! traverse the fixed-vertex loop. The fixtures below exercise both
//! code paths so the perf JSON captures the cheap short-circuit cost
//! separately from the fixed-vertex loop.
//!
//! Fixture catalogue:
//!   * `textbook` — 6-vertex undirected path; cheap min-degree=1
//!     short-circuit fires immediately under `checks=true`.
//!   * `layered/LxW` — layered directed network; not strongly connected
//!     (sink has no out-edges), so cheap `strongly_connected_components`
//!     short-circuit returns 0 under `checks=true`.
//!   * `fixed_vertex/RING-N` — undirected ring `C_N`; cheap checks
//!     leave the result undecided (connected, min-degree=2), so the
//!     fixed-vertex loop runs in full and returns 2. This is the
//!     worst-case path for FL-016 on a sparse graph.
//!   * `fixed_vertex/K-N` — complete undirected `K_N`; no complete-graph
//!     shortcut for `edge_connectivity`, so the fixed-vertex loop runs
//!     V-1 max-flow computations on a dense graph (≈ V edges per cut).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, edge_connectivity};

/// 6-vertex undirected path (`edge_connectivity` = 1; cheap
/// min-degree short-circuit fires under `checks=true`).
fn textbook() -> Graph {
    let mut g = Graph::new(6, false).expect("graph init");
    for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 5)] {
        g.add_edge(u, v).expect("edge in range");
    }
    g
}

/// Layered directed network — same shape as FL-013/FL-014/FL-015 benches.
/// Not strongly connected, so `edge_connectivity` = 0 under any
/// `checks` setting (with checks: cheap SCC short-circuit; without:
/// fixed-vertex loop finds an unreachable pair early and exits).
fn layered(layers: u32, width: u32) -> Graph {
    let n_inner = layers * width;
    let n = n_inner + 2;
    let source = 0u32;
    let sink = n - 1;
    let mut g = Graph::new(n, true).expect("graph init");
    let idx = |layer: u32, col: u32| 1 + layer * width + col;

    for col in 0..width {
        g.add_edge(source, idx(0, col)).expect("edge");
    }
    for layer in 0..(layers.saturating_sub(1)) {
        for a in 0..width {
            for b in 0..width {
                g.add_edge(idx(layer, a), idx(layer + 1, b)).expect("edge");
            }
        }
    }
    for col in 0..width {
        g.add_edge(idx(layers - 1, col), sink).expect("edge");
    }
    g
}

/// Undirected ring on `n` vertices. Connected, min-degree=2, so the
/// cheap short-circuits all pass and the fixed-vertex loop runs in
/// full. Returns ec = 2.
fn ring(n: u32) -> Graph {
    let mut g = Graph::new(n, false).expect("graph init");
    for i in 0..n {
        let j = (i + 1) % n;
        g.add_edge(i, j).expect("edge");
    }
    g
}

/// Complete undirected graph `K_n`. No complete-graph shortcut for
/// `edge_connectivity`, so the fixed-vertex loop runs V-1 max-flow
/// computations on a dense input. Returns ec = n-1.
fn complete(n: u32) -> Graph {
    let mut g = Graph::new(n, false).expect("graph init");
    for i in 0..n {
        for j in (i + 1)..n {
            g.add_edge(i, j).expect("edge");
        }
    }
    g
}

fn bench_textbook(c: &mut Criterion) {
    let g = textbook();
    c.bench_function(
        "edge_connectivity/textbook (6v path undirected, checks=true)",
        |b| {
            b.iter(|| edge_connectivity(&g, true).expect("ec"));
        },
    );
}

fn bench_layered_checks(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_connectivity/layered_checks_true");
    for (layers, width) in [(4u32, 8u32), (6, 16), (8, 32)] {
        let g = layered(layers, width);
        group.throughput(Throughput::Elements(g.ecount() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("L{layers}xW{width}")),
            &g,
            |b, g| {
                b.iter(|| edge_connectivity(g, true).expect("ec"));
            },
        );
    }
    group.finish();
}

fn bench_fixed_vertex_ring(c: &mut Criterion) {
    // Fixed-vertex loop on rings — cheap checks all pass, so this
    // measures the worst-case path for sparse non-complete graphs.
    let mut group = c.benchmark_group("edge_connectivity/fixed_vertex_ring");
    for n in [6u32, 8, 12] {
        let g = ring(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(format!("C{n}")), &g, |b, g| {
            b.iter(|| edge_connectivity(g, true).expect("ec"));
        });
    }
    group.finish();
}

fn bench_fixed_vertex_complete(c: &mut Criterion) {
    // K_n with no complete-graph shortcut: V-1 dense max-flow runs.
    let mut group = c.benchmark_group("edge_connectivity/fixed_vertex_complete");
    for n in [4u32, 6, 8] {
        let g = complete(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(format!("K{n}")), &g, |b, g| {
            b.iter(|| edge_connectivity(g, true).expect("ec"));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_textbook,
    bench_layered_checks,
    bench_fixed_vertex_ring,
    bench_fixed_vertex_complete
);
criterion_main!(benches);
