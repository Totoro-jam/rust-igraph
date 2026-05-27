use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, trussness};

#[allow(clippy::cast_precision_loss)]
fn bench_trussness(c: &mut Criterion) {
    let mut group = c.benchmark_group("trussness/sparse_chain");
    for n in [100u32, 500, 1000] {
        // Sparse graph: chain of triangles (0-1-2, 2-3-4, 4-5-6, ...)
        let mut g = Graph::with_vertices(n);
        for i in (0..n.saturating_sub(2)).step_by(2) {
            g.add_edge(i, i + 1).unwrap();
            g.add_edge(i, i + 2).unwrap();
            g.add_edge(i + 1, i + 2).unwrap();
        }
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| trussness(&g));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("trussness/dense_clique");
    for n in [20u32, 50, 100] {
        // Dense graph: complete graph K_n
        let mut g = Graph::with_vertices(n);
        for u in 0..n {
            for v in (u + 1)..n {
                g.add_edge(u, v).unwrap();
            }
        }
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| trussness(&g));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("trussness/band_graph");
    for n in [200u32, 1000, 5000] {
        // Band graph: each vertex connects to next 4 neighbors
        let mut g = Graph::with_vertices(n);
        for v in 0..n {
            for offset in 1..=4u32 {
                let u = (v + offset) % n;
                if u > v {
                    g.add_edge(v, u).unwrap();
                }
            }
        }
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| trussness(&g));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_trussness);
criterion_main!(benches);
