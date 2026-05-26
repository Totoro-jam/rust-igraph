//! Max-flow (Dinic) baseline benchmarks for ALGO-FL-002.
//!
//! Run: `cargo bench --bench bench_max_flow`.
//! Results land under `target/criterion/`. Headline numbers are recorded
//! in `.codefuse/tracking/perf/ALGO-FL-002.json`.
//!
//! Compares against (a) a textbook 6-vertex CLRS instance and (b)
//! synthetic layered networks where the answer is known by construction
//! (sum of bottleneck arcs in each layer).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, max_flow_value};

/// CLRS 26.1-1 max-flow textbook instance (6 vertices, 9 arcs, max flow = 23).
fn textbook() -> (Graph, Vec<f64>, u32, u32) {
    let mut g = Graph::new(6, true).expect("graph init");
    let arcs = [
        (0u32, 1u32),
        (0, 2),
        (1, 2),
        (1, 3),
        (2, 1),
        (2, 4),
        (3, 2),
        (3, 5),
        (4, 3),
        (4, 5),
    ];
    let caps = vec![16.0, 13.0, 10.0, 12.0, 4.0, 14.0, 9.0, 20.0, 7.0, 4.0];
    for (u, v) in arcs {
        g.add_edge(u, v).expect("edge in range");
    }
    (g, caps, 0, 5)
}

/// Layered directed network: `layers` layers of `width` vertices each.
/// Source (0) connects to layer-0; layer-i fully connects to layer-(i+1);
/// the final layer connects to a single sink. All capacities = 1.
/// Max flow equals the min layer cut = `width`.
fn layered(layers: u32, width: u32) -> (Graph, Vec<f64>, u32, u32) {
    let n_inner = layers * width;
    let n = n_inner + 2; // +source +sink
    let source = 0u32;
    let sink = n - 1;
    let mut g = Graph::new(n, true).expect("graph init");
    let mut caps: Vec<f64> = Vec::new();
    let idx = |layer: u32, col: u32| 1 + layer * width + col;

    // source → layer 0
    for col in 0..width {
        g.add_edge(source, idx(0, col)).expect("edge");
        caps.push(1.0);
    }
    // bipartite full connect between consecutive layers
    for layer in 0..(layers.saturating_sub(1)) {
        for a in 0..width {
            for b in 0..width {
                g.add_edge(idx(layer, a), idx(layer + 1, b)).expect("edge");
                caps.push(1.0);
            }
        }
    }
    // last layer → sink
    for col in 0..width {
        g.add_edge(idx(layers - 1, col), sink).expect("edge");
        caps.push(1.0);
    }
    (g, caps, source, sink)
}

fn bench_textbook(c: &mut Criterion) {
    let (g, caps, s, t) = textbook();
    c.bench_function("max_flow_value/textbook (6v 10e directed)", |b| {
        b.iter(|| max_flow_value(&g, s, t, Some(&caps)).expect("max_flow"));
    });
}

fn bench_layered(c: &mut Criterion) {
    let mut group = c.benchmark_group("max_flow_value/layered");
    for (layers, width) in [(4u32, 8u32), (6, 16), (8, 32)] {
        let (g, caps, s, t) = layered(layers, width);
        group.throughput(Throughput::Elements(g.ecount() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("L{layers}xW{width}")),
            &(g, caps, s, t),
            |b, (g, caps, s, t)| {
                b.iter(|| max_flow_value(g, *s, *t, Some(caps)).expect("max_flow"));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_textbook, bench_layered);
criterion_main!(benches);
