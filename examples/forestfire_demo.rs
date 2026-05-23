//! ALGO-GN-006 example: forest-fire model diagnostics.
//!
//! Grows a directed forest-fire graph on 2 000 vertices and prints
//! summary statistics that highlight the model's heavy-tailed in-degree
//! distribution — the property that motivated Leskovec, Kleinberg and
//! Faloutsos (KDD'05) to introduce the model in the first place.
//!
//! What we look at:
//!   * mean in- / out-degree (out is always close to `ambs` plus a
//!     geometric burn tail; in is heavy-tailed),
//!   * the top-10 in-degree hubs,
//!   * a small in-degree histogram on a log binning showing the slow
//!     tail decay.
//!
//! Run: `cargo run --example forestfire_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use rust_igraph::{Graph, forest_fire_game};

fn in_degrees(g: &Graph) -> Vec<u32> {
    let n = g.vcount();
    let mut ind = vec![0u32; n as usize];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (_src, dst) = g.edge(eid).expect("edge id in bounds for example");
        ind[dst as usize] += 1;
    }
    ind
}

fn out_degrees(g: &Graph) -> Vec<u32> {
    let n = g.vcount();
    let mut outd = vec![0u32; n as usize];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (src, _dst) = g.edge(eid).expect("edge id in bounds for example");
        outd[src as usize] += 1;
    }
    outd
}

fn mean(xs: &[u32]) -> f64 {
    let sum: u64 = xs.iter().map(|&x| u64::from(x)).sum();
    sum as f64 / xs.len() as f64
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n: u32 = 2_000;
    let fw_prob = 0.37;
    let bw_factor = 0.32;
    let ambs: u32 = 2;

    let g = forest_fire_game(n, fw_prob, bw_factor, ambs, true, 0xF0_1E57)?;

    let ind = in_degrees(&g);
    let outd = out_degrees(&g);

    println!("forest fire: n = {n}, fw_prob = {fw_prob}, bw_factor = {bw_factor}, ambs = {ambs}",);
    println!("  edges        = {}", g.ecount());
    println!("  mean in-deg  = {:.2}", mean(&ind));
    println!("  mean out-deg = {:.2}", mean(&outd));

    // Top-10 in-degree hubs — the model's signature output.
    let mut idx: Vec<u32> = (0..n).collect();
    idx.sort_by_key(|&i| std::cmp::Reverse(ind[i as usize]));
    println!("  top-10 in-degree hubs:");
    for &i in idx.iter().take(10) {
        println!("    vertex {i:>5}  in-degree = {}", ind[i as usize]);
    }

    // Log-binned in-degree histogram (powers of 2): a power-law tail
    // shows a roughly straight downward slope in counts-per-bin.
    let max_in = *ind.iter().max().unwrap_or(&0);
    let n_bins = if max_in == 0 {
        1
    } else {
        f64::from(max_in).log2().ceil() as usize + 1
    };
    let mut bins = vec![0u32; n_bins.max(1)];
    for &d in &ind {
        let b = if d == 0 {
            0
        } else {
            f64::from(d).log2() as usize
        };
        bins[b] += 1;
    }
    println!("  in-degree histogram (powers of 2):");
    for (i, &count) in bins.iter().enumerate() {
        let lo = if i == 0 { 0 } else { 1 << i };
        let hi = (1u32 << (i + 1)) - 1;
        println!("    [{lo:>4}, {hi:>4}]  count = {count}");
    }

    Ok(())
}
