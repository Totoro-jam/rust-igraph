//! Minimum-spanning-tree benchmarks (ALGO-MST-001).
//!
//! Run: `cargo bench --bench bench_mst`.
//! Results land under `target/criterion/`. A snapshot of the baseline
//! lives at `.codefuse/tracking/perf/ALGO-MST-001.json`.
//!
//! We exercise three input shapes that stress the three dispatch
//! branches:
//!   - Karate (34v / 78e) — small dense-ish graph, all three variants.
//!   - Synthetic "path + skip-7" sparse graph at 1k / 10k vertices —
//!     measures the asymptotic Prim/Kruskal cost on sparse data.
//!   - Complete `K_n` at n=64 — quadratic edge count, surfaces the
//!     Kruskal-sort overhead vs Prim's heap behaviour.

#![allow(clippy::cast_precision_loss)]

use std::fs::File;
use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, MstAlgorithm, minimum_spanning_tree, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

/// Synthetic ER-like sparse graph: path + skip-7 cross-edges (same as
/// `bench_bfs::synthetic`).
fn synthetic(n: u32) -> Graph {
    let mut g = Graph::with_vertices(n);
    for i in 0..n.saturating_sub(1) {
        g.add_edge(i, i + 1).expect("synthetic add_edge");
    }
    for i in 0..n.saturating_sub(7) {
        g.add_edge(i, i + 7).expect("synthetic add_edge");
    }
    g
}

/// Complete `K_n`.
fn complete(n: u32) -> Graph {
    let mut g = Graph::with_vertices(n);
    for u in 0..n {
        for v in (u + 1)..n {
            g.add_edge(u, v).expect("complete add_edge");
        }
    }
    g
}

/// Deterministic weights derived from edge index, distinct so the MST
/// is uniquely determined and the variants exercise sorted-vs-heap
/// extraction without tiebreak instability.
fn weights_for(g: &Graph) -> Vec<f64> {
    (0..g.ecount()).map(|i| (i as f64) + 1.0).collect()
}

fn bench_mst_karate(c: &mut Criterion) {
    let g = karate();
    let w = weights_for(&g);
    let mut group = c.benchmark_group("mst/karate (34v 78e)");
    group.bench_function("unweighted", |b| {
        b.iter(|| minimum_spanning_tree(&g, None, MstAlgorithm::Unweighted).unwrap());
    });
    group.bench_function("kruskal", |b| {
        b.iter(|| minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Kruskal).unwrap());
    });
    group.bench_function("prim", |b| {
        b.iter(|| minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Prim).unwrap());
    });
    group.bench_function("automatic_weighted", |b| {
        b.iter(|| minimum_spanning_tree(&g, Some(&w), MstAlgorithm::Automatic).unwrap());
    });
    group.finish();
}

fn bench_mst_synthetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("mst/synthetic_sparse");
    for n in [1_000u32, 10_000] {
        let g = synthetic(n);
        let w = weights_for(&g);
        group.throughput(Throughput::Elements(
            u64::try_from(g.ecount()).expect("ecount fits u64"),
        ));
        group.bench_with_input(
            BenchmarkId::new("kruskal", n),
            &(g.clone(), w.clone()),
            |b, (g, w)| {
                b.iter(|| minimum_spanning_tree(g, Some(w), MstAlgorithm::Kruskal).unwrap());
            },
        );
        group.bench_with_input(
            BenchmarkId::new("prim", n),
            &(g.clone(), w.clone()),
            |b, (g, w)| {
                b.iter(|| minimum_spanning_tree(g, Some(w), MstAlgorithm::Prim).unwrap());
            },
        );
        group.bench_with_input(BenchmarkId::new("unweighted_bfs", n), &g, |b, g| {
            b.iter(|| minimum_spanning_tree(g, None, MstAlgorithm::Unweighted).unwrap());
        });
    }
    group.finish();
}

fn bench_mst_complete(c: &mut Criterion) {
    let mut group = c.benchmark_group("mst/complete_K");
    for n in [32u32, 64] {
        let g = complete(n);
        let w = weights_for(&g);
        group.throughput(Throughput::Elements(
            u64::try_from(g.ecount()).expect("ecount fits u64"),
        ));
        group.bench_with_input(
            BenchmarkId::new("kruskal", n),
            &(g.clone(), w.clone()),
            |b, (g, w)| {
                b.iter(|| minimum_spanning_tree(g, Some(w), MstAlgorithm::Kruskal).unwrap());
            },
        );
        group.bench_with_input(
            BenchmarkId::new("prim", n),
            &(g.clone(), w.clone()),
            |b, (g, w)| {
                b.iter(|| minimum_spanning_tree(g, Some(w), MstAlgorithm::Prim).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_mst_karate,
    bench_mst_synthetic,
    bench_mst_complete
);
criterion_main!(benches);
