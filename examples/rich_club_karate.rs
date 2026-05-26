//! `rich_club_sequence` (ALGO-PR-040) — rich-club coefficient sequence
//! on Zachary's karate club.
//!
//! Run with `cargo run --release --example rich_club_karate`.
//!
//! The "rich-club" phenomenon asks whether the high-degree vertices of a
//! network are densely interconnected among themselves. The classical
//! rich-club coefficient `phi(k)` is the edge density restricted to
//! vertices of degree `> k`. This example builds the same picture from
//! the *vertex-ordering* form returned by `rich_club_sequence`:
//!
//!   1. Sort vertices by *ascending* degree, breaking ties by id.
//!   2. Feed that order to `rich_club_sequence(..., normalized = true)`.
//!      The returned `seq[i]` is then the edge density of the subgraph
//!      that survives after peeling off the `i` lowest-degree vertices —
//!      i.e. it is `phi` evaluated at the i-th degree threshold.
//!   3. Print the resulting curve. A rising tail means the surviving
//!      "rich" core is denser than the network as a whole.
//!
//! For karate (34 vertices, 78 edges), the trailing entries climb from
//! the global density (~0.139) toward 1.0 — the very last finite entry
//! is the density of the two-vertex subgraph induced by the two highest-
//! degree vertices (the instructor and the president), which is `1.0`
//! when they are connected and `0.0` when they are not. The final entry
//! `NaN` corresponds to the trailing single-vertex subgraph where the
//! denominator `n(n-1)/2 = 0`.

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{Graph, read_edgelist, rich_club_sequence};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    let g: Graph = read_edgelist(File::open(path)?)?;
    let n = g.vcount();
    println!("Karate club: {} vertices, {} edges", n, g.ecount());

    // Degree of every vertex.
    let mut deg: Vec<(u32, u32)> = (0..n)
        .map(|v| {
            (
                v,
                u32::try_from(g.neighbors(v).expect("vertex in range").len()).unwrap(),
            )
        })
        .collect();
    // Ascending degree, tie-break by vertex id — peel low-degree first.
    deg.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    let vertex_order: Vec<u32> = deg.iter().map(|&(v, _)| v).collect();

    let seq = rich_club_sequence(&g, None, &vertex_order, true, false, false)?;

    println!("\nRich-club coefficient phi(i) after peeling i lowest-degree vertices:");
    println!("  (vertex_order = ascending degree, id tiebreak)\n");
    println!(
        "    {:>3}  {:>10}  {:>10}  {:>12}",
        "i", "remaining", "min_deg", "phi(i)"
    );
    for (i, &phi) in seq.iter().enumerate() {
        let remaining = (n as usize) - i;
        let min_deg_after = if i < n as usize { deg[i].1 } else { u32::MAX };
        let phi_str = if phi.is_nan() {
            "      NaN".to_string()
        } else {
            format!("{phi:.6}")
        };
        println!("    {i:>3}  {remaining:>10}  {min_deg_after:>10}  {phi_str:>12}");
    }

    // Highlight the most "rich" trailing prefix: the densest non-trivial
    // surviving subgraph.
    let best = seq
        .iter()
        .enumerate()
        .filter(|(i, v)| *i + 2 <= n as usize && v.is_finite())
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("finite"));
    if let Some((i, phi)) = best {
        let remaining = n as usize - i;
        println!(
            "\nDensest non-trivial surviving subgraph: after peeling {i} vertices, \
             {remaining} survive with phi = {phi:.6}."
        );
    }

    Ok(())
}
