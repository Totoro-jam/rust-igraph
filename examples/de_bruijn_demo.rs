//! ALGO-CN-012 example: De Bruijn graph `B(m, n)` (`igraph_de_bruijn`).
//!
//! Walks through canonical specialisations and verifies the standard
//! structural invariants for each one. Because `B(m, n)` is always
//! directed and every vertex has out-degree `m` and in-degree `m`
//! (for `n >= 1`), the total degree is `2m`.
//!
//! | m | n | result          | vcount | ecount (= `m^(n+1)`) | note                              |
//! |---|---|-----------------|--------|----------------------|-----------------------------------|
//! | 2 | 1 | directed `K_2`+ | 2      | 4                    | both self-loops and both arcs     |
//! | 2 | 2 | `B(2, 2)`       | 4      | 8                    | smallest non-trivial case         |
//! | 2 | 3 | `B(2, 3)`       | 8      | 16                   | adds one symbol of context        |
//! | 3 | 2 | `B(3, 2)`       | 9      | 27                   | ternary alphabet                  |
//! | 1 | 1 | single self-loop| 1      | 1                    | degenerate `m == 1` case          |
//!
//! Run: `cargo run --example de_bruijn_demo`.
//!
//! The arc rewrite is `(i, (i * m mod m^n) + b)` for `b ∈ [0, m)`, so
//! from any vertex `i` the `m` successors lie in a contiguous block
//! `[basis, basis + m)`. The example asserts that property on `B(3, 2)`
//! so you can see the arithmetic encoding in action.

use rust_igraph::{Graph, de_bruijn};

fn print_summary(label: &str, g: &Graph) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
}

fn assert_total_degree(g: &Graph, expected_total: usize) {
    for v in 0..g.vcount() {
        let d = g.degree(v).expect("vertex in range");
        assert_eq!(
            d, expected_total,
            "vertex {v} should have total degree (= out + in) {expected_total}"
        );
    }
}

fn main() {
    // m = 1, n = 1 — the single string "0" maps to itself.
    let b11 = de_bruijn(1, 1).expect("B(1, 1)");
    print_summary("B(1, 1) = de_bruijn(1, 1)", &b11);
    assert_eq!(b11.vcount(), 1);
    assert_eq!(b11.ecount(), 1);
    assert!(b11.is_directed());
    // The lone arc is the self-loop (0, 0).
    assert_eq!(b11.edge(0).expect("only arc"), (0, 0));

    // B(2, 1) — directed K_2 plus both self-loops.
    let b21 = de_bruijn(2, 1).expect("B(2, 1)");
    print_summary("B(2, 1) = de_bruijn(2, 1)", &b21);
    assert_eq!(b21.vcount(), 2);
    assert_eq!(b21.ecount(), 4);
    // Total degree (out + in) per vertex is 2m = 4 (out-degree m = 2,
    // in-degree m = 2 — every vertex of B(m, 1) is incident to every
    // other vertex including itself via both orientations).
    assert_total_degree(&b21, 4);

    // B(2, 2) — 4 vertices, 8 arcs. Smallest case where the rewrite is
    // non-trivial.
    let b22 = de_bruijn(2, 2).expect("B(2, 2)");
    print_summary("B(2, 2) = de_bruijn(2, 2)", &b22);
    assert_eq!(b22.vcount(), 4);
    assert_eq!(b22.ecount(), 8);
    assert_total_degree(&b22, 4);

    // B(2, 3) — adds another symbol of context.
    let b23 = de_bruijn(2, 3).expect("B(2, 3)");
    print_summary("B(2, 3) = de_bruijn(2, 3)", &b23);
    assert_eq!(b23.vcount(), 8);
    assert_eq!(b23.ecount(), 16);
    assert_total_degree(&b23, 4);

    // B(3, 2) — ternary alphabet, 9 vertices and 27 arcs.
    let b32 = de_bruijn(3, 2).expect("B(3, 2)");
    print_summary("B(3, 2) = de_bruijn(3, 2)", &b32);
    assert_eq!(b32.vcount(), 9);
    assert_eq!(b32.ecount(), 27);
    assert_total_degree(&b32, 6);

    // Demonstrate the rewrite: every arc (u, v) of B(3, 2) satisfies
    // v ∈ [(u * 3) mod 9, (u * 3) mod 9 + 3).
    let m: u32 = 3;
    let n: u32 = 2;
    let vcount = b32.vcount();
    let ec = u32::try_from(b32.ecount()).expect("ecount fits u32");
    for eid in 0..ec {
        let (u, v) = b32.edge(eid).expect("arc in range");
        let basis = (u * m) % vcount;
        assert!(
            v >= basis && v < basis + m,
            "B({m}, {n}) arc ({u} → {v}) violates rewrite (basis = {basis})"
        );
    }

    println!("\nall structural invariants OK ✓");
}
