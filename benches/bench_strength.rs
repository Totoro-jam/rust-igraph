use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, StrengthMode, diversity, strength, strength_with_mode};

#[allow(clippy::cast_precision_loss)]
fn bench_strength(c: &mut Criterion) {
    let mut group = c.benchmark_group("strength/undirected_all");
    for n in [100u32, 1000, 10_000] {
        let mut g = Graph::with_vertices(n);
        for v in 0..n {
            for u in (v + 1)..n.min(v + 5) {
                g.add_edge(v, u).unwrap();
            }
        }
        let weights: Vec<f64> = (0..g.ecount()).map(|i| (i as f64 + 1.0) * 0.1).collect();
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| strength(&g, &weights));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("strength/directed_out");
    for n in [100u32, 1000, 10_000] {
        let mut g = Graph::new(n, true).unwrap();
        for v in 0..n {
            for u in (v + 1)..n.min(v + 5) {
                g.add_edge(v, u).unwrap();
            }
        }
        let weights: Vec<f64> = (0..g.ecount()).map(|i| (i as f64 + 1.0) * 0.1).collect();
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| strength_with_mode(&g, &weights, StrengthMode::Out, true));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("diversity/simple_undirected");
    for n in [100u32, 1000, 10_000] {
        let mut g = Graph::with_vertices(n);
        for v in 0..n {
            for u in (v + 1)..n.min(v + 5) {
                g.add_edge(v, u).unwrap();
            }
        }
        let weights: Vec<f64> = (0..g.ecount()).map(|i| (i as f64 + 1.0) * 0.1).collect();
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| diversity(&g, &weights));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_strength);
criterion_main!(benches);
