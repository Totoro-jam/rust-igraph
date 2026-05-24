//! ALGO-GN-017 example: `cited_type_game` — type-weighted growing
//! citation network.
//!
//! Grows a 2 000-vertex directed citation graph where each vertex has a
//! pre-assigned type and incoming-citation probability is proportional
//! to `pref[type]`. Three types with sharply different attractivity:
//! `pref = [10.0, 1.0, 0.05]` — type 0 collects 100x more citation
//! mass than type 2 at parity. The output highlights three signatures
//! that distinguish `cited_type_game` from sibling citation models:
//!   * vertex counts per type follow the deterministic round-robin
//!     assignment exactly (types are INPUT to the game, not sampled),
//!   * mean in-degree concentrates heavily on the high-pref type,
//!   * with all-positive pref no self-loops occur, but multi-edges
//!     ARE allowed (multiple citations at one step may pick the same
//!     target).
//!
//! Run: `cargo run --example cited_type_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::many_single_char_names
)]

use rust_igraph::{Graph, cited_type_game};

fn count_per_type(types: &[u32], num_types: usize) -> Vec<u32> {
    let mut counts = vec![0u32; num_types];
    for &t in types {
        counts[t as usize] += 1;
    }
    counts
}

/// In-degree per type bucket. `cited_type_game` always pushes edges as
/// `(i, target)` so the destination accumulates in-degree.
fn in_degree_per_type(g: &Graph, types: &[u32], num_types: usize) -> Vec<u64> {
    let mut deg = vec![0u64; num_types];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (_s, d) = g.edge(eid).expect("edge id in bounds for example");
        deg[types[d as usize] as usize] += 1;
    }
    deg
}

/// Counts self-loops and parallel-edge bundles.
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n: u32 = 2_000;
    let num_types: u32 = 3;
    let edges_per_step: u32 = 3;
    // Round-robin type assignment: each type gets exactly n/num_types
    // vertices (give or take 1 from modulo).
    let types: Vec<u32> = (0..n).map(|v| v % num_types).collect();
    // Sharply skewed attractivity: type 0 collects ~200x more mass
    // than type 2 at parity.
    let pref = vec![10.0, 1.0, 0.05];

    let g = cited_type_game(n, &types, &pref, edges_per_step, true, 0xC17E_0E70_u64)?;

    let counts = count_per_type(&types, num_types as usize);
    let in_deg = in_degree_per_type(&g, &types, num_types as usize);
    let (loops, multi_edges) = count_loops_and_multi(&g);
    let m = g.ecount();

    println!(
        "cited_type: n = {n}, types = {num_types} (round-robin), edges_per_step = {edges_per_step}, pref = [10.0, 1.0, 0.05]"
    );
    println!("  edges                    = {m}");
    println!("  expected edges (n-1)*eps = {}", (n - 1) * edges_per_step);
    println!("  vertex counts per type   = {counts:?}");
    println!("  self-loops               = {loops}");
    println!("  edges in multi-bundles   = {multi_edges}");

    println!("  in-degree share per type:");
    for t in 0..num_types as usize {
        let cnt = counts[t];
        let deg = in_deg[t];
        let share = if m == 0 {
            0.0
        } else {
            100.0 * deg as f64 / m as f64
        };
        let mean = if cnt == 0 {
            0.0
        } else {
            deg as f64 / cnt as f64
        };
        println!(
            "    type {t}  pref = {:>6}  count = {cnt:>4}  in_deg = {deg:>5}  share = {share:>5.1}%  mean_in_deg = {mean:.2}",
            pref[t]
        );
    }

    // Sanity: ratio of in-degrees should track ratio of prefs (with
    // noise — late-arriving high-pref vertices need a few steps to
    // accumulate citations).
    let total: f64 = pref.iter().sum();
    println!("  pref-implied share targets:");
    for (t, &p) in pref.iter().enumerate().take(num_types as usize) {
        println!("    type {t}: {:>5.1}%", 100.0 * p / total);
    }

    Ok(())
}
