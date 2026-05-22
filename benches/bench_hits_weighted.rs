//! Weighted HITS (hub & authority) benchmark. ALGO-PR-017b.
//!
//! Run: `cargo bench --bench bench_hits_weighted`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-PR-017b.json`.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, hub_and_authority_scores_weighted, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

// Directed cycle of length n: dense enough to drive several iterations,
// small enough to stay tractable in a microbench.
fn directed_ring(n: u32) -> Graph {
    let mut g = Graph::new(n, true).expect("graph init");
    for i in 0..n {
        g.add_edge(i, (i + 1) % n).expect("ring edge");
    }
    g
}

fn bench_weighted_hits_karate(c: &mut Criterion) {
    // Undirected karate under unit weights → exercises the shifted
    // power-iter path on (W + I).
    let g = karate();
    let weights = vec![1.0; g.ecount() as usize];
    c.bench_function(
        "hits_weighted/karate (34v 78e, undirected, unit weights)",
        |b| {
            b.iter(|| hub_and_authority_scores_weighted(&g, &weights).unwrap());
        },
    );
}

fn bench_weighted_hits_directed_ring_500(c: &mut Criterion) {
    // Directed power-iteration path on a 500-cycle with linearly increasing
    // weights — keeps every edge active and stresses the W · (Wᵀ · h)
    // walk.
    let g = directed_ring(500);
    #[allow(clippy::cast_precision_loss)]
    let weights: Vec<f64> = (1..=g.ecount()).map(|i| i as f64).collect();
    c.bench_function(
        "hits_weighted/directed-ring-500 (varied weights, power iter)",
        |b| {
            b.iter(|| hub_and_authority_scores_weighted(&g, &weights).unwrap());
        },
    );
}

criterion_group!(
    benches,
    bench_weighted_hits_karate,
    bench_weighted_hits_directed_ring_500
);
criterion_main!(benches);
