//! ALGO-GN-019 example: `recent_degree_game` — sliding-window
//! preferential attachment.
//!
//! Grows a 2 000-vertex directed graph where each step emits `m = 3`
//! outgoing citations weighted by `pow(recent_in_degree, power) +
//! zero_appeal`. The "recent" in-degree only counts citations received
//! within the last `time_window = 25` steps; older citations are
//! expired from the BIT-tree.
//!
//! The output illustrates four signatures of the recent-degree model:
//!   * NEVER self-loops (the psumtree only ranges over previously-added
//!     vertices);
//!   * the in-degree distribution is heavy-tailed (rich-get-richer
//!     amplifies through each window refresh — once a vertex is hot it
//!     keeps being cited, refreshing its window timer);
//!   * the heaviest hitters tend to cluster on the *earliest* cohort
//!     because those vertices had the most opportunities to accumulate
//!     citations before the window started expiring their early hits;
//!   * `zero_appeal` keeps cold vertices in play but with vanishing
//!     probability when the BIT-tree is dominated by hot vertices.
//!
//! Run: `cargo run --example recent_degree_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::many_single_char_names
)]

use rust_igraph::{Graph, recent_degree_game};

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
    let m: u32 = 3;
    let power = 1.0;
    let time_window: u32 = 25;
    let zero_appeal = 1.0;

    let g = recent_degree_game(
        n,
        power,
        time_window,
        m,
        None,
        false,
        zero_appeal,
        true,
        0x2D57_DEF0_u64,
    )?;

    let edge_count = g.ecount();
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
        "recent_degree: n = {n}, m = {m}, power = {power}, time_window = {time_window}, zero_appeal = {zero_appeal}"
    );
    println!("  edges                    = {edge_count}");
    println!("  expected edges (n-1)*m   = {}", (n - 1) * m);
    println!("  self-loops               = {loops}  (must be 0 — recent_degree never self-loops)");
    println!("  edges in multi-bundles   = {multi_edges}  (multi-edges allowed when m ≥ 2)");
    println!();
    println!("  in-degree summary:");
    println!("    mean   = {mean:.2}");
    println!("    Q1     = {q1}");
    println!("    median = {median}");
    println!("    Q3     = {q3}");
    println!("    max    = {max_in}");
    println!("    uncited vertices = {uncited} / {n}");

    // Sample high-in-degree vertices. Because the window expires old
    // edges, the heaviest hitters should NOT be the very oldest
    // cohort (cf. lastcit, which can refresh forever); they should
    // cluster in the steady-state regime after warm-up.
    let mut indexed: Vec<(u32, u32)> = in_deg
        .iter()
        .enumerate()
        .map(|(v, &d)| (u32::try_from(v).expect("vertex index fits u32"), d))
        .collect();
    indexed.sort_by(|a, b| b.1.cmp(&a.1));
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
