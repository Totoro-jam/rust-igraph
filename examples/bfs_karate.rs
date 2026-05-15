//! Phase 0 smoke test: load Zachary's Karate Club, BFS from vertex 0,
//! print the visit order. Used by the BOOT-11 oracle test as a reference.
//!
//! Run from the repo root: `cargo run --example bfs_karate`.

use std::fs::File;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // CARGO_MANIFEST_DIR is the `igraph` crate; fixture lives at repo-root/fixtures.
    path.push("../../fixtures/karate.edges");
    let path = path.canonicalize()?;

    let file = File::open(&path)?;
    let g = igraph::read_edgelist(file)?;
    println!(
        "loaded {} ({} vertices, {} edges)",
        path.display(),
        g.vcount(),
        g.ecount()
    );

    let order = igraph::bfs(&g, 0)?;
    println!("BFS from 0: {order:?}");
    println!("  visited {} of {} vertices", order.len(), g.vcount());

    Ok(())
}
