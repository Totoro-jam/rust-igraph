//! `DrL` 3D layout baseline (ALGO-LO-015).
//!
//! Run: `cargo bench --bench bench_drl_3d`.
//! Results land under `target/criterion/`. A snapshot is committed to
//! `.codefuse/tracking/perf/ALGO-LO-015.json`.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{DrlOptions, DrlTemplate, Graph, layout_drl_3d};

#[allow(clippy::cast_possible_truncation)]
fn path_graph(n: usize) -> Graph {
    let mut g = Graph::with_vertices(n as u32);
    for i in 0..n as u32 - 1 {
        g.add_edge(i, i + 1).unwrap();
    }
    g
}

#[allow(clippy::cast_possible_truncation)]
fn cycle_graph(n: usize) -> Graph {
    let n32 = n as u32;
    let mut g = Graph::with_vertices(n32);
    for i in 0..n32 {
        g.add_edge(i, (i + 1) % n32).unwrap();
    }
    g
}

#[allow(clippy::cast_possible_truncation)]
fn bench_vertex_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("drl_3d/vertex_count");
    let opts = DrlOptions::from_template(DrlTemplate::Refine);
    for &n in &[10usize, 50, 200] {
        let g = cycle_graph(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| layout_drl_3d(g, None, &opts, None).expect("drl_3d"));
        });
    }
    group.finish();
}

fn bench_templates(c: &mut Criterion) {
    let mut group = c.benchmark_group("drl_3d/template");
    let g = path_graph(30);
    for templ in [
        DrlTemplate::Refine,
        DrlTemplate::Final,
        DrlTemplate::Default,
    ] {
        let opts = DrlOptions::from_template(templ);
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{templ:?}")),
            &g,
            |b, g| {
                b.iter(|| layout_drl_3d(g, None, &opts, None).expect("drl_3d"));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_vertex_count, bench_templates);
criterion_main!(benches);
