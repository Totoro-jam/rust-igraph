//! ALGO-GN-015 example: establishment / sample-traits diagnostics.
//!
//! Grows a 2 000-vertex undirected establishment graph with `types = 3`
//! and a strongly assortative pref matrix (high diagonal, low
//! off-diagonal). The output highlights the two signatures of the
//! Caldarelli et al. (2002) model:
//!   * the per-type vertex count tracks `type_dist` after thinning,
//!   * within-type edges dominate the edge set when the pref matrix
//!     is diagonal-heavy.
//!
//! Run: `cargo run --example establishment_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::many_single_char_names
)]

use rust_igraph::{Graph, establishment_game};

fn count_per_type(types: &[u32], num_types: usize) -> Vec<u32> {
    let mut counts = vec![0u32; num_types];
    for &t in types {
        counts[t as usize] += 1;
    }
    counts
}

fn within_vs_cross(g: &Graph, types: &[u32]) -> (u64, u64) {
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    let mut within: u64 = 0;
    let mut cross: u64 = 0;
    for eid in 0..m {
        let (s, d) = g.edge(eid).expect("edge id in bounds for example");
        if types[s as usize] == types[d as usize] {
            within += 1;
        } else {
            cross += 1;
        }
    }
    (within, cross)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n: u32 = 2_000;
    let num_types: u32 = 3;
    let k: u32 = 4;
    // Skewed type distribution: type 0 is twice as common as the other
    // two on the new-vertex draw.
    let type_dist = vec![0.50, 0.25, 0.25];
    // Assortative pref: 0.30 within, 0.02 across.
    let pref = vec![
        vec![0.30, 0.02, 0.02],
        vec![0.02, 0.30, 0.02],
        vec![0.02, 0.02, 0.30],
    ];

    let (g, types) = establishment_game(
        n,
        num_types,
        k,
        Some(&type_dist),
        &pref,
        false,
        0xE5AB_0E70_u64,
    )?;

    let counts = count_per_type(&types, num_types as usize);
    let (within, cross) = within_vs_cross(&g, &types);
    let m = within + cross;

    println!(
        "establishment: n = {n}, types = {num_types}, k = {k}, type_dist = [0.50, 0.25, 0.25]"
    );
    println!("  edges                    = {m}");
    println!("  vertex counts per type   = {counts:?}");
    println!(
        "  type proportions         = [{:.3}, {:.3}, {:.3}]",
        counts[0] as f64 / n as f64,
        counts[1] as f64 / n as f64,
        counts[2] as f64 / n as f64
    );
    if m > 0 {
        println!(
            "  within-type edges        = {within} ({:.1}%)",
            100.0 * within as f64 / m as f64
        );
        println!(
            "  cross-type edges         = {cross} ({:.1}%)",
            100.0 * cross as f64 / m as f64
        );
    }

    // Per-type mean degree gives a quick read on whether the diagonal
    // pref values are translating into a dense intra-type subgraph.
    let mut deg_sum = vec![0u64; num_types as usize];
    let edges_m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..edges_m {
        let (s, d) = g.edge(eid).expect("edge id in bounds for example");
        deg_sum[types[s as usize] as usize] += 1;
        deg_sum[types[d as usize] as usize] += 1;
    }
    println!("  mean degree per type:");
    for t in 0..num_types as usize {
        let c = counts[t];
        let mean = if c == 0 {
            0.0
        } else {
            deg_sum[t] as f64 / c as f64
        };
        println!("    type {t}  count = {c:>4}  mean_deg = {mean:.2}");
    }

    Ok(())
}
