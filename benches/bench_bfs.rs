//! BFS baseline benchmarks.
//!
//! Run: `cargo bench --bench bench_bfs`.
//! Results land under `target/criterion/`. Numbers are committed to
//! `.codefuse/tracking/perf/ALGO-TR-001.json` (Phase 0 baseline).

use std::fs::File;
use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, bfs, erdos_renyi_gnp, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

/// Synthetic sparse ER graph at average degree ≈ 4 — exercises BFS on
/// a realistic adjacency-list workload (constant average branching).
fn synthetic(n: u32) -> Graph {
    let p = 4.0 / f64::from(n.saturating_sub(1).max(1));
    erdos_renyi_gnp(n, p, false, false, 0x0BF5_BE0C_0001).expect("ER synthetic")
}

fn bench_bfs_karate(c: &mut Criterion) {
    let g = karate();
    c.bench_function("bfs/karate (34v 78e)", |b| {
        b.iter(|| bfs(&g, 0).unwrap());
    });
}

fn bench_bfs_synthetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("bfs/synthetic");
    for n in [100u32, 1_000, 10_000] {
        let g = synthetic(n);
        group.throughput(Throughput::Elements(u64::from(g.vcount())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| bfs(g, 0).unwrap());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_bfs_karate, bench_bfs_synthetic);
criterion_main!(benches);
