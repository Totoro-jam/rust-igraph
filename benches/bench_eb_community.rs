//! Edge-betweenness community detection benchmark. ALGO-CO-006.
//!
//! Run: `cargo bench --bench bench_eb_community`. Numbers land in
//! `.codefuse/tracking/perf/ALGO-CO-006.json`. Cells exercise small
//! exemplars (path / cycle / two-K4-bridge), the karate club, and a
//! ring-of-cliques graph.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, edge_betweenness_community, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn two_k4_bridge() -> Graph {
    let mut g = Graph::with_vertices(8);
    for u in 0..4 {
        for v in (u + 1)..4 {
            g.add_edge(u, v).expect("clique edge");
        }
    }
    for u in 4..8 {
        for v in (u + 1)..8 {
            g.add_edge(u, v).expect("clique edge");
        }
    }
    g.add_edge(3, 4).expect("bridge edge");
    g
}

fn ring_of_cliques(num_cliques: u32, clique_size: u32) -> Graph {
    let n = num_cliques * clique_size;
    let mut g = Graph::with_vertices(n);
    for c in 0..num_cliques {
        let base = c * clique_size;
        for u in 0..clique_size {
            for v in (u + 1)..clique_size {
                g.add_edge(base + u, base + v).expect("clique edge");
            }
        }
        let next_base = ((c + 1) % num_cliques) * clique_size;
        g.add_edge(base, next_base).expect("bridge edge");
    }
    g
}

fn bench_path_10(c: &mut Criterion) {
    let mut g = Graph::with_vertices(10);
    for i in 0..9u32 {
        g.add_edge(i, i + 1).expect("path edge");
    }
    c.bench_function("eb_community/path-10 (10v 9e)", |b| {
        b.iter(|| edge_betweenness_community(&g).unwrap());
    });
}

fn bench_two_k4_bridge(c: &mut Criterion) {
    let g = two_k4_bridge();
    c.bench_function("eb_community/two-K4-bridge (8v 13e)", |b| {
        b.iter(|| edge_betweenness_community(&g).unwrap());
    });
}

fn bench_ring_of_cliques_4x5(c: &mut Criterion) {
    let g = ring_of_cliques(4, 5);
    c.bench_function("eb_community/ring-of-cliques 4x5 (20v 44e)", |b| {
        b.iter(|| edge_betweenness_community(&g).unwrap());
    });
}

fn bench_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("eb_community/karate (34v 78e)", |b| {
        b.iter(|| edge_betweenness_community(&g).unwrap());
    });
}

fn bench_directed_path_10(c: &mut Criterion) {
    let mut g = Graph::new(10, true).expect("directed graph");
    for i in 0..9u32 {
        g.add_edge(i, i + 1).expect("directed path edge");
    }
    c.bench_function("eb_community/directed-path-10 (10v 9e)", |b| {
        b.iter(|| edge_betweenness_community(&g).unwrap());
    });
}

fn bench_directed_two_triangles_bridge(c: &mut Criterion) {
    let mut g = Graph::new(6, true).expect("directed graph");
    for &(u, v) in &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
        g.add_edge(u, v).expect("directed edge");
    }
    c.bench_function("eb_community/directed-two-triangles-bridge (6v 7e)", |b| {
        b.iter(|| edge_betweenness_community(&g).unwrap());
    });
}

criterion_group!(
    benches,
    bench_path_10,
    bench_two_k4_bridge,
    bench_ring_of_cliques_4x5,
    bench_karate,
    bench_directed_path_10,
    bench_directed_two_triangles_bridge,
);
criterion_main!(benches);
