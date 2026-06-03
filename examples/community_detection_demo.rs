//! Community detection comparison on a synthetic two-community graph.
//!
//! Run: `cargo run --example community_detection_demo`

use rust_igraph::Graph;

fn main() {
    // Build a graph with two dense clusters connected by a sparse bridge.
    let mut edges: Vec<(u32, u32)> = Vec::new();

    // Cluster A: vertices 0..10 (near-complete)
    for i in 0u32..10 {
        for j in (i + 1)..10 {
            if (i + j) % 7 != 0 {
                edges.push((i, j));
            }
        }
    }

    // Cluster B: vertices 10..20 (near-complete)
    for i in 10u32..20 {
        for j in (i + 1)..20 {
            if (i + j) % 7 != 0 {
                edges.push((i, j));
            }
        }
    }

    // Inter-cluster bridges (sparse)
    edges.push((3, 12));
    edges.push((7, 15));

    let g = Graph::from_edges(&edges, false, None).unwrap();
    println!("Graph: {} vertices, {} edges", g.vcount(), g.ecount());
    println!();

    // Louvain
    let louvain = g.louvain().unwrap();
    let louvain_k = *louvain.membership.iter().max().unwrap_or(&0) + 1;
    println!(
        "Louvain:  {louvain_k} communities, modularity = {:.4}",
        louvain.modularity
    );

    // Leiden
    let leiden = g.leiden().unwrap();
    println!(
        "Leiden:   {} communities, quality    = {:.4}",
        leiden.nb_clusters, leiden.quality
    );

    // Structural properties
    let density = g.density().unwrap().unwrap_or(0.0);
    let transitivity = g.transitivity().unwrap().unwrap_or(0.0);
    println!();
    println!("Density:       {density:.4}");
    println!("Transitivity:  {transitivity:.4}");
    println!("Connected:     {}", g.is_connected().unwrap());
    println!("Bridges:       {:?}", g.bridges().unwrap());
}
