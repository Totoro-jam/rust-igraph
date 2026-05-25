//! ALGO-CN-009 example: d-dimensional square lattice (`igraph_square_lattice`).
//!
//! Walks every flag combination supported by the constructor and
//! verifies the standard structural invariants.
//!
//! Truth table for `square_lattice(dim, 1, directed, mutual, periodic)`:
//!
//! | dim          | periodic         | directed | mutual | vcount | ecount | shape                |
//! |--------------|------------------|----------|--------|--------|--------|----------------------|
//! | `[]`         | n/a              | false    | false  | 1      | 0      | singleton (0-d)      |
//! | `[3]`        | none             | false    | false  | 3      | 2      | path `P_3`           |
//! | `[3]`        | `[true]`         | false    | false  | 3      | 3      | cycle `C_3`          |
//! | `[3, 3]`     | none             | false    | false  | 9      | 12     | 2-D grid             |
//! | `[3, 3]`     | `[true, true]`   | false    | false  | 9      | 18     | 2-D torus (4-reg.)   |
//! | `[2, 2, 2]`  | none             | false    | false  | 8      | 12     | cube `Q_3`           |
//! | `[3]`        | none             | true     | true   | 3      | 4      | bidir. `P_3` arcs    |
//!
//! Vertex IDs follow the little-endian convention: the lattice site at
//! coordinates `(i_0, i_1, …, i_{d-1})` in a shape `(n_0, n_1, …)`
//! lattice carries id `i_0 + n_0·i_1 + n_0·n_1·i_2 + …`.
//!
//! Run: `cargo run --example square_lattice_demo`.

use rust_igraph::{Graph, hypercube, square_lattice};

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

fn assert_regular(g: &Graph, expected: usize) {
    for v in 0..g.vcount() {
        let d = g.degree(v).expect("vertex in range");
        assert_eq!(d, expected, "vertex {v} should have degree {expected}");
    }
}

fn main() {
    // 1-D path.
    let path = square_lattice(&[3], 1, false, false, None).expect("P_3");
    print_summary("dim=[3] non-periodic — path P_3", &path);
    assert_eq!(path.vcount(), 3);
    assert_eq!(path.ecount(), 2);

    // 1-D periodic cycle.
    let cycle = square_lattice(&[3], 1, false, false, Some(&[true])).expect("C_3");
    print_summary("dim=[3] periodic — cycle C_3", &cycle);
    assert_eq!(cycle.ecount(), 3);

    // 2-D 3×3 grid.
    let grid = square_lattice(&[3, 3], 1, false, false, None).expect("3x3 grid");
    print_summary("dim=[3,3] non-periodic — 9-vertex grid", &grid);
    assert_eq!(grid.vcount(), 9);
    assert_eq!(grid.ecount(), 12);

    // 2-D 3×3 torus — every vertex 4-regular.
    let torus = square_lattice(&[3, 3], 1, false, false, Some(&[true, true])).expect("torus");
    print_summary("dim=[3,3] periodic — 3x3 torus", &torus);
    assert_regular(&torus, 4);
    assert_eq!(torus.ecount(), 18);

    // 3-D cube equals Q_3.
    let cube = square_lattice(&[2, 2, 2], 1, false, false, None).expect("cube");
    let q3 = hypercube(3, false).expect("Q_3");
    print_summary("dim=[2,2,2] — cube ≡ Q_3", &cube);
    assert_eq!(cube.vcount(), q3.vcount());
    assert_eq!(cube.ecount(), q3.ecount());
    assert_eq!(
        dump_edges(&cube),
        dump_edges(&q3),
        "cube edge sequence should equal Q_3"
    );

    // Directed mutual P_3 — every undirected edge becomes a pair of arcs.
    let dir_mut = square_lattice(&[3], 1, true, true, None).expect("dir mut P_3");
    print_summary("dim=[3] directed+mutual — bidirectional arcs", &dir_mut);
    assert!(dir_mut.is_directed());
    assert_eq!(dir_mut.ecount(), 4);

    println!("\nall structural invariants OK ✓");
}
