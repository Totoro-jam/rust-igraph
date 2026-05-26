//! `mincut_value` baseline benchmarks for ALGO-FL-017.
//!
//! Run: `cargo bench --bench bench_mincut_value`.
//! Results land under `target/criterion/`. Headline numbers are
//! recorded in `.codefuse/tracking/perf/ALGO-FL-017.json`.
//!
//! Unlike `edge_connectivity` (FL-016), `mincut_value` does **not**
//! short-circuit on disconnected graphs or min-degree-1 graphs — there
//! is no `checks` parameter in the igraph C API, so every input
//! traverses the fixed-vertex loop (modulo the V-1 max-flow early-exit
//! at value 0). The fixtures below cover three regimes:
//!
//!   * `textbook` — undirected ring `C_5` with unit caps; reproduces
//!     the demo's headline case.
//!   * `layered/LxW` — layered directed network with unit caps; not
//!     strongly connected, so each call into FL-002 short-circuits at
//!     the BFS step. Demonstrates the cost of the disconnected
//!     fixed-vertex loop without short-circuit help.
//!   * `fixed_vertex_ring/RING-N` — undirected ring `C_N` with unit
//!     caps; the worst-case sparse-connected path: V-1 FL-002 calls,
//!     each returning 2.
//!   * `fixed_vertex_complete/K-N` — complete `K_N` with unit caps;
//!     V-1 dense max-flow runs that converge quickly.
//!   * `weighted_ring/RING-N` — weighted ring `C_N` with non-uniform
//!     caps to exercise the weighted code path through `f64`
//!     comparisons.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, mincut_value};

fn textbook() -> Graph {
    let mut g = Graph::new(5, false).expect("graph init");
    for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 0)] {
        g.add_edge(u, v).expect("edge in range");
    }
    g
}

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

fn ring(n: u32) -> Graph {
    let mut g = Graph::new(n, false).expect("graph init");
    for i in 0..n {
        let j = (i + 1) % n;
        g.add_edge(i, j).expect("edge");
    }
    g
}

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
    c.bench_function("mincut_value/textbook (C_5 undirected unit caps)", |b| {
        b.iter(|| mincut_value(&g, None).expect("mc"));
    });
}

fn bench_layered(c: &mut Criterion) {
    let mut group = c.benchmark_group("mincut_value/layered_unit_caps");
    for (layers, width) in [(4u32, 8u32), (6, 16), (8, 32)] {
        let g = layered(layers, width);
        group.throughput(Throughput::Elements(g.ecount() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("L{layers}xW{width}")),
            &g,
            |b, g| {
                b.iter(|| mincut_value(g, None).expect("mc"));
            },
        );
    }
    group.finish();
}

fn bench_fixed_vertex_ring(c: &mut Criterion) {
    let mut group = c.benchmark_group("mincut_value/fixed_vertex_ring");
    for n in [6u32, 8, 12] {
        let g = ring(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(format!("C{n}")), &g, |b, g| {
            b.iter(|| mincut_value(g, None).expect("mc"));
        });
    }
    group.finish();
}

fn bench_fixed_vertex_complete(c: &mut Criterion) {
    let mut group = c.benchmark_group("mincut_value/fixed_vertex_complete");
    for n in [4u32, 6, 8] {
        let g = complete(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(format!("K{n}")), &g, |b, g| {
            b.iter(|| mincut_value(g, None).expect("mc"));
        });
    }
    group.finish();
}

fn bench_weighted_ring(c: &mut Criterion) {
    // Same ring sizes, but exercise the weighted code path with a
    // non-uniform capacity vector (no edge has capacity exactly 1.0).
    let mut group = c.benchmark_group("mincut_value/weighted_ring");
    for n in [6u32, 8, 12] {
        let g = ring(n);
        let caps: Vec<f64> = (0..n).map(|i| 1.0 + f64::from(i) * 0.5).collect();
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(format!("C{n}")), &g, |b, g| {
            b.iter(|| mincut_value(g, Some(&caps)).expect("mc"));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_textbook,
    bench_layered,
    bench_fixed_vertex_ring,
    bench_fixed_vertex_complete,
    bench_weighted_ring,
);
criterion_main!(benches);
