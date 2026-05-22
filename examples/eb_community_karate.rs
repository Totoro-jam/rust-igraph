//! ALGO-CO-006 demo: load Zachary's karate club, run Girvan-Newman
//! edge-betweenness community detection, and print the best-modularity
//! partition along with the first few edge removals from the dendrogram.
//!
//! Run from the repo root: `cargo run --example eb_community_karate`.

#![allow(clippy::many_single_char_names)]

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{EdgeBetweennessResult, edge_betweenness_community, modularity};

fn print_result(g: &rust_igraph::Graph, r: &EdgeBetweennessResult) {
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

    let preview = r.removed_edges.len().min(5);
    println!("  first {preview} edges removed (id, betweenness@removal):");
    for i in 0..preview {
        let eid = r.removed_edges[i];
        let eb = r.edge_betweenness[i];
        let (u, v) = g.edge(eid).expect("removed edge id is valid");
        println!("    #{i}: eid={eid} ({u}-{v})  eb = {eb:.4}");
    }

    println!("  dendrogram: {} merges total", r.merges.len());
    let merge_preview = r.merges.len().min(3);
    if merge_preview > 0 {
        println!("  last {merge_preview} merges (each row is [c1, c2] -> n + i):");
        let start = r.merges.len() - merge_preview;
        for (idx, row) in r.merges.iter().enumerate().skip(start) {
            println!("    merge #{idx}: {row:?}  Q = {:.6}", r.modularity[idx]);
        }
    }
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

    let r = edge_betweenness_community(&g)?;
    print_result(&g, &r);

    Ok(())
}
