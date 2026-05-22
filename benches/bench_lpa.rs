//! Label propagation community detection benchmark. ALGO-CO-004.
//!
//! Run: `cargo bench --bench bench_lpa`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-CO-004.json`. Covers the major code
//! paths: unweighted Fast variant (default), unit-weighted, fixed-seed
//! determinism, the three variants on karate, and a denser
//! ring-of-cliques benchmark that exercises convergence on a graph
//! with sharper community structure.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{
    Graph, LpaOptions, LpaVariant, label_propagation, label_propagation_weighted,
    label_propagation_with_options, read_edgelist,
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
    c.bench_function("lpa/karate (34v 78e, fast, unweighted)", |b| {
        b.iter(|| label_propagation(&g).unwrap());
    });
}

fn bench_karate_weighted(c: &mut Criterion) {
    let g = karate();
    let w: Vec<f64> = vec![1.0; g.ecount()];
    c.bench_function("lpa/karate unit-weighted (fast)", |b| {
        b.iter(|| label_propagation_weighted(&g, &w).unwrap());
    });
}

fn bench_karate_fixed_seed(c: &mut Criterion) {
    let g = karate();
    let opts = LpaOptions {
        seed: 42,
        ..LpaOptions::default()
    };
    c.bench_function("lpa/karate fixed seed (deterministic)", |b| {
        b.iter(|| label_propagation_with_options(&g, None, &opts).unwrap());
    });
}

fn bench_karate_dominance(c: &mut Criterion) {
    let g = karate();
    let opts = LpaOptions {
        variant: LpaVariant::Dominance,
        seed: 0,
        ..LpaOptions::default()
    };
    c.bench_function("lpa/karate dominance variant", |b| {
        b.iter(|| label_propagation_with_options(&g, None, &opts).unwrap());
    });
}

fn bench_karate_retention(c: &mut Criterion) {
    let g = karate();
    let opts = LpaOptions {
        variant: LpaVariant::Retention,
        seed: 0,
        ..LpaOptions::default()
    };
    c.bench_function("lpa/karate retention variant", |b| {
        b.iter(|| label_propagation_with_options(&g, None, &opts).unwrap());
    });
}

fn bench_ring_of_cliques_8x10(c: &mut Criterion) {
    // 8 cliques × 10 vertices each = 80v, 8·45 internal + 8 bridge = 368e.
    // LPA converges in O(few) passes; this measures the per-iteration
    // adjacency walk + label-tally on a graph with sharp communities.
    let g = ring_of_cliques(8, 10);
    c.bench_function("lpa/ring-of-cliques 8x10 (80v 368e, fast)", |b| {
        b.iter(|| label_propagation(&g).unwrap());
    });
}

criterion_group!(
    benches,
    bench_karate_unweighted,
    bench_karate_weighted,
    bench_karate_fixed_seed,
    bench_karate_dominance,
    bench_karate_retention,
    bench_ring_of_cliques_8x10,
);
criterion_main!(benches);
