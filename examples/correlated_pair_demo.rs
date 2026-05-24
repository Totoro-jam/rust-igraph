//! ALGO-GN-023 example: `correlated_pair_game` — sample a pair of
//! Erdős–Rényi graphs whose adjacency vectors have a tunable Pearson
//! correlation while keeping each graph's marginal edge density at `p`.
//!
//! At each `corr` in `{0.0, 0.25, 0.5, 0.75, 0.9, 1.0}` we draw a
//! `(g1, g2)` pair on `n = 200` vertices with `p = 0.1`, then report:
//!   * the two ecounts (should both sit near `C(200, 2) · p = 1990`);
//!   * the size of the edge intersection `|E(g1) ∩ E(g2)|`;
//!   * the Jaccard overlap `|E∩| / |E∪|`;
//!   * the empirical Pearson correlation of the two adjacency vectors,
//!     which should track the requested `corr` once we average across
//!     enough vertex pairs.
//!
//! The expected pattern: `corr = 0` ⇒ intersection ≈ `p · |E(g1)|`
//! (independent graphs), `corr = 1` ⇒ intersection = `|E(g1)|`
//! (exact copy). Intermediate values trace out the model's
//! contingency-table behavior.
//!
//! Run: `cargo run --example correlated_pair_demo --release`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::collections::HashSet;

use rust_igraph::{Graph, correlated_pair_game};

const N: u32 = 200;
const P: f64 = 0.1;

fn edge_set(g: &Graph) -> HashSet<(u32, u32)> {
    let m = u32::try_from(g.ecount()).expect("ecount fits u32 for demo");
    (0..m)
        .map(|eid| {
            let (u, v) = g.edge(eid).expect("edge id in bounds for demo");
            if u <= v { (u, v) } else { (v, u) }
        })
        .collect()
}

fn empirical_corr(g1: &Graph, g2: &Graph) -> f64 {
    let n = N as usize;
    let total = n * (n - 1) / 2; // unordered pairs
    let e1 = edge_set(g1);
    let e2 = edge_set(g2);
    let m1 = e1.len();
    let m2 = e2.len();
    // |E1 ∩ E2| in one pass — iterate the smaller set.
    let (small, big) = if m1 <= m2 { (&e1, &e2) } else { (&e2, &e1) };
    let inter = small.iter().filter(|p| big.contains(p)).count();

    // Pearson correlation of two {0,1}-valued vectors over `total`
    // positions can be written as
    //   r = (P11 − p1 · p2) / sqrt(p1 · (1 − p1) · p2 · (1 − p2))
    // where p1 = m1/total, p2 = m2/total, P11 = inter/total.
    let t = total as f64;
    let p1 = m1 as f64 / t;
    let p2 = m2 as f64 / t;
    let p11 = inter as f64 / t;
    let cov = p11 - p1 * p2;
    let denom = (p1 * (1.0 - p1) * p2 * (1.0 - p2)).sqrt();
    if denom == 0.0 { 0.0 } else { cov / denom }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "correlated_pair_game: n = {N}, p = {P}, undirected; sweeping corr ∈ {{0, .25, .5, .75, .9, 1}}"
    );
    println!(
        "expected per-graph ecount ≈ C(n,2)·p = {:.0}",
        f64::from(N) * f64::from(N - 1) / 2.0 * P
    );
    println!(
        "{:>5}  {:>8}  {:>8}  {:>8}  {:>8}  {:>8}",
        "corr", "|E1|", "|E2|", "|E∩|", "jaccard", "ρ̂"
    );
    println!(
        "{:->5}--{:->8}--{:->8}--{:->8}--{:->8}--{:->8}",
        "", "", "", "", "", ""
    );

    let seed: u64 = 0x10_23_45_67_u64.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for corr_milli in [0u32, 250, 500, 750, 900, 1000] {
        let corr = f64::from(corr_milli) / 1000.0;
        let (g1, g2) = correlated_pair_game(
            N,
            corr,
            P,
            false,
            None,
            seed.wrapping_add(corr_milli.into()),
        )?;
        let e1 = edge_set(&g1);
        let e2 = edge_set(&g2);
        let m1 = e1.len();
        let m2 = e2.len();
        let inter = e1.intersection(&e2).count();
        let union = e1.union(&e2).count();
        let jaccard = if union == 0 {
            0.0
        } else {
            inter as f64 / union as f64
        };
        let rho = empirical_corr(&g1, &g2);
        println!("{corr:>5.2}  {m1:>8}  {m2:>8}  {inter:>8}  {jaccard:>8.3}  {rho:>8.3}");
    }
    println!();
    println!("note: ρ̂ → corr as n grows; here n = {N} ⇒ noise ~±0.05 per row.");
    Ok(())
}
