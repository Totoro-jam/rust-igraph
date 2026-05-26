//! ALGO-GN-030 example: bipartite Erdős-Rényi diagnostics.
//!
//! Builds two bipartite random graphs that highlight the difference
//! between the G(n1, n2, p) Bernoulli sampler and the G(n1, n2, m)
//! fixed-edge sampler, then prints the degree distributions on each
//! side of the bipartition.
//!
//! What we look at:
//!   * vcount, ecount and bipartite type vector for each graph
//!   * mean bottom- and top-side degrees (for G(n,p) they should hover
//!     near n2·p and n1·p respectively)
//!   * the top-5 bottom-side and top-5 top-side hubs (degree-sorted)
//!
//! Run: `cargo run --example bipartite_game_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use rust_igraph::{BipartiteGraph, BipartiteMode, Graph, bipartite_game_gnm, bipartite_game_gnp};

fn degrees(g: &Graph) -> Vec<u32> {
    let mut deg = vec![0u32; g.vcount() as usize];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (u, v) = g.edge(eid).expect("edge id in bounds for example");
        deg[u as usize] += 1;
        deg[v as usize] += 1;
    }
    deg
}

fn mean(xs: &[u32]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let sum: u64 = xs.iter().map(|&x| u64::from(x)).sum();
    sum as f64 / xs.len() as f64
}

fn report(label: &str, bp: &BipartiteGraph, n1: u32, n2: u32) {
    let g = &bp.graph;
    let deg = degrees(g);

    let bottom: Vec<u32> = deg[..n1 as usize].to_vec();
    let top: Vec<u32> = deg[n1 as usize..(n1 + n2) as usize].to_vec();

    println!(
        "{label}: n1 = {n1}, n2 = {n2}, directed = {}",
        g.is_directed()
    );
    println!("  vcount       = {}", g.vcount());
    println!("  ecount       = {}", g.ecount());
    println!("  mean bot-deg = {:.2}", mean(&bottom));
    println!("  mean top-deg = {:.2}", mean(&top));

    let mut bot_idx: Vec<u32> = (0..n1).collect();
    bot_idx.sort_by_key(|&i| std::cmp::Reverse(bottom[i as usize]));
    println!("  top-5 bottom-side hubs:");
    for &i in bot_idx.iter().take(5) {
        println!("    vertex {i:>4} (bot)   degree = {}", bottom[i as usize]);
    }

    let mut top_idx: Vec<u32> = (0..n2).collect();
    top_idx.sort_by_key(|&i| std::cmp::Reverse(top[i as usize]));
    println!("  top-5 top-side hubs:");
    for &i in top_idx.iter().take(5) {
        let global = n1 + i;
        println!(
            "    vertex {global:>4} (top)   degree = {}",
            top[i as usize]
        );
    }

    // Quick sanity printout of the types vector head.
    let head: Vec<String> = bp
        .types
        .iter()
        .take(10)
        .map(|&t| if t { "T" } else { "B" }.to_string())
        .collect();
    println!("  types head   = [{}]", head.join(", "));
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n1: u32 = 200;
    let n2: u32 = 150;

    // G(n1, n2, p): sparse Bernoulli sampler. mode=All on an undirected
    // graph just means "every cross-pair sampled once".
    let p = 0.05;
    let gnp = bipartite_game_gnp(n1, n2, p, false, BipartiteMode::All, 0xB1_AE_DE_01)?;
    report(&format!("G(n1, n2, p={p}) undirected/All"), &gnp, n1, n2);

    // G(n1, n2, m): fixed-edge sampler with a directed Out arc set.
    let m: u64 = 600;
    let gnm = bipartite_game_gnm(n1, n2, m, true, BipartiteMode::Out, 0xB1_AE_DE_02)?;
    report(&format!("G(n1, n2, m={m}) directed/Out"), &gnm, n1, n2);

    Ok(())
}
