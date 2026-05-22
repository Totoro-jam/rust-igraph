//! `community_to_membership` (ALGO-CM-013) — replay walktrap's dendrogram.
//!
//! Run with `cargo run --release --example community_to_membership_walktrap_karate`.
//! Drives [`walktrap`] on the karate club, then re-cuts the resulting
//! dendrogram at several depths via [`community_to_membership`] to
//! show how the partition coarsens as we feed the cut more merges.

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{community_to_membership, modularity, read_edgelist, walktrap};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    let g = read_edgelist(File::open(path)?)?;
    let n = g.vcount();

    let wt = walktrap(&g)?;
    println!(
        "Walktrap on karate ({} vertices): best-Q cut at k = {}, modularity = {:.4}",
        n,
        wt.nb_clusters,
        wt.modularity.last().copied().unwrap_or(0.0),
    );
    println!("dendrogram: {} merges", wt.merges.len());

    let max_steps = u32::try_from(wt.merges.len())?;
    println!("\nRe-cutting via community_to_membership:");
    println!("{:>6} {:>6} {:>10}  csize", "steps", "k", "Q");
    for &steps in &[
        0u32,
        max_steps / 4,
        max_steps / 2,
        max_steps - wt.nb_clusters,
        max_steps,
    ] {
        if steps > max_steps {
            continue;
        }
        let cut = community_to_membership(&wt.merges, n, steps)?;
        let q = modularity(&g, &cut.membership, 1.0)?.unwrap_or(0.0);
        let mut sizes = cut.csize.clone();
        sizes.sort_unstable();
        sizes.reverse();
        let preview: Vec<String> = sizes.iter().take(8).map(u32::to_string).collect();
        let tail = if cut.csize.len() > 8 { ", ..." } else { "" };
        println!(
            "{:>6} {:>6} {:>10.4}  [{}{}]",
            steps,
            cut.csize.len(),
            q,
            preview.join(", "),
            tail,
        );
    }

    Ok(())
}
