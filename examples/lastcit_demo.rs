//! ALGO-GN-018 example: `lastcit_game` — recency-decay citation network.
//!
//! Grows a 2 000-vertex directed citation graph where each step emits
//! `edges_per_node = 3` outgoing citations. Cited vertices' weights
//! decay across `agebins = 4` age buckets:
//!   * bucket 0 (just cited): weight 8.0
//!   * bucket 1: 4.0
//!   * bucket 2: 2.0
//!   * bucket 3: 1.0
//!   * "never cited" (bucket agebins): 0.5
//!
//! The output illustrates three signatures of the lastcit model:
//!   * NEVER self-loops (the psumtree only ranges over previously-added
//!     vertices);
//!   * a large fraction of vertices is never cited (only their
//!     "never-cited" preference slot keeps them in play);
//!   * the in-degree distribution is heavy-tailed (max in-degree well
//!     above the mean). Citation events tend to refresh already-cited
//!     vertices back into the top bucket, so the highest-in-degree
//!     vertices skew toward the earliest cohorts that had the most
//!     opportunities to accumulate refresh events.
//!
//! Run: `cargo run --example lastcit_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::many_single_char_names
)]

use rust_igraph::{Graph, lastcit_game};

fn in_degree(g: &Graph) -> Vec<u32> {
    let mut deg = vec![0u32; g.vcount() as usize];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (_s, d) = g.edge(eid).expect("edge id in bounds for example");
        deg[d as usize] += 1;
    }
    deg
}

fn count_loops_and_multi(g: &Graph) -> (u64, u64) {
    use std::collections::HashMap;

    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    let directed = g.is_directed();
    let mut loops: u64 = 0;
    let mut counter: HashMap<(u32, u32), u32> = HashMap::with_capacity(m as usize);
    for eid in 0..m {
        let (s, d) = g.edge(eid).expect("edge id in bounds for example");
        if s == d {
            loops += 1;
        }
        let pair = if directed || s <= d { (s, d) } else { (d, s) };
        *counter.entry(pair).or_insert(0) += 1;
    }
    let multi: u64 = counter
        .values()
        .filter(|&&c| c >= 2)
        .map(|&c| u64::from(c))
        .sum();
    (loops, multi)
}

fn quartile_summary(in_deg: &[u32]) -> (u32, u32, u32, u32) {
    let mut s = in_deg.to_vec();
    s.sort_unstable();
    let n = s.len();
    let q1 = s[n / 4];
    let median = s[n / 2];
    let q3 = s[3 * n / 4];
    let max = *s.last().unwrap_or(&0);
    (q1, median, q3, max)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n: u32 = 2_000;
    let edges_per_node: u32 = 3;
    let agebins: u32 = 4;
    let preference = vec![8.0, 4.0, 2.0, 1.0, 0.5];

    let g = lastcit_game(
        n,
        edges_per_node,
        agebins,
        &preference,
        true,
        0x1A57_DEF0_u64,
    )?;

    let m = g.ecount();
    let (loops, multi_edges) = count_loops_and_multi(&g);
    let in_deg = in_degree(&g);
    let total: u64 = in_deg.iter().map(|&d| u64::from(d)).sum();
    let mean = if in_deg.is_empty() {
        0.0
    } else {
        total as f64 / in_deg.len() as f64
    };
    let (q1, median, q3, max_in) = quartile_summary(&in_deg);
    let uncited = in_deg.iter().filter(|&&d| d == 0).count();

    println!(
        "lastcit: n = {n}, edges_per_node = {edges_per_node}, agebins = {agebins}, preference = {preference:?}"
    );
    println!("  edges                    = {m}");
    println!("  expected edges (n-1)*eps = {}", (n - 1) * edges_per_node);
    println!("  self-loops               = {loops}  (must be 0 — lastcit never self-loops)");
    println!("  edges in multi-bundles   = {multi_edges}  (multi-edges allowed when eps ≥ 2)");
    println!();
    println!("  in-degree summary:");
    println!("    mean   = {mean:.2}");
    println!("    Q1     = {q1}");
    println!("    median = {median}");
    println!("    Q3     = {q3}");
    println!("    max    = {max_in}");
    println!("    uncited vertices = {uncited} / {n}");

    // Sample a few high-in-degree vertices and show their step indices —
    // recency-biased preference should concentrate them in the middle/late
    // part of the index range rather than at the very oldest vertices.
    let mut indexed: Vec<(u32, u32)> = in_deg
        .iter()
        .enumerate()
        .map(|(v, &d)| (u32::try_from(v).expect("vertex index fits u32"), d))
        .collect();
    indexed.sort_by_key(|entry| std::cmp::Reverse(entry.1));
    println!();
    println!("  top-10 vertices by in-degree (vertex_index, in_degree):");
    for (v, d) in indexed.iter().take(10) {
        let cohort = if *v < n / 4 {
            "old"
        } else if *v < n / 2 {
            "early-mid"
        } else if *v < 3 * n / 4 {
            "late-mid"
        } else {
            "recent"
        };
        println!("    v = {v:>5} ({cohort:<9}) in_deg = {d}");
    }

    Ok(())
}
