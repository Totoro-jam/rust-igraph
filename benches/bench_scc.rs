//! Strongly-connected-components baseline benchmarks. ALGO-CC-002.
//!
//! Run: `cargo bench --bench bench_scc`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-CC-002.json`.

use std::fs::File;
use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, read_edgelist, strongly_connected_components};

/// Karate as undirected: SCC delegates to weak `connected_components`,
/// so this measures the dispatch path on a real fixture.
fn karate_undirected() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

/// Karate's edge list re-emitted as a directed graph (each undirected
/// edge becomes a directed edge `(u, v)` plus its reverse `(v, u)`).
/// Forces the iterative two-pass DFS path through every vertex.
fn karate_directed() -> Graph {
    let undirected = karate_undirected();
    let mut g = Graph::new(undirected.vcount(), true).expect("directed init");
    let m = u32::try_from(undirected.ecount()).expect("karate fits in u32");
    for e in 0..m {
        let (u, v) = undirected.edge(e).expect("edge id");
        g.add_edge(u, v).expect("forward");
        if u != v {
            g.add_edge(v, u).expect("reverse");
        }
    }
    g
}

/// `n`-vertex directed cycle (single SCC; worst-case post-order
/// stack depth `n`).
fn directed_cycle(n: u32) -> Graph {
    let mut g = Graph::new(n, true).expect("directed init");
    for i in 0..n {
        g.add_edge(i, (i + 1) % n).expect("add cycle edge");
    }
    g
}

fn bench_scc_karate_undirected(c: &mut Criterion) {
    let g = karate_undirected();
    c.bench_function("scc/karate-undirected (34v 78e)", |b| {
        b.iter(|| strongly_connected_components(&g).unwrap());
    });
}

fn bench_scc_karate_directed(c: &mut Criterion) {
    let g = karate_directed();
    c.bench_function("scc/karate-directed (34v 156e)", |b| {
        b.iter(|| strongly_connected_components(&g).unwrap());
    });
}

fn bench_scc_directed_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("scc/directed-cycle");
    for n in [100u32, 1_000, 10_000] {
        let g = directed_cycle(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| strongly_connected_components(g).unwrap());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_scc_karate_undirected,
    bench_scc_karate_directed,
    bench_scc_directed_cycle,
);
criterion_main!(benches);
