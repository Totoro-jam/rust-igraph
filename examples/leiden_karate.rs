//! ALGO-CO-003 demo: load Zachary's karate club, run Leiden
//! (Traag-Waltman-van Eck 2019) community detection with all three
//! built-in objectives, and print the discovered partitions along with
//! the final quality scores and per-iteration history.
//!
//! Run from the repo root: `cargo run --example leiden_karate`.

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{LeidenObjective, LeidenOptions, LeidenResult, leiden_with_options};

fn print_partition(name: &str, g_n: u32, r: &LeidenResult) {
    let k = r.nb_clusters;
    println!(
        "{name}: Q = {:.6}, k = {}, iterations = {}",
        r.quality, k, r.n_iterations_run
    );
    println!("  per-iteration quality:");
    for (i, q) in r.qualities.iter().enumerate() {
        println!("    iter {i}: {q:.6}");
    }
    let mut by_community: Vec<Vec<u32>> = vec![Vec::new(); k as usize];
    for (v, &c) in r.membership.iter().enumerate() {
        by_community[c as usize].push(u32::try_from(v).expect("vertex id fits u32"));
    }
    println!("  communities (of {g_n} vertices):");
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

    // Modularity objective (γ=1) — the canonical Leiden call. On a
    // loop-free graph this equals Newman-Girvan modularity, so we
    // cross-check against the standalone modularity() function.
    let mod_opts = LeidenOptions {
        objective: LeidenObjective::Modularity,
        resolution: 1.0,
        seed: 42,
        ..LeidenOptions::default()
    };
    let r_mod = leiden_with_options(&g, None, &mod_opts)?;
    print_partition("Modularity (γ=1, seed=42)", g.vcount(), &r_mod);
    if let Some(q) = rust_igraph::modularity(&g, &r_mod.membership, 1.0)? {
        println!(
            "  cross-check vs modularity(): {q:.6} (Δ = {:.2e})",
            (r_mod.quality - q).abs()
        );
    }

    // CPM objective with a small γ — finds a coarser partition than
    // Modularity at γ=1. With γ → 0 the whole graph becomes one
    // community; with γ → ∞ it collapses to singletons.
    let cpm_opts = LeidenOptions {
        objective: LeidenObjective::Cpm,
        resolution: 0.05,
        seed: 42,
        ..LeidenOptions::default()
    };
    let r_cpm = leiden_with_options(&g, None, &cpm_opts)?;
    print_partition("CPM (γ=0.05, seed=42)", g.vcount(), &r_cpm);

    // ER objective — Reichardt-Bornholdt null model based on a flat
    // edge probability p = m / C(n,2) rather than degree-degree.
    let er_opts = LeidenOptions {
        objective: LeidenObjective::Er,
        resolution: 1.0,
        seed: 42,
        ..LeidenOptions::default()
    };
    let r_er = leiden_with_options(&g, None, &er_opts)?;
    print_partition("ER (γ=1, seed=42)", g.vcount(), &r_er);

    Ok(())
}
