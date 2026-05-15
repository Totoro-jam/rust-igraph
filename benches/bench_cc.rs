//! Connected-components baseline benchmarks. ALGO-CC-001.
//!
//! Run: `cargo bench --bench bench_cc`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-CC-001.json`.

use std::fs::File;
use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, connected_components, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn synthetic_disjoint(n: u32, components: u32) -> Graph {
    // n vertices, divided into `components` chains.
    let mut g = Graph::with_vertices(n);
    let per = n / components.max(1);
    for c in 0..components {
        let start = c * per;
        let end = if c + 1 == components {
            n
        } else {
            (c + 1) * per
        };
        for i in start..end.saturating_sub(1) {
            g.add_edge(i, i + 1).unwrap();
        }
    }
    g
}

fn bench_cc_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("cc/karate (34v 78e)", |b| {
        b.iter(|| connected_components(&g).unwrap());
    });
}

fn bench_cc_synthetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("cc/synthetic-multi-component");
    for n in [100u32, 1_000, 10_000] {
        let g = synthetic_disjoint(n, n / 10); // ~10 vertices per component
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| connected_components(g).unwrap());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_cc_karate, bench_cc_synthetic);
criterion_main!(benches);
