//! Articulation-points benchmark. ALGO-CC-010.
//!
//! Run: `cargo bench --bench bench_articulation`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-CC-010.json`.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, articulation_points, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn path_graph(n: u32) -> Graph {
    let mut g = Graph::with_vertices(n);
    for i in 0..n.saturating_sub(1) {
        g.add_edge(i, i + 1).unwrap();
    }
    g
}

fn bench_articulation_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("articulation/karate (34v 78e)", |b| {
        b.iter(|| articulation_points(&g).unwrap());
    });
}

fn bench_articulation_path_1k(c: &mut Criterion) {
    let g = path_graph(1_000);
    c.bench_function("articulation/path-1k (998 APs)", |b| {
        b.iter(|| articulation_points(&g).unwrap());
    });
}

criterion_group!(
    benches,
    bench_articulation_karate,
    bench_articulation_path_1k
);
criterion_main!(benches);
