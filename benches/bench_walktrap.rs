//! Walktrap (Pons-Latapy 2005) benchmark. ALGO-CO-008.
//!
//! Run: `cargo bench --bench bench_walktrap`. Numbers land in
//! `.codefuse/tracking/perf/ALGO-CO-008.json`. Cells exercise the small
//! C reference fixtures (triangle, ring-6 weighted), a bridged-cliques
//! exemplar, the karate club, and a ring of cliques.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, read_edgelist, walktrap, walktrap_weighted};

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

fn bench_triangle(c: &mut Criterion) {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).expect("triangle edge");
    g.add_edge(1, 2).expect("triangle edge");
    g.add_edge(2, 0).expect("triangle edge");
    c.bench_function("walktrap/triangle (3v 3e)", |b| {
        b.iter(|| walktrap(&g).unwrap());
    });
}

fn bench_ring6_weighted(c: &mut Criterion) {
    let mut g = Graph::with_vertices(6);
    g.add_edge(0, 1).expect("ring edge");
    g.add_edge(1, 2).expect("ring edge");
    g.add_edge(2, 3).expect("ring edge");
    g.add_edge(3, 4).expect("ring edge");
    g.add_edge(4, 5).expect("ring edge");
    g.add_edge(5, 0).expect("ring edge");
    let weights = vec![1.0, 0.5, 0.25, 0.75, 1.25, 1.5];
    c.bench_function("walktrap/ring-6 weighted (6v 6e)", |b| {
        b.iter(|| walktrap_weighted(&g, &weights).unwrap());
    });
}

fn bench_two_k5_bridge(c: &mut Criterion) {
    let g = two_k5_bridge();
    c.bench_function("walktrap/two-K5-bridge (10v 21e)", |b| {
        b.iter(|| walktrap(&g).unwrap());
    });
}

fn bench_ring_of_cliques_4x5(c: &mut Criterion) {
    let g = ring_of_cliques(4, 5);
    c.bench_function("walktrap/ring-of-cliques 4x5 (20v 44e)", |b| {
        b.iter(|| walktrap(&g).unwrap());
    });
}

fn bench_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("walktrap/karate (34v 78e)", |b| {
        b.iter(|| walktrap(&g).unwrap());
    });
}

criterion_group!(
    benches,
    bench_triangle,
    bench_ring6_weighted,
    bench_two_k5_bridge,
    bench_ring_of_cliques_4x5,
    bench_karate,
);
criterion_main!(benches);
