//! ALGO-GN-031 example: independent-edge-allocation multigraph stats.
//!
//! Builds three IEA multigraphs that highlight the model's defining
//! properties — exact edge count, multi-edges from saturation, optional
//! self-loops — then prints distribution diagnostics.
//!
//! What we look at:
//!   * vcount, ecount, directed flag
//!   * self-loop count (0 when loops=false, ~m/n when loops=true)
//!   * max edge multiplicity (the "multi" part of multigraph)
//!   * mean and max per-vertex degree
//!   * top-5 endpoint hits (the model has a uniform marginal)
//!
//! Run: `cargo run --example iea_game_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::unusual_byte_groupings
)]

use rust_igraph::{Graph, iea_game};
use std::collections::HashMap;

fn endpoint_stats(g: &Graph) -> (Vec<u32>, u32, HashMap<(u32, u32), u32>) {
    let n = g.vcount() as usize;
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    let mut deg = vec![0u32; n];
    let mut self_loops = 0u32;
    let mut mult: HashMap<(u32, u32), u32> = HashMap::new();
    for eid in 0..m {
        let (u, v) = g.edge(eid).expect("edge id in bounds for example");
        deg[u as usize] += 1;
        deg[v as usize] += 1;
        if u == v {
            self_loops += 1;
        }
        // For undirected graphs, normalise to (min, max) before counting
        // multiplicity so (a, b) and (b, a) collapse into the same pair.
        let key = if g.is_directed() || u <= v {
            (u, v)
        } else {
            (v, u)
        };
        *mult.entry(key).or_default() += 1;
    }
    (deg, self_loops, mult)
}

fn mean(xs: &[u32]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let sum: u64 = xs.iter().map(|&x| u64::from(x)).sum();
    sum as f64 / xs.len() as f64
}

fn report(label: &str, n: u32, m: u64, directed: bool, loops: bool, seed: u64) {
    let g = iea_game(n, m, directed, loops, seed).expect("iea_game succeeds for example");
    let (deg, self_loops, mult) = endpoint_stats(&g);
    let max_mult = mult.values().copied().max().unwrap_or(0);
    let max_deg = deg.iter().copied().max().unwrap_or(0);

    println!("{label}");
    println!("  vcount       = {}", g.vcount());
    println!("  ecount       = {}", g.ecount());
    println!("  directed     = {directed}");
    println!("  loops allowed= {loops}");
    println!("  self-loops   = {self_loops}");
    println!("  max edge mult= {max_mult}");
    println!("  mean degree  = {:.2}", mean(&deg));
    println!("  max  degree  = {max_deg}");

    let mut idx: Vec<u32> = (0..n).collect();
    idx.sort_by_key(|&i| std::cmp::Reverse(deg[i as usize]));
    println!("  top-5 endpoint hubs (by total degree):");
    for &i in idx.iter().take(5) {
        println!("    vertex {i:>4}   degree = {}", deg[i as usize]);
    }
    println!();
}

fn main() {
    // 1) Directed with self-loops. Expected self-loops ~ m / n = 100.
    report(
        "G_1 = IEA(n=50, m=5000, directed=true, loops=true)",
        50,
        5_000,
        true,
        true,
        0x1EA_DE_01,
    );

    // 2) Directed no self-loops — saturation regime; multi-edges abound.
    report(
        "G_2 = IEA(n=10, m=2000, directed=true, loops=false)",
        10,
        2_000,
        true,
        false,
        0x1EA_DE_02,
    );

    // 3) Undirected no self-loops — small graph with exact m.
    report(
        "G_3 = IEA(n=20, m=300, directed=false, loops=false)",
        20,
        300,
        false,
        false,
        0x1EA_DE_03,
    );
}
