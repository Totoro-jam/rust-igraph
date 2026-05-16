//! Girth benchmark. ALGO-PR-001.
//!
//! Run: `cargo bench --bench bench_girth`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-PR-001.json`.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, girth, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn ring(n: u32) -> Graph {
    let mut g = Graph::with_vertices(n);
    for i in 0..n {
        g.add_edge(i, (i + 1) % n).unwrap();
    }
    g
}

fn bench_girth_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("girth/karate (34v 78e)", |b| {
        b.iter(|| girth(&g).unwrap());
    });
}

fn bench_girth_ring_100(c: &mut Criterion) {
    // Worst case per upstream — full BFS from every vertex.
    let g = ring(100);
    c.bench_function("girth/ring-100 (worst case)", |b| {
        b.iter(|| girth(&g).unwrap());
    });
}

criterion_group!(benches, bench_girth_karate, bench_girth_ring_100);
criterion_main!(benches);
