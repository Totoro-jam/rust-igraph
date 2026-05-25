//! ALGO-CN-023 example: planar triangular lattice
//! (`igraph_triangular_lattice`).
//!
//! Walks each shape branch (triangle, rectangle, hexagon) and every
//! flag combination supported by the constructor, asserting the
//! structural invariants documented in upstream igraph.
//!
//! Truth table for `triangular_lattice(dims, directed, mutual)`:
//!
//! | dims           | directed | mutual | vcount | ecount | shape                  |
//! |----------------|----------|--------|--------|--------|------------------------|
//! | `[1]`          | true     | false  | 1      | 0      | single vertex          |
//! | `[3]`          | false    | false  | 6      | 9      | triangle side 3        |
//! | `[5]`          | true     | false  | 15     | 30     | triangle side 5        |
//! | `[2, 2]`       | false    | false  | 4      | 5      | 2x2 quasi-rectangle    |
//! | `[2, 2]`       | true     | true   | 4      | 10     | 2x2 dir+mutual         |
//! | `[3, 4, 5]`    | false    | true   | 36     | 87     | hexagon (3,4,5)        |
//! | `[3, 0]`       | false    | false  | 0      | 0      | empty (zero dim guard) |
//!
//! Vertex indexing follows the upstream `lex_ordering = false` rule:
//! the lattice site at `(i, j)` carries id
//! `prefix_sum[j] + (i - row_start[j])`. Every interior vertex has
//! degree at most six (the triangular lattice is the planar dual of
//! the hexagonal lattice).
//!
//! Run: `cargo run --example triangular_lattice_demo`.

use rust_igraph::{Graph, triangular_lattice};

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
    // Triangle side 1: a single isolated vertex (directed flag preserved).
    let triangle_1 = triangular_lattice(&[1], true, false).expect("triangle 1");
    print_summary("dims=[1] directed — single vertex", &triangle_1);
    assert_eq!(triangle_1.vcount(), 1);
    assert_eq!(triangle_1.ecount(), 0);
    assert!(triangle_1.is_directed());

    // Triangle side 3: 6 vertices, 9 edges, max degree 6 reached at
    // the interior vertex (id 3).
    let triangle_3 = triangular_lattice(&[3], false, false).expect("triangle 3");
    print_summary("dims=[3] undirected — triangle side 3", &triangle_3);
    assert_eq!(triangle_3.vcount(), 6);
    assert_eq!(triangle_3.ecount(), 9);
    assert!(max_degree(&triangle_3) <= 6);

    // Triangle side 5 (the canonical .out fixture): 15 vertices, 30 arcs.
    let triangle_5 = triangular_lattice(&[5], true, false).expect("triangle 5");
    print_summary("dims=[5] directed — triangle side 5", &triangle_5);
    assert_eq!(triangle_5.vcount(), 15);
    assert_eq!(triangle_5.ecount(), 30);

    // 2x2 quasi-rectangle: matches python-igraph testTriangularLattice.
    let rect_2x2 = triangular_lattice(&[2, 2], false, false).expect("2x2 rect");
    print_summary("dims=[2, 2] undirected — 2x2 rectangle", &rect_2x2);
    assert_eq!(rect_2x2.vcount(), 4);
    assert_eq!(rect_2x2.ecount(), 5);

    // 2x2 rectangle directed + mutual: edge count doubles.
    let rect_2x2_mut = triangular_lattice(&[2, 2], true, true).expect("2x2 rect mut");
    print_summary("dims=[2, 2] directed+mutual — 2x2 rectangle", &rect_2x2_mut);
    assert!(rect_2x2_mut.is_directed());
    assert_eq!(rect_2x2_mut.ecount(), 10);

    // Hexagonal lattice (3, 4, 5): canonical upstream fixture.
    let hex = triangular_lattice(&[3, 4, 5], false, true).expect("hex 3,4,5");
    print_summary("dims=[3, 4, 5] — hexagonal lattice", &hex);
    assert_eq!(hex.vcount(), 36);
    assert_eq!(hex.ecount(), 87);
    assert!(max_degree(&hex) <= 6);

    // Zero in any dim collapses to the empty graph (upstream guard).
    let empty = triangular_lattice(&[3, 0], false, false).expect("empty");
    print_summary("dims=[3, 0] — empty (zero-dim guard)", &empty);
    assert_eq!(empty.vcount(), 0);
    assert_eq!(empty.ecount(), 0);

    println!("\nall structural invariants OK ✓");
}
