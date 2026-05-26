//! ALGO-GN-029 example: `citing_cited_type_game` — citing×cited-type
//! growing citation network.
//!
//! Generalises `cited_type_game` by letting the **citing** vertex's
//! type also matter: every vertex `i` cites `edges_per_step` earlier
//! vertices, drawing each target from a per-type Fenwick BIT weighted
//! by `pref[type[i]][type[j]]`. The output highlights three signatures
//! that distinguish `citing_cited_type_game` from sibling citation
//! models:
//!   * vertex counts per type follow the deterministic round-robin
//!     assignment exactly (types are INPUT to the game, not sampled),
//!   * the citation-flow matrix `flow[s][t]` concentrates along the
//!     diagonal when `pref` is diagonal (assortative regime),
//!   * with all-positive pref no self-loops occur, but multi-edges
//!     ARE allowed (multiple draws at one step may pick the same
//!     target).
//!
//! Run: `cargo run --example citing_cited_type_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::many_single_char_names
)]

use rust_igraph::{Graph, citing_cited_type_game};

fn count_per_type(types: &[u32], num_types: usize) -> Vec<u32> {
    let mut counts = vec![0u32; num_types];
    for &t in types {
        counts[t as usize] += 1;
    }
    counts
}

/// Citation-flow matrix: `flow[s][t]` = number of edges from a
/// type-`s` citing vertex to a type-`t` cited vertex.
fn flow_matrix(g: &Graph, types: &[u32], num_types: usize) -> Vec<Vec<u64>> {
    let mut flow = vec![vec![0u64; num_types]; num_types];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (s, d) = g.edge(eid).expect("edge id in bounds for example");
        let st = types[s as usize] as usize;
        let dt = types[d as usize] as usize;
        flow[st][dt] += 1;
    }
    flow
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n: u32 = 2_000;
    let num_types: u32 = 3;
    let edges_per_step: u32 = 3;
    let types: Vec<u32> = (0..n).map(|v| v % num_types).collect();
    // Sharply assortative 3x3 pref: each citing type prefers its own
    // cited type 100x over the others.
    let pref_rows: Vec<Vec<f64>> = vec![
        vec![10.0, 0.1, 0.1],
        vec![0.1, 10.0, 0.1],
        vec![0.1, 0.1, 10.0],
    ];
    let pref_views: Vec<&[f64]> = pref_rows.iter().map(Vec::as_slice).collect();

    let g = citing_cited_type_game(
        n,
        &types,
        &pref_views,
        edges_per_step,
        true,
        0xC17C_0E70_u64,
    )?;

    let counts = count_per_type(&types, num_types as usize);
    let flow = flow_matrix(&g, &types, num_types as usize);
    let (loops, multi_edges) = count_loops_and_multi(&g);
    let m = g.ecount();

    println!(
        "citing_cited_type: n = {n}, types = {num_types} (round-robin), edges_per_step = {edges_per_step}, pref = diag(10) + off(0.1)"
    );
    println!("  edges                    = {m}");
    println!("  expected edges (n-1)*eps = {}", (n - 1) * edges_per_step);
    println!("  vertex counts per type   = {counts:?}");
    println!("  self-loops               = {loops}");
    println!("  edges in multi-bundles   = {multi_edges}");

    println!("  citation-flow matrix (rows = citing type, cols = cited type):");
    print!("           ");
    for t in 0..(num_types as usize) {
        print!("    type {t} ");
    }
    println!();
    for (s, row) in flow.iter().enumerate().take(num_types as usize) {
        print!("    type {s} ");
        for &cell in row.iter().take(num_types as usize) {
            print!("    {cell:>6} ");
        }
        let row_total: u64 = row.iter().sum();
        let diag_share = if row_total == 0 {
            0.0
        } else {
            100.0 * row[s] as f64 / row_total as f64
        };
        println!("   (diag share = {diag_share:>5.1}%)");
    }

    // Sanity: each row's diagonal share should track the pref ratio
    // 10 / (10 + 0.1 * (T-1)) ≈ 98.0% in the steady state.
    let row_total_pref: f64 = pref_rows[0].iter().sum();
    let target = 100.0 * pref_rows[0][0] / row_total_pref;
    println!("  pref-implied diagonal target per row ≈ {target:>5.1}%");

    Ok(())
}
