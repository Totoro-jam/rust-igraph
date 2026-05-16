//! Triangle counting / transitivity benchmark. ALGO-PR-002.
//!
//! Run: `cargo bench --bench bench_triangles`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-PR-002.json`.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, count_triangles, read_edgelist, transitivity_undirected};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn bench_count_triangles_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("count_triangles/karate", |b| {
        b.iter(|| count_triangles(&g).unwrap());
    });
}

fn bench_transitivity_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("transitivity_undirected/karate", |b| {
        b.iter(|| transitivity_undirected(&g).unwrap());
    });
}

criterion_group!(
    benches,
    bench_count_triangles_karate,
    bench_transitivity_karate
);
criterion_main!(benches);
