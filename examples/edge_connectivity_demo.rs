//! ALGO-FL-016 example: global edge connectivity (graph adhesion) —
//! the minimum number of edges whose removal disconnects the graph.
//! Equivalent to `min_{v != 0} st_edge_connectivity(0, v)` (both
//! directions for directed graphs). The alias
//! [`adhesion`](rust_igraph::adhesion) mirrors igraph C's
//! `igraph_adhesion`.
//!
//! Unlike global vertex connectivity, edge connectivity has no
//! complete-graph short-circuit because multigraphs may have
//! lambda > n-1 (parallel edges raise the min cut). Even `K_n` must
//! traverse the fixed-vertex loop.
//!
//! Run: `cargo run --example edge_connectivity_demo`.

use rust_igraph::{Graph, adhesion, edge_connectivity};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) Undirected 5-cycle — every pair of vertices admits two
    //    edge-disjoint paths around the ring → ec = 2. No cheap
    //    short-circuit fires (connected, min-degree=2, no
    //    complete-graph shortcut for edge_connectivity), so the
    //    fixed-vertex loop runs.
    let mut ring = Graph::new(5, false)?;
    for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 0)] {
        ring.add_edge(u, v)?;
    }
    println!("Undirected 5-cycle C_5");
    println!(
        "  edge_connectivity = {}  (expected 2)",
        edge_connectivity(&ring, true)?
    );
    println!(
        "  adhesion alias    = {}  (expected 2)",
        adhesion(&ring, true)?
    );

    // 2) Undirected path on 5 vertices — endpoints have degree 1 → ec = 1
    //    (cheap min-degree short-circuit fires under checks=true).
    let mut path = Graph::new(5, false)?;
    for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4)] {
        path.add_edge(u, v)?;
    }
    println!("\nUndirected path P_5");
    println!(
        "  edge_connectivity = {}  (expected 1)",
        edge_connectivity(&path, true)?
    );

    // 3) Two disconnected edges — graph not connected → ec = 0
    //    (cheap connectedness short-circuit fires).
    let mut disconnected = Graph::new(4, false)?;
    disconnected.add_edge(0, 1)?;
    disconnected.add_edge(2, 3)?;
    println!("\nTwo isolated edges {{0-1, 2-3}}");
    println!(
        "  edge_connectivity = {}  (expected 0)",
        edge_connectivity(&disconnected, true)?
    );

    // 4) Complete graph K_4 — no complete-graph shortcut for
    //    edge_connectivity (multigraph caveat), so the fixed-vertex
    //    loop runs three max-flow computations and returns n - 1 = 3.
    let mut k4 = Graph::new(4, false)?;
    for i in 0u32..4 {
        for j in (i + 1)..4 {
            k4.add_edge(i, j)?;
        }
    }
    println!("\nComplete graph K_4 (no complete-graph shortcut for edge_connectivity)");
    println!(
        "  edge_connectivity = {}  (expected 3)",
        edge_connectivity(&k4, true)?
    );

    // 5) Directed out-tree — leaves cannot reach the root → ec = 0
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
        "  edge_connectivity = {}  (expected 0)",
        edge_connectivity(&out_tree, true)?
    );

    // 6) Multigraph: two parallel edges between vertices 0 and 1.
    //    Demonstrates why edge_connectivity has no complete-graph
    //    shortcut — lambda(G) = 2 here even though vcount - 1 = 1.
    let mut multi = Graph::new(2, false)?;
    multi.add_edge(0, 1)?;
    multi.add_edge(0, 1)?;
    println!("\nMultigraph: 2 parallel edges between 0-1");
    println!(
        "  edge_connectivity = {}  (expected 2 — parallel edges raise lambda)",
        edge_connectivity(&multi, true)?
    );

    // 7) Directed cycle 0 -> 1 -> 2 -> 0 — strongly connected, every
    //    cut needs to remove one edge in each direction → ec = 1.
    //    Fixed-vertex loop runs both (0,v) and (v,0) for v in {1,2}.
    let mut dcycle = Graph::new(3, true)?;
    dcycle.add_edge(0, 1)?;
    dcycle.add_edge(1, 2)?;
    dcycle.add_edge(2, 0)?;
    println!("\nDirected 3-cycle 0->1->2->0");
    println!(
        "  edge_connectivity = {}  (expected 1)",
        edge_connectivity(&dcycle, true)?
    );

    Ok(())
}
