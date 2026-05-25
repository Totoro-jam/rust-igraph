//! ALGO-CN-030 example: dense **real-valued** adjacency-matrix
//! constructor (`igraph_weighted_adjacency`).
//!
//! Sibling of [`adjacency`](rust_igraph::adjacency): instead of an integer
//! matrix whose entries are edge multiplicities, this constructor takes an
//! `n × n` matrix of `f64` weights, emits exactly one edge per qualifying
//! cell, and returns a `(Graph, Vec<f64>)` pair where the second component
//! is the per-edge weight vector in emission order.
//!
//! Mode + loops semantics match upstream igraph (see
//! `references/igraph/src/constructors/adjacency.c`):
//!
//! * `Directed` walks the full matrix column-major.
//! * `Undirected` walks the lower triangle row-major and requires
//!   bit-equal symmetry — except that pairs where both sides are `NaN`
//!   are accepted (matching upstream's NaN-tolerant check).
//! * `Max` / `Min` reduce per off-diagonal pair using `max(A[i][j],
//!   A[j][i])` / `min(...)` with NaN propagation.
//! * `Plus` sums the two halves: `A[i][j] + A[j][i]`.
//! * `Upper` / `Lower` keep only one triangle.
//! * Loop handling: `NoLoops` drops the diagonal, `Twice` halves diagonal
//!   weights (except for `Directed` / `Upper` / `Lower` which collapse
//!   `Twice` → `Once`).
//!
//! Run: `cargo run --example weighted_adjacency_demo`.
//!
//! [`adjacency`]: rust_igraph::adjacency

use rust_igraph::{AdjacencyMode, Graph, LoopsMode, weighted_adjacency};

fn dump_edges(g: &Graph) -> Vec<(u32, u32)> {
    let m = u32::try_from(g.ecount()).expect("ecount fits u32 in example");
    (0..m)
        .map(|e| g.edge(e).expect("edge id in bounds for example"))
        .collect()
}

fn print_summary(label: &str, g: &Graph, w: &[f64]) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
    println!("  edges    = {:?}", dump_edges(g));
    println!("  weights  = {w:?}");
}

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

fn main() {
    // 3x3 sample matrix — asymmetric.
    let m3: &[&[f64]] = &[&[2.0, 0.5, 0.0], &[1.5, 0.0, 2.0], &[0.0, 2.5, 3.0]];

    // Symmetric counterpart for Undirected.
    let m3_sym: &[&[f64]] = &[&[2.0, 0.5, 0.0], &[0.5, 0.0, 2.0], &[0.0, 2.0, 3.0]];

    // DIRECTED, NoLoops: column-major walk drops the diagonal. Four
    // non-zero off-diagonal entries -> 4 edges, weights in column-major
    // visit order = [1.5, 0.5, 2.5, 2.0].
    let (g, w) = weighted_adjacency(m3, AdjacencyMode::Directed, LoopsMode::NoLoops).expect("ok");
    print_summary("M3 directed no_loops", &g, &w);
    assert_eq!(g.ecount(), 4);
    assert!(g.is_directed());

    // DIRECTED, Once: include diagonal as self-loops (2.0 and 3.0).
    let (g, w) = weighted_adjacency(m3, AdjacencyMode::Directed, LoopsMode::Once).expect("ok");
    print_summary("M3 directed loops_once", &g, &w);
    assert_eq!(g.ecount(), 6);
    assert!(w.contains(&2.0) && w.contains(&3.0));

    // DIRECTED, Twice: collapses to Once for Directed -> diagonal NOT
    // halved. Same edge multiset & weights as the Once case.
    let (g, w_twice) =
        weighted_adjacency(m3, AdjacencyMode::Directed, LoopsMode::Twice).expect("ok");
    print_summary("M3 directed loops_twice (collapses to once)", &g, &w_twice);
    assert_eq!(w_twice, w);

    // UNDIRECTED requires symmetry; M3_SYM passes. Twice halves diagonal
    // weights: 2.0 -> 1.0, 3.0 -> 1.5.
    let (g, w) =
        weighted_adjacency(m3_sym, AdjacencyMode::Undirected, LoopsMode::Twice).expect("ok");
    print_summary("M3_SYM undirected loops_twice", &g, &w);
    assert!(!g.is_directed());
    assert!(approx_eq(w[0], 1.0), "diag 2.0 halved to 1.0");

    // MAX, NoLoops: each off-diagonal pair contributes max(A[i][j],
    // A[j][i]). On M3: (0,1) max(0.5, 1.5)=1.5; (0,2) max(0, 0)=0 skip;
    // (1,2) max(2.0, 2.5)=2.5. -> 2 edges.
    let (g, w) = weighted_adjacency(m3, AdjacencyMode::Max, LoopsMode::NoLoops).expect("ok");
    print_summary("M3 max no_loops", &g, &w);
    assert_eq!(g.ecount(), 2);

    // MIN, NoLoops: min(A[i][j], A[j][i]) per pair. (0,1)=0.5;
    // (0,2)=0 skip; (1,2)=2.0. -> 2 edges with smaller weights.
    let (g, w) = weighted_adjacency(m3, AdjacencyMode::Min, LoopsMode::NoLoops).expect("ok");
    print_summary("M3 min no_loops", &g, &w);
    assert_eq!(g.ecount(), 2);

    // PLUS, NoLoops: A[i][j] + A[j][i] per pair. (0,1)=0.5+1.5=2.0;
    // (0,2)=0; (1,2)=2.0+2.5=4.5. -> 2 edges.
    let (g, w) = weighted_adjacency(m3, AdjacencyMode::Plus, LoopsMode::NoLoops).expect("ok");
    print_summary("M3 plus no_loops", &g, &w);
    assert_eq!(g.ecount(), 2);

    // UPPER, Twice: collapses to Once for Upper. Strict upper triangle:
    // (0,1)=0.5; (1,2)=2.0. Diagonal entries kept as self-loops,
    // un-halved. -> 4 edges.
    let (g, w) = weighted_adjacency(m3, AdjacencyMode::Upper, LoopsMode::Twice).expect("ok");
    print_summary("M3 upper loops_twice (collapses to once)", &g, &w);
    assert_eq!(g.ecount(), 4);

    // LOWER, NoLoops: strict lower triangle. (1,0)=1.5; (2,1)=2.5;
    // (2,0)=0 skip. -> 2 edges. The Graph canonicalises undirected
    // edges so (1,0) and (2,1) read back as (0,1) and (1,2).
    let (g, w) = weighted_adjacency(m3, AdjacencyMode::Lower, LoopsMode::NoLoops).expect("ok");
    print_summary("M3 lower no_loops", &g, &w);
    assert_eq!(g.ecount(), 2);

    // NaN-tolerant undirected: both sides NaN counts as symmetric.
    let m_nan: &[&[f64]] = &[&[0.0, f64::NAN], &[f64::NAN, 0.0]];
    let (g, w) =
        weighted_adjacency(m_nan, AdjacencyMode::Undirected, LoopsMode::NoLoops).expect("ok");
    print_summary("symmetric-NaN undirected no_loops", &g, &w);
    assert_eq!(g.ecount(), 1);
    assert!(w[0].is_nan());

    // Empty 0 x 0 -> empty graph.
    let empty: &[&[f64]] = &[];
    let (g, w) =
        weighted_adjacency(empty, AdjacencyMode::Directed, LoopsMode::NoLoops).expect("ok");
    print_summary("empty 0x0 directed", &g, &w);
    assert_eq!(g.vcount(), 0);
    assert_eq!(g.ecount(), 0);

    println!("\nall mode/loops dispatches behave as upstream igraph");
}
