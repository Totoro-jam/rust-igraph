//! Dense integer adjacency-matrix constructor benchmarks (ALGO-CN-029).
//!
//! Run: `cargo bench --bench bench_adjacency`. Results land under
//! `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-CN-029.json`.
//!
//! `igraph_adjacency(matrix, mode, loops)` traverses an `n × n` integer
//! matrix and emits one edge for every unit of weight after the
//! per-mode dispatch. Cost is `O(n²)` for the traversal plus
//! `O(|E|)` for the edge emission, so throughput is reported in *edges*
//! (the emitted edge count, pre-computed per shape).
//!
//! Shapes:
//!
//! * **Tiny `M3` family** (3 × 3, the three matrices from
//!   `tests/unit/igraph_adjacency.c`) — per-call overhead, including
//!   matrix-shape validation and the per-mode dispatch.
//! * **Sparse 128 × 128, single edge per row** — keeps emission count
//!   linear in `n` so the `O(n²)` traversal dominates.
//! * **Dense 128 × 128, unit weights** — `n²` emission cost, exercises
//!   the symmetric-mode triangular walks.
//! * **Dense 512 × 512, unit weights, MAX** — large undirected build
//!   bound by the per-pair `i < j` walk (~`n²/2` comparisons).

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{AdjacencyMode, LoopsMode, adjacency};

struct Shape {
    label: &'static str,
    matrix: Vec<Vec<i64>>,
    mode: AdjacencyMode,
    loops: LoopsMode,
    /// Pre-computed edge count for throughput reporting.
    edges: u64,
}

fn build_shapes() -> Vec<Shape> {
    // Upstream-C-test matrices (igraph_adjacency.c lines 75-89).
    let m3: Vec<Vec<i64>> = vec![vec![4, 2, 0], vec![3, 0, 4], vec![0, 5, 6]];
    let m3_sym: Vec<Vec<i64>> = vec![vec![4, 2, 0], vec![2, 0, 4], vec![0, 4, 6]];

    // Sparse pattern: one entry per row (a directed permutation-ish edge
    // set) so |E| = n on a 128 × 128 board.
    let n_sparse = 128usize;
    let mut sparse = vec![vec![0i64; n_sparse]; n_sparse];
    for i in 0..n_sparse {
        sparse[i][(i + 1) % n_sparse] = 1;
    }

    // Dense unit matrix 128 × 128 — every entry is 1.
    let n_dense = 128usize;
    let dense: Vec<Vec<i64>> = vec![vec![1i64; n_dense]; n_dense];

    // Larger dense unit matrix for MAX (drives the symmetric path).
    let n_large = 512usize;
    let large_sym: Vec<Vec<i64>> = vec![vec![1i64; n_large]; n_large];

    vec![
        // Tiny: M3, all seven modes — exercises per-mode dispatch.
        Shape {
            label: "m3_directed_no_loops",
            matrix: m3.clone(),
            mode: AdjacencyMode::Directed,
            loops: LoopsMode::NoLoops,
            edges: 14,
        },
        Shape {
            label: "m3_directed_loops_once",
            matrix: m3.clone(),
            mode: AdjacencyMode::Directed,
            loops: LoopsMode::Once,
            edges: 24,
        },
        Shape {
            label: "m3_undirected_loops_twice",
            matrix: m3_sym.clone(),
            mode: AdjacencyMode::Undirected,
            loops: LoopsMode::Twice,
            edges: 15, // 5 from diagonal halved + 10 off-diagonal triangle reads
        },
        Shape {
            label: "m3_max_no_loops",
            matrix: m3.clone(),
            mode: AdjacencyMode::Max,
            loops: LoopsMode::NoLoops,
            edges: 8,
        },
        Shape {
            label: "m3_min_no_loops",
            matrix: vec![vec![4, 2, 0], vec![3, 0, 5], vec![0, 4, 6]],
            mode: AdjacencyMode::Min,
            loops: LoopsMode::NoLoops,
            edges: 6,
        },
        Shape {
            label: "m3_plus_no_loops",
            matrix: m3.clone(),
            mode: AdjacencyMode::Plus,
            loops: LoopsMode::NoLoops,
            edges: 14,
        },
        Shape {
            label: "m3_upper_loops_once",
            matrix: m3.clone(),
            mode: AdjacencyMode::Upper,
            loops: LoopsMode::Once,
            edges: 16,
        },
        // Sparse 128 × 128 directed (|E| ≈ n).
        Shape {
            label: "n128_sparse_directed_no_loops",
            matrix: sparse,
            mode: AdjacencyMode::Directed,
            loops: LoopsMode::NoLoops,
            edges: n_sparse as u64,
        },
        // Dense 128 × 128, unit weights, directed (|E| = n²).
        Shape {
            label: "n128_dense_directed_no_loops",
            matrix: dense.clone(),
            mode: AdjacencyMode::Directed,
            loops: LoopsMode::NoLoops,
            edges: (n_dense * (n_dense - 1)) as u64,
        },
        // Dense 128 × 128, unit weights, MAX (|E| = n(n-1)/2).
        Shape {
            label: "n128_dense_max_no_loops",
            matrix: dense,
            mode: AdjacencyMode::Max,
            loops: LoopsMode::NoLoops,
            edges: (n_dense * (n_dense - 1) / 2) as u64,
        },
        // Large 512 × 512 dense MAX — stress the symmetric triangular walk.
        Shape {
            label: "n512_dense_max_no_loops",
            matrix: large_sym,
            mode: AdjacencyMode::Max,
            loops: LoopsMode::NoLoops,
            edges: (n_large * (n_large - 1) / 2) as u64,
        },
    ]
}

fn bench_adjacency(c: &mut Criterion) {
    let mut group = c.benchmark_group("adjacency");
    let shapes = build_shapes();
    for shape in &shapes {
        // Borrow rows once per shape; the cost we want to measure is the
        // adjacency() call, not the &[&[i64]] view materialisation.
        let rows: Vec<&[i64]> = shape.matrix.iter().map(Vec::as_slice).collect();
        group.throughput(Throughput::Elements(shape.edges));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &(&rows, shape.mode, shape.loops),
            |b, &(rows, mode, loops)| {
                b.iter(|| adjacency(rows, mode, loops).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_adjacency);
criterion_main!(benches);
