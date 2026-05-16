//! Single-source unweighted-distances benchmark. ALGO-SP-006.
//!
//! Run: `cargo bench --bench bench_distances`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-SP-006.json`.

use std::fs::File;
use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, distances, read_edgelist};

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

fn bench_distances_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("distances/karate (34v 78e)", |b| {
        b.iter(|| distances(&g, 0).unwrap());
    });
}

fn bench_distances_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("distances/path");
    for n in [100u32, 1_000, 10_000] {
        let g = path_graph(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| distances(g, 0).unwrap());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_distances_karate, bench_distances_path);
criterion_main!(benches);
