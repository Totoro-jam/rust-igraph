//! Bipartite matching benchmark (unweighted + weighted).
//!
//! Run: `cargo bench --bench bench_matching`.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{create, maximum_bipartite_matching, maximum_bipartite_matching_weighted};

fn complete_bipartite(a: u32, b: u32) -> (rust_igraph::Graph, Vec<bool>) {
    let mut edges = Vec::with_capacity((a * b) as usize);
    for i in 0..a {
        for j in 0..b {
            edges.push((i, a + j));
        }
    }
    let n = a + b;
    let g = create(&edges, n, false).expect("complete bipartite");
    let types: Vec<bool> = (0..n).map(|i| i >= a).collect();
    (g, types)
}

fn bench_unweighted(c: &mut Criterion) {
    let mut group = c.benchmark_group("matching_unweighted");
    for &(a, b) in &[(10u32, 10), (50, 50), (100, 100)] {
        let (g, types) = complete_bipartite(a, b);
        let n = a + b;
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("K{a},{b}")),
            &(g, types),
            |bench, (g, t)| {
                bench.iter(|| maximum_bipartite_matching(g, t).unwrap());
            },
        );
    }
    group.finish();
}

fn bench_weighted(c: &mut Criterion) {
    let mut group = c.benchmark_group("matching_weighted");
    for &(a, b) in &[(10u32, 10), (50, 50)] {
        let (g, types) = complete_bipartite(a, b);
        let ne = g.ecount();
        #[allow(clippy::cast_precision_loss)]
        let weights: Vec<f64> = (0..ne).map(|i| (i as f64) + 1.0).collect();
        let n = a + b;
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("K{a},{b}")),
            &(g, types, weights),
            |bench, (g, t, w)| {
                bench.iter(|| maximum_bipartite_matching_weighted(g, t, w, 0.0).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_unweighted, bench_weighted);
criterion_main!(benches);
