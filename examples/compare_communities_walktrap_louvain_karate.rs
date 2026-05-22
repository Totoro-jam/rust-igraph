//! `compare_communities` (ALGO-CM-015) — quantify how two community
//! detection algorithms partition the same graph.
//!
//! Run with `cargo run --release --example
//! compare_communities_walktrap_louvain_karate`. Runs walktrap and louvain
//! on Zachary's karate club, then prints every partition-distance metric
//! (VI, NMI, split-join, Rand, adjusted Rand) for the two results.

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{
    CommunityComparison, WalktrapOptions, compare_communities, louvain, read_edgelist, walktrap,
    walktrap_with_options,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    let g = read_edgelist(File::open(path)?)?;
    println!("Karate club: {} vertices, {} edges", g.vcount(), g.ecount());

    // Walktrap and louvain are both deterministic on undirected graphs.
    let wt = walktrap(&g)?;
    let lv = louvain(&g)?;

    let best_q = wt
        .modularity
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let wt_k = wt.membership.iter().copied().max().map_or(0, |m| m + 1);
    let lv_k = lv.membership.iter().copied().max().map_or(0, |m| m + 1);

    println!("\nwalktrap best-Q cut: k = {wt_k}, Q = {best_q:.4}");
    println!("  membership = {:?}", wt.membership);
    println!(
        "louvain final partition: k = {lv_k}, Q = {:.4}",
        lv.modularity
    );
    println!("  membership = {:?}", lv.membership);

    println!("\nPair-wise partition distances (walktrap vs louvain):");
    for (label, method) in [
        (
            "VariationOfInformation",
            CommunityComparison::VariationOfInformation,
        ),
        (
            "NormalizedMutualInformation",
            CommunityComparison::NormalizedMutualInformation,
        ),
        ("SplitJoin", CommunityComparison::SplitJoin),
        ("Rand", CommunityComparison::Rand),
        ("AdjustedRand", CommunityComparison::AdjustedRand),
    ] {
        let d = compare_communities(&wt.membership, &lv.membership, method)?;
        println!("  {label:30} = {d:.6}");
    }

    // Also compare a deeper walktrap cut (steps = 8) against the default
    // — answers "how stable is walktrap's partition under its own
    // hyper-parameter?".
    let wt_deep = walktrap_with_options(&g, None, WalktrapOptions { steps: 8 })?;
    println!("\nWalktrap stability (default steps=4 vs steps=8):");
    for (label, method) in [
        (
            "VariationOfInformation",
            CommunityComparison::VariationOfInformation,
        ),
        (
            "NormalizedMutualInformation",
            CommunityComparison::NormalizedMutualInformation,
        ),
        ("AdjustedRand", CommunityComparison::AdjustedRand),
    ] {
        let d = compare_communities(&wt.membership, &wt_deep.membership, method)?;
        println!("  {label:30} = {d:.6}");
    }

    Ok(())
}
