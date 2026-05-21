//! HITS (hub & authority) benchmark. ALGO-PR-017.
//!
//! Run: `cargo bench --bench bench_hits`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-PR-017.json`.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, hub_and_authority_scores, read_edgelist};

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

fn bench_hits_karate(c: &mut Criterion) {
    // Undirected karate → delegates to eigenvector_centrality, so this
    // captures the fast-path cost.
    let g = karate();
    c.bench_function("hits/karate (34v 78e, undirected → eigenvector)", |b| {
        b.iter(|| hub_and_authority_scores(&g).unwrap());
    });
}

fn bench_hits_directed_ring_500(c: &mut Criterion) {
    // Directed power-iteration path, A·Aᵀ converges fast on a ring.
    let g = directed_ring(500);
    c.bench_function("hits/directed-ring-500 (power iter)", |b| {
        b.iter(|| hub_and_authority_scores(&g).unwrap());
    });
}

criterion_group!(benches, bench_hits_karate, bench_hits_directed_ring_500);
criterion_main!(benches);
