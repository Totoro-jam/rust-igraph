//! ALGO-CN-008 example: d-dimensional Hamming graph `H(n, q)` (`igraph_hamming`).
//!
//! `H(n, q)` has `q^n` vertices indexed by all length-`n` strings in
//! `{0, 1, …, q-1}^n`. Vertex IDs follow the little-endian base-`q`
//! convention: the string `(x_0, x_1, …, x_{n-1})` maps to
//! `Σ_i x_i · q^i`. Two vertices share an edge iff their strings differ
//! in **exactly one position**.
//!
//! Truth table for `hamming(n, q, directed)`:
//!
//! | n | q | directed | vcount | ecount | shape                            |
//! |---|---|----------|--------|--------|----------------------------------|
//! | 0 | * | false    | 1      | 0      | singleton (any `q`, even `q=0`)  |
//! | 2 | 0 | false    | 0      | 0      | null graph                       |
//! | 1 | 3 | false    | 3      | 3      | complete `K_3`                   |
//! | 2 | 3 | false    | 9      | 18     | 4-regular                        |
//! | 2 | 4 | false    | 16     | 48     | 6-regular                        |
//! | 3 | 2 | false    | 8      | 12     | equivalent to hypercube `Q_3`    |
//! | 3 | 2 | true     | 8      | 12     | low → high orientation           |
//!
//! Properties demonstrated:
//!
//! * `n·(q-1)`-regular: every vertex has degree `n·(q-1)`.
//! * Edge count: `q^n · (q-1) · n / 2`.
//! * Canonical edge ordering: every emitted pair has `u < v` (the
//!   directed variant stores them as `u → v` arcs).
//! * `q = 2` coincides with the hypercube `Q_n`.
//!
//! Run: `cargo run --example hamming_demo`.

#![allow(clippy::cast_possible_truncation)]

use rust_igraph::{Graph, hamming, hypercube};

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

fn assert_n_q_minus_one_regular(g: &Graph, n: u32, q: u32) {
    let expected = (n as usize) * ((q as usize) - 1);
    for v in 0..g.vcount() {
        let d = g.degree(v).expect("vertex in range");
        assert_eq!(
            d, expected,
            "H({n},{q}) vertex {v} should have degree {expected}",
        );
    }
}

fn assert_canonical_orientation(g: &Graph) {
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32");
    for e in 0..m {
        let (u, v) = g.edge(e).expect("edge in range");
        assert!(u < v, "edge ({u}, {v}) must be canonically ordered");
    }
}

fn main() {
    // H(1, 3) — complete K_3 (every singleton string differs from every other).
    let k3 = hamming(1, 3, false).expect("H(1,3)");
    print_summary("H(1, 3) — K_3", &k3);
    assert_n_q_minus_one_regular(&k3, 1, 3);

    // H(2, 3) — 9-vertex, 4-regular.
    let h_2_3 = hamming(2, 3, false).expect("H(2,3)");
    print_summary("H(2, 3) — 9 vertices, 4-regular", &h_2_3);
    assert_n_q_minus_one_regular(&h_2_3, 2, 3);

    // H(2, 4) — 16-vertex, 6-regular.
    let h_2_4 = hamming(2, 4, false).expect("H(2,4)");
    print_summary("H(2, 4) — 16 vertices, 6-regular", &h_2_4);
    assert_n_q_minus_one_regular(&h_2_4, 2, 4);

    // H(3, 2) ≡ Q_3 — match against the standalone hypercube constructor.
    let h_3_2 = hamming(3, 2, false).expect("H(3,2)");
    let q3 = hypercube(3, false).expect("Q_3");
    print_summary("H(3, 2) — the cube, ≡ Q_3", &h_3_2);
    assert_eq!(h_3_2.vcount(), q3.vcount());
    assert_eq!(h_3_2.ecount(), q3.ecount());
    assert_eq!(
        dump_edges(&h_3_2),
        dump_edges(&q3),
        "H(3,2) edge sequence should equal Q_3"
    );

    // Directed variant — same canonical edges, stored low → high.
    let h_3_2_d = hamming(3, 2, true).expect("H(3,2) directed");
    print_summary("H(3, 2) (directed) — low → high arcs", &h_3_2_d);
    assert_canonical_orientation(&h_3_2_d);

    println!("\nall structural invariants OK ✓");
}
