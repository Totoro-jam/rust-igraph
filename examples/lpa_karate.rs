//! ALGO-CO-004 demo: load Zachary's karate club, run label propagation
//! (Raghavan–Albert–Kumara 2007; Traag–Šubelj 2023 fast variant) with
//! all three built-in variants, and print the discovered partitions
//! along with the modularity score (computed from the membership via
//! `modularity()`, since LPA itself does not optimise an objective).
//!
//! Run from the repo root: `cargo run --example lpa_karate`.

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{LpaOptions, LpaResult, LpaVariant, label_propagation_with_options, modularity};

fn print_partition(name: &str, g: &rust_igraph::Graph, r: &LpaResult) {
    let k = r.nb_clusters;
    let q = modularity(g, &r.membership, 1.0)
        .ok()
        .flatten()
        .unwrap_or(0.0);
    println!("{name}: k = {k}, Q = {q:.6}");
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

    // Fast variant (default) — Traag–Šubelj 2023 queue-based kernel.
    let fast_opts = LpaOptions {
        variant: LpaVariant::Fast,
        seed: 42,
        ..LpaOptions::default()
    };
    let r_fast = label_propagation_with_options(&g, None, &fast_opts)?;
    print_partition("Fast (seed=42)", &g, &r_fast);

    // Dominance variant — Raghavan–Albert–Kumara alternating control/
    // update iterations; converges when no vertex's label changes.
    let dom_opts = LpaOptions {
        variant: LpaVariant::Dominance,
        seed: 42,
        ..LpaOptions::default()
    };
    let r_dom = label_propagation_with_options(&g, None, &dom_opts)?;
    print_partition("Dominance (seed=42)", &g, &r_dom);

    // Retention variant — keep the current label when it remains
    // dominant, otherwise pick a random majority-label neighbour.
    let ret_opts = LpaOptions {
        variant: LpaVariant::Retention,
        seed: 42,
        ..LpaOptions::default()
    };
    let r_ret = label_propagation_with_options(&g, None, &ret_opts)?;
    print_partition("Retention (seed=42)", &g, &r_ret);

    Ok(())
}
