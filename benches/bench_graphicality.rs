use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{EdgeTypeFilter, is_bigraphical, is_graphical};

fn bench_graphicality(c: &mut Criterion) {
    let mut group = c.benchmark_group("graphicality/simple_undirected");
    for n in [100u32, 1000, 10_000] {
        let degrees: Vec<u32> = (0..n).map(|i| i % (n - 1)).collect();
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| is_graphical(&degrees, None, EdgeTypeFilter::Simple));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("graphicality/loopy_simple_undirected");
    for n in [100u32, 1000, 10_000] {
        let degrees: Vec<u32> = (0..n).map(|i| i % (n + 1)).collect();
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| is_graphical(&degrees, None, EdgeTypeFilter::LoopsSimple));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("graphicality/simple_directed");
    for n in [100u32, 1000, 10_000] {
        let out_deg: Vec<u32> = (0..n).map(|i| i % (n - 1)).collect();
        let in_deg: Vec<u32> = (0..n).rev().map(|i| i % (n - 1)).collect();
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| is_graphical(&out_deg, Some(&in_deg), EdgeTypeFilter::Simple));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("graphicality/bigraphical_simple");
    for n in [100u32, 1000, 10_000] {
        let d1: Vec<u32> = (0..n).map(|i| i % (n / 2 + 1)).collect();
        let d2: Vec<u32> = (0..=n / 2).map(|_| n / 2).collect();
        group.bench_function(format!("n={n}"), |b| {
            b.iter(|| is_bigraphical(&d1, &d2, EdgeTypeFilter::Simple));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_graphicality);
criterion_main!(benches);
