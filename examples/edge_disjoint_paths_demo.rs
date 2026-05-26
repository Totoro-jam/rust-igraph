//! ALGO-FL-012 example: maximum number of edge-disjoint `source →
//! target` paths on textbook instances. By Menger's theorem (1927) this
//! equals the s-t edge connectivity, which on unit capacities equals
//! the s-t maximum flow value.
//!
//! Run: `cargo run --example edge_disjoint_paths_demo`.

use rust_igraph::{Graph, edge_disjoint_paths};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) igraph C unit-test fixture (6 vertices, 9 directed arcs incl.
    //    self-loop at vertex 3 — see tests/unit/igraph_edge_disjoint_paths.c).
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
        (3, 3),
    ] {
        g.add_edge(u, v)?;
    }
    println!("igraph_edge_disjoint_paths.c fixture (6v, 9 arcs incl. loop)");
    println!(
        "  edge_disjoint_paths(0 → 5) = {}  (expected 2)",
        edge_disjoint_paths(&g, 0, 5)?
    );
    println!(
        "  edge_disjoint_paths(0 → 3) = {}  (expected 1)",
        edge_disjoint_paths(&g, 0, 3)?
    );
    println!(
        "  edge_disjoint_paths(3 → 0) = {}  (expected 0)",
        edge_disjoint_paths(&g, 3, 0)?
    );
    println!(
        "  edge_disjoint_paths(3 → 5) = {}  (expected 2)",
        edge_disjoint_paths(&g, 3, 5)?
    );

    // 2) Two parallel unit paths 0→1→3 and 0→2→3 → 2 edge-disjoint paths.
    let mut h = Graph::new(4, true)?;
    h.add_edge(0, 1)?;
    h.add_edge(1, 3)?;
    h.add_edge(0, 2)?;
    h.add_edge(2, 3)?;
    println!("\nTwo parallel unit paths (4v)");
    println!(
        "  edge_disjoint_paths(0 → 3) = {}  (expected 2)",
        edge_disjoint_paths(&h, 0, 3)?
    );

    // 3) K_5 undirected — every pair has 4 edge-disjoint paths.
    let mut k5 = Graph::new(5, false)?;
    for i in 0u32..5 {
        for j in (i + 1)..5 {
            k5.add_edge(i, j)?;
        }
    }
    println!("\nK_5 undirected (every pair)");
    println!(
        "  edge_disjoint_paths(0 → 1) = {}  (expected 4)",
        edge_disjoint_paths(&k5, 0, 1)?
    );

    // 4) Parallel arcs — each parallel arc counts as a distinct
    //    1-edge path (no internal vertex sharing required).
    let mut m = Graph::new(2, true)?;
    for _ in 0..4 {
        m.add_edge(0, 1)?;
    }
    println!("\n4 parallel arcs 0→1");
    println!(
        "  edge_disjoint_paths(0 → 1) = {}  (expected 4)",
        edge_disjoint_paths(&m, 0, 1)?
    );

    // 5) Disconnected endpoints — no path → 0.
    let isolated = Graph::new(4, true)?;
    println!("\nDisconnected endpoints (4 isolated vertices)");
    println!(
        "  edge_disjoint_paths(0 → 3) = {}  (expected 0)",
        edge_disjoint_paths(&isolated, 0, 3)?
    );

    Ok(())
}
