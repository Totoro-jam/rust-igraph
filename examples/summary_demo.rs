//! Demonstrates the `graph_summary` function for quick graph inspection.

use rust_igraph::{Graph, famous, full_graph, graph_summary, graph_summary_string};

fn main() {
    println!("=== Graph Summary Demo ===\n");

    // Famous graph: Petersen
    let petersen = famous("petersen").unwrap();
    let s = graph_summary(&petersen).unwrap();
    println!("Petersen graph: {s}");
    println!();

    // Complete graph K6
    let k6 = full_graph(6, false, false).unwrap();
    let s = graph_summary(&k6).unwrap();
    println!("K6: {s}");
    println!();

    // Custom graph from adjacency matrix
    let adj = vec![
        vec![0.0, 1.0, 0.0, 0.0, 1.0],
        vec![1.0, 0.0, 1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0, 1.0],
        vec![1.0, 0.0, 0.0, 1.0, 0.0],
    ];
    let cycle5 = Graph::from_adjacency_matrix(&adj, false).unwrap();
    let s = graph_summary(&cycle5).unwrap();
    println!("C5 (from adjacency matrix): {s}");
    println!();

    // Disconnected graph
    let mut g = Graph::with_vertices(8);
    // Component 1: triangle
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();
    // Component 2: path
    g.add_edge(3, 4).unwrap();
    g.add_edge(4, 5).unwrap();
    // Vertices 6 and 7 are isolated

    println!("Disconnected graph (detailed):");
    println!("{}", graph_summary_string(&g).unwrap());
}
