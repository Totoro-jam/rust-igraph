//! ALGO-CN-024 example: planar hexagonal (honeycomb) lattice
//! (`igraph_hexagonal_lattice`).
//!
//! Walks every shape branch (triangle, quasi-rectangle, hexagon) and
//! every flag combination, asserting the structural invariants documented
//! in upstream igraph: every vertex carries degree at most three (the
//! hexagonal lattice is the planar dual of [`rust_igraph::triangular_lattice`]).
//!
//! Truth table for `hexagonal_lattice(dims, directed, mutual)`:
//!
//! | dims           | directed | mutual | vcount | ecount | shape                    |
//! |----------------|----------|--------|--------|--------|--------------------------|
//! | `[1]`          | true     | false  | 6      | 6      | single hexagon (`C_6`)     |
//! | `[5]`          | true     | false  | 46     | 60     | triangle side 5          |
//! | `[2, 2]`       | false    | false  | 16     | 19     | 2x2 quasi-rectangle      |
//! | `[2, 2]`       | true     | true   | 16     | 38     | 2x2 dir+mutual           |
//! | `[4, 5]`       | true     | true   | 58     | 154    | 4x5 rectangle dir+mutual |
//! | `[3, 4, 5]`    | false    | true   | 94     | 129    | hexagon (3,4,5)          |
//! | `[3, 0]`       | false    | false  | 0      | 0      | empty (zero dim guard)   |
//!
//! Vertex indexing follows the upstream `lex_ordering = false` rule:
//! the lattice site at `(i, j)` carries id
//! `prefix_sum[j] + (i - row_start[j])`. Edges connect `(i, j)` to
//! `(i + 1, j)` always and to `(i - 1, j + 1)` only when `i` is odd.
//!
//! Run: `cargo run --example hexagonal_lattice_demo`.

use rust_igraph::{Graph, hexagonal_lattice};

fn dump_edges(g: &Graph) -> Vec<(u32, u32)> {
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    (0..m)
        .map(|e| g.edge(e).expect("edge id in bounds for example"))
        .collect()
}

fn print_summary(label: &str, g: &Graph) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
    println!("  edges    = {:?}", dump_edges(g));
}

fn max_degree(g: &Graph) -> usize {
    (0..g.vcount())
        .map(|v| g.degree(v).expect("vertex in range"))
        .max()
        .unwrap_or(0)
}

fn main() {
    // Single hexagon: dims=[1] yields a `C_6` with 6 vertices and 6 edges.
    let single_hex = hexagonal_lattice(&[1], true, false).expect("single hex");
    print_summary("dims=[1] directed — single hexagon (`C_6`)", &single_hex);
    assert_eq!(single_hex.vcount(), 6);
    assert_eq!(single_hex.ecount(), 6);
    assert!(single_hex.is_directed());
    assert!(max_degree(&single_hex) <= 3);

    // Triangle side 5 (canonical upstream .out fixture): 46 vertices, 60 arcs.
    let triangle_5 = hexagonal_lattice(&[5], true, false).expect("triangle 5");
    print_summary("dims=[5] directed — triangle side 5", &triangle_5);
    assert_eq!(triangle_5.vcount(), 46);
    assert_eq!(triangle_5.ecount(), 60);
    assert!(max_degree(&triangle_5) <= 3);

    // 2x2 quasi-rectangle (python-igraph testHexagonalLattice fixture).
    let rect_2x2 = hexagonal_lattice(&[2, 2], false, false).expect("2x2 rect");
    print_summary("dims=[2, 2] undirected — 2x2 rectangle", &rect_2x2);
    assert_eq!(rect_2x2.vcount(), 16);
    assert_eq!(rect_2x2.ecount(), 19);
    assert!(max_degree(&rect_2x2) <= 3);

    // 2x2 rectangle directed + mutual: edge count doubles.
    let rect_2x2_mut = hexagonal_lattice(&[2, 2], true, true).expect("2x2 rect mut");
    print_summary("dims=[2, 2] directed+mutual — 2x2 rectangle", &rect_2x2_mut);
    assert!(rect_2x2_mut.is_directed());
    assert_eq!(rect_2x2_mut.ecount(), 38);

    // 4x5 rectangle directed + mutual (canonical upstream .out fixture).
    let rect_4x5 = hexagonal_lattice(&[4, 5], true, true).expect("4x5 rect mut");
    print_summary("dims=[4, 5] directed+mutual — 4x5 rectangle", &rect_4x5);
    assert_eq!(rect_4x5.vcount(), 58);
    assert_eq!(rect_4x5.ecount(), 154);
    assert!(max_degree(&rect_4x5) <= 6); // each arc counted twice → in+out ≤ 6

    // Hexagonal lattice (3, 4, 5) — canonical upstream fixture; undirected
    // silently collapses the `mutual` flag, so the result has 129 edges.
    let hex = hexagonal_lattice(&[3, 4, 5], false, true).expect("hex 3,4,5");
    print_summary("dims=[3, 4, 5] — hexagonal lattice", &hex);
    assert_eq!(hex.vcount(), 94);
    assert_eq!(hex.ecount(), 129);
    assert!(max_degree(&hex) <= 3);

    // Zero in any dim collapses to the empty graph (upstream guard).
    let empty = hexagonal_lattice(&[3, 0], false, false).expect("empty");
    print_summary("dims=[3, 0] — empty (zero-dim guard)", &empty);
    assert_eq!(empty.vcount(), 0);
    assert_eq!(empty.ecount(), 0);

    println!("\nall structural invariants OK ✓");
}
