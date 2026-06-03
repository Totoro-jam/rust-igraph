//! Complete graph analysis pipeline showcasing rust-igraph's API.
//!
//! Run with: `cargo run --example quickstart`

use rust_igraph::{
    Graph, GraphBuilder, betweenness, connected_components, degree_sequence, density, louvain,
    pagerank,
};

fn main() {
    // 1. Build a graph using the fluent builder
    let graph = GraphBuilder::undirected()
        .vertices(9)
        .edges(&[
            (0, 1),
            (0, 2),
            (1, 2),
            (1, 3),
            (2, 3),
            // cluster 1: vertices 0-3 are tightly connected
            (0, 3),
            // cluster 2: vertices 5-8
            (5, 6),
            (5, 7),
            (6, 7),
            (6, 8),
            (7, 8),
            // bridge between clusters
            (3, 4),
            (4, 5),
        ])
        .build()
        .unwrap();

    let vcount = graph.vcount();
    let ecount = graph.ecount();
    let d = density(&graph).unwrap().unwrap_or(0.0);
    println!("=== Graph Overview ===");
    println!("Vertices: {vcount}");
    println!("Edges: {ecount}");
    println!("Density: {d:.4}");
    println!("Directed: {}", graph.is_directed());
    println!();

    // 2. Connectivity analysis
    let components = connected_components(&graph).unwrap();
    let comp_count = components.count;
    let largest = components.membership.iter().filter(|&&c| c == 0).count();
    println!("=== Connectivity ===");
    println!("Connected components: {comp_count}");
    println!("Largest component size: {largest}");
    println!();

    // 3. Degree analysis
    let degrees = degree_sequence(&graph, rust_igraph::DegreeMode::All).unwrap();
    let max_degree = degrees.iter().max().copied().unwrap_or(0);
    let avg_degree = f64::from(degrees.iter().sum::<u32>()) / f64::from(graph.vcount());
    println!("=== Degree Statistics ===");
    println!("Degree sequence: {degrees:?}");
    println!("Max degree: {max_degree}");
    println!("Average degree: {avg_degree:.2}");
    println!();

    // 4. Centrality measures
    let pr = pagerank(&graph).unwrap();
    let bc = betweenness(&graph).unwrap();

    println!("=== Centrality (top 3 by PageRank) ===");
    let mut ranked: Vec<(usize, f64)> = pr.iter().copied().enumerate().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (rank, &(vertex, score)) in ranked.iter().take(3).enumerate() {
        let r = rank + 1;
        let b = bc[vertex];
        println!("  #{r}: vertex {vertex} (PageRank={score:.4}, betweenness={b:.2})");
    }
    println!();

    // 5. Community detection
    let communities = louvain(&graph).unwrap();
    let modularity = communities.modularity;
    println!("=== Community Detection (Louvain) ===");
    println!("Modularity: {modularity:.4}");
    println!("Membership: {:?}", communities.membership);

    let num_communities = *communities.membership.iter().max().unwrap_or(&0) + 1;
    println!("Number of communities: {num_communities}");
    println!();

    // 6. Iterate over edges (idiomatic Rust)
    println!("=== Bridge Edges (betweenness > 4.0) ===");
    for (u, v) in &graph {
        let edge_bc = f64::midpoint(bc[u as usize], bc[v as usize]);
        if edge_bc > 4.0 {
            println!("  {u} -- {v} (avg vertex betweenness: {edge_bc:.2})");
        }
    }

    // 7. Graph algebra
    let subgraph_a = Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], false, None).unwrap();
    let subgraph_b = Graph::from_edges(&[(1, 2), (2, 3), (3, 1)], false, None).unwrap();
    let merged = &subgraph_a | &subgraph_b;
    let mv = merged.vcount();
    let me = merged.ecount();
    println!();
    println!("=== Graph Algebra ===");
    println!("Union of two triangles: {mv} vertices, {me} edges");
}
