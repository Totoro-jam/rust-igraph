//! ALGO-CN-001 example: ring / path / cycle constructors.
//!
//! Builds a few common shapes from the same `ring_graph` factory and
//! prints summary diagnostics — vertex / edge counts, the degree
//! sequence (so the parity invariants `path => 1,2,2,...,2,1` and
//! `cycle => 2,2,...,2` are visible at a glance), and a small ASCII
//! edge list for the smallest cycle so the wrap-around closing the
//! ring is obvious.
//!
//! Run: `cargo run --example ring_demo`.

#![allow(clippy::cast_possible_truncation)]

use rust_igraph::{Graph, cycle_graph, path_graph, ring_graph};

fn degree_sequence(g: &Graph) -> Vec<usize> {
    let n = g.vcount();
    (0..n)
        .map(|v| g.neighbors(v).expect("vertex id in bounds").len())
        .collect()
}

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
    println!("  degrees  = {:?}", degree_sequence(g));
}

fn main() {
    let p7 = path_graph(7, false, false).expect("path P7");
    print_summary("undirected path P7", &p7);

    let c7 = cycle_graph(7, false, false).expect("cycle C7");
    print_summary("undirected cycle C7", &c7);
    println!("  edges    = {:?}", dump_edges(&c7));

    let dc7 = cycle_graph(7, true, false).expect("directed cycle C7");
    print_summary("directed cycle C7 (forward only)", &dc7);

    let dmc4 = ring_graph(4, true, true, true).expect("directed mutual cycle");
    print_summary("directed mutual cycle C4", &dmc4);
    println!("  edges    = {:?}", dump_edges(&dmc4));

    // Degenerate shapes — useful sanity samples.
    let self_loop = ring_graph(1, false, false, true).expect("n=1 cycle");
    print_summary("singleton cycle (self-loop)", &self_loop);
    println!("  edges    = {:?}", dump_edges(&self_loop));
}
