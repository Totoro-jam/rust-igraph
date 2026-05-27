//! ALGO-SP-012 example: path length histogram demo.
//!
//! Run: `cargo run --example path_length_hist_demo`.

use rust_igraph::{Graph, create, path_length_hist};

fn main() {
    println!("=== Undirected path 0-1-2-3-4 ===");
    let mut g = Graph::with_vertices(5);
    for i in 0..4 {
        g.add_edge(i, i + 1).unwrap();
    }
    let r = path_length_hist(&g, false).unwrap();
    println!("  Histogram: {:?}", r.hist);
    println!("  Unconnected: {}", r.unconnected);

    println!("\n=== Undirected cycle 0-1-2-3-4-0 ===");
    let mut g = Graph::with_vertices(5);
    for i in 0..5 {
        g.add_edge(i, (i + 1) % 5).unwrap();
    }
    let r = path_length_hist(&g, false).unwrap();
    println!("  Histogram: {:?}", r.hist);
    println!("  Unconnected: {}", r.unconnected);

    println!("\n=== K4 (complete on 4 vertices) ===");
    let g = create(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)], 4, false).unwrap();
    let r = path_length_hist(&g, false).unwrap();
    println!("  Histogram: {:?}", r.hist);
    println!("  Unconnected: {}", r.unconnected);

    println!("\n=== Two components {{0,1}} + {{2,3,4}} ===");
    let g = create(&[(0, 1), (2, 3), (3, 4)], 5, false).unwrap();
    let r = path_length_hist(&g, false).unwrap();
    println!("  Histogram: {:?}", r.hist);
    println!("  Unconnected: {}", r.unconnected);

    println!("\n=== Directed path 0->1->2->3 (directed mode) ===");
    let g = create(&[(0, 1), (1, 2), (2, 3)], 4, true).unwrap();
    let r_dir = path_length_hist(&g, true).unwrap();
    println!("  Directed histogram: {:?}", r_dir.hist);
    println!("  Directed unconnected: {}", r_dir.unconnected);
    let r_undir = path_length_hist(&g, false).unwrap();
    println!("  Undirected histogram: {:?}", r_undir.hist);
    println!("  Undirected unconnected: {}", r_undir.unconnected);
}
