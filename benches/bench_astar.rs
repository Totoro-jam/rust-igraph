//! A* shortest-path benchmark. ALGO-SP-036.
//!
//! Run: `cargo bench --bench bench_astar`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-SP-036.json`.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{DijkstraMode, Graph, get_shortest_path_astar, read_edgelist};

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

fn bench_karate_null_heuristic(c: &mut Criterion) {
    let g = karate();
    c.bench_function("astar/karate 0→33 null heuristic (34v 78e)", |b| {
        b.iter(|| {
            get_shortest_path_astar(&g, 0, 33, None, DijkstraMode::All, None).expect("astar");
        });
    });
}

fn bench_ring_1000_null(c: &mut Criterion) {
    let g = ring(1000);
    c.bench_function("astar/ring-1000 0→500 null heuristic", |b| {
        b.iter(|| {
            get_shortest_path_astar(&g, 0, 500, None, DijkstraMode::All, None).expect("astar");
        });
    });
}

fn bench_ring_1000_admissible(c: &mut Criterion) {
    let g = ring(1000);
    let h = |v: u32, to: u32| {
        let d = v.abs_diff(to);
        f64::from(d.min(1000 - d))
    };
    c.bench_function("astar/ring-1000 0→500 admissible heuristic", |b| {
        b.iter(|| {
            get_shortest_path_astar(&g, 0, 500, None, DijkstraMode::All, Some(&h)).expect("astar");
        });
    });
}

criterion_group!(
    benches,
    bench_karate_null_heuristic,
    bench_ring_1000_null,
    bench_ring_1000_admissible
);
criterion_main!(benches);
