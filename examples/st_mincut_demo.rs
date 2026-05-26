//! ALGO-FL-010 example: scalar s-t minimum-cut value on textbook
//! instances, illustrating max-flow / min-cut duality.
//!
//! Run: `cargo run --example st_mincut_demo`.

use rust_igraph::{Graph, max_flow_value, st_mincut_value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) CLRS 26.1-1: 6-vertex / 10-arc instance, min s-t cut = 23.
    let mut g = Graph::new(6, true)?;
    let arcs = [
        (0u32, 1u32),
        (0, 2),
        (1, 2),
        (1, 3),
        (2, 1),
        (2, 4),
        (3, 2),
        (3, 5),
        (4, 3),
        (4, 5),
    ];
    let caps = [16.0, 13.0, 10.0, 12.0, 4.0, 14.0, 9.0, 20.0, 7.0, 4.0];
    for (u, v) in arcs {
        g.add_edge(u, v)?;
    }
    let cut = st_mincut_value(&g, 0, 5, Some(&caps))?;
    let flow = max_flow_value(&g, 0, 5, Some(&caps))?;
    println!("CLRS textbook (6v, 10 arcs)");
    println!("  source = 0, sink = 5");
    println!("  st_mincut_value = {cut}  (expected 23)");
    println!("  max_flow_value  = {flow}  (duality: must equal cut)");

    // 2) Two parallel unit-cap paths: 0→1→3 and 0→2→3. Min cut = 2
    // (must cut both bottleneck edges to disconnect source from sink).
    let mut h = Graph::new(4, true)?;
    h.add_edge(0, 1)?;
    h.add_edge(1, 3)?;
    h.add_edge(0, 2)?;
    h.add_edge(2, 3)?;
    let cut2 = st_mincut_value(&h, 0, 3, Some(&[1.0, 1.0, 1.0, 1.0]))?;
    println!("\nTwo parallel unit-cap paths (4v)");
    println!("  st_mincut_value(0 → 3) = {cut2}  (expected 2)");

    // 3) Disconnected endpoints — the empty cut already separates them.
    let isolated = Graph::new(4, true)?;
    let cut3 = st_mincut_value(&isolated, 0, 3, None)?;
    println!("\nDisconnected endpoints (4 isolated vertices)");
    println!("  st_mincut_value(0 → 3) = {cut3}  (expected 0)");

    // 4) Undirected igraph_maxflow.c reference: matches the C unit test
    // input, with the cut interpretation. Same 4-vertex undirected graph
    // edges (0-1,0-2,1-2,1-3,2-3) and caps (4,2,10,2,2) → cut = 4.
    let mut und = Graph::new(4, false)?;
    let uedges = [(0u32, 1u32), (0, 2), (1, 2), (1, 3), (2, 3)];
    let ucaps = [4.0, 2.0, 10.0, 2.0, 2.0];
    for (src, dst) in uedges {
        und.add_edge(src, dst)?;
    }
    let cut4 = st_mincut_value(&und, 0, 3, Some(&ucaps))?;
    println!("\nUndirected igraph_maxflow.c reference (4v)");
    println!("  caps = {ucaps:?}");
    println!("  st_mincut_value(0 → 3) = {cut4}  (expected 4)");

    Ok(())
}
