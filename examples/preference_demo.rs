//! ALGO-GN-014 example: preference and asymmetric-preference samplers.
//!
//! Grows a 4-type symmetric preference graph and a 2x3 asymmetric
//! preference graph, then prints per-type degree summaries that show
//! how the block structure manifests in the resulting adjacency.
//!
//! What we look at:
//!   * symmetric model — assortative diagonal pref, equal-size blocks
//!     via `fixed_sizes=true`. Per-type mean degree should track
//!     `(size_t - 1) * p_in` plus the (small) off-diagonal contribution.
//!   * asymmetric model — directed 2x3 with one hot-spot cell. Per
//!     `(out_type, in_type)` we report the count of edges, illustrating
//!     how the joint type distribution shapes the directed adjacency.
//!
//! Run: `cargo run --example preference_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]

use rust_igraph::{Graph, asymmetric_preference_game, preference_game};

fn count_edges_by_type(
    g: &Graph,
    types_a: &[u32],
    types_b: &[u32],
    n_a: usize,
    n_b: usize,
) -> Vec<Vec<u32>> {
    let mut counts = vec![vec![0u32; n_b]; n_a];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (u, v) = g.edge(eid).expect("edge id in bounds for example");
        let a = types_a[u as usize] as usize;
        let b = types_b[v as usize] as usize;
        counts[a][b] += 1;
    }
    counts
}

fn type_sizes(types: &[u32], k: usize) -> Vec<u32> {
    let mut sizes = vec![0u32; k];
    for &t in types {
        sizes[t as usize] += 1;
    }
    sizes
}

fn run_symmetric() -> Result<(), Box<dyn std::error::Error>> {
    let n: u32 = 200;
    let k: u32 = 4;
    let p_in = 0.3;
    let p_off = 0.01;
    let pref: Vec<Vec<f64>> = (0..k as usize)
        .map(|i| {
            (0..k as usize)
                .map(|j| if i == j { p_in } else { p_off })
                .collect()
        })
        .collect();

    let (g, types) = preference_game(n, k, None, true, &pref, false, false, 0xFE14_DE40_u64)?;

    println!("symmetric preference: n = {n}, k = {k}, p_in = {p_in}, p_off = {p_off}, fixed_sizes");
    println!("  edges = {}", g.ecount());

    let sizes = type_sizes(&types, k as usize);
    let counts = count_edges_by_type(&g, &types, &types, k as usize, k as usize);
    println!("  per-type block edge counts (rows = u-type, cols = v-type):");
    for (i, row) in counts.iter().enumerate() {
        let cells: Vec<String> = row.iter().map(|c| format!("{c:>4}")).collect();
        println!("    type {i} (size {:>3}):  {}", sizes[i], cells.join(" "));
    }
    Ok(())
}

fn run_asymmetric() -> Result<(), Box<dyn std::error::Error>> {
    let n: u32 = 300;
    let no_out: u32 = 2;
    let no_in: u32 = 3;
    // Hotspot at (out=0, in=1); softer mass elsewhere.
    let pref: Vec<Vec<f64>> = vec![vec![0.05, 0.30, 0.02], vec![0.02, 0.05, 0.10]];
    let (g, out_types, in_types) =
        asymmetric_preference_game(n, no_out, no_in, None, &pref, false, 0xFE14_A55E_u64)?;

    println!("\nasymmetric preference: n = {n}, no_out = {no_out}, no_in = {no_in}, no loops");
    println!("  edges (directed) = {}", g.ecount());
    let out_sizes = type_sizes(&out_types, no_out as usize);
    let in_sizes = type_sizes(&in_types, no_in as usize);
    println!("  out-type sizes = {out_sizes:?}");
    println!("  in-type sizes  = {in_sizes:?}");

    let counts = count_edges_by_type(&g, &out_types, &in_types, no_out as usize, no_in as usize);
    println!("  joint (out_type → in_type) edge counts vs pref:");
    for o in 0..no_out as usize {
        for i in 0..no_in as usize {
            println!(
                "    out={o} in={i}: {:>5} edges  (pref = {:.3})",
                counts[o][i], pref[o][i]
            );
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_symmetric()?;
    run_asymmetric()?;
    Ok(())
}
