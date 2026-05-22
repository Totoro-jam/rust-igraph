//! ALGO-CO-007 demo: load Zachary's karate club, run Clauset-Newman-Moore
//! fast greedy modularity community detection, and print the best-Q
//! partition along with the dendrogram modularity trajectory.
//!
//! Run from the repo root: `cargo run --example fast_greedy_karate`.

#![allow(clippy::many_single_char_names)]

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{FastGreedyResult, fast_greedy_modularity, modularity};

fn print_result(g: &rust_igraph::Graph, r: &FastGreedyResult) {
    let k = r.nb_clusters;
    let q = modularity(g, &r.membership, 1.0)
        .ok()
        .flatten()
        .unwrap_or(0.0);
    println!("best partition: k = {k}, Q = {q:.6}");
    let mut by_community: Vec<Vec<u32>> = vec![Vec::new(); k as usize];
    for (v, &c) in r.membership.iter().enumerate() {
        by_community[c as usize].push(u32::try_from(v).expect("vertex id fits u32"));
    }
    println!("  communities (of {} vertices):", g.vcount());
    for (cid, members) in by_community.iter().enumerate() {
        println!("    c{cid} ({} vertices): {members:?}", members.len());
    }

    println!("  dendrogram: {} merges total", r.merges.len());
    let merge_preview = r.merges.len().min(5);
    if merge_preview > 0 {
        println!("  first {merge_preview} merges (each row is [c1, c2] -> n + i):");
        for (idx, row) in r.merges.iter().take(merge_preview).enumerate() {
            println!(
                "    merge #{idx}: {row:?}  Q after = {:.6}",
                r.modularity[idx + 1]
            );
        }
    }

    let (best_step, best_q) = r
        .modularity
        .iter()
        .enumerate()
        .filter(|(_, q)| !q.is_nan())
        .fold((0usize, f64::NEG_INFINITY), |(bi, bq), (i, &q)| {
            if q > bq { (i, q) } else { (bi, bq) }
        });
    println!(
        "  best-Q cut at step {best_step} of {} (Q = {best_q:.6})",
        r.merges.len()
    );
}

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

    let r = fast_greedy_modularity(&g)?;
    print_result(&g, &r);

    Ok(())
}
