//! Eulerian existence benchmark. ALGO-CC-040.
//!
//! Run: `cargo bench --bench bench_eulerian`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-CC-040.json`.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, is_eulerian, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn cycle(n: u32) -> Graph {
    let mut g = Graph::with_vertices(n);
    for i in 0..n {
        g.add_edge(i, (i + 1) % n).unwrap();
    }
    g
}

fn bench_is_eulerian_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("is_eulerian/karate (34v 78e)", |b| {
        b.iter(|| is_eulerian(&g).unwrap());
    });
}

fn bench_is_eulerian_cycle_1k(c: &mut Criterion) {
    let g = cycle(1_000);
    c.bench_function("is_eulerian/cycle-1k (always-true)", |b| {
        b.iter(|| is_eulerian(&g).unwrap());
    });
}

criterion_group!(
    benches,
    bench_is_eulerian_karate,
    bench_is_eulerian_cycle_1k
);
criterion_main!(benches);
