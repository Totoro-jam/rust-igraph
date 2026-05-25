//! ALGO-CN-013 example: Kautz graph `K(m, n)` (`igraph_kautz`).
//!
//! Walks through canonical specialisations and verifies the standard
//! structural invariants for each one. `K(m, n)` is always directed and
//! every vertex has out-degree `m` and in-degree `m` for `n >= 1`
//! (total degree `2m`); for `n == 0` it collapses to the directed
//! complete graph `K_{m+1}` with no self-loops.
//!
//! | m | n | result                | vcount | ecount (= `m·(m+1)·m^n`)   | note                              |
//! |---|---|-----------------------|--------|----------------------------|-----------------------------------|
//! | 0 | 0 | singleton             | 1      | 0                          | degenerate `m == 0 ∧ n == 0`      |
//! | 0 | 5 | empty                 | 0      | 0                          | degenerate `m == 0 ∧ n >= 1`      |
//! | 5 | 0 | directed `K_6`        | 6      | 30                         | `n == 0` → directed complete      |
//! | 2 | 1 | smallest non-trivial  | 6      | 12                         | the `test_kautz.c` canonical case |
//! | 2 | 2 | binary alphabet       | 12     | 24                         | smallest with both index tables   |
//! | 3 | 2 | quaternary alphabet   | 36     | 108                        | richer alphabet stresses cursor   |
//!
//! Run: `cargo run --example kautz_demo`.
//!
//! The Kautz rewrite: a vertex is a length-`n+1` string over an
//! alphabet of `m+1` letters with no two consecutive equal letters.
//! From `v = (a_0, …, a_n)` an arc goes to every `w = (a_1, …, a_n, b)`
//! with `b ≠ a_n` — so out-degree is exactly `m`. The example asserts
//! the loopless and source-target lastdigit-distinct properties on
//! `K(3, 2)` so you can see the rewrite in action.

use rust_igraph::{Graph, kautz};

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
    // K(0, 0) — the n == 0 path with m == 0 collapses to a singleton.
    let k00 = kautz(0, 0).expect("K(0, 0)");
    print_summary("K(0, 0) = kautz(0, 0)", &k00);
    assert_eq!(k00.vcount(), 1);
    assert_eq!(k00.ecount(), 0);
    assert!(k00.is_directed());

    // K(0, 5) — the only-symbol alphabet cannot satisfy the
    // no-consecutive-equal constraint, so the graph is empty.
    let k05 = kautz(0, 5).expect("K(0, 5)");
    print_summary("K(0, 5) = kautz(0, 5)", &k05);
    assert_eq!(k05.vcount(), 0);
    assert_eq!(k05.ecount(), 0);

    // K(5, 0) — the n == 0 path lands on directed K_6 with no
    // self-loops: vcount = m+1, ecount = (m+1)·m = 30.
    let k50 = kautz(5, 0).expect("K(5, 0)");
    print_summary("K(5, 0) = kautz(5, 0)  (directed K_6, no loops)", &k50);
    assert_eq!(k50.vcount(), 6);
    assert_eq!(k50.ecount(), 30);
    // Total degree = 2(m+1-1) = 2·5 = 10.
    assert_total_degree(&k50, 10);

    // K(2, 1) — the smallest non-trivial case (also the canonical
    // upstream unit test in `test_kautz.c`). 6 vertices, 12 arcs,
    // every vertex has out-degree 2 and in-degree 2.
    let k21 = kautz(2, 1).expect("K(2, 1)");
    print_summary("K(2, 1) = kautz(2, 1)", &k21);
    assert_eq!(k21.vcount(), 6);
    assert_eq!(k21.ecount(), 12);
    assert_total_degree(&k21, 4);

    // K(2, 2) — same alphabet, one more letter of context. The dense
    // vertex layout now matters because both index1 and index2 are
    // exercised.
    let k22 = kautz(2, 2).expect("K(2, 2)");
    print_summary("K(2, 2) = kautz(2, 2)", &k22);
    assert_eq!(k22.vcount(), 12);
    assert_eq!(k22.ecount(), 24);
    assert_total_degree(&k22, 4);

    // K(3, 2) — quaternary alphabet, 36 vertices and 108 arcs.
    let k32 = kautz(3, 2).expect("K(3, 2)");
    print_summary("K(3, 2) = kautz(3, 2)", &k32);
    assert_eq!(k32.vcount(), 36);
    assert_eq!(k32.ecount(), 108);
    assert_total_degree(&k32, 6);

    // Kautz graphs are loopless. The rewrite forces the source's
    // lastdigit and the target's lastdigit to differ (no two
    // consecutive equal letters in the underlying string), so no arc
    // has the same source and target.
    let ec = u32::try_from(k32.ecount()).expect("ecount fits u32");
    for eid in 0..ec {
        let (u, v) = k32.edge(eid).expect("arc in range");
        assert_ne!(u, v, "K(3, 2) is loopless: arc ({u}, {v}) is a self-loop");
    }

    println!("\nall structural invariants OK ✓");
}
