//! ALGO-GN-008 example: k-regular sampler diagnostics.
//!
//! Generates three graphs from the same `k_regular_game` entry point —
//! an undirected simple 4-regular graph on 12 vertices, a directed
//! simple 3-regular graph on 8 vertices, and an undirected multigraph
//! 6-regular graph on 5 vertices — then prints the degree histogram of
//! each so the regularity invariant is visible in plain text.
//!
//! Run: `cargo run --example k_regular_demo`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::similar_names
)]

use rust_igraph::{Graph, k_regular_game};

fn undirected_degree_histogram(g: &Graph) -> Vec<u64> {
    let mut deg = vec![0u64; g.vcount() as usize];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (u, v) = g.edge(eid).expect("edge id in bounds for example");
        deg[u as usize] += 1;
        // Self-loop contributes 2 to undirected degree; the two
        // increments happen to coincide when u == v.
        deg[v as usize] += 1;
    }
    deg
}

fn directed_degree_histograms(g: &Graph) -> (Vec<u64>, Vec<u64>) {
    let n = g.vcount() as usize;
    let mut out_deg = vec![0u64; n];
    let mut in_deg = vec![0u64; n];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (u, v) = g.edge(eid).expect("edge id in bounds for example");
        out_deg[u as usize] += 1;
        in_deg[v as usize] += 1;
    }
    (out_deg, in_deg)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // (1) Undirected, simple, 4-regular on 12 vertices.
    let g_us = k_regular_game(12, 4, false, false, 0x15_1B4D_5EED)?;
    println!(
        "undirected simple 4-regular on 12 vertices: edges = {}, directed = {}",
        g_us.ecount(),
        g_us.is_directed()
    );
    let deg = undirected_degree_histogram(&g_us);
    println!("  per-vertex degrees (every entry should be 4):");
    for (v, d) in deg.iter().enumerate() {
        let marker = if *d == 4 { " ok " } else { "MISS" };
        println!("    v{v:>2}  deg = {d}  {marker}");
    }

    // (2) Directed, simple, 3-regular on 8 vertices.
    println!();
    let g_ds = k_regular_game(8, 3, true, false, 0x15_1B4D_5EEE)?;
    println!(
        "directed simple 3-regular on 8 vertices: edges = {}, directed = {}",
        g_ds.ecount(),
        g_ds.is_directed()
    );
    let (out_deg, in_deg) = directed_degree_histograms(&g_ds);
    println!("  per-vertex (out, in) degrees (every entry should be (3, 3)):");
    for v in 0..g_ds.vcount() as usize {
        let o = out_deg[v];
        let i = in_deg[v];
        let marker = if o == 3 && i == 3 { " ok " } else { "MISS" };
        println!("    v{v}  (out, in) = ({o}, {i})  {marker}");
    }

    // (3) Undirected multigraph 6-regular on 5 vertices (n*k = 30 is
    // even but k > n-1 = 4, so the simple sampler would reject — the
    // multigraph path uses the configuration model).
    println!();
    let g_um = k_regular_game(5, 6, false, true, 0x15_1B4D_5EEF)?;
    println!(
        "undirected multi 6-regular on 5 vertices: edges = {}, directed = {}",
        g_um.ecount(),
        g_um.is_directed()
    );
    let deg = undirected_degree_histogram(&g_um);
    println!("  per-vertex degrees (every entry should be 6):");
    for (v, d) in deg.iter().enumerate() {
        let marker = if *d == 6 { " ok " } else { "MISS" };
        println!("    v{v}  deg = {d}  {marker}");
    }

    Ok(())
}
