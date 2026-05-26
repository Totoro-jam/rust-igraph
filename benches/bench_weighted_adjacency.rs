//! Dense real-valued adjacency-matrix constructor benchmarks (ALGO-CN-030).
//!
//! Run: `cargo bench --bench bench_weighted_adjacency`. Results land
//! under `target/criterion/`. A snapshot of the baseline lives at
//! `.codefuse/tracking/perf/ALGO-CN-030.json`.
//!
//! Sibling of `bench_adjacency` for the `igraph_weighted_adjacency`
//! constructor. Cost shape is identical (`O(n²)` traversal + `O(|E|)`
//! emission) but each emit writes a single `f64` weight rather than
//! pushing N copies of an integer multiplicity, so throughput is
//! reported in **edges emitted**.
//!
//! Shapes:
//!
//! * **Tiny `M3` family** (3 × 3) — per-call overhead, exercises the
//!   per-mode dispatch and the symmetric-matrix check for Undirected.
//! * **Sparse 128 × 128 (single non-zero per row)** — the `O(n²)`
//!   traversal dominates over emission.
//! * **Dense 128 × 128, weight 1.0** — `n²` emission cost, exercises
//!   the symmetric-mode triangular walks.
//! * **Dense 512 × 512, MAX** — large undirected build bound by the
//!   per-pair `j < i` walk.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rust_igraph::{AdjacencyMode, LoopsMode, weighted_adjacency};

struct Shape {
    label: &'static str,
    matrix: Vec<Vec<f64>>,
    mode: AdjacencyMode,
    loops: LoopsMode,
    /// Pre-computed emitted edge count for throughput reporting.
    edges: u64,
}

fn build_shapes() -> Vec<Shape> {
    // Asymmetric 3 × 3 sample, mirroring the M3 used by the integer
    // CN-029 bench so the two snapshots are directly comparable.
    let m3: Vec<Vec<f64>> = vec![
        vec![2.0, 0.5, 0.0],
        vec![1.5, 0.0, 2.0],
        vec![0.0, 2.5, 3.0],
    ];
    let m3_sym: Vec<Vec<f64>> = vec![
        vec![2.0, 0.5, 0.0],
        vec![0.5, 0.0, 2.0],
        vec![0.0, 2.0, 3.0],
    ];

    // Sparse pattern: one non-zero per row -> |E| = n on a 128 board.
    let n_sparse = 128usize;
    let mut sparse = vec![vec![0.0f64; n_sparse]; n_sparse];
    for (i, row) in sparse.iter_mut().enumerate() {
        row[(i + 1) % n_sparse] = 1.0;
    }

    // Dense 128 × 128 unit matrix.
    let n_dense = 128usize;
    let dense: Vec<Vec<f64>> = vec![vec![1.0f64; n_dense]; n_dense];

    // Larger dense unit matrix for MAX.
    let n_large = 512usize;
    let large_sym: Vec<Vec<f64>> = vec![vec![1.0f64; n_large]; n_large];

    vec![
        // M3 family — per-mode dispatch overhead.
        Shape {
            label: "m3_directed_no_loops",
            matrix: m3.clone(),
            mode: AdjacencyMode::Directed,
            loops: LoopsMode::NoLoops,
            // 4 non-zero off-diagonal entries.
            edges: 4,
        },
        Shape {
            label: "m3_directed_loops_once",
            matrix: m3.clone(),
            mode: AdjacencyMode::Directed,
            loops: LoopsMode::Once,
            // 4 off-diagonal + 2 diagonal non-zeros.
            edges: 6,
        },
        Shape {
            label: "m3_undirected_loops_twice",
            matrix: m3_sym.clone(),
            mode: AdjacencyMode::Undirected,
            loops: LoopsMode::Twice,
            // lower triangle non-zeros: (1,0), (2,1); diag non-zeros at 0 and 2.
            edges: 4,
        },
        Shape {
            label: "m3_max_no_loops",
            matrix: m3.clone(),
            mode: AdjacencyMode::Max,
            loops: LoopsMode::NoLoops,
            // pairs (0,1) max=1.5, (0,2) max=0 skip, (1,2) max=2.5
            edges: 2,
        },
        Shape {
            label: "m3_min_no_loops",
            matrix: m3.clone(),
            mode: AdjacencyMode::Min,
            loops: LoopsMode::NoLoops,
            // (0,1) min=0.5, (0,2) min=0 skip, (1,2) min=2.0
            edges: 2,
        },
        Shape {
            label: "m3_plus_no_loops",
            matrix: m3.clone(),
            mode: AdjacencyMode::Plus,
            loops: LoopsMode::NoLoops,
            // (0,1) 0.5+1.5=2.0, (0,2) 0, (1,2) 2.0+2.5=4.5
            edges: 2,
        },
        Shape {
            label: "m3_upper_loops_once",
            matrix: m3.clone(),
            mode: AdjacencyMode::Upper,
            loops: LoopsMode::Once,
            // upper triangle non-zeros: (0,1), (1,2); diag non-zeros at 0 and 2.
            edges: 4,
        },
        // Sparse 128 × 128 directed.
        Shape {
            label: "n128_sparse_directed_no_loops",
            matrix: sparse,
            mode: AdjacencyMode::Directed,
            loops: LoopsMode::NoLoops,
            edges: n_sparse as u64,
        },
        // Dense 128 × 128 directed.
        Shape {
            label: "n128_dense_directed_no_loops",
            matrix: dense.clone(),
            mode: AdjacencyMode::Directed,
            loops: LoopsMode::NoLoops,
            edges: (n_dense * (n_dense - 1)) as u64,
        },
        // Dense 128 × 128 MAX.
        Shape {
            label: "n128_dense_max_no_loops",
            matrix: dense,
            mode: AdjacencyMode::Max,
            loops: LoopsMode::NoLoops,
            edges: (n_dense * (n_dense - 1) / 2) as u64,
        },
        // Large 512 × 512 dense MAX.
        Shape {
            label: "n512_dense_max_no_loops",
            matrix: large_sym,
            mode: AdjacencyMode::Max,
            loops: LoopsMode::NoLoops,
            edges: (n_large * (n_large - 1) / 2) as u64,
        },
    ]
}

fn bench_weighted_adjacency(c: &mut Criterion) {
    let mut group = c.benchmark_group("weighted_adjacency");
    let shapes = build_shapes();
    for shape in &shapes {
        let rows: Vec<&[f64]> = shape.matrix.iter().map(Vec::as_slice).collect();
        group.throughput(Throughput::Elements(shape.edges));
        group.bench_with_input(
            BenchmarkId::from_parameter(shape.label),
            &(&rows, shape.mode, shape.loops),
            |b, &(rows, mode, loops)| {
                b.iter(|| weighted_adjacency(rows, mode, loops).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_weighted_adjacency);
criterion_main!(benches);
