//! ALGO-GN-028 example: `degree_sequence_game_edge_switching_simple` —
//! sample a *simple* graph realising a given degree sequence via a
//! two-phase strategy: deterministic Havel–Hakimi (undirected) /
//! Kleitman–Wang (directed) INDEX seed, followed by `10 · |E|`
//! degree-preserving edge-switching MCMC trials (mirrors
//! `IGRAPH_DEGSEQ_EDGE_SWITCHING_SIMPLE`).
//!
//! How it compares to its siblings:
//!   * vs `degree_sequence_game_configuration` (ALGO-GN-024): GN-024
//!     permits self-loops and multi-edges; GN-028 rewires until the
//!     graph is simple while keeping the exact degree sequence.
//!   * vs `degree_sequence_game_vl` (ALGO-GN-025): VL also yields a
//!     simple graph and additionally enforces *connectivity*; GN-028
//!     drops the connectivity guarantee and gains linear-in-|E|
//!     scaling at any density.
//!   * vs `degree_sequence_game_fast_heur_simple` (ALGO-GN-026):
//!     `FAST_HEUR_SIMPLE` is faster but gives no MCMC mixing guarantee.
//!     `EDGE_SWITCHING_SIMPLE` costs more per call but the output
//!     distribution becomes uniform as the chain mixes.
//!   * vs `degree_sequence_game_configuration_simple` (ALGO-GN-027):
//!     `CONFIGURATION_SIMPLE` is uniform but uses rejection sampling
//!     that degrades exponentially with density
//!     (`exp(O((Σd/n)²))` expected restarts). `EDGE_SWITCHING_SIMPLE`
//!     stays linear in `|E|` for *any* graphical input, so pick it
//!     for dense / skewed sequences.
//!
//! The demo walks four scenarios:
//!
//! 1. **3-regular undirected** at `n = 10`. Tractable for every
//!    sibling; baseline for comparison.
//! 2. **Dense regime** at `n = 10`, degrees `[5,4,4,3,3,3,2,2,2,2]`
//!    (Σ=30, |E|=15, density 3) — a regime where `CONFIGURATION_SIMPLE`
//!    rejection-samples heavily but `EDGE_SWITCHING_SIMPLE` handles in
//!    stride.
//! 3. **Directed mixed in/out** at `n = 10`, the exact upstream
//!    `outarr` / `inarr` from the C test suite (Σ=28). Demonstrates
//!    the directed branch: independent out/in INDEX seed,
//!    out-adjacency multi-arc detection only.
//! 4. **Isolated singleton** at `n = 1`, degree `[0]` — accepted via
//!    the empty/early-exit branch.
//!
//! Run: `cargo run --example degree_sequence_edge_switching_simple_demo --release`.

#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use std::collections::HashSet;

use rust_igraph::{Graph, degree_sequence_game_edge_switching_simple};

fn observed_out_in(g: &Graph) -> (Vec<u32>, Vec<u32>) {
    let vcount = g.vcount() as usize;
    let mut out = vec![0u32; vcount];
    let mut inv = vec![0u32; vcount];
    let ecount = u32::try_from(g.ecount()).expect("ecount fits u32 for demo");
    for eid in 0..ecount {
        let (src, dst) = g.edge(eid).expect("edge id in bounds for demo");
        if g.is_directed() {
            out[src as usize] = out[src as usize].saturating_add(1);
            inv[dst as usize] = inv[dst as usize].saturating_add(1);
        } else {
            out[src as usize] = out[src as usize].saturating_add(1);
            out[dst as usize] = out[dst as usize].saturating_add(1);
        }
    }
    (out, inv)
}

fn count_self_loops_and_multi(g: &Graph) -> (u32, u32) {
    let ecount = u32::try_from(g.ecount()).expect("ecount fits u32 for demo");
    let mut loops = 0u32;
    let mut seen: HashSet<(u32, u32)> = HashSet::with_capacity(ecount as usize);
    let mut multi = 0u32;
    for eid in 0..ecount {
        let (a, b) = g.edge(eid).expect("edge id in bounds for demo");
        if a == b {
            loops = loops.saturating_add(1);
            continue;
        }
        let key = if g.is_directed() || a < b {
            (a, b)
        } else {
            (b, a)
        };
        if !seen.insert(key) {
            multi = multi.saturating_add(1);
        }
    }
    (loops, multi)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Scenario 1: 3-regular undirected (baseline).
    {
        let n: usize = 10;
        let d: u32 = 3;
        let seq: Vec<u32> = vec![d; n];
        let g = degree_sequence_game_edge_switching_simple(&seq, None, 0xC0DE_C0DE_u64)?;
        let (out, _) = observed_out_in(&g);
        let (loops, multi) = count_self_loops_and_multi(&g);
        println!("Scenario 1: 3-regular undirected, n = {n}");
        println!(
            "  |E| = {} (expected n*d/2 = {})",
            g.ecount(),
            n * d as usize / 2
        );
        println!("  degree sequence preserved exactly = {}", out == seq);
        println!("  self-loops = {loops}, multi-edges = {multi}  (both must be 0)");
        println!();
    }

    // Scenario 2: dense regime — EDGE_SWITCHING_SIMPLE shines here.
    {
        let seq: Vec<u32> = vec![5, 4, 4, 3, 3, 3, 2, 2, 2, 2];
        let sum: u64 = seq.iter().map(|&d| u64::from(d)).sum();
        let g = degree_sequence_game_edge_switching_simple(&seq, None, 0xFEED_FACE_u64)?;
        let (out, _) = observed_out_in(&g);
        let (loops, multi) = count_self_loops_and_multi(&g);
        println!(
            "Scenario 2: dense regime (density Σd/n = {:.1}), n = {}",
            sum as f64 / seq.len() as f64,
            seq.len()
        );
        println!(
            "  Σd = {sum}, |E| = {} (expected Σd/2 = {})",
            g.ecount(),
            sum / 2
        );
        println!("  observed degrees   = {out:?}");
        println!("  expected degrees   = {seq:?}");
        println!("  preserved exactly  = {}", out == seq);
        println!("  self-loops = {loops}, multi-edges = {multi}  (both must be 0)");
        println!();
    }

    // Scenario 3: directed mixed in/out — upstream C-test inputs verbatim.
    {
        let out_seq: Vec<u32> = vec![2, 3, 2, 3, 3, 3, 3, 1, 4, 4];
        let in_seq: Vec<u32> = vec![3, 6, 2, 0, 2, 2, 4, 3, 3, 3];
        let sum_out: u64 = out_seq.iter().map(|&d| u64::from(d)).sum();
        let sum_in: u64 = in_seq.iter().map(|&d| u64::from(d)).sum();
        assert_eq!(sum_out, sum_in);
        let g =
            degree_sequence_game_edge_switching_simple(&out_seq, Some(&in_seq), 0xBEEF_BABE_u64)?;
        let (out, inv) = observed_out_in(&g);
        let (loops, multi) = count_self_loops_and_multi(&g);
        println!(
            "Scenario 3: directed mixed (C-test verbatim), n = {}",
            out_seq.len()
        );
        println!("  Σout = {sum_out}, |E| = {} (must equal Σout)", g.ecount());
        println!("  observed out  = {out:?}");
        println!("  expected out  = {out_seq:?}");
        println!("  observed in   = {inv:?}");
        println!("  expected in   = {in_seq:?}");
        println!("  preserved exactly = {}", out == out_seq && inv == in_seq);
        println!("  self-loops = {loops}, multi-arcs = {multi}  (both must be 0)");
        println!();
    }

    // Scenario 4: isolated singleton (early-exit branch).
    {
        let seq: Vec<u32> = vec![0];
        let g = degree_sequence_game_edge_switching_simple(&seq, None, 0x0000_0001_u64)?;
        let (out, _) = observed_out_in(&g);
        println!("Scenario 4: isolated singleton, n = 1");
        println!("  |E| = {} (expected 0)", g.ecount());
        println!("  observed degrees = {out:?}");
        println!();
    }

    println!("EDGE_SWITCHING_SIMPLE yields a simple realisation of the input degree sequence,");
    println!("running in O(n² + |E|) wall-clock independent of density. Pick this sampler when");
    println!("density is high or CONFIGURATION_SIMPLE (ALGO-GN-027) hits its restart budget.");
    println!("For connectivity, use degree_sequence_game_vl (ALGO-GN-025); for fastest sampling,");
    println!("use degree_sequence_game_fast_heur_simple (ALGO-GN-026).");
    Ok(())
}
