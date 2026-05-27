use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{
    CentralizationMode, LoopMode, centralization, centralization_betweenness_tmax,
    centralization_closeness_tmax, centralization_degree_tmax, centralization_eigenvector_tmax,
};

fn bench_centralization(c: &mut Criterion) {
    let mut group = c.benchmark_group("centralization/score");
    for n in [100u32, 1000, 10_000] {
        let scores: Vec<f64> = (0..n).map(|i| f64::from(i % 50)).collect();
        let tmax = centralization_degree_tmax(n, false, CentralizationMode::All, LoopMode::NoLoops);
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| centralization(&scores, tmax, true));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("centralization/degree_tmax");
    for n in [100u32, 1000, 10_000] {
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| {
                centralization_degree_tmax(n, false, CentralizationMode::All, LoopMode::NoLoops)
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("centralization/betweenness_tmax");
    for n in [100u32, 1000, 10_000] {
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| centralization_betweenness_tmax(n, false));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("centralization/closeness_tmax");
    for n in [100u32, 1000, 10_000] {
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| centralization_closeness_tmax(n, CentralizationMode::All));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("centralization/eigenvector_tmax");
    for n in [100u32, 1000, 10_000] {
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| centralization_eigenvector_tmax(n, CentralizationMode::All));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_centralization);
criterion_main!(benches);
