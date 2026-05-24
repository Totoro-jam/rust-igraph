//! ALGO-CN-003 example: wheel constructor (`igraph_wheel`).
//!
//! Builds the wheel W5 (centre + 4 rim vertices) under each of the
//! four `WheelMode` variants (Out / In / Mutual / Undirected) and
//! prints the resulting edge list. The wheel is a star **plus** a rim
//! cycle through the leaves; the edge order is `[star block, rim
//! forward sweep, (optional) rim reverse arcs]`.
//!
//! The four-mode truth table for `wheel_graph(n=5, mode, center=0)`:
//!
//! | mode        | directed | ecount       | first edge | last edge   |
//! |-------------|----------|--------------|------------|-------------|
//! | Out         | true     | 2(n-1) = 8   | (0, 1)     | (4, 1)      |
//! | In          | true     | 2(n-1) = 8   | (1, 0)     | (4, 1)      |
//! | Mutual      | true     | 4(n-1) = 16  | (0, 1)     | (2, 1)      |
//! | Undirected  | false    | 2(n-1) = 8   | (0, 1)*    | (1, 4)*     |
//!
//! `*` Undirected storage canonicalises endpoints as `(min, max)`.
//!
//! Degenerate shapes (documented in `wheel_graph`'s module docs):
//!
//! * `n = 2` — rim collapses to a 1-cycle, so the only rim vertex
//!   carries a self-loop.
//! * `n = 3` — rim collapses to a 2-cycle: the forward and wrap-around
//!   rim arcs are parallel.
//!
//! Run: `cargo run --example wheel_demo`.

#![allow(clippy::cast_possible_truncation)]

use rust_igraph::{Graph, WheelMode, wheel_graph};

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
    let out = wheel_graph(5, WheelMode::Out, 0).expect("out wheel");
    print_summary("Out  wheel W5 (centre = 0)", &out);

    let inn = wheel_graph(5, WheelMode::In, 0).expect("in wheel");
    print_summary("In   wheel W5 (centre = 0)", &inn);

    let mut_wheel = wheel_graph(5, WheelMode::Mutual, 0).expect("mutual wheel");
    print_summary("Mutual wheel W5 (centre = 0, both arcs)", &mut_wheel);

    let undirected = wheel_graph(5, WheelMode::Undirected, 0).expect("undirected wheel");
    print_summary("Undirected wheel W5 (centre = 0)", &undirected);

    // Non-zero centre — rim skips the centre vertex while still
    // visiting rim vertices in vertex-id order.
    let off_centre = wheel_graph(5, WheelMode::Out, 2).expect("off-centre out wheel");
    print_summary("Out wheel W5 with centre = 2", &off_centre);

    // Degenerate cases — wheel ≡ star for n ≤ 1, and rim collapses
    // into a self-loop (n=2) or parallel rim edges (n=3).
    let w2 = wheel_graph(2, WheelMode::Out, 0).expect("W2");
    print_summary("Out wheel n=2 (rim is self-loop on vertex 1)", &w2);
    let w3 = wheel_graph(3, WheelMode::Out, 0).expect("W3");
    print_summary("Out wheel n=3 (rim is parallel (1,2) and (2,1))", &w3);
}
