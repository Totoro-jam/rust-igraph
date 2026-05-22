//! ALGO-CO-002 demo: load Zachary's karate club, run Louvain multilevel
//! community detection, and print the discovered partition along with
//! the final modularity and the per-level history.
//!
//! Run from the repo root: `cargo run --example louvain_karate`.

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

    let r = rust_igraph::louvain(&g)?;
    let k = r.membership.iter().copied().max().map_or(0, |m| m + 1);
    println!(
        "Louvain converged: Q = {:.6}, k = {}, levels = {}",
        r.modularity,
        k,
        r.levels.len(),
    );

    // Cross-check against the standalone modularity() function — they
    // must agree to f64 precision on the returned partition.
    if let Some(q) = rust_igraph::modularity(&g, &r.membership, 1.0)? {
        println!(
            "cross-check vs modularity(): {q:.6} (Δ = {:.2e})",
            (r.modularity - q).abs()
        );
    }

    // Print per-level modularity history.
    println!("per-level modularity:");
    for (i, q) in r.modularities.iter().enumerate() {
        println!("  level {i}: {q:.6}");
    }

    // Group vertices by community for human-readable output.
    let mut by_community: Vec<Vec<u32>> = vec![Vec::new(); k as usize];
    for (v, &c) in r.membership.iter().enumerate() {
        by_community[c as usize].push(u32::try_from(v).expect("vertex id fits u32"));
    }
    println!("communities:");
    for (cid, members) in by_community.iter().enumerate() {
        println!("  c{cid} ({} vertices): {members:?}", members.len());
    }

    Ok(())
}
