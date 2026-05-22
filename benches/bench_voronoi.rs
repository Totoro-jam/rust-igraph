//! `voronoi` (ALGO-SP-007) benchmark.
//!
//! Run: `cargo bench --bench bench_voronoi`. Numbers land in
//! `.codefuse/tracking/perf/ALGO-SP-007.json`.
//!
//! Three scenarios exercise the two inner loops (BFS / Dijkstra):
//!
//! - `voronoi/karate-unweighted-3gens`: 34v / 78e undirected; mirrors the
//!   reproducible portion of `references/igraph/tests/unit/igraph_voronoi.c`
//!   (Zachary karate, generators=[0,32,24], mode=ALL, FIRST tiebreaker).
//!   Drives the unweighted BFS inner loop.
//! - `voronoi/grid-weighted-3gens`: 30×30 lattice (900v) with edge
//!   weights `1.0 + i mod 5`. Drives the weighted Dijkstra inner loop and
//!   the `mindist`-tracked subtree pruning.
//! - `voronoi/grid-many-gens`: same lattice with 10 generators sampled at
//!   a stride; stresses the cross-generator min-tracking + tiebreaker
//!   path that triggers most often in dense generator regimes.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{DijkstraMode, Graph, VoronoiTiebreaker, read_edgelist, voronoi};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

/// `rows × cols` 4-connected grid as an undirected graph. Weighting:
/// every edge gets `1.0 + (idx % 5)` so Dijkstra has a non-trivial
/// distance distribution.
fn grid(rows: u32, cols: u32) -> (Graph, Vec<f64>) {
    let n = rows
        .checked_mul(cols)
        .expect("grid dimensions overflow u32");
    let mut g = Graph::with_vertices(n);
    let mut weights: Vec<f64> = Vec::new();
    let mut next_w = 0u32;
    for r in 0..rows {
        for c in 0..cols {
            let v = r * cols + c;
            if c + 1 < cols {
                g.add_edge(v, v + 1).expect("horizontal edge");
                weights.push(1.0 + f64::from(next_w % 5));
                next_w = next_w.wrapping_add(1);
            }
            if r + 1 < rows {
                g.add_edge(v, v + cols).expect("vertical edge");
                weights.push(1.0 + f64::from(next_w % 5));
                next_w = next_w.wrapping_add(1);
            }
        }
    }
    (g, weights)
}

fn bench_karate_unweighted_3gens(c: &mut Criterion) {
    let g = karate();
    let gens = vec![0u32, 32, 24];
    c.bench_function("voronoi/karate-unweighted-3gens (34v 78e)", |b| {
        b.iter(|| {
            voronoi(
                &g,
                None,
                DijkstraMode::All,
                &gens,
                VoronoiTiebreaker::First,
                42,
            )
            .unwrap()
        });
    });
}

fn bench_grid_weighted_3gens(c: &mut Criterion) {
    let (g, w) = grid(30, 30);
    // Generators at three corners — drives long, partly-overlapping
    // Dijkstra fronts.
    let gens: Vec<u32> = vec![0, (30 * 30) - 1, 30 * 29];
    c.bench_function("voronoi/grid-weighted-3gens (900v 4-connected)", |b| {
        b.iter(|| {
            voronoi(
                &g,
                Some(&w),
                DijkstraMode::All,
                &gens,
                VoronoiTiebreaker::First,
                42,
            )
            .unwrap()
        });
    });
}

fn bench_grid_many_gens(c: &mut Criterion) {
    let (g, w) = grid(30, 30);
    // 10 generators spread by stride 90 → uniform-ish coverage; lots of
    // tie regions, so the tiebreaker / mindist path is exercised heavily.
    let n = g.vcount();
    let gens: Vec<u32> = (0..n).step_by(90).take(10).collect();
    c.bench_function(
        "voronoi/grid-many-gens (900v, 10 generators, last-tiebreaker)",
        |b| {
            b.iter(|| {
                voronoi(
                    &g,
                    Some(&w),
                    DijkstraMode::All,
                    &gens,
                    VoronoiTiebreaker::Last,
                    42,
                )
                .unwrap()
            });
        },
    );
}

criterion_group!(
    benches,
    bench_karate_unweighted_3gens,
    bench_grid_weighted_3gens,
    bench_grid_many_gens,
);
criterion_main!(benches);
