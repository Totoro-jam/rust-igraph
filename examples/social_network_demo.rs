//! Comprehensive social network analysis demo.
//!
//! Demonstrates rust-igraph's breadth: graph construction, community
//! detection, centrality, shortest paths, and structural properties
//! — all in one coherent workflow on Zachary's karate club.
//!
//! Run: `cargo run --example social_network_demo`

use std::fs::File;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. Load graph ---
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    let file = File::open(path.canonicalize()?)?;
    let g = rust_igraph::read_edgelist(file)?;
    println!("=== Zachary's Karate Club ===");
    println!("{g}");
    println!();

    // --- 2. Basic properties ---
    #[allow(clippy::cast_precision_loss)]
    let density = g.ecount() as f64 / (f64::from(g.vcount()) * (f64::from(g.vcount()) - 1.0) / 2.0);
    let is_conn = rust_igraph::is_connected(&g, rust_igraph::ConnectednessMode::Weak)?;
    println!("Density: {density:.4}");
    println!("Connected: {is_conn}");
    let diam = rust_igraph::diameter(&g)?;
    println!(
        "Diameter: {}",
        diam.map_or("N/A".to_string(), |d| d.to_string())
    );
    println!();

    // --- 3. Centrality ---
    let pr = rust_igraph::pagerank(&g)?;
    let bc = rust_igraph::betweenness(&g)?;
    let cl = rust_igraph::closeness(&g)?;

    let mut top_pr: Vec<(usize, f64)> = pr.iter().copied().enumerate().collect();
    top_pr.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    println!("Top-5 by PageRank:");
    for (v, score) in top_pr.iter().take(5) {
        println!(
            "  vertex {v:>2}: PR={score:.4}, betweenness={:.1}, closeness={:.4}",
            bc[*v],
            cl[*v].unwrap_or(0.0)
        );
    }
    println!();

    // --- 4. Community detection ---
    let louvain = rust_igraph::louvain(&g)?;
    let n_communities = louvain
        .membership
        .iter()
        .copied()
        .max()
        .map_or(0, |m| m + 1);
    println!(
        "Louvain: {n_communities} communities, Q = {:.4}",
        louvain.modularity
    );

    for c in 0..n_communities {
        let members: Vec<u32> = g
            .vertex_ids()
            .filter(|&v| louvain.membership[v as usize] == c)
            .collect();
        println!(
            "  Community {c}: {} members {:?}",
            members.len(),
            if members.len() > 8 {
                &members[..8]
            } else {
                &members
            }
        );
    }
    println!();

    // --- 5. Shortest paths ---
    let paths = rust_igraph::distances(&g, 0)?;
    let reachable: Vec<f64> = paths.iter().filter_map(|d| d.map(f64::from)).collect();
    #[allow(clippy::cast_precision_loss)]
    let avg_dist = reachable.iter().sum::<f64>() / (reachable.len().max(1) as f64);
    println!("Average distance from vertex 0: {avg_dist:.2}");

    // --- 6. Structural properties ---
    let transitivity = rust_igraph::transitivity_undirected(&g)?;
    println!(
        "Global clustering coefficient: {:.4}",
        transitivity.unwrap_or(0.0)
    );

    let components = rust_igraph::connected_components(&g)?;
    println!("Connected components: {}", components.count);

    println!("\nDone — {} algorithms demonstrated in one workflow.", 8);
    Ok(())
}
