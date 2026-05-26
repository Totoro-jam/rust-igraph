//! `edge_disjoint_paths` baseline benchmarks for ALGO-FL-012.
//!
//! Run: `cargo bench --bench bench_edge_disjoint_paths`.
//! Results land under `target/criterion/`. Headline numbers are
//! recorded in `.codefuse/tracking/perf/ALGO-FL-012.json`.
//!
//! `edge_disjoint_paths` is a thin wrapper over `max_flow_value` with
//! unit capacities and an integer cast (igraph C's
//! `igraph_edge_disjoint_paths` at flow.c:2326 is a 15-line redirect,
//! identical in shape to `igraph_st_edge_connectivity`). By Menger's
//! theorem the two functions return the same value on every input;
//! these benches exist to (a) certify the wrapper introduces no
//! measurable overhead vs. the bare delegate on unit caps and (b) keep
//! a perf snapshot per AWU per the SOP.
//! Fixtures intentionally mirror `bench_st_edge_connectivity.rs` so the
//! two reports can be compared cell-for-cell.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, edge_disjoint_paths};

/// 6-vertex directed instance from `tests/unit/igraph_edge_disjoint_paths.c`
/// (no self-loop variant — the unweighted bench doesn't care, and a
/// self-loop contributes nothing to s-t flow).
fn textbook() -> (Graph, u32, u32) {
    let mut g = Graph::new(6, true).expect("graph init");
    let arcs = [
        (0u32, 1u32),
        (0, 2),
        (1, 2),
        (1, 3),
        (2, 4),
        (3, 4),
        (3, 5),
        (4, 5),
    ];
    for (u, v) in arcs {
        g.add_edge(u, v).expect("edge in range");
    }
    (g, 0, 5)
}

/// Layered directed network: `layers` layers of `width` vertices each.
/// Edge connectivity (and so edge-disjoint paths) equals layer width.
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
    c.bench_function("edge_disjoint_paths/textbook (6v 8e directed)", |b| {
        b.iter(|| edge_disjoint_paths(&g, s, t).expect("paths"));
    });
}

fn bench_layered(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_disjoint_paths/layered");
    for (layers, width) in [(4u32, 8u32), (6, 16), (8, 32)] {
        let (g, s, t) = layered(layers, width);
        group.throughput(Throughput::Elements(g.ecount() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("L{layers}xW{width}")),
            &(g, s, t),
            |b, (g, s, t)| {
                b.iter(|| edge_disjoint_paths(g, *s, *t).expect("paths"));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_textbook, bench_layered);
criterion_main!(benches);
