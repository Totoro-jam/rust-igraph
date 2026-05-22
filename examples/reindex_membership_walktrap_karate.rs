//! `reindex_membership` (ALGO-CM-014) — densify a hand-built membership.
//!
//! Run with `cargo run --release --example reindex_membership_walktrap_karate`.
//! Builds a messy membership over the karate club (sparse cluster ids
//! plus a singleton in a far-away id) and shows how
//! [`reindex_membership`] compresses the labels to `0..k-1` without
//! changing the partition. Then verifies the densified membership
//! produces the same modularity as the original one.

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{modularity, read_edgelist, reindex_membership};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    let g = read_edgelist(File::open(path)?)?;
    let n = g.vcount();

    // Hand-built "messy" membership: a real karate-style 2-cluster
    // split, but with cluster ids 1_000 and 4_242 and one singleton
    // tagged 9_999 to force the sparse branch.
    let mr_hi = 1_000u32;
    let officer = 4_242u32;
    let outsider = 9_999u32;
    let messy: Vec<u32> = (0..n)
        .map(|v| match v {
            // The "officer" / John A side (rough karate split).
            0..=8 | 10..=11 | 16..=17 | 19 | 21 => mr_hi,
            // Lone wolf — sits in its own cluster id.
            33 => outsider,
            _ => officer,
        })
        .collect();

    println!("Karate club ({n} vertices). Messy input membership:");
    println!("  {messy:?}");
    let q_before = modularity(&g, &messy, 1.0)?.unwrap_or(0.0);
    println!("  modularity Q = {q_before:.4}");

    let r = reindex_membership(&messy)?;
    println!(
        "\nDensified to {} clusters via reindex_membership:",
        r.nb_clusters()
    );
    println!("  membership = {:?}", r.membership);
    println!("  new_to_old = {:?}", r.new_to_old);

    let q_after = modularity(&g, &r.membership, 1.0)?.unwrap_or(0.0);
    println!("  modularity Q = {q_after:.4}");

    if (q_before - q_after).abs() < 1e-12 {
        println!("\nQ before = Q after to 1e-12 — densification preserves the partition.");
    } else {
        println!(
            "\nWARNING: Q differs by {} — densification changed the partition?",
            (q_before - q_after).abs()
        );
    }
    Ok(())
}
