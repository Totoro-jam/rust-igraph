//! ALGO-CO-006b demo: weighted Girvan-Newman edge-betweenness community
//! detection on Zachary's karate club. Runs the algorithm twice — once
//! with unit weights (which must agree with the unweighted slice) and
//! once with synthetic weights that boost the bridge between the two
//! coaches, demonstrating how weighted betweenness shifts the removal
//! order.
//!
//! Run from the repo root: `cargo run --example eb_community_weighted_karate`.

#![allow(clippy::many_single_char_names)]

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{
    EdgeBetweennessResult, edge_betweenness_community_weighted, modularity_weighted,
};

fn print_result(g: &rust_igraph::Graph, r: &EdgeBetweennessResult, weights: &[f64], tag: &str) {
    let k = r.nb_clusters;
    let q = modularity_weighted(g, &r.membership, 1.0, weights)
        .ok()
        .flatten()
        .unwrap_or(0.0);
    println!("[{tag}] best partition: k = {k}, weighted-Q = {q:.6}");
    let mut by_community: Vec<Vec<u32>> = vec![Vec::new(); k as usize];
    for (v, &c) in r.membership.iter().enumerate() {
        by_community[c as usize].push(u32::try_from(v).expect("vertex id fits u32"));
    }
    for (cid, members) in by_community.iter().enumerate() {
        println!("    c{cid} ({} vertices): {members:?}", members.len());
    }

    let preview = r.removed_edges.len().min(5);
    println!("  first {preview} edges removed (id, weighted-eb@removal):");
    for i in 0..preview {
        let eid = r.removed_edges[i];
        let eb = r.edge_betweenness[i];
        let (u, v) = g.edge(eid).expect("removed edge id is valid");
        println!(
            "    #{i}: eid={eid} ({u}-{v}) w={:.3}  eb={eb:.4}",
            weights[eid as usize]
        );
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

    // Pass 1: unit weights — equivalent to the unweighted slice.
    let unit = vec![1.0_f64; g.ecount()];
    let r_unit = edge_betweenness_community_weighted(&g, &unit)?;
    print_result(&g, &r_unit, &unit, "unit");

    // Pass 2: synthetic weights — make the (0, 33) coach-to-coach edge
    // cheap if present, otherwise leave everything 1.0. A cheap edge
    // attracts cross-community shortest paths and gets removed first.
    let mut tilted = vec![1.0_f64; g.ecount()];
    for (eid, slot) in tilted.iter_mut().enumerate() {
        let eid_u32 = u32::try_from(eid).expect("edge id fits u32");
        if let Ok((u, v)) = g.edge(eid_u32) {
            if (u == 0 && v == 33) || (u == 33 && v == 0) {
                *slot = 0.1;
            }
        }
    }
    let r_tilted = edge_betweenness_community_weighted(&g, &tilted)?;
    print_result(&g, &r_tilted, &tilted, "tilted (w(0,33)=0.1)");

    Ok(())
}
