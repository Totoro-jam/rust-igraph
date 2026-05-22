//! `community_voronoi` (ALGO-CO-009) benchmark.
//!
//! Run: `cargo bench --bench bench_community_voronoi`. Numbers land in
//! `.codefuse/tracking/perf/ALGO-CO-009.json`.
//!
//! Three scenarios exercise the two main code paths (fixed-r vs auto-r):
//!
//! - `community_voronoi/karate-fixed-r1`: 34v / 78e undirected; mirrors
//!   the Zachary karate reference scenario but with a fixed `r=1.0` so
//!   only one inner Voronoi call runs. Drives the LRD generator-picker
//!   plus a single fixed-r assignment.
//! - `community_voronoi/karate-auto-r`: same graph but `r=-1`. Drives
//!   the Brent quadratic-fit optimizer (≤25 iterations) wrapping the
//!   inner generator-picker + Voronoi call.
//! - `community_voronoi/grid-fixed-r1`: 20×20 lattice (400v) with edge
//!   weights `1.0 + i mod 5`. Drives weighted-distance LRD on a larger
//!   sparse graph where the heaps see real work.
//!
//! `grid-auto-r` is intentionally omitted: the auto-r outer loop is
//! O(25) inner calls on a 400-vertex graph and would dominate the
//! benchmark wall-clock without revealing anything new.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{DijkstraMode, Graph, community_voronoi, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

/// `rows × cols` 4-connected grid as an undirected graph. Weighting:
/// every edge gets `1.0 + (idx % 5)` so the weighted LRD has a
/// non-trivial distance distribution.
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

fn bench_karate_fixed_r1(c: &mut Criterion) {
    let g = karate();
    c.bench_function("community_voronoi/karate-fixed-r1 (34v 78e)", |b| {
        b.iter(|| community_voronoi(&g, None, None, DijkstraMode::All, 1.0).unwrap());
    });
}

fn bench_karate_auto_r(c: &mut Criterion) {
    let g = karate();
    c.bench_function("community_voronoi/karate-auto-r (34v 78e)", |b| {
        b.iter(|| community_voronoi(&g, None, None, DijkstraMode::All, -1.0).unwrap());
    });
}

fn bench_grid_fixed_r1(c: &mut Criterion) {
    let (g, w) = grid(20, 20);
    c.bench_function(
        "community_voronoi/grid-fixed-r1 (400v 4-connected weighted)",
        |b| {
            b.iter(|| community_voronoi(&g, Some(&w), None, DijkstraMode::All, 1.0).unwrap());
        },
    );
}

criterion_group!(
    benches,
    bench_karate_fixed_r1,
    bench_karate_auto_r,
    bench_grid_fixed_r1,
);
criterion_main!(benches);
