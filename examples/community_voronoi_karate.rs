//! `community_voronoi` (ALGO-CO-009) — Voronoi-based community detection
//! on Zachary's karate club.
//!
//! Run with `cargo run --release --example community_voronoi_karate`.
//! Mirrors the deterministic portion of
//! `references/igraph/tests/unit/igraph_community_voronoi.c`: the
//! Zachary karate club, undirected, unweighted, with the auto-r
//! optimizer (`r = -1`) that picks `r` to maximise Newman-Girvan
//! modularity.
//!
//! Prints:
//!
//! * the generator vertices picked by the LRD greedy step,
//! * the modularity at the chosen `r` (when auto-r is used),
//! * the cell sizes and members of the resulting partition.
//!
//! On Zachary, the reference C implementation picks generators
//! `[33, 0, 24]` and produces 3 communities; our self-rolled
//! implementation reproduces both numbers bit-exactly.

#![allow(clippy::cast_possible_truncation)]

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{DijkstraMode, community_voronoi, read_edgelist};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    let g = read_edgelist(File::open(path)?)?;
    println!("Karate club: {} vertices, {} edges", g.vcount(), g.ecount());

    // r = -1 ⇒ Brent quadratic-fit search for the modularity-maximising r.
    let res = community_voronoi(&g, None, None, DijkstraMode::All, -1.0)?;

    println!("\nauto-r result:");
    println!("  generators : {:?}", res.generators);
    match res.modularity {
        Some(q) => println!("  modularity : {q:.6}"),
        None => println!("  modularity : (n/a)"),
    }

    let k = res.generators.len();
    let mut sizes = vec![0usize; k];
    let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); k];
    for (v, &m) in res.membership.iter().enumerate() {
        let i = m as usize;
        sizes[i] += 1;
        buckets[i].push(v as u32);
    }
    for i in 0..k {
        let members: Vec<String> = buckets[i].iter().map(u32::to_string).collect();
        println!(
            "  cell {i} (generator {:>2}): size = {:>2} / {}",
            res.generators[i],
            sizes[i],
            res.membership.len()
        );
        println!("    members: [{}]", members.join(", "));
    }

    Ok(())
}
