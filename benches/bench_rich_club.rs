//! Rich-club coefficient sequence (ALGO-PR-040) benchmark.
//!
//! Run: `cargo bench --bench bench_rich_club`. Numbers land in
//! `.codefuse/tracking/perf/ALGO-PR-040.json`.
//!
//! `rich_club_sequence` is `O(|V| + |E|)`: one inverted-permutation
//! build, one pass over the edge list to bucket each edge into the step
//! at which its first endpoint disappears, one reverse cumulative sweep,
//! and an optional per-step normalization that is `O(|V|)`. The three
//! scenarios stress the dominant `|E|` term across sparsity regimes:
//!
//! - `rich_club/karate-inorder-normalized` — Zachary karate (34v / 78e).
//!   Tiny graph; mostly the per-edge bucket fill and the |V|-length
//!   reverse sum. Sets the micro-bench baseline.
//! - `rich_club/grid-30x30-inorder-normalized` — 30×30 grid (900v /
//!   ~1740e). Sparse but large; isolates the linear-in-`|V|`
//!   normalization sweep.
//! - `rich_club/gnp-1000-p005-inorder-normalized` — G(1000, 0.05) random
//!   graph (~25 000 edges expected). Dense regime; isolates the
//!   linear-in-`|E|` bucket-fill loop.

use std::fs::File;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_igraph::{Graph, erdos_renyi_gnp, read_edgelist, rich_club_sequence};

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

fn in_order(n: u32) -> Vec<u32> {
    (0..n).collect()
}

fn bench_karate(c: &mut Criterion) {
    let g = karate();
    let order = in_order(g.vcount());
    c.bench_function("rich_club/karate-inorder-normalized (34v 78e)", |b| {
        b.iter(|| rich_club_sequence(&g, None, &order, true, false, false).unwrap());
    });
}

fn bench_grid(c: &mut Criterion) {
    let g = grid(30, 30);
    let order = in_order(g.vcount());
    c.bench_function(
        "rich_club/grid-30x30-inorder-normalized (900v ~1740e)",
        |b| b.iter(|| rich_club_sequence(&g, None, &order, true, false, false).unwrap()),
    );
}

fn bench_gnp(c: &mut Criterion) {
    let g = erdos_renyi_gnp(1000, 0.05, false, false, 0xCAFE_F00D).expect("build G(n, p)");
    let order = in_order(g.vcount());
    let label = format!(
        "rich_club/gnp-1000-p005-inorder-normalized ({}v {}e)",
        g.vcount(),
        g.ecount()
    );
    c.bench_function(&label, |b| {
        b.iter(|| rich_club_sequence(&g, None, &order, true, false, false).unwrap());
    });
}

criterion_group!(benches, bench_karate, bench_grid, bench_gnp);
criterion_main!(benches);
