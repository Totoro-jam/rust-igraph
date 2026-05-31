//! Shortest-paths Dijkstra benchmark. ALGO-SP-038.
//!
//! Run: `cargo bench --bench bench_sp_dijkstra`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-SP-038.json`.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, get_shortest_paths_dijkstra, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn ring(n: u32) -> Graph {
    let mut g = Graph::with_vertices(n);
    for i in 0..n {
        g.add_edge(i, (i + 1) % n).expect("ring edge");
    }
    g
}

fn bench_karate_unit_weights(c: &mut Criterion) {
    let g = karate();
    let w = vec![1.0; g.ecount()];
    c.bench_function("sp_dij/karate unit weights from 0 (34v 78e)", |b| {
        b.iter(|| {
            get_shortest_paths_dijkstra(&g, 0, &w).expect("sp_dij");
        });
    });
}

fn bench_ring_500_unit(c: &mut Criterion) {
    let g = ring(500);
    let w = vec![1.0; g.ecount()];
    c.bench_function("sp_dij/ring-500 from 0 unit weights", |b| {
        b.iter(|| {
            get_shortest_paths_dijkstra(&g, 0, &w).expect("sp_dij");
        });
    });
}

criterion_group!(benches, bench_karate_unit_weights, bench_ring_500_unit);
criterion_main!(benches);
