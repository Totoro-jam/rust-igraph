//! GMRES `PageRank` (ALGO-PR-011c) benchmark — head-to-head vs. power
//! iteration (ALGO-PR-011).
//!
//! Run: `cargo bench --bench bench_pagerank_linsys`. Numbers land in
//! `.codefuse/tracking/perf/ALGO-PR-011c.json`.
//!
//! Both backends solve the same fixed point `(I - α · Mᵀ) · pr =
//! (1-α)/N · 1` with `α = 0.85`. Per-matvec cost is identical
//! (`O(|V| + |E|)` plus a dangling-sum sweep) — what differs is the
//! number of matvecs:
//!
//! - PR-011 power iteration: drives the residual by a factor of `α` per
//!   step, so it needs `O(log(1/ε) / log(1/α)) ≈ 142` matvecs to hit
//!   `ε = 1e-10`.
//! - PR-011c GMRES(30, restart 50): minimises the residual over the
//!   full Krylov subspace, so it usually converges in one restart
//!   cycle (≤ 30 matvecs).
//!
//! The three scenarios stress sparsity regimes the same way as
//! `bench_rich_club`:
//!
//! - `pagerank_linsys/karate (34v 78e)` — tiny graph, matvec overhead
//!   dominates so the difference is small.
//! - `pagerank_linsys/grid-30x30 (900v ~1740e)` — sparse but large.
//! - `pagerank_linsys/gnp-1000-p005 (~25 000 e)` — denser, larger
//!   spectral gap → GMRES should pull ahead.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, erdos_renyi_gnp, pagerank, pagerank_linsys, read_edgelist};

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

fn bench_karate_gmres(c: &mut Criterion) {
    let g = karate();
    c.bench_function("pagerank_linsys/karate (34v 78e)", |b| {
        b.iter(|| pagerank_linsys(&g).unwrap());
    });
}

fn bench_karate_power(c: &mut Criterion) {
    let g = karate();
    c.bench_function("pagerank/karate-power (34v 78e)", |b| {
        b.iter(|| pagerank(&g).unwrap());
    });
}

fn bench_grid_gmres(c: &mut Criterion) {
    let g = grid(30, 30);
    c.bench_function("pagerank_linsys/grid-30x30 (900v ~1740e)", |b| {
        b.iter(|| pagerank_linsys(&g).unwrap());
    });
}

fn bench_grid_power(c: &mut Criterion) {
    let g = grid(30, 30);
    c.bench_function("pagerank/grid-30x30-power (900v ~1740e)", |b| {
        b.iter(|| pagerank(&g).unwrap());
    });
}

fn bench_gnp_gmres(c: &mut Criterion) {
    let g = erdos_renyi_gnp(1000, 0.05, false, false, 0xCAFE_F00D).expect("build G(n, p)");
    let label = format!(
        "pagerank_linsys/gnp-1000-p005 ({}v {}e)",
        g.vcount(),
        g.ecount()
    );
    c.bench_function(&label, |b| {
        b.iter(|| pagerank_linsys(&g).unwrap());
    });
}

fn bench_gnp_power(c: &mut Criterion) {
    let g = erdos_renyi_gnp(1000, 0.05, false, false, 0xCAFE_F00D).expect("build G(n, p)");
    let label = format!(
        "pagerank/gnp-1000-p005-power ({}v {}e)",
        g.vcount(),
        g.ecount()
    );
    c.bench_function(&label, |b| {
        b.iter(|| pagerank(&g).unwrap());
    });
}

criterion_group!(
    benches,
    bench_karate_gmres,
    bench_karate_power,
    bench_grid_gmres,
    bench_grid_power,
    bench_gnp_gmres,
    bench_gnp_power
);
criterion_main!(benches);
