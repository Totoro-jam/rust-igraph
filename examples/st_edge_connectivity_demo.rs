//! ALGO-FL-011 example: scalar s-t edge connectivity on textbook
//! instances, illustrating the wrapper around unit-capacity max-flow.
//!
//! Run: `cargo run --example st_edge_connectivity_demo`.

use rust_igraph::{Graph, st_edge_connectivity};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) igraph C unit-test fixture (6 vertices, 8 directed arcs, ec = 2).
    let mut g = Graph::new(6, true)?;
    for (u, v) in [
        (0u32, 1u32),
        (0, 2),
        (1, 2),
        (1, 3),
        (2, 4),
        (3, 4),
        (3, 5),
        (4, 5),
    ] {
        g.add_edge(u, v)?;
    }
    println!("igraph_st_edge_connectivity.c fixture (6v, 8 arcs)");
    println!(
        "  st_edge_connectivity(0 → 5) = {}  (expected 2)",
        st_edge_connectivity(&g, 0, 5)?
    );

    // 2) Two parallel unit paths 0→1→3 and 0→2→3 → ec = 2.
    let mut h = Graph::new(4, true)?;
    h.add_edge(0, 1)?;
    h.add_edge(1, 3)?;
    h.add_edge(0, 2)?;
    h.add_edge(2, 3)?;
    println!("\nTwo parallel unit paths (4v)");
    println!(
        "  st_edge_connectivity(0 → 3) = {}  (expected 2)",
        st_edge_connectivity(&h, 0, 3)?
    );

    // 3) K_5 undirected — every pair has ec = 4 (vertex degree).
    let mut k5 = Graph::new(5, false)?;
    for i in 0u32..5 {
        for j in (i + 1)..5 {
            k5.add_edge(i, j)?;
        }
    }
    println!("\nK_5 undirected (every pair)");
    println!(
        "  st_edge_connectivity(0 → 1) = {}  (expected 4)",
        st_edge_connectivity(&k5, 0, 1)?
    );

    // 4) Disconnected endpoints — empty cut → ec = 0.
    let isolated = Graph::new(4, true)?;
    println!("\nDisconnected endpoints (4 isolated vertices)");
    println!(
        "  st_edge_connectivity(0 → 3) = {}  (expected 0)",
        st_edge_connectivity(&isolated, 0, 3)?
    );

    Ok(())
}
