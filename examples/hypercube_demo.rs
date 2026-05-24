//! ALGO-CN-007 example: n-dimensional hypercube `Q_n` (`igraph_hypercube`).
//!
//! The hypercube graph `Q_n` has `2^n` vertices and `2^(n-1) · n` edges.
//! Vertices are zero-based IDs in `[0, 2^n)`; two vertices share an edge
//! iff their binary representations differ in **exactly one bit**.
//!
//! Truth table for `hypercube(n, directed)`:
//!
//! | n | directed | vcount | ecount | shape                           |
//! |---|----------|--------|--------|---------------------------------|
//! | 0 | false    | 1      | 0      | singleton                       |
//! | 1 | false    | 2      | 1      | `K_2`                           |
//! | 2 | false    | 4      | 4      | 4-cycle                         |
//! | 3 | false    | 8      | 12     | the cube (3-regular, bipartite) |
//! | 3 | true     | 8      | 12     | cube, low → high orientation    |
//! | 4 | false    | 16     | 32     | tesseract                       |
//!
//! Properties demonstrated:
//!
//! * `n`-regular: every vertex has degree `n`.
//! * Bipartite: vertices split by parity of `popcount(id)`.
//! * Canonical edge ordering: every emitted pair has `u < v` (the directed
//!   variant stores them as `u → v` arcs).
//!
//! Run: `cargo run --example hypercube_demo`.

#![allow(clippy::cast_possible_truncation)]

use rust_igraph::{Graph, hypercube};

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

fn assert_n_regular(g: &Graph, n: u32) {
    for v in 0..g.vcount() {
        let d = g.degree(v).expect("vertex in range");
        assert_eq!(d, n as usize, "Q_{n} vertex {v} should have degree {n}");
    }
}

fn assert_bipartite_by_popcount(g: &Graph) {
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32");
    for e in 0..m {
        let (u, v) = g.edge(e).expect("edge in range");
        assert_ne!(
            u.count_ones() % 2,
            v.count_ones() % 2,
            "edge ({u}, {v}) must cross popcount parity"
        );
    }
}

fn main() {
    // Q_2 — the 4-cycle.
    let q2 = hypercube(2, false).expect("Q_2");
    print_summary("Q_2 (undirected) — 4-cycle", &q2);
    assert_n_regular(&q2, 2);

    // Q_3 — 8-vertex cube.
    let q3 = hypercube(3, false).expect("Q_3");
    print_summary("Q_3 (undirected) — the cube", &q3);
    assert_n_regular(&q3, 3);
    assert_bipartite_by_popcount(&q3);

    // Q_3 directed — same canonical edges, stored as low → high arcs.
    let q3d = hypercube(3, true).expect("Q_3 directed");
    print_summary("Q_3 (directed)   — low → high orientation", &q3d);

    // Q_4 — tesseract.
    let q4 = hypercube(4, false).expect("Q_4");
    print_summary("Q_4 (undirected) — the tesseract", &q4);
    assert_n_regular(&q4, 4);
    assert_bipartite_by_popcount(&q4);

    println!("\nall structural invariants OK ✓");
}
