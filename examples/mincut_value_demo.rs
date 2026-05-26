//! ALGO-FL-017 example: global minimum-cut value (weighted
//! generalisation of [`edge_connectivity`](rust_igraph::edge_connectivity)).
//! Equivalent to `min_{v != 0} max_flow_value(graph, 0, v, capacity)`
//! (both directions for directed graphs).
//!
//! With unit capacities the result equals
//! `edge_connectivity(graph, true) as f64`. With per-edge capacities,
//! the bottleneck edge dominates the cut and the result reports the
//! total weight that must be removed to disconnect some pair.
//!
//! Run: `cargo run --example mincut_value_demo`.

use rust_igraph::{Graph, edge_connectivity, mincut_value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) Empty graph — no pair to separate, mincut_value = +inf
    //    (mirrors IGRAPH_INFINITY init at flow.c:1699).
    let empty = Graph::new(0, false)?;
    println!("Empty graph");
    let mc = mincut_value(&empty, None)?;
    println!("  mincut_value = {mc}  (expected +inf)");

    // 2) Single-vertex graph — same +inf corner case.
    let single = Graph::new(1, false)?;
    println!("\nSingle vertex");
    println!(
        "  mincut_value = {}  (expected +inf)",
        mincut_value(&single, None)?
    );

    // 3) Undirected ring on 5 vertices with unit capacities — matches
    //    edge_connectivity(C_5) = 2.
    let mut ring = Graph::new(5, false)?;
    for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 0)] {
        ring.add_edge(u, v)?;
    }
    println!("\nUndirected 5-cycle C_5 (unit caps)");
    println!(
        "  mincut_value = {}  (expected 2.0)",
        mincut_value(&ring, None)?
    );
    let ec = edge_connectivity(&ring, true)?;
    println!("  edge_connectivity = {ec}  (matches mincut_value at unit caps)");

    // 4) Same ring, but a single weak edge — bridge weight 0.5, rest
    //    weight 10. The cheapest 2-edge cut is bridge + a neighbour:
    //    0.5 + 10 = 10.5.
    let caps = vec![10.0, 10.0, 0.5, 10.0, 10.0];
    println!("\nC_5 with capacities {caps:?}");
    let mc = mincut_value(&ring, Some(&caps))?;
    println!("  mincut_value = {mc}  (expected 10.5)");

    // 5) Two disconnected edges — min cut is empty, value = 0.
    let mut disc = Graph::new(4, false)?;
    disc.add_edge(0, 1)?;
    disc.add_edge(2, 3)?;
    println!("\nTwo isolated edges {{0-1, 2-3}}");
    println!(
        "  mincut_value = {}  (expected 0.0 — graph not connected)",
        mincut_value(&disc, None)?
    );

    // 6) Directed 3-cycle with mixed arc capacities — every arc is a
    //    directed bridge, so the bottleneck arc determines the cut.
    let mut dcycle = Graph::new(3, true)?;
    dcycle.add_edge(0, 1)?;
    dcycle.add_edge(1, 2)?;
    dcycle.add_edge(2, 0)?;
    let dcaps = vec![3.0, 1.0, 2.0];
    println!("\nDirected 3-cycle 0→1→2→0 with caps {dcaps:?}");
    println!(
        "  mincut_value = {}  (expected 1.0 — bottleneck arc 1→2)",
        mincut_value(&dcycle, Some(&dcaps))?
    );

    // 7) Multigraph: two parallel edges between 0 and 1 — parallel
    //    edges raise the min cut to 2 even though "K_2".
    let mut multi = Graph::new(2, false)?;
    multi.add_edge(0, 1)?;
    multi.add_edge(0, 1)?;
    println!("\nMultigraph: 2 parallel edges between 0-1");
    println!(
        "  mincut_value = {}  (expected 2.0 — parallel edges raise the cut)",
        mincut_value(&multi, None)?
    );

    // 8) Complete K_5 with non-uniform capacities — pick capacities
    //    so that isolating vertex 0 (sum of its incident edge weights)
    //    is the unique minimum cut.
    let mut k5 = Graph::new(5, false)?;
    let mut k5_caps = Vec::new();
    for i in 0u32..5 {
        for j in (i + 1)..5 {
            k5.add_edge(i, j)?;
            // Edges incident on vertex 0 get weight 1; all other edges
            // get weight 100 — isolating vertex 0 costs 4·1 = 4.
            k5_caps.push(if i == 0 || j == 0 { 1.0 } else { 100.0 });
        }
    }
    println!("\nK_5 with cheap edges on vertex 0 (rest = 100)");
    println!(
        "  mincut_value = {}  (expected 4.0 — isolate vertex 0)",
        mincut_value(&k5, Some(&k5_caps))?
    );

    Ok(())
}
