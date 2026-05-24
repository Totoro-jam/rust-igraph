//! ALGO-GN-022 example: `dot_product_game` — random dot-product graphs
//! with community-structured latent positions.
//!
//! Builds a 120-vertex undirected graph from latent vectors arranged as
//! three communities. Each community's vectors are concentrated near a
//! distinct unit-simplex corner (`e1`, `e2`, `e3` in 3-D), so within-
//! community pairs see `<v_i, v_j> ≈ a²` while across-community pairs
//! see `<v_i, v_j> ≈ a·b` with `a >> b`. The resulting edge density is
//! dramatically higher inside communities than across them — the
//! defining property of the RDPG model.
//!
//! The output prints, for each pair of communities, the connection
//! probability (edge count / possible pairs) along with the overall
//! intra- vs inter-community contrast.
//!
//! Run: `cargo run --example dot_product_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::many_single_char_names
)]

use rust_igraph::{Graph, dot_product_game_with_warnings};

const N_PER_COMMUNITY: usize = 40;
const N_COMMUNITIES: usize = 3;
const N_TOTAL: usize = N_PER_COMMUNITY * N_COMMUNITIES; // 120

/// Latent vector for vertex `v`: place it near the basis vector that
/// matches its community, with `a` on the chosen axis and `b` on the
/// other two. Choosing `a = 0.85`, `b = 0.10` gives intra-prob
/// `≈ 0.85² + 0.10² + 0.10² = 0.7425` and inter-prob
/// `≈ 0.85·0.10 + 0.85·0.10 + 0.10² = 0.18` — a ~4× contrast.
fn latent(v: usize) -> Vec<f64> {
    let a = 0.85_f64;
    let b = 0.10_f64;
    let community = v / N_PER_COMMUNITY;
    (0..N_COMMUNITIES)
        .map(|axis| if axis == community { a } else { b })
        .collect()
}

fn community_of(v: u32) -> usize {
    (v as usize) / N_PER_COMMUNITY
}

fn pair_block_counts(g: &Graph) -> [[u32; N_COMMUNITIES]; N_COMMUNITIES] {
    let mut counts = [[0u32; N_COMMUNITIES]; N_COMMUNITIES];
    let m = u32::try_from(g.ecount()).expect("ecount fits u32 for example");
    for eid in 0..m {
        let (s, d) = g.edge(eid).expect("edge id in bounds for example");
        let cs = community_of(s);
        let cd = community_of(d);
        // Canonicalise: store into the upper-triangular slot. Each
        // undirected edge contributes to exactly one (a, b) block with
        // a <= b, matching the unordered pair capacity used below.
        let (a, b) = if cs <= cd { (cs, cd) } else { (cd, cs) };
        counts[a][b] += 1;
    }
    counts
}

fn block_pair_capacity(a: usize, b: usize) -> u32 {
    if a == b {
        // Unordered intra-community pairs.
        let k = N_PER_COMMUNITY as u32;
        k * (k - 1) / 2
    } else {
        // Unordered cross-community pairs.
        let k = N_PER_COMMUNITY as u32;
        k * k
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vecs: Vec<Vec<f64>> = (0..N_TOTAL).map(latent).collect();
    let seed: u64 = 0x22_AB_BE_EF_u64.wrapping_mul(0x9E37_79B9_7F4A_7C15);

    println!(
        "dot_product_game: n = {N_TOTAL}, d = {N_COMMUNITIES}, undirected, 3 planted communities"
    );
    println!("------------------------------------------------------------");

    let (g, warnings) = dot_product_game_with_warnings(&vecs, false, seed)?;

    println!("vcount = {}, ecount = {}", g.vcount(), g.ecount());
    println!(
        "had_negative = {}, had_over_one = {}  (both expected false for these vectors)",
        warnings.had_negative, warnings.had_over_one
    );
    println!();

    let counts = pair_block_counts(&g);
    println!("connection probability per (community_a, community_b) block:");
    let mut intra_p = 0.0_f64;
    let mut inter_p = 0.0_f64;
    let mut intra_blocks = 0u32;
    let mut inter_blocks = 0u32;
    for (a, row) in counts.iter().enumerate() {
        for (b, &pairs) in row.iter().enumerate().skip(a) {
            let cap = block_pair_capacity(a, b);
            let p = f64::from(pairs) / f64::from(cap);
            println!("  ({a}, {b}): edges = {pairs:>4} / {cap:>4} → p ≈ {p:.3}");
            if a == b {
                intra_p += p;
                intra_blocks += 1;
            } else {
                inter_p += p;
                inter_blocks += 1;
            }
        }
    }

    let intra_avg = intra_p / intra_blocks as f64;
    let inter_avg = inter_p / inter_blocks as f64;
    println!();
    println!("average intra-community p = {intra_avg:.3} (theory ≈ 0.743)");
    println!("average inter-community p = {inter_avg:.3} (theory ≈ 0.180)");
    println!("contrast intra / inter    = {:.2}×", intra_avg / inter_avg);

    Ok(())
}
