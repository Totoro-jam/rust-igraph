//! `minimum_size_separators` baseline benchmarks for ALGO-CN-031.
//!
//! Run: `cargo bench --bench bench_minimum_size_separators`.
//! Results land under `target/criterion/`; headline numbers recorded in
//! `.codefuse/tracking/perf/ALGO-CN-031.json`.
//!
//! Kanevsky's algorithm computes the vertex connectivity `k`, then drives a
//! max-flow / `all_st_mincuts` loop over the Even-Tarjan reduction for each of
//! the `k` highest-degree vertices against its non-neighbours. The cost is
//! therefore dominated by `O(k · n)` max-flow computations on the doubled graph
//! plus the (output-sensitive) min-cut enumeration. Workloads stay in the main
//! branch (`2 ≤ k ≤ n-2`) so the full pipeline runs:
//!   * **Cycle** `C_n` — connectivity 2; every non-adjacent vertex pair is a
//!     minimum separator, so the output grows quadratically while the per-cut
//!     work stays cheap (n = 20/40/80). Primary scaling driver.
//!   * **Square grid** `m × m` lattice — connectivity 2 with a richer flow
//!     structure than the cycle; stresses the reduction + relabelling on a
//!     denser, more clustered graph (m = 4/6/8 → 16/36/64 vertices).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{Graph, minimum_size_separators};

/// Undirected cycle `C_n` on `n` vertices: edges `i — (i+1) mod n`.
/// Connectivity 2.
fn cycle(n: u32) -> Graph {
    let mut g = Graph::new(n, false).expect("graph init");
    for i in 0..n {
        g.add_edge(i, (i + 1) % n).expect("edge in range");
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

fn bench_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("minimum_size_separators/cycle");
    for n in [20u32, 40, 80] {
        let g = cycle(n);
        group.throughput(Throughput::Elements(u64::from(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &g, |b, g| {
            b.iter(|| minimum_size_separators(g).expect("minimum_size_separators"));
        });
    }
    group.finish();
}

fn bench_grid(c: &mut Criterion) {
    let mut group = c.benchmark_group("minimum_size_separators/grid");
    for m in [4u32, 6, 8] {
        let g = grid(m);
        group.throughput(Throughput::Elements(u64::from(m * m)));
        group.bench_with_input(BenchmarkId::from_parameter(m), &g, |b, g| {
            b.iter(|| minimum_size_separators(g).expect("minimum_size_separators"));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_cycle, bench_grid);
criterion_main!(benches);
