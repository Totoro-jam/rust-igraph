//! Eigenvector centrality benchmark. ALGO-PR-012b.
//!
//! Run: `cargo bench --bench bench_eigenvector`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-PR-012b.json`. Covers all four code
//! paths: undirected/unweighted, undirected/weighted, directed/unweighted,
//! directed/weighted.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{
    EigenvectorMode, Graph, eigenvector_centrality, eigenvector_centrality_directed,
    eigenvector_centrality_directed_weighted, eigenvector_centrality_weighted, read_edgelist,
};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn directed_ring(n: u32) -> Graph {
    let mut g = Graph::new(n, true).expect("graph init");
    for i in 0..n {
        g.add_edge(i, (i + 1) % n).expect("ring edge");
    }
    g
}

fn bench_undirected_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("eigen/karate (34v 78e, undirected unweighted)", |b| {
        b.iter(|| eigenvector_centrality(&g).unwrap());
    });
}

#[allow(clippy::cast_precision_loss)]
fn bench_undirected_karate_weighted(c: &mut Criterion) {
    let g = karate();
    let w: Vec<f64> = (0..g.ecount())
        .map(|i| 1.0 + (i % 5) as f64 * 0.5)
        .collect();
    c.bench_function("eigen/karate weighted (varied)", |b| {
        b.iter(|| eigenvector_centrality_weighted(&g, &w).unwrap());
    });
}

fn bench_directed_ring_500(c: &mut Criterion) {
    let g = directed_ring(500);
    c.bench_function("eigen/directed-ring-500 (Out mode)", |b| {
        b.iter(|| eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap());
    });
}

#[allow(clippy::cast_precision_loss)]
fn bench_directed_ring_500_weighted(c: &mut Criterion) {
    let g = directed_ring(500);
    let w: Vec<f64> = (0..g.ecount())
        .map(|i| 1.0 + (i % 7) as f64 * 0.25)
        .collect();
    c.bench_function("eigen/directed-ring-500 weighted (Out mode)", |b| {
        b.iter(|| eigenvector_centrality_directed_weighted(&g, EigenvectorMode::Out, &w).unwrap());
    });
}

criterion_group!(
    benches,
    bench_undirected_karate,
    bench_undirected_karate_weighted,
    bench_directed_ring_500,
    bench_directed_ring_500_weighted,
);
criterion_main!(benches);
