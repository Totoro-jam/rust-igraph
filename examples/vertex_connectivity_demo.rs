//! ALGO-FL-015 example: global vertex connectivity (graph cohesion) —
//! the minimum number of internal vertices whose removal disconnects
//! some pair of vertices. Equivalent to
//! `min_{s ≠ t} st_vertex_connectivity(s, t, NumberOfNodes)`.
//! The alias [`cohesion`](rust_igraph::cohesion) mirrors igraph C's
//! `igraph_cohesion`.
//!
//! Run: `cargo run --example vertex_connectivity_demo`.

use rust_igraph::{Graph, cohesion, vertex_connectivity};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) Undirected 5-cycle — every pair of vertices admits two
    //    internally disjoint paths around the ring → vc = 2.
    let mut ring = Graph::new(5, false)?;
    for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 0)] {
        ring.add_edge(u, v)?;
    }
    println!("Undirected 5-cycle C_5");
    println!(
        "  vertex_connectivity = {}  (expected 2)",
        vertex_connectivity(&ring, true)?
    );
    println!(
        "  cohesion alias      = {}  (expected 2)",
        cohesion(&ring, true)?
    );

    // 2) Undirected path on 5 vertices — endpoints have degree 1 → vc = 1
    //    (cheap min-degree short-circuit fires under checks=true).
    let mut path = Graph::new(5, false)?;
    for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4)] {
        path.add_edge(u, v)?;
    }
    println!("\nUndirected path P_5");
    println!(
        "  vertex_connectivity = {}  (expected 1)",
        vertex_connectivity(&path, true)?
    );

    // 3) Two disconnected edges — no s-t pair is reachable between the
    //    components → vc = 0 (cheap connectedness short-circuit fires).
    let mut disconnected = Graph::new(4, false)?;
    disconnected.add_edge(0, 1)?;
    disconnected.add_edge(2, 3)?;
    println!("\nTwo isolated edges {{0-1, 2-3}}");
    println!(
        "  vertex_connectivity = {}  (expected 0)",
        vertex_connectivity(&disconnected, true)?
    );

    // 4) Complete graph K_6 — every pair is adjacent so removing any
    //    n-2 vertices still leaves the pair connected → vc = n - 1 = 5
    //    (cheap is_complete short-circuit fires).
    let mut k6 = Graph::new(6, false)?;
    for i in 0u32..6 {
        for j in (i + 1)..6 {
            k6.add_edge(i, j)?;
        }
    }
    println!("\nComplete graph K_6");
    println!(
        "  vertex_connectivity = {}  (expected 5)",
        vertex_connectivity(&k6, true)?
    );

    // 5) Directed out-tree — leaves cannot reach the root → vc = 0
    //    (cheap strong-connectedness short-circuit fires).
    let mut out_tree = Graph::new(10, true)?;
    for (u, v) in [
        (0u32, 1u32),
        (0, 2),
        (0, 3),
        (1, 4),
        (1, 5),
        (1, 6),
        (2, 7),
        (2, 8),
        (2, 9),
    ] {
        out_tree.add_edge(u, v)?;
    }
    println!("\nDirected out-tree on 10 vertices");
    println!(
        "  vertex_connectivity = {}  (expected 0)",
        vertex_connectivity(&out_tree, true)?
    );

    // 6) Cycle with a chord (4 vertices) — pairwise FL-013 loop runs
    //    (no cheap short-circuit applies) and returns 2.
    let mut chorded = Graph::new(4, false)?;
    for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 0), (0, 2)] {
        chorded.add_edge(u, v)?;
    }
    println!("\n4-cycle with chord 0-2 (no cheap short-circuit applies)");
    println!(
        "  vertex_connectivity = {}  (expected 2)",
        vertex_connectivity(&chorded, true)?
    );

    Ok(())
}
