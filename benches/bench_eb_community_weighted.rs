//! Weighted edge-betweenness community detection benchmark. ALGO-CO-006b.
//!
//! Run: `cargo bench --bench bench_eb_community_weighted`. Numbers land
//! in `.codefuse/tracking/perf/ALGO-CO-006b.json`. Same cell set as the
//! unweighted bench so the Dijkstra-Brandes overhead vs. BFS-Brandes is
//! visible side-by-side.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, edge_betweenness_community_weighted, read_edgelist};

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

fn bench_path_10_unit(c: &mut Criterion) {
    let mut g = Graph::with_vertices(10);
    for i in 0..9u32 {
        g.add_edge(i, i + 1).expect("path edge");
    }
    let w = vec![1.0_f64; g.ecount()];
    c.bench_function("eb_community_weighted/path-10 unit (10v 9e)", |b| {
        b.iter(|| edge_betweenness_community_weighted(&g, &w).unwrap());
    });
}

fn bench_two_k4_bridge_unit(c: &mut Criterion) {
    let g = two_k4_bridge();
    let w = vec![1.0_f64; g.ecount()];
    c.bench_function("eb_community_weighted/two-K4-bridge unit (8v 13e)", |b| {
        b.iter(|| edge_betweenness_community_weighted(&g, &w).unwrap());
    });
}

fn bench_ring_of_cliques_4x5_unit(c: &mut Criterion) {
    let g = ring_of_cliques(4, 5);
    let w = vec![1.0_f64; g.ecount()];
    c.bench_function(
        "eb_community_weighted/ring-of-cliques 4x5 unit (20v 44e)",
        |b| b.iter(|| edge_betweenness_community_weighted(&g, &w).unwrap()),
    );
}

fn bench_karate_unit(c: &mut Criterion) {
    let g = karate();
    let w = vec![1.0_f64; g.ecount()];
    c.bench_function("eb_community_weighted/karate unit (34v 78e)", |b| {
        b.iter(|| edge_betweenness_community_weighted(&g, &w).unwrap());
    });
}

criterion_group!(
    benches,
    bench_path_10_unit,
    bench_two_k4_bridge_unit,
    bench_ring_of_cliques_4x5_unit,
    bench_karate_unit,
);
criterion_main!(benches);
