//! `st_vertex_connectivity` baseline benchmarks for ALGO-FL-013.
//!
//! Run: `cargo bench --bench bench_st_vertex_connectivity`.
//! Results land under `target/criterion/`. Headline numbers are
//! recorded in `.codefuse/tracking/perf/ALGO-FL-013.json`.
//!
//! `st_vertex_connectivity` reduces to a unit-cap max-flow on a
//! split-vertex graph with `2n` vertices and `m + n` arcs (or
//! `2m + n` arcs for undirected input after the MUTUAL conversion).
//! That's strictly more work than `st_edge_connectivity` on the same
//! input, so we deliberately mirror its fixture shape (textbook 6v
//! directed + layered `L × W` lattices) to make the two perf JSONs
//! comparable cell-for-cell.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, VconnNei, st_vertex_connectivity};

/// 6-vertex directed instance from
/// `tests/unit/igraph_st_vertex_connectivity.c` (path graph 0-1-2-3-4-5
/// undirected; converted to directed mutual internally — vc(0,5)=1).
fn textbook() -> (Graph, u32, u32) {
    let mut g = Graph::new(6, false).expect("graph init");
    for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 5)] {
        g.add_edge(u, v).expect("edge in range");
    }
    (g, 0, 5)
}

/// Layered directed network: `layers` layers of `width` vertices each.
/// Vertex connectivity (s → sink) equals `width` (one disjoint path
/// per column once we split through every layer).
fn layered(layers: u32, width: u32) -> (Graph, u32, u32) {
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
    (g, source, sink)
}

fn bench_textbook(c: &mut Criterion) {
    let (g, s, t) = textbook();
    c.bench_function(
        "st_vertex_connectivity/textbook (6v path undirected)",
        |b| {
            b.iter(|| st_vertex_connectivity(&g, s, t, VconnNei::Error).expect("vc"));
        },
    );
}

fn bench_layered(c: &mut Criterion) {
    let mut group = c.benchmark_group("st_vertex_connectivity/layered");
    for (layers, width) in [(4u32, 8u32), (6, 16), (8, 32)] {
        let (g, s, t) = layered(layers, width);
        group.throughput(Throughput::Elements(g.ecount() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("L{layers}xW{width}")),
            &(g, s, t),
            |b, (g, s, t)| {
                b.iter(|| st_vertex_connectivity(g, *s, *t, VconnNei::Error).expect("vc"));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_textbook, bench_layered);
criterion_main!(benches);
