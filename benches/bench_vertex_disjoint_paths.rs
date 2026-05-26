//! `vertex_disjoint_paths` baseline benchmarks for ALGO-FL-014.
//!
//! Run: `cargo bench --bench bench_vertex_disjoint_paths`.
//! Results land under `target/criterion/`. Headline numbers are
//! recorded in `.codefuse/tracking/perf/ALGO-FL-014.json`.
//!
//! `vertex_disjoint_paths` is a thin wrapper that calls
//! `st_vertex_connectivity(Ignore)` and adds back the count of direct
//! `s → t` edges (one `get_all_eids_between` lookup). Cost should
//! track FL-013's split-graph + max-flow work almost exactly, with
//! O(deg(s)) overhead from the eid lookup. Same fixture shape as
//! `bench_st_vertex_connectivity.rs` so the perf JSONs compare
//! cell-for-cell.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, vertex_disjoint_paths};

/// 6-vertex undirected path (`vertex_disjoint_paths(0, 5)` = 1; same
/// fixture as the FL-013 bench so the costs are directly comparable).
fn textbook() -> (Graph, u32, u32) {
    let mut g = Graph::new(6, false).expect("graph init");
    for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 5)] {
        g.add_edge(u, v).expect("edge in range");
    }
    (g, 0, 5)
}

/// Layered directed network: `layers` layers of `width` vertices each
/// between a source and sink. `vertex_disjoint_paths(s, t) = width`
/// (one disjoint path per column, no direct s→t edge so the wrapper
/// adds 0 on top of `st_vertex_connectivity` under `Ignore`).
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
    c.bench_function("vertex_disjoint_paths/textbook (6v path undirected)", |b| {
        b.iter(|| vertex_disjoint_paths(&g, s, t).expect("vdp"));
    });
}

fn bench_layered(c: &mut Criterion) {
    let mut group = c.benchmark_group("vertex_disjoint_paths/layered");
    for (layers, width) in [(4u32, 8u32), (6, 16), (8, 32)] {
        let (g, s, t) = layered(layers, width);
        group.throughput(Throughput::Elements(g.ecount() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("L{layers}xW{width}")),
            &(g, s, t),
            |b, (g, s, t)| {
                b.iter(|| vertex_disjoint_paths(g, *s, *t).expect("vdp"));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_textbook, bench_layered);
criterion_main!(benches);
