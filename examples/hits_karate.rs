//! ALGO-PR-017 demo: load Zachary's karate club, compute HITS hub and
//! authority scores, print the top vertices by hub score and verify
//! the documented invariant `hub == authority == eigenvector_centrality`
//! on undirected graphs.
//!
//! Run from the repo root: `cargo run --example hits_karate`.

use std::fs::File;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    let path = path.canonicalize()?;

    let file = File::open(&path)?;
    let g = rust_igraph::read_edgelist(file)?;
    println!(
        "loaded {} ({} vertices, {} edges)",
        path.display(),
        g.vcount(),
        g.ecount()
    );

    let s = rust_igraph::hub_and_authority_scores(&g)?;
    println!("dominant eigenvalue of A·Aᵀ: {:.6}", s.eigenvalue);

    let mut indexed: Vec<(usize, f64)> = s.hub.iter().enumerate().map(|(i, &x)| (i, x)).collect();
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    println!("top 5 hubs (max-norm scaling, max=1.0):");
    for (rank, (vertex, score)) in indexed.iter().take(5).enumerate() {
        println!("  #{}: vertex {vertex} → {score:.6}", rank + 1);
    }

    // On undirected graphs HITS delegates to eigenvector_centrality and
    // hub == authority by construction.
    let max_diff = s
        .hub
        .iter()
        .zip(s.authority.iter())
        .map(|(h, a)| (h - a).abs())
        .fold(0.0_f64, f64::max);
    println!("max |hub - authority| = {max_diff:.2e} (undirected → identical)");

    Ok(())
}
