//! Edge clustering coefficient (ALGO-PR-031) benchmark.
//!
//! Run: `cargo bench --bench bench_ecc`. Numbers land in
//! `.codefuse/tracking/perf/ALGO-PR-031.json`.
//!
//! Three scenarios cover the dominant cost regimes:
//!
//! - `ecc/karate-k3-normalize`: Zachary karate (34v / 78e). The k=3
//!   path is essentially one adjlist intersection per edge — the same
//!   acyclic-style scan used by ALGO-PR-002 triangles, so the timing
//!   sits in the same micro-bench class.
//! - `ecc/karate-k4-normalize`: same fixture, k=4. Each edge now
//!   iterates over its smaller-degree endpoint's neighbours and
//!   intersects `N(hi) ∩ N(v3)` for each, so this is ~`d_avg` × the
//!   k=3 cost.
//! - `ecc/grid-30x30-k3-normalize`: 30×30 grid (900v, ~1740e). Stress
//!   test for the per-edge intersection loop on a sparse-but-large
//!   graph; exercises the linear-merge intersection on uniformly small
//!   adjacency lists (degree ≤ 4 everywhere).

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, ecc, read_edgelist};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn grid(rows: u32, cols: u32) -> Graph {
    let n = rows
        .checked_mul(cols)
        .expect("grid dimensions overflow u32");
    let mut g = Graph::with_vertices(n);
    for r in 0..rows {
        for c in 0..cols {
            let v = r * cols + c;
            if c + 1 < cols {
                g.add_edge(v, v + 1).expect("horizontal edge");
            }
            if r + 1 < rows {
                g.add_edge(v, v + cols).expect("vertical edge");
            }
        }
    }
    g
}

fn bench_karate_k3(c: &mut Criterion) {
    let g = karate();
    c.bench_function("ecc/karate-k3-normalize (34v 78e)", |b| {
        b.iter(|| ecc(&g, None, 3, false, true).unwrap());
    });
}

fn bench_karate_k4(c: &mut Criterion) {
    let g = karate();
    c.bench_function("ecc/karate-k4-normalize (34v 78e)", |b| {
        b.iter(|| ecc(&g, None, 4, false, true).unwrap());
    });
}

fn bench_grid_k3(c: &mut Criterion) {
    let g = grid(30, 30);
    c.bench_function("ecc/grid-30x30-k3-normalize (900v ~1740e)", |b| {
        b.iter(|| ecc(&g, None, 3, false, true).unwrap());
    });
}

criterion_group!(benches, bench_karate_k3, bench_karate_k4, bench_grid_k3);
criterion_main!(benches);
