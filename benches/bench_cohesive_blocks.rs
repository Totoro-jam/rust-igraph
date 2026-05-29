//! `cohesive_blocks` baseline benchmarks for ALGO-CN-032.
//!
//! Run: `cargo bench --bench bench_cohesive_blocks`.
//! Results land under `target/criterion/`; headline numbers recorded in
//! `.codefuse/tracking/perf/ALGO-CN-032.json`.
//!
//! Cohesive blocking (Moody-White 2003) recursively removes minimum-size
//! vertex separators and keeps each subgraph whose connectivity is strictly
//! higher than its parent's. Every queue entry pays a `vertex_connectivity`
//! plus a full `minimum_size_separators` (Kanevsky) call, so the cost is
//! dominated by the per-block max-flow / min-cut enumeration multiplied by the
//! number of blocks discovered. Two workload families:
//!   * **Clique path** — `p` copies of `K_4` chained by single bridge edges.
//!     Each clique is a cohesion-3 block inside the cohesion-1 whole, so the
//!     block count grows linearly in `p` while each block stays small
//!     (p = 3/5/8 → 12/20/32 vertices). Drives block-count scaling.
//!   * **Square grid** `m × m` lattice — connectivity 2 with a clustered flow
//!     structure; stresses the separator + reduction pipeline on each block
//!     (m = 3/4/5 → 9/16/25 vertices).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, cohesive_blocks};

/// `p` disjoint `K_4` cliques chained by a single bridge edge between
/// consecutive cliques. Vertices `4*i .. 4*i+3` form clique `i`; a bridge
/// joins `4*i+3` to `4*(i+1)`.
fn clique_path(p: u32) -> Graph {
    let mut g = Graph::new(p * 4, false).expect("graph init");
    for i in 0..p {
        let base = i * 4;
        for a in 0..4 {
            for b in (a + 1)..4 {
                g.add_edge(base + a, base + b).expect("edge in range");
            }
        }
        if i + 1 < p {
            g.add_edge(base + 3, base + 4).expect("bridge in range");
        }
    }
    g
}

/// Undirected `m × m` square grid (non-periodic lattice). Vertex `(r, c)`
/// is indexed `r * m + c`; connectivity 2.
fn grid(m: u32) -> Graph {
    let mut g = Graph::new(m * m, false).expect("graph init");
    for r in 0..m {
        for c in 0..m {
            let v = r * m + c;
            if c + 1 < m {
                g.add_edge(v, v + 1).expect("edge in range");
            }
            if r + 1 < m {
                g.add_edge(v, v + m).expect("edge in range");
            }
        }
    }
    g
}

fn bench_clique_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("cohesive_blocks/clique_path");
    for p in [3u32, 5, 8] {
        let g = clique_path(p);
        group.throughput(Throughput::Elements(u64::from(p * 4)));
        group.bench_with_input(BenchmarkId::from_parameter(p), &g, |b, g| {
            b.iter(|| cohesive_blocks(g).expect("cohesive_blocks"));
        });
    }
    group.finish();
}

fn bench_grid(c: &mut Criterion) {
    let mut group = c.benchmark_group("cohesive_blocks/grid");
    for m in [3u32, 4, 5] {
        let g = grid(m);
        group.throughput(Throughput::Elements(u64::from(m * m)));
        group.bench_with_input(BenchmarkId::from_parameter(m), &g, |b, g| {
            b.iter(|| cohesive_blocks(g).expect("cohesive_blocks"));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_clique_path, bench_grid);
criterion_main!(benches);
