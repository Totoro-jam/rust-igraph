//! ALGO-CL-002 example: chordality test demo.
//!
//! Builds several small graphs, runs MCS and the chordal test, printing
//! the MCS ordering and whether fill-in edges are needed.
//!
//! Run: `cargo run --example chordality_demo`.

use rust_igraph::{Graph, create, is_chordal, maximum_cardinality_search};

fn demo(label: &str, g: &Graph) {
    let n = g.vcount();
    println!("{label} ({n} vertices, {} edges)", g.ecount());

    let mcs = maximum_cardinality_search(g).expect("MCS succeeds");
    println!("  MCS alpha:    {:?}", &mcs.alpha[..n as usize]);

    let cr = is_chordal(g, Some(&mcs.alpha)).expect("is_chordal succeeds");
    println!("  Chordal:      {}", cr.chordal);
    if !cr.fill_in.is_empty() {
        println!("  Fill-in edges: {:?}", cr.fill_in);
    }
    println!();
}

fn main() {
    let empty = Graph::new(0, false).expect("ok");
    demo("Empty graph", &empty);

    let singleton = Graph::new(1, false).expect("ok");
    demo("Singleton", &singleton);

    let triangle = create(&[(0, 1), (1, 2), (2, 0)], 3, false).expect("ok");
    demo("Triangle (K3) - chordal", &triangle);

    let c4 = create(&[(0, 1), (1, 2), (2, 3), (3, 0)], 4, false).expect("ok");
    demo("4-cycle (C4) - NOT chordal", &c4);

    let c4_chord = create(&[(0, 1), (1, 2), (2, 3), (3, 0), (0, 2)], 4, false).expect("ok");
    demo("C4 + diagonal (chordal)", &c4_chord);

    let path = create(&[(0, 1), (1, 2), (2, 3), (3, 4)], 5, false).expect("ok");
    demo("Path P5 - chordal (tree)", &path);

    let k4 = create(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)], 4, false).expect("ok");
    demo("Complete K4 - chordal", &k4);

    let disconnected = Graph::new(5, false).expect("ok");
    demo("5 isolated vertices - chordal", &disconnected);
}
