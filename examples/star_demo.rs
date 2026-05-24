//! ALGO-CN-002 example: star constructor (`igraph_star`).
//!
//! Builds the K1,4 star under each of the four `StarMode` variants
//! (Out / In / Mutual / Undirected) and prints the resulting edge
//! list so the per-mode arc orientation is immediately visible.
//!
//! The four-mode truth table for `star_graph(n=5, mode, center=0)`:
//!
//! | mode        | directed | ecount  | first edge | last edge  |
//! |-------------|----------|---------|------------|------------|
//! | Out         | true     | n-1 = 4 | (0, 1)     | (0, 4)     |
//! | In          | true     | n-1 = 4 | (1, 0)     | (4, 0)     |
//! | Mutual      | true     | 2(n-1) = 8 | (0, 1)  | (4, 0)     |
//! | Undirected  | false    | n-1 = 4 | (0, 1)*    | (0, 4)*    |
//!
//! `*` Undirected storage canonicalises endpoints as `(min, max)`.
//!
//! Run: `cargo run --example star_demo`.

#![allow(clippy::cast_possible_truncation)]

use rust_igraph::{Graph, StarMode, star_graph};

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

fn main() {
    let out = star_graph(5, StarMode::Out, 0).expect("out star");
    print_summary("Out  star K1,4 (centre = 0)", &out);

    let inn = star_graph(5, StarMode::In, 0).expect("in star");
    print_summary("In   star K1,4 (centre = 0)", &inn);

    let mut_star = star_graph(5, StarMode::Mutual, 0).expect("mutual star");
    print_summary("Mutual star K1,4 (centre = 0, both arcs)", &mut_star);

    let undirected = star_graph(5, StarMode::Undirected, 0).expect("undirected star");
    print_summary("Undirected star K1,4 (centre = 0)", &undirected);

    // Centre at a non-zero vertex — leaves are visited in vertex-id
    // order [0, center) then (center, n), which the edge sequence
    // makes obvious for Out mode with centre = 2.
    let off_centre = star_graph(5, StarMode::Out, 2).expect("off-centre out star");
    print_summary("Out star K1,4 with centre = 2", &off_centre);
}
