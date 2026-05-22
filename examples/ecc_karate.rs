//! `ecc` (ALGO-PR-031) — edge clustering coefficient on Zachary's
//! karate club.
//!
//! Run with `cargo run --release --example ecc_karate`.
//!
//! Computes the Radicchi 2004 edge clustering coefficient for every
//! edge of the karate club, under both the k=3 (triangle-based) and
//! k=4 (square-based) variants in their default (`offset=false,
//! normalize=true`) form. For each variant we print:
//!
//!   * the minimum, maximum, and mean coefficient (skipping NaN
//!     entries — those mark edges with a degree-1 endpoint, where the
//!     denominator vanishes),
//!   * a small leaderboard of the five edges with the highest
//!     coefficient (the "deeply embedded" edges Radicchi's algorithm
//!     keeps), and the five with the lowest (the candidate
//!     inter-community bridges).
//!
//! The first four edges (in `karate.edges` id order) should reproduce
//! `0.875, 0.555556, 1.0, 1.0` — the leading entries of
//! `references/igraph/tests/unit/igraph_ecc.out` for `IGRAPH_ECC_3`.

#![allow(clippy::cast_precision_loss)]

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{Graph, ecc, read_edgelist};

fn summarise(label: &str, g: &Graph, values: &[f64]) {
    let mut finite: Vec<(u32, f64)> = values
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| {
            if v.is_finite() {
                Some((u32::try_from(i).expect("edge id fits in u32"), v))
            } else {
                None
            }
        })
        .collect();
    let nan_count = values.len() - finite.len();
    let n = finite.len();
    let (mut lo, mut hi, mut sum) = (f64::INFINITY, f64::NEG_INFINITY, 0.0);
    for &(_, v) in &finite {
        lo = lo.min(v);
        hi = hi.max(v);
        sum += v;
    }
    let mean = if n == 0 { f64::NAN } else { sum / n as f64 };

    println!("\n{label}:");
    println!(
        "  edges: {} (finite) / {} (NaN: {})",
        n,
        values.len(),
        nan_count
    );
    println!("  min  = {lo:.6}   max  = {hi:.6}   mean = {mean:.6}");

    // Top-5 most embedded (highest coefficient): community-internal.
    finite.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("no NaN here"));
    println!("  top-5 (most embedded):");
    for &(eid, v) in finite.iter().take(5) {
        let (u, w) = g.edge(eid).expect("edge id in range");
        println!("    eid {eid:>3}: ({u:>2},{w:>2})  C = {v:.6}");
    }

    // Bottom-5 least embedded: candidate inter-community bridges.
    finite.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("no NaN here"));
    println!("  bottom-5 (likely bridges):");
    for &(eid, v) in finite.iter().take(5) {
        let (u, w) = g.edge(eid).expect("edge id in range");
        println!("    eid {eid:>3}: ({u:>2},{w:>2})  C = {v:.6}");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    let g = read_edgelist(File::open(path)?)?;
    println!("Karate club: {} vertices, {} edges", g.vcount(), g.ecount());

    let c3 = ecc(&g, None, 3, false, true)?;
    let c4 = ecc(&g, None, 4, false, true)?;

    // First four entries mirror references/igraph/tests/unit/igraph_ecc.out
    // line 73 (IGRAPH_ECC_3 on karate): 0.875 0.555556 1 1.
    println!(
        "\nFirst four k=3 values (matches igraph C .out): {:.6} {:.6} {:.6} {:.6}",
        c3[0], c3[1], c3[2], c3[3]
    );

    summarise("k=3 (triangle-based)", &g, &c3);
    summarise("k=4 (square-based)", &g, &c4);

    Ok(())
}
