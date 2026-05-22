//! Leiden community detection benchmark. ALGO-CO-003.
//!
//! Run: `cargo bench --bench bench_leiden`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-CO-003.json`. Covers the major code
//! paths: unweighted modularity (default), weighted modularity, CPM
//! objective, ER objective, fixed-seed determinism, and a denser
//! ring-of-cliques benchmark that exercises the aggregation loop.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{
    Graph, LeidenObjective, LeidenOptions, leiden, leiden_weighted, leiden_with_options,
    read_edgelist,
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

fn bench_karate_unweighted(c: &mut Criterion) {
    let g = karate();
    c.bench_function("leiden/karate (34v 78e, modularity, unweighted)", |b| {
        b.iter(|| leiden(&g).unwrap());
    });
}

#[allow(clippy::cast_precision_loss)]
fn bench_karate_weighted(c: &mut Criterion) {
    let g = karate();
    let w: Vec<f64> = (0..g.ecount())
        .map(|i| 1.0 + (i % 5) as f64 * 0.5)
        .collect();
    c.bench_function("leiden/karate weighted (varied, modularity)", |b| {
        b.iter(|| leiden_weighted(&g, &w).unwrap());
    });
}

fn bench_karate_fixed_seed(c: &mut Criterion) {
    let g = karate();
    let opts = LeidenOptions {
        seed: 42,
        ..LeidenOptions::default()
    };
    c.bench_function("leiden/karate fixed seed (deterministic)", |b| {
        b.iter(|| leiden_with_options(&g, None, &opts).unwrap());
    });
}

fn bench_karate_cpm(c: &mut Criterion) {
    let g = karate();
    let opts = LeidenOptions {
        objective: LeidenObjective::Cpm,
        resolution: 0.05,
        ..LeidenOptions::default()
    };
    c.bench_function("leiden/karate CPM (γ=0.05)", |b| {
        b.iter(|| leiden_with_options(&g, None, &opts).unwrap());
    });
}

fn bench_karate_er(c: &mut Criterion) {
    let g = karate();
    let opts = LeidenOptions {
        objective: LeidenObjective::Er,
        resolution: 1.0,
        ..LeidenOptions::default()
    };
    c.bench_function("leiden/karate ER (γ=1.0)", |b| {
        b.iter(|| leiden_with_options(&g, None, &opts).unwrap());
    });
}

fn bench_ring_of_cliques_8x10(c: &mut Criterion) {
    // 8 cliques × 10 vertices each = 80v, 8·45 internal + 8 bridge = 368e.
    // Multi-level: first pass shrinks to ~8 super-vertices, then the
    // refinement+aggregation phases dominate.
    let g = ring_of_cliques(8, 10);
    c.bench_function("leiden/ring-of-cliques 8x10 (80v 368e)", |b| {
        b.iter(|| leiden(&g).unwrap());
    });
}

criterion_group!(
    benches,
    bench_karate_unweighted,
    bench_karate_weighted,
    bench_karate_fixed_seed,
    bench_karate_cpm,
    bench_karate_er,
    bench_ring_of_cliques_8x10,
);
criterion_main!(benches);
