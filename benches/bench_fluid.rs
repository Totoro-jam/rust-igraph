//! Fluid communities benchmark. ALGO-CO-005.
//!
//! Run: `cargo bench --bench bench_fluid`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-CO-005.json`. Covers karate at k = 2
//! and k = 4, a fixed-seed determinism cell, and a ring-of-cliques
//! benchmark that exercises convergence on a graph with sharp community
//! structure.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{
    FluidOptions, Graph, fluid_communities, fluid_communities_with_options, read_edgelist,
};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
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

fn bench_karate_k2(c: &mut Criterion) {
    let g = karate();
    c.bench_function("fluid/karate k=2 (34v 78e)", |b| {
        b.iter(|| fluid_communities(&g, 2).unwrap());
    });
}

fn bench_karate_k4(c: &mut Criterion) {
    let g = karate();
    c.bench_function("fluid/karate k=4", |b| {
        b.iter(|| fluid_communities(&g, 4).unwrap());
    });
}

fn bench_karate_fixed_seed(c: &mut Criterion) {
    let g = karate();
    let opts = FluidOptions {
        seed: 42,
        ..FluidOptions::default()
    };
    c.bench_function("fluid/karate k=3 fixed seed (deterministic)", |b| {
        b.iter(|| fluid_communities_with_options(&g, 3, &opts).unwrap());
    });
}

fn bench_ring_of_cliques_8x10_k8(c: &mut Criterion) {
    // 8 cliques × 10 vertices each = 80v, 8·45 internal + 8 bridge = 368e.
    let g = ring_of_cliques(8, 10);
    c.bench_function("fluid/ring-of-cliques 8x10 (80v 368e, k=8)", |b| {
        b.iter(|| fluid_communities(&g, 8).unwrap());
    });
}

criterion_group!(
    benches,
    bench_karate_k2,
    bench_karate_k4,
    bench_karate_fixed_seed,
    bench_ring_of_cliques_8x10_k8,
);
criterion_main!(benches);
