//! ALGO-FL-002 example: max-flow on the CLRS textbook instance + a
//! layered network where the answer is known by construction.
//!
//! Run: `cargo run --example max_flow_demo`.

use rust_igraph::{Graph, max_flow_value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) CLRS 26.1-1: 6-vertex / 10-arc instance, max flow = 23.
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
    let f = max_flow_value(&g, 0, 5, Some(&caps))?;
    println!("CLRS textbook (6v, 10 arcs)");
    println!("  source = 0, sink = 5");
    println!("  max flow = {f}  (expected 23)");

    // 2) Layered unit-capacity network: 4 layers × 8 columns, plus
    // source + sink. Min layer cut = 8, so max flow = 8.
    let layers = 4u32;
    let width = 8u32;
    let n = layers * width + 2;
    let source = 0u32;
    let sink = n - 1;
    let mut h = Graph::new(n, true)?;
    let mut hcaps: Vec<f64> = Vec::new();
    let idx = |layer: u32, col: u32| 1 + layer * width + col;
    for col in 0..width {
        h.add_edge(source, idx(0, col))?;
        hcaps.push(1.0);
    }
    for layer in 0..(layers - 1) {
        for a in 0..width {
            for b in 0..width {
                h.add_edge(idx(layer, a), idx(layer + 1, b))?;
                hcaps.push(1.0);
            }
        }
    }
    for col in 0..width {
        h.add_edge(idx(layers - 1, col), sink)?;
        hcaps.push(1.0);
    }
    let f2 = max_flow_value(&h, source, sink, Some(&hcaps))?;
    println!("\nLayered unit-cap network (4 layers, 8 wide)");
    println!("  vertices = {}, edges = {}", h.vcount(), h.ecount());
    println!("  max flow = {f2}  (expected {width})");

    // 3) Undirected demo: same conformance fixture as the C unit test.
    let mut und = Graph::new(4, false)?;
    let uedges = [(0u32, 1u32), (0, 2), (1, 2), (1, 3), (2, 3)];
    let ucaps = [4.0, 2.0, 10.0, 2.0, 2.0];
    for (src, dst) in uedges {
        und.add_edge(src, dst)?;
    }
    let f3 = max_flow_value(&und, 0, 3, Some(&ucaps))?;
    println!("\nUndirected igraph_maxflow.c reference (4v)");
    println!("  caps = {ucaps:?}");
    println!("  max flow = {f3}  (expected 4)");

    Ok(())
}
