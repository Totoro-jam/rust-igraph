//! `pagerank_linsys` (ALGO-PR-011c) — GMRES backend for `PageRank`,
//! head-to-head with the PR-011 power-iteration backend on Zachary's
//! karate club.
//!
//! Run with `cargo run --release --example pagerank_linsys_demo`.
//!
//! Both backends solve the same fixed point
//! `(I - α · Mᵀ) · pr = (1 - α)/N · 1` at `α = 0.85`. Power iteration
//! drives the residual by a factor of `α` per step (so it needs roughly
//! `log(ε) / log(α)` iterations to reach a target `ε`); restarted GMRES
//! minimises the residual over a Krylov subspace and typically lands
//! within ≤30 Arnoldi steps. The two backends agree to better than
//! `1e-9` elementwise (PR-011 stops at `eps = 1e-10` so its result is
//! the looser of the two); this demo verifies that bound and prints
//! the per-vertex ranking.

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{Graph, pagerank, pagerank_linsys, read_edgelist};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    let g: Graph = read_edgelist(File::open(path)?)?;
    let n = g.vcount() as usize;
    println!("Karate club: {} vertices, {} edges", n, g.ecount());

    let pr_power = pagerank(&g)?;
    let pr_gmres = pagerank_linsys(&g)?;
    assert_eq!(pr_power.len(), n);
    assert_eq!(pr_gmres.len(), n);

    // Parity sanity check.
    let mut max_diff = 0.0_f64;
    for i in 0..n {
        max_diff = max_diff.max((pr_power[i] - pr_gmres[i]).abs());
    }
    println!("\nMax |power − GMRES| over all vertices: {max_diff:.3e}");
    assert!(max_diff < 1e-9, "backends disagree by {max_diff:e}");

    // Top-10 ranking (1-indexed for readability vs. the classic karate
    // figures where vertex 1 is the instructor and vertex 34 is the
    // president).
    let mut ranked: Vec<(usize, f64)> = pr_gmres.iter().copied().enumerate().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("finite ranks"));

    println!("\nTop-10 vertices by GMRES PageRank:");
    println!(
        "    {:>4}  {:>4}  {:>14}  {:>14}",
        "rank", "v", "pr_power", "pr_gmres"
    );
    for (rank, &(v, pr)) in ranked.iter().take(10).enumerate() {
        let pp = pr_power[v];
        println!(
            "    {:>4}  {:>4}  {:>14.10}  {:>14.10}",
            rank + 1,
            v + 1,
            pp,
            pr
        );
    }

    // Probability-distribution sanity: both must sum to 1.
    let s_power: f64 = pr_power.iter().sum();
    let s_gmres: f64 = pr_gmres.iter().sum();
    println!("\n‖pr_power‖₁ = {s_power:.12}, ‖pr_gmres‖₁ = {s_gmres:.12}");

    Ok(())
}
