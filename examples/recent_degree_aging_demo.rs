//! ALGO-GN-032 example: recent-degree aging PA graph stats.
//!
//! Builds three recent-degree-aging graphs that highlight how the
//! sliding-window + vertex-aging mechanism shapes topology, then prints
//! distribution diagnostics.
//!
//! What we look at:
//!   * vcount, ecount, directed flag
//!   * mean and max in-degree (targets of citations)
//!   * share of edges aimed at the youngest 25 % of vertices
//!
//! Run: `cargo run --example recent_degree_aging_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_arguments,
    clippy::similar_names,
    clippy::unusual_byte_groupings
)]

use rust_igraph::recent_degree_aging_game;

fn report(
    label: &str,
    nodes: u32,
    m: u32,
    outpref: bool,
    pa_exp: f64,
    aging_exp: f64,
    aging_bins: u32,
    time_window: u32,
    zero_appeal: f64,
    directed: bool,
    seed: u64,
) {
    let g = recent_degree_aging_game(
        nodes,
        m,
        None,
        outpref,
        pa_exp,
        aging_exp,
        aging_bins,
        time_window,
        zero_appeal,
        directed,
        seed,
    )
    .expect("recent_degree_aging_game succeeds for example");

    let n = g.vcount() as usize;
    let ecount = u32::try_from(g.ecount()).expect("ecount fits u32");
    let mut in_deg = vec![0u32; n];
    for eid in 0..ecount {
        let (_u, v) = g.edge(eid).expect("edge in bounds");
        in_deg[v as usize] += 1;
    }
    let mean_in: f64 = in_deg.iter().map(|&d| f64::from(d)).sum::<f64>() / n as f64;
    let max_in = in_deg.iter().copied().max().unwrap_or(0);

    let young_cutoff = nodes * 3 / 4;
    let young_edges: u32 = (0..ecount)
        .filter(|&eid| {
            let (_u, v) = g.edge(eid).expect("edge in bounds");
            v >= young_cutoff
        })
        .count() as u32;
    let young_share = f64::from(young_edges) / f64::from(ecount) * 100.0;

    println!("{label}");
    println!("  vcount          = {}", g.vcount());
    println!("  ecount          = {}", g.ecount());
    println!("  directed        = {directed}");
    println!("  mean in-degree  = {mean_in:.2}");
    println!("  max  in-degree  = {max_in}");
    println!("  young-25% share = {young_share:.1}%");
    println!();
}

fn main() {
    report(
        "G1: no aging (aging_exp=0), short window=5, directed",
        200,
        3,
        false,
        1.0,
        0.0,
        10,
        5,
        1.0,
        true,
        0xA6E_DE_01,
    );

    report(
        "G2: strong aging (aging_exp=-2), window=20, directed",
        200,
        3,
        false,
        1.0,
        -2.0,
        10,
        20,
        1.0,
        true,
        0xA6E_DE_02,
    );

    report(
        "G3: outpref + moderate aging, undirected",
        200,
        3,
        true,
        1.0,
        -0.5,
        8,
        15,
        0.5,
        false,
        0xA6E_DE_03,
    );
}
