//! ALGO-GN-010 example: stochastic block model community structure.
//!
//! Generates an assortative SBM on `n = 100` vertices split into 4
//! equal blocks of 25, with within-block connection probability `0.30`
//! and between-block probability `0.02`. Then reports per-block edge
//! counts and degree distributions to make the planted communities
//! visible at the command line.
//!
//! What we look at:
//!   * total edges and the within-/between-block split,
//!   * per-block density (edges / max-possible),
//!   * per-block mean degree (which should run far above the between-
//!     block contribution),
//!   * top-3 highest-degree vertices in each block.
//!
//! Run: `cargo run --example sbm_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]

use rust_igraph::{Graph, sbm_game};

/// Map a vertex id to its block, given the prefix-sum `offsets`. The
/// `offsets` array has length `k + 1` with `offsets[k] = n`.
fn block_of(vertex: u32, offsets: &[u32]) -> usize {
    // Linear scan — fine for the tiny k = 4 in this demo.
    for (b, w) in offsets.windows(2).enumerate() {
        if vertex >= w[0] && vertex < w[1] {
            return b;
        }
    }
    offsets.len().saturating_sub(2)
}

fn prefix_offsets(sizes: &[u32]) -> Vec<u32> {
    let mut acc = 0u32;
    let mut out = Vec::with_capacity(sizes.len() + 1);
    out.push(0);
    for &s in sizes {
        acc += s;
        out.push(acc);
    }
    out
}

fn degrees(g: &Graph) -> Vec<u32> {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 4 communities of 25, assortative.
    let block_sizes: Vec<u32> = vec![25, 25, 25, 25];
    let k = block_sizes.len();
    let p_in = 0.30_f64;
    let p_off = 0.02_f64;
    let pref: Vec<Vec<f64>> = (0..k)
        .map(|i| (0..k).map(|j| if i == j { p_in } else { p_off }).collect())
        .collect();

    let g = sbm_game(&pref, &block_sizes, false, false, false, 0x5B10_10DE)?;
    let offsets = prefix_offsets(&block_sizes);
    let n = g.vcount();

    println!("sbm_game: n = {n}, k = {k}, p_in = {p_in}, p_off = {p_off}");
    println!("  total edges = {}", g.ecount());

    // Tally within- vs between-block edges and a k×k block-edge matrix.
    let mut within = vec![0u64; k];
    let mut between = 0u64;
    let mut block_edges = vec![vec![0u64; k]; k];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (u, v) = g.edge(eid).expect("edge id in bounds for example");
        let bu = block_of(u, &offsets);
        let bv = block_of(v, &offsets);
        let (lo, hi) = if bu <= bv { (bu, bv) } else { (bv, bu) };
        block_edges[lo][hi] += 1;
        if bu == bv {
            within[bu] += 1;
        } else {
            between += 1;
        }
    }
    let total_within: u64 = within.iter().sum();
    println!("  within-block edges  = {total_within}");
    println!("  between-block edges = {between}");

    println!("  per-block diagnostics:");
    println!("    block | size | edges | density  | mean deg");
    println!("    ------+------+-------+----------+---------");
    let deg = degrees(&g);
    for (b, (&size, &edges)) in block_sizes.iter().zip(within.iter()).enumerate() {
        let maxpairs = u64::from(size) * u64::from(size - 1) / 2;
        let density = if maxpairs == 0 {
            0.0
        } else {
            edges as f64 / maxpairs as f64
        };
        let lo = offsets[b] as usize;
        let hi = offsets[b + 1] as usize;
        let block_deg: u32 = deg[lo..hi].iter().sum();
        let mean_deg = f64::from(block_deg) / f64::from(size);
        println!("       {b:>2} | {size:>4} | {edges:>5} | {density:>8.4} | {mean_deg:>7.2}");
    }

    println!("  block × block edge matrix (symmetric, diagonal = within):");
    print!("        ");
    for j in 0..k {
        print!("  b{j:<2}");
    }
    println!();
    for (i, row) in block_edges.iter().enumerate() {
        print!("    b{i:<2} ");
        for (j, &cell) in row.iter().enumerate() {
            let val = if i <= j { cell } else { block_edges[j][i] };
            print!(" {val:>4}");
        }
        println!();
    }

    // Top-3 highest-degree vertices per block.
    println!("  top-3 highest-degree vertices per block:");
    for (b, win) in offsets.windows(2).enumerate() {
        let lo = win[0] as usize;
        let hi = win[1] as usize;
        let mut idx: Vec<u32> = (lo as u32..hi as u32).collect();
        idx.sort_by_key(|&i| std::cmp::Reverse(deg[i as usize]));
        let top: Vec<String> = idx
            .iter()
            .take(3)
            .map(|&i| format!("v{i} (deg={})", deg[i as usize]))
            .collect();
        println!("    block {b}: {}", top.join(", "));
    }

    Ok(())
}
