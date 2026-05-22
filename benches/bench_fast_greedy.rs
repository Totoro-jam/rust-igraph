//! Fast greedy modularity benchmark. ALGO-CO-007.
//!
//! Run: `cargo bench --bench bench_fast_greedy`. Numbers land in
//! `.codefuse/tracking/perf/ALGO-CO-007.json`. Cells exercise a path,
//! a bridged-cliques exemplar, the karate club, and a ring-of-cliques.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, fast_greedy_modularity, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn two_k5_bridge() -> Graph {
    let mut g = Graph::with_vertices(10);
    for u in 0..5u32 {
        for v in (u + 1)..5 {
            g.add_edge(u, v).expect("clique edge");
        }
    }
    for u in 5..10u32 {
        for v in (u + 1)..10 {
            g.add_edge(u, v).expect("clique edge");
        }
    }
    g.add_edge(0, 5).expect("bridge edge");
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
    c.bench_function("fast_greedy/path-10 (10v 9e)", |b| {
        b.iter(|| fast_greedy_modularity(&g).unwrap());
    });
}

fn bench_two_k5_bridge(c: &mut Criterion) {
    let g = two_k5_bridge();
    c.bench_function("fast_greedy/two-K5-bridge (10v 21e)", |b| {
        b.iter(|| fast_greedy_modularity(&g).unwrap());
    });
}

fn bench_ring_of_cliques_4x5(c: &mut Criterion) {
    let g = ring_of_cliques(4, 5);
    c.bench_function("fast_greedy/ring-of-cliques 4x5 (20v 44e)", |b| {
        b.iter(|| fast_greedy_modularity(&g).unwrap());
    });
}

fn bench_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("fast_greedy/karate (34v 78e)", |b| {
        b.iter(|| fast_greedy_modularity(&g).unwrap());
    });
}

criterion_group!(
    benches,
    bench_path_10,
    bench_two_k5_bridge,
    bench_ring_of_cliques_4x5,
    bench_karate,
);
criterion_main!(benches);
