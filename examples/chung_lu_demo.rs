//! ALGO-GN-012 example: Chung–Lu expected-degree random graph.
//!
//! Three side-by-side runs over the same power-law-flavoured weight
//! vector compare the three connection-probability variants:
//!
//! ```text
//!   w_i = 1 + 0.4 · i,   n = 200,   so weights span [1, 80.6]
//!   variants:
//!     Original :  p_ij = min(q, 1)             (Chung–Lu 2002)
//!     Maxent   :  p_ij = q / (1 + q)           (Park & Newman 2004)
//!     Nr       :  p_ij = 1 − exp(−q)           (Norros–Reittu 2006)
//!   where q = w_i · w_j / Σ w_k.
//! ```
//!
//! For each realisation we print:
//!
//!   * vcount / ecount / mean degree;
//!   * Pearson correlation between the requested weight `w_i` and the
//!     observed degree `d_i` — the Chung–Lu guarantee is that `E[d_i] ≈
//!     w_i` in the sparse-graph limit, so this correlation should be
//!     close to 1 for every variant;
//!   * the top-5 hub vertices ranked by degree.
//!
//! We then run the same weight vector through the directed code path
//! (`in_weights = out_weights`) and print the analogous
//! out-/in-degree correlations, demonstrating that the same
//! expected-degree contract holds per-direction.
//!
//! Run: `cargo run --example chung_lu_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::unusual_byte_groupings
)]

use rust_igraph::{ChungLuVariant, Graph, chung_lu_game};

const N: usize = 200;
const BASE: f64 = 1.0;
const STEP: f64 = 0.4;

fn weights() -> Vec<f64> {
    (0..N).map(|i| BASE + STEP * (i as f64)).collect()
}

fn undirected_degrees(g: &Graph) -> Vec<u32> {
    let n = g.vcount();
    let mut d = vec![0u32; n as usize];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (u, v) = g.edge(eid).expect("edge id in bounds for example");
        d[u as usize] += 1;
        if u != v {
            d[v as usize] += 1;
        }
    }
    d
}

fn directed_degrees(g: &Graph) -> (Vec<u32>, Vec<u32>) {
    let n = g.vcount();
    let mut out = vec![0u32; n as usize];
    let mut din = vec![0u32; n as usize];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (u, v) = g.edge(eid).expect("edge id in bounds for example");
        out[u as usize] += 1;
        din[v as usize] += 1;
    }
    (out, din)
}

/// Pearson correlation between two equal-length vectors.
fn pearson(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len());
    let n = x.len() as f64;
    let mx: f64 = x.iter().sum::<f64>() / n;
    let my: f64 = y.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut sxx = 0.0;
    let mut syy = 0.0;
    for (&xi, &yi) in x.iter().zip(y.iter()) {
        let dx = xi - mx;
        let dy = yi - my;
        num += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    let denom = (sxx * syy).sqrt();
    if denom == 0.0 { 0.0 } else { num / denom }
}

fn summarise_undirected(label: &str, g: &Graph, w: &[f64]) {
    let deg = undirected_degrees(g);
    let deg_f: Vec<f64> = deg.iter().map(|&d| f64::from(d)).collect();
    let mean: f64 = deg_f.iter().sum::<f64>() / (deg.len() as f64);
    let corr = pearson(w, &deg_f);

    println!("--- {label} ---");
    println!(
        "  vcount = {}, ecount = {}, mean degree = {:.3}",
        g.vcount(),
        g.ecount(),
        mean
    );
    println!("  Pearson(w_i, deg_i) = {corr:.4}   (closer to 1.0 → tighter match)");

    let mut ranked: Vec<(u32, u32, f64)> = deg
        .iter()
        .enumerate()
        .map(|(i, &d)| (i as u32, d, w[i]))
        .collect();
    ranked.sort_by_key(|tup| std::cmp::Reverse(tup.1));
    println!("  top-5 hubs (vertex, observed_deg, requested_w):");
    for &(v, d, wv) in ranked.iter().take(5) {
        println!("     v={v:>3}  deg={d:>4}  w={wv:>6.2}");
    }
}

fn summarise_directed(label: &str, g: &Graph, w: &[f64]) {
    let (out, din) = directed_degrees(g);
    let out_f: Vec<f64> = out.iter().map(|&d| f64::from(d)).collect();
    let in_f: Vec<f64> = din.iter().map(|&d| f64::from(d)).collect();
    let mean_out: f64 = out_f.iter().sum::<f64>() / (out.len() as f64);
    let corr_out = pearson(w, &out_f);
    let corr_in = pearson(w, &in_f);

    println!("--- {label} ---");
    println!(
        "  vcount = {}, ecount = {}, mean out-degree = {:.3} (= mean in-degree)",
        g.vcount(),
        g.ecount(),
        mean_out,
    );
    println!("  Pearson(w_i, out_deg_i) = {corr_out:.4}");
    println!("  Pearson(w_i, in_deg_i ) = {corr_in:.4}");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let w = weights();
    let total: f64 = w.iter().sum();
    println!(
        "chung_lu_demo: n = {N}, weights w_i = {BASE} + {STEP}·i (range [{:.2}, {:.2}], Σ w = {:.1})",
        w[0],
        w[N - 1],
        total,
    );
    println!(
        "Expected mean degree in the sparse limit ≈ Σ w / n = {:.2}",
        total / (N as f64)
    );
    println!();

    for (label, variant, seed) in [
        (
            "Original (min(q,1))",
            ChungLuVariant::Original,
            0x5C_1200_01,
        ),
        ("Maxent   (q/(1+q))", ChungLuVariant::Maxent, 0x5C_1200_02),
        ("Nr       (1-exp(-q))", ChungLuVariant::Nr, 0x5C_1200_03),
    ] {
        let g = chung_lu_game(&w, None, false, variant, seed)?;
        summarise_undirected(label, &g, &w);
        println!();
    }

    let g_dir = chung_lu_game(&w, Some(&w), false, ChungLuVariant::Maxent, 0x5C_1200_04)?;
    summarise_directed("Directed Maxent (in_weights = out_weights)", &g_dir, &w);
    println!();

    println!("Every variant places the high-w vertices at the top of the hub");
    println!("ranking and yields a strong (typically ≥ 0.95) Pearson");
    println!("correlation between the requested weight and the observed");
    println!("degree — the defining guarantee of the Chung–Lu model.");
    Ok(())
}
