//! ALGO-CN-011 example: circulant graph `G(n, shifts)`
//! (`igraph_circulant`).
//!
//! Walks through canonical specialisations and verifies the standard
//! structural invariants for each one:
//!
//! | n  | shifts        | result                | vcount | ecount | note                            |
//! |----|---------------|-----------------------|--------|--------|---------------------------------|
//! | 5  | [1]           | `C_5`                 | 5      | 5      | undirected cycle                |
//! | 6  | [1, 3]        | `C_6` + matching      | 6      | 9      | antipodal shift halves emission |
//! | 7  | [1, 2]        | squared cycle         | 7      | 14     | 4-regular                       |
//! | 4  | [1, 2]        | `K_4`                 | 4      | 6      | complete via full shift set     |
//! | 8  | [1, 3] (dir)  | directed circulant    | 8      | 16     | both shifts kept distinct       |
//!
//! Run: `cargo run --example circulant_demo`.

use rust_igraph::{Graph, circulant};

fn print_summary(label: &str, g: &Graph) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
}

fn assert_regularity(g: &Graph, expected_degree: usize) {
    for v in 0..g.vcount() {
        let d = g.degree(v).expect("vertex in range");
        assert_eq!(
            d, expected_degree,
            "vertex {v} should have degree {expected_degree}"
        );
    }
}

fn main() {
    // C_5 — the simplest non-trivial cycle.
    let c5 = circulant(5, &[1], false).expect("C_5");
    print_summary("C_5 = circulant(5, [1], false)", &c5);
    assert_eq!(c5.vcount(), 5);
    assert_eq!(c5.ecount(), 5);
    assert_regularity(&c5, 2);

    // C_6 plus the antipodal perfect matching (shift 3 = n/2 halves the
    // emission). The result is 3-regular (cycle deg 2 + matching deg 1).
    let c6m = circulant(6, &[1, 3], false).expect("C_6 + matching");
    print_summary("C_6 + matching = circulant(6, [1, 3], false)", &c6m);
    assert_eq!(c6m.vcount(), 6);
    assert_eq!(c6m.ecount(), 9);
    assert_regularity(&c6m, 3);

    // Squared cycle on 7 vertices.
    let sq = circulant(7, &[1, 2], false).expect("squared cycle");
    print_summary("squared cycle = circulant(7, [1, 2], false)", &sq);
    assert_eq!(sq.vcount(), 7);
    assert_eq!(sq.ecount(), 14);
    assert_regularity(&sq, 4);

    // K_4 — complete graph through every distinct undirected shift.
    let k4 = circulant(4, &[1, 2], false).expect("K_4");
    print_summary("K_4 = circulant(4, [1, 2], false)", &k4);
    assert_eq!(k4.vcount(), 4);
    assert_eq!(k4.ecount(), 6);
    assert_regularity(&k4, 3);

    // Directed circulant with two distinct shifts (kept distinct
    // because the undirected folding doesn't apply).
    let d8 = circulant(8, &[1, 3], true).expect("directed circulant");
    print_summary("directed circulant = circulant(8, [1, 3], true)", &d8);
    assert_eq!(d8.vcount(), 8);
    assert_eq!(d8.ecount(), 16);
    assert!(d8.is_directed());

    println!("\nall structural invariants OK ✓");
}
