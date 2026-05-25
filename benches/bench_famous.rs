//! Famous named-graph constructor benchmarks (ALGO-CN-020).
//!
//! Run: `cargo bench --bench bench_famous`.
//! A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-CN-020.json`.
//!
//! Coverage: a single-name lookup at three representative sizes
//! (`Bull` = 5v/5e, `Petersen` = 10v/15e, `Meredith` = 70v/140e), plus a
//! "walk the entire catalogue" bench that materialises every graph
//! `famous_names()` reports so the steady-state dispatch + edge-copy
//! cost is exercised end-to-end.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{famous, famous_names};

fn bench_famous_by_name(c: &mut Criterion) {
    let mut group = c.benchmark_group("famous/by_name");
    for &(name, vcount) in &[("Bull", 5u64), ("Petersen", 10), ("Meredith", 70)] {
        group.throughput(Throughput::Elements(vcount));
        group.bench_with_input(BenchmarkId::from_parameter(name), &name, |b, &name| {
            b.iter(|| famous(name).unwrap());
        });
    }
    group.finish();
}

fn bench_famous_walk_catalogue(c: &mut Criterion) {
    let mut group = c.benchmark_group("famous/walk_catalogue");
    let names: Vec<&'static str> = famous_names().to_vec();
    group.throughput(Throughput::Elements(names.len() as u64));
    group.bench_function("all_31", |b| {
        b.iter(|| {
            for n in &names {
                let _ = famous(n).unwrap();
            }
        });
    });
    group.finish();
}

criterion_group!(benches, bench_famous_by_name, bench_famous_walk_catalogue);
criterion_main!(benches);
