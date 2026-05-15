//! DFS baseline benchmarks. ALGO-TR-002.
//!
//! Run: `cargo bench --bench bench_dfs`. Numbers go into
//! `.codefuse/tracking/perf/ALGO-TR-002.json`.

use std::fs::File;
use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, dfs, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn synthetic(n: u32) -> Graph {
    let mut g = Graph::with_vertices(n);
    for i in 0..n.saturating_sub(1) {
        g.add_edge(i, i + 1).unwrap();
    }
    for i in 0..n.saturating_sub(7) {
        g.add_edge(i, i + 7).unwrap();
    }
    g
}

fn bench_dfs_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("dfs/karate (34v 78e)", |b| {
        b.iter(|| dfs(&g, 0).unwrap());
    });
}

fn bench_dfs_synthetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("dfs/synthetic");
    for n in [100u32, 1_000, 10_000] {
        let g = synthetic(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| dfs(g, 0).unwrap());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_dfs_karate, bench_dfs_synthetic);
criterion_main!(benches);
