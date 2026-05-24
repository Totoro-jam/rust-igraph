//! ALGO-GN-021 example: `barabasi_aging_game` — preferential
//! attachment with vertex aging.
//!
//! Compares two 2 000-vertex directed graphs that differ only in
//! `aging_exp`:
//!   * baseline run (`aging_exp = 0`, age term collapses to a
//!     constant) — recovers classical Barabási–Albert; old hubs
//!     dominate the in-degree tail because they have had the most
//!     time to accumulate citations;
//!   * aging run (`aging_exp = -1`, linear age suppression) — newer
//!     vertices are favoured; the heaviest hitters are pushed away
//!     from the very oldest cohort, demonstrating the suppression
//!     behaviour described in Albert & Barabási (2002).
//!
//! The output prints, for each run, the top-10 hubs with their cohort
//! (`old / early-mid / late-mid / recent`) plus the share of total
//! in-degree captured by the young half of the graph.
//!
//! Run: `cargo run --example barabasi_aging_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]

use rust_igraph::{Graph, barabasi_aging_game};

fn in_degree(g: &Graph) -> Vec<u32> {
    let mut deg = vec![0u32; g.vcount() as usize];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (_s, d) = g.edge(eid).expect("edge id in bounds for example");
        deg[d as usize] += 1;
    }
    deg
}

fn cohort(v: u32, n: u32) -> &'static str {
    if v < n / 4 {
        "old"
    } else if v < n / 2 {
        "early-mid"
    } else if v < 3 * n / 4 {
        "late-mid"
    } else {
        "recent"
    }
}

fn young_share(in_deg: &[u32]) -> f64 {
    let n = in_deg.len();
    if n == 0 {
        return 0.0;
    }
    let half = n / 2;
    let total: u64 = in_deg.iter().map(|&d| u64::from(d)).sum();
    if total == 0 {
        return 0.0;
    }
    let young: u64 = in_deg.iter().skip(half).map(|&d| u64::from(d)).sum();
    young as f64 / total as f64
}

fn report(label: &str, g: &Graph) {
    let n = g.vcount();
    let in_deg = in_degree(g);
    let mut indexed: Vec<(u32, u32)> = in_deg
        .iter()
        .enumerate()
        .map(|(v, &d)| (u32::try_from(v).expect("vertex index fits u32"), d))
        .collect();
    indexed.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{label}");
    println!("  edges                    = {}", g.ecount());
    println!(
        "  young-half in-degree share = {:.3}  (1/2 = no preference)",
        young_share(&in_deg)
    );
    println!("  top-10 hubs (vertex_index, in_degree, cohort):");
    for (v, d) in indexed.iter().take(10) {
        println!(
            "    v = {v:>5} (in_deg = {d:>3}, cohort = {})",
            cohort(*v, n)
        );
    }
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n: u32 = 2_000;
    let m: u32 = 3;
    let aging_bins: u32 = 30;
    let seed: u64 = 0x21_AB_BE_EF_u64.wrapping_mul(0x9E37_79B9_7F4A_7C15);

    println!("barabasi_aging_game: n = {n}, m = {m}, aging_bins = {aging_bins}, directed");
    println!("--------------------------------------------------------------------");

    // Baseline: pa=1, aging=0, zero_age_appeal=1 — age term collapses
    // to a constant `(1·1 + 1) = 2`, recovering classical BA.
    let baseline = barabasi_aging_game(
        n, m, None, false, 1.0, 0.0, aging_bins, 1.0, 1.0, 1.0, 1.0, true, seed,
    )?;
    report("baseline (aging_exp = 0, classical BA):", &baseline);

    // Aging run: pa=1, aging=-1, zero_age_appeal=0 — the age term
    // becomes `pow(age, -1)` with no floor, so the oldest cohort's
    // weight decays to 1/age while the newest stays near 1. This makes
    // the aging effect dramatic enough to see in 2000 vertices.
    let aging = barabasi_aging_game(
        n, m, None, false, 1.0, -1.0, aging_bins, 1.0, 0.0, 1.0, 1.0, true, seed,
    )?;
    report(
        "aging (aging_exp = -1, zero_age_appeal = 0, sharp age decay):",
        &aging,
    );

    // Sanity check: aging run pushes more in-degree onto the young
    // half. Print the directional gap.
    let base_share = young_share(&in_degree(&baseline));
    let aging_share = young_share(&in_degree(&aging));
    println!(
        "young-half share gap: aging - baseline = {:+.3}  (positive ⇒ aging lifts the young half)",
        aging_share - base_share
    );

    Ok(())
}
