//! `split_join_distance` (ALGO-CM-016) — detect sub-partition / refinement
//! relationships by inspecting the asymmetric `(d12, d21)` pair.
//!
//! Run with `cargo run --release --example
//! split_join_distance_louvain_vs_leiden_karate`. Runs louvain and leiden
//! on Zachary's karate club, then prints:
//!
//! * the asymmetric pair `(d12, d21)`,
//! * the total (symmetric) split-join distance `d12 + d21`,
//! * a one-line interpretation: which partition (if either) is a
//!   sub-partition of the other, or whether they are identical.
//!
//! Also re-runs leiden with `iterations = 6` against the default
//! (`iterations = 2`) to demonstrate the asymmetry pair for hyperparameter
//! stability checks.

#![allow(clippy::similar_names)]

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{
    LeidenOptions, leiden, leiden_with_options, louvain, read_edgelist, split_join_distance,
};

fn label_partition(d12: u64, d21: u64) -> &'static str {
    match (d12, d21) {
        (0, 0) => "identical partitions",
        (0, _) => "comm1 is a sub-partition of comm2 (comm2 is coarser)",
        (_, 0) => "comm2 is a sub-partition of comm1 (comm1 is coarser)",
        _ => "neither is a refinement of the other",
    }
}

fn report(label1: &str, label2: &str, comm1: &[u32], comm2: &[u32]) {
    let r = split_join_distance(comm1, comm2).expect("split_join_distance");
    println!("\n{label1} vs {label2}:");
    println!(
        "  d12 (projection of {label1:>20} from {label2:>20}) = {}",
        r.d12
    );
    println!(
        "  d21 (projection of {label2:>20} from {label1:>20}) = {}",
        r.d21
    );
    println!(
        "  total (symmetric split-join)                          = {}",
        r.total()
    );
    println!("  interpretation: {}", label_partition(r.d12, r.d21));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    let g = read_edgelist(File::open(path)?)?;
    println!("Karate club: {} vertices, {} edges", g.vcount(), g.ecount());

    let lv = louvain(&g)?;
    let ld = leiden(&g)?;
    let ld_deep = leiden_with_options(
        &g,
        None,
        &LeidenOptions {
            n_iterations: 6,
            ..LeidenOptions::default()
        },
    )?;

    let lv_k = lv.membership.iter().copied().max().map_or(0, |m| m + 1);
    let ld_k = ld.membership.iter().copied().max().map_or(0, |m| m + 1);
    let ld_deep_k = ld_deep
        .membership
        .iter()
        .copied()
        .max()
        .map_or(0, |m| m + 1);

    println!(
        "\nlouvain         partition: k = {lv_k}, Q = {:.4}",
        lv.modularity
    );
    println!(
        "leiden default  partition: k = {ld_k}, Q = {:.4}",
        ld.quality
    );
    println!(
        "leiden iter=6   partition: k = {ld_deep_k}, Q = {:.4}",
        ld_deep.quality
    );

    report("louvain", "leiden(default)", &lv.membership, &ld.membership);
    report(
        "leiden(default)",
        "leiden(iter=6)",
        &ld.membership,
        &ld_deep.membership,
    );

    // Identical partition check (sanity): split_join(x, x) == (0, 0).
    report("louvain", "louvain", &lv.membership, &lv.membership);

    Ok(())
}
