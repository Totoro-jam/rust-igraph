//! Graphlet decomposition benchmark. ALGO-CL-020.
//!
//! Run: `cargo bench --bench bench_graphlets`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-CL-020.json`.

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, graphlets, graphlets_candidate_basis, graphlets_project};

fn triangle_with_tail() -> (Graph, Vec<f64>) {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).expect("e");
    g.add_edge(0, 2).expect("e");
    g.add_edge(1, 2).expect("e");
    g.add_edge(2, 3).expect("e");
    let w = vec![3.0, 3.0, 3.0, 1.0];
    (g, w)
}

fn dense_ring(n: u32) -> (Graph, Vec<f64>) {
    let mut g = Graph::with_vertices(n);
    let mut weights = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            if j - i <= 2 || (i == 0 && j == n - 1) {
                g.add_edge(i, j).expect("e");
                weights.push(2.0);
            }
        }
    }
    (g, weights)
}

fn bench_candidate_basis_small(c: &mut Criterion) {
    let (g, w) = triangle_with_tail();
    c.bench_function("graphlets/candidate_basis triangle+tail (4v 4e)", |b| {
        b.iter(|| graphlets_candidate_basis(&g, &w).expect("basis"));
    });
}

fn bench_project_small(c: &mut Criterion) {
    let (g, w) = triangle_with_tail();
    let basis = graphlets_candidate_basis(&g, &w).expect("basis");
    c.bench_function("graphlets/project triangle+tail 1000 iters", |b| {
        b.iter(|| graphlets_project(&g, &w, &basis.cliques, None, 1000).expect("project"));
    });
}

fn bench_graphlets_ring20(c: &mut Criterion) {
    let (g, w) = dense_ring(20);
    c.bench_function("graphlets/full ring-20 (20v)", |b| {
        b.iter(|| graphlets(&g, &w, 1000).expect("graphlets"));
    });
}

criterion_group!(
    benches,
    bench_candidate_basis_small,
    bench_project_small,
    bench_graphlets_ring20
);
criterion_main!(benches);
