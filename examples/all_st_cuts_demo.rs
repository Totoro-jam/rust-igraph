//! ALGO-FL-031 example: list every (s,t) edge cut of a directed graph
//! via the Provan-Shier paradigm.
//!
//! Run: `cargo run --example all_st_cuts_demo`.
//!
//! We (1) build the canonical 6-node directed graph, (2) call
//! `all_st_cuts(graph, source, target)`, then (3) print each cut's
//! source-side partition `X` (always holds `source`, never `target`) and
//! the ids of the edges leaving `X`. This graph has 9 distinct cuts.

use rust_igraph::{Graph, all_st_cuts};

fn main() {
    let edges = [(0u32, 1u32), (1, 2), (1, 3), (2, 4), (3, 4), (1, 5), (5, 4)];
    let mut g = Graph::new(6, true).expect("directed graph");
    for (u, v) in edges {
        g.add_edge(u, v).expect("add edge");
    }

    // Label the edges so the cut ids below are easy to read back.
    println!("edges (id: from -> to):");
    for (eid, (u, v)) in edges.iter().enumerate() {
        println!("  {eid}: {u} -> {v}");
    }
    println!();

    let (source, target) = (0u32, 4u32);
    let res = all_st_cuts(&g, source, target).expect("compute cuts");

    println!("{} cuts from {source} to {target}:", res.cuts.len());
    for (i, (part, cut)) in res.partition1s.iter().zip(res.cuts.iter()).enumerate() {
        println!("  cut {i}: source-side X = {part:?}, edges = {cut:?}");
    }
}
