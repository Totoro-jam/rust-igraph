//! ALGO-CO-005 demo: load Zachary's karate club, run Fluid Communities
//! (Parés et al. 2017) for several values of `k`, and print the
//! discovered partitions with their modularity (computed from the
//! membership via `modularity()`, since Fluid does not optimise an
//! objective).
//!
//! Run from the repo root: `cargo run --example fluid_karate`.

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{FluidOptions, FluidResult, fluid_communities_with_options, modularity};

fn print_partition(name: &str, g: &rust_igraph::Graph, r: &FluidResult) {
    let k = r.nb_clusters;
    let q = modularity(g, &r.membership, 1.0)
        .ok()
        .flatten()
        .unwrap_or(0.0);
    println!(
        "{name}: k = {k}, Q = {q:.6}, iters = {}",
        r.n_iterations_run
    );
    let mut by_community: Vec<Vec<u32>> = vec![Vec::new(); k as usize];
    for (v, &c) in r.membership.iter().enumerate() {
        by_community[c as usize].push(u32::try_from(v).expect("vertex id fits u32"));
    }
    println!("  communities (of {} vertices):", g.vcount());
    for (cid, members) in by_community.iter().enumerate() {
        println!("    c{cid} ({} vertices): {members:?}", members.len());
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

    for k in [2u32, 3, 4] {
        let opts = FluidOptions {
            seed: 42,
            ..FluidOptions::default()
        };
        let r = fluid_communities_with_options(&g, k, &opts)?;
        print_partition(&format!("k={k} (seed=42)"), &g, &r);
    }

    Ok(())
}
