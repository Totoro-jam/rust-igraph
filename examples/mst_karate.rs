//! ALGO-MST-001 example: minimum spanning tree of Zachary's Karate Club.
//!
//! Loads `fixtures/karate.edges` (34 vertices, 78 edges), assigns
//! deterministic synthetic edge weights, and runs all three MST
//! variants (Unweighted, Kruskal, Prim) plus Automatic dispatch.
//! The weighted variants must agree on total weight (matroid
//! optimality); the unweighted variant returns a BFS spanning tree.
//!
//! Run: `cargo run --example mst_karate`.

#![allow(clippy::cast_precision_loss)]

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{MstAlgorithm, minimum_spanning_tree, read_edgelist};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    let g = read_edgelist(File::open(&path)?)?;
    println!(
        "loaded karate: {} vertices, {} edges",
        g.vcount(),
        g.ecount()
    );

    // Deterministic weights: w_i = i + 1.0 — distinct, so the MST is
    // unique and the variants agree on the edge set.
    let weights: Vec<f64> = (0..g.ecount()).map(|i| (i as f64) + 1.0).collect();

    let unweighted = minimum_spanning_tree(&g, None, MstAlgorithm::Unweighted)?;
    let kruskal = minimum_spanning_tree(&g, Some(&weights), MstAlgorithm::Kruskal)?;
    let prim = minimum_spanning_tree(&g, Some(&weights), MstAlgorithm::Prim)?;
    let automatic = minimum_spanning_tree(&g, Some(&weights), MstAlgorithm::Automatic)?;

    let total = |edges: &[u32]| -> f64 { edges.iter().map(|&e| weights[e as usize]).sum() };

    println!(
        "unweighted BFS forest: {} edges (expected vcount-1 = {})",
        unweighted.len(),
        g.vcount() - 1
    );
    println!(
        "kruskal: {} edges, total weight {:.2}",
        kruskal.len(),
        total(&kruskal)
    );
    println!(
        "prim:    {} edges, total weight {:.2}",
        prim.len(),
        total(&prim)
    );
    println!(
        "auto:    {} edges, total weight {:.2} (dispatched to kruskal)",
        automatic.len(),
        total(&automatic)
    );

    // Matroid optimality: every weighted variant agrees on total cost.
    assert!((total(&kruskal) - total(&prim)).abs() < 1e-12);
    assert!((total(&kruskal) - total(&automatic)).abs() < 1e-12);
    println!("matroid invariant: kruskal == prim == automatic ✓");

    Ok(())
}
