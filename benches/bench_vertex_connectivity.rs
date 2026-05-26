//! `vertex_connectivity` baseline benchmarks for ALGO-FL-015.
//!
//! Run: `cargo bench --bench bench_vertex_connectivity`.
//! Results land under `target/criterion/`. Headline numbers are
//! recorded in `.codefuse/tracking/perf/ALGO-FL-015.json`.
//!
//! `vertex_connectivity` is the global cohesion: it dispatches to the
//! cheap short-circuits (connectedness, min-degree, complete) when
//! `checks=true`, and otherwise runs the `O(V^2)` pairwise loop of
//! `st_vertex_connectivity` in `NumberOfNodes` mode. The fixtures
//! below exercise both code paths so the perf JSON captures the cheap
//! short-circuit cost separately from the brute-force pairwise loop.
//!
//! Fixture catalogue:
//!   * `textbook` — 6-vertex undirected path; cheap min-degree=1
//!     short-circuit fires immediately under `checks=true`.
//!   * `layered/LxW` — layered directed network; not strongly connected
//!     (sink has no out-edges), so cheap `strongly_connected_components`
//!     short-circuit returns 0 under `checks=true`. With `checks=false`
//!     it runs the full pairwise loop and is dramatically slower —
//!     covered in the `pairwise_loop` group below.
//!   * `pairwise_loop/RING-N` — undirected ring `C_N`; cheap checks
//!     leave `min_conn` at n-1 (connected, min-degree=2, not complete),
//!     so the pairwise FL-013 loop runs in full and returns 2. This is
//!     the worst-case path for FL-015 on a sparse graph.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, vertex_connectivity};

/// 6-vertex undirected path (`vertex_connectivity` = 1; cheap
/// min-degree short-circuit fires under `checks=true`).
fn textbook() -> Graph {
    let mut g = Graph::new(6, false).expect("graph init");
    for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 5)] {
        g.add_edge(u, v).expect("edge in range");
    }
    g
}

/// Layered directed network — same shape as FL-013/FL-014 benches.
/// Not strongly connected, so `vertex_connectivity` = 0 under any
/// `checks` setting (with checks: cheap SCC short-circuit; without:
/// pairwise loop finds an unreachable pair early and exits).
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

/// Undirected ring on `n` vertices. Connected, min-degree=2, not
/// complete (for n >= 4), so the cheap short-circuits all pass and
/// the pairwise FL-013 loop runs in full. Returns vc = 2.
fn ring(n: u32) -> Graph {
    let mut g = Graph::new(n, false).expect("graph init");
    for i in 0..n {
        let j = (i + 1) % n;
        g.add_edge(i, j).expect("edge");
    }
    g
}

fn bench_textbook(c: &mut Criterion) {
    let g = textbook();
    c.bench_function(
        "vertex_connectivity/textbook (6v path undirected, checks=true)",
        |b| {
            b.iter(|| vertex_connectivity(&g, true).expect("vc"));
        },
    );
}

fn bench_layered_checks(c: &mut Criterion) {
    let mut group = c.benchmark_group("vertex_connectivity/layered_checks_true");
    for (layers, width) in [(4u32, 8u32), (6, 16), (8, 32)] {
        let g = layered(layers, width);
        group.throughput(Throughput::Elements(g.ecount() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("L{layers}xW{width}")),
            &g,
            |b, g| {
                b.iter(|| vertex_connectivity(g, true).expect("vc"));
            },
        );
    }
    group.finish();
}

fn bench_pairwise_loop(c: &mut Criterion) {
    // Pairwise FL-013 loop — cheap checks all pass on rings, so this
    // measures the worst-case path for sparse non-complete graphs.
    let mut group = c.benchmark_group("vertex_connectivity/pairwise_loop_ring");
    for n in [6u32, 8, 12] {
        let g = ring(n);
        group.throughput(Throughput::Elements(
            u64::from(g.vcount()) * u64::from(g.vcount()),
        ));
        group.bench_with_input(BenchmarkId::from_parameter(format!("C{n}")), &g, |b, g| {
            b.iter(|| vertex_connectivity(g, true).expect("vc"));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_textbook,
    bench_layered_checks,
    bench_pairwise_loop
);
criterion_main!(benches);
