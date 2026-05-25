//! ALGO-CN-010 example: generalized Petersen graph G(n, k)
//! (`igraph_generalized_petersen`).
//!
//! Walks the most famous specializations of the family and verifies the
//! standard structural invariants (3-regular, undirected, exactly
//! `2n` vertices and `3n` edges).
//!
//! | n  | k | name                          | vcount | ecount | note                |
//! |----|---|-------------------------------|--------|--------|---------------------|
//! | 5  | 2 | Petersen                      | 10     | 15     | smallest snark-free |
//! | 8  | 3 | Möbius–Kantor                 | 16     | 24     | bipartite, girth 6  |
//! | 10 | 3 | Desargues                     | 20     | 30     | bipartite           |
//! | 12 | 5 | Nauru                         | 24     | 36     | symmetric           |
//!
//! Run: `cargo run --example generalized_petersen_demo`.

use rust_igraph::{Graph, generalized_petersen};

fn print_summary(label: &str, g: &Graph) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
}

fn assert_three_regular(g: &Graph) {
    for v in 0..g.vcount() {
        let d = g.degree(v).expect("vertex in range");
        assert_eq!(d, 3, "vertex {v} should have degree 3");
    }
}

fn main() {
    // Petersen — the smallest 3-regular graph with girth 5.
    let petersen = generalized_petersen(5, 2).expect("Petersen graph");
    print_summary("G(5, 2) — Petersen", &petersen);
    assert_eq!(petersen.vcount(), 10);
    assert_eq!(petersen.ecount(), 15);
    assert!(!petersen.is_directed());
    assert_three_regular(&petersen);

    // Möbius–Kantor — 3-regular, bipartite, girth 6.
    let mobius_kantor = generalized_petersen(8, 3).expect("Möbius–Kantor");
    print_summary("G(8, 3) — Möbius–Kantor", &mobius_kantor);
    assert_eq!(mobius_kantor.vcount(), 16);
    assert_eq!(mobius_kantor.ecount(), 24);
    assert_three_regular(&mobius_kantor);

    // Desargues — distance-transitive 3-regular bipartite graph.
    let desargues = generalized_petersen(10, 3).expect("Desargues");
    print_summary("G(10, 3) — Desargues", &desargues);
    assert_eq!(desargues.vcount(), 20);
    assert_eq!(desargues.ecount(), 30);
    assert_three_regular(&desargues);

    // Nauru — symmetric 3-regular graph.
    let nauru = generalized_petersen(12, 5).expect("Nauru");
    print_summary("G(12, 5) — Nauru", &nauru);
    assert_eq!(nauru.vcount(), 24);
    assert_eq!(nauru.ecount(), 36);
    assert_three_regular(&nauru);

    println!("\nall structural invariants OK ✓");
}
