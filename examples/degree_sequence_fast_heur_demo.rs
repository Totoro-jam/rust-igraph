//! ALGO-GN-026 example: `degree_sequence_game_fast_heur_simple` — sample
//! a *simple* (no self-loops, no multi-edges) graph realising a given
//! degree sequence via the fast-heuristic algorithm of igraph C
//! (`IGRAPH_DEGSEQ_FAST_HEUR_SIMPLE`).
//!
//! Unlike the configuration-model generator (ALGO-GN-024) the output is
//! guaranteed simple; unlike the Viger-Latapy generator (ALGO-GN-025)
//! the output is NOT guaranteed connected, in exchange for one to two
//! orders of magnitude faster sampling.
//!
//! The demo walks four scenarios:
//!
//! 1. **3-regular undirected** at `n = 30`. Always graphical; we verify
//!    degree preservation and simplicity.
//! 2. **Skewed power-law-like** at `n = 12` with degrees
//!    `[5,4,4,3,3,3,2,2,2,2]` (Σ=30, |E|=15). High-degree hubs survive
//!    because every accepted pair preserves the residual multiset.
//! 3. **Directed balanced** at `n = 8`, out/in = `[3,2,2,1,1,1,1,1]`,
//!    `[2,2,1,2,1,1,1,2]` (Σ=12). Demonstrates the directed branch:
//!    separate out/in stub bags and a per-source sorted-Vec adjacency.
//! 4. **Isolated singleton** at `n = 1`, degree `[0]`. Edge-free graphs
//!    are accepted.
//!
//! Run: `cargo run --example degree_sequence_fast_heur_demo --release`.

#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use std::collections::HashSet;

use rust_igraph::{Graph, degree_sequence_game_fast_heur_simple};

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
    // Scenario 1: 3-regular undirected.
    {
        let n: usize = 30;
        let d: u32 = 3;
        let seq: Vec<u32> = vec![d; n];
        let g = degree_sequence_game_fast_heur_simple(&seq, None, 0xC0DE_C0DE_u64)?;
        let (out, _) = observed_out_in(&g);
        let (loops, multi) = count_self_loops_and_multi(&g);
        println!("Scenario 1: 3-regular undirected, n = {n}");
        println!(
            "  |E| = {} (expected n·d/2 = {})",
            g.ecount(),
            n * d as usize / 2
        );
        println!("  degree sequence preserved exactly = {}", out == seq);
        println!("  self-loops = {loops}, multi-edges = {multi}  (both must be 0)");
        println!();
    }

    // Scenario 2: skewed power-law-like sequence.
    {
        let seq: Vec<u32> = vec![5, 4, 4, 3, 3, 3, 2, 2, 2, 2];
        let sum: u64 = seq.iter().map(|&d| u64::from(d)).sum();
        let g = degree_sequence_game_fast_heur_simple(&seq, None, 0xFEED_FACE_u64)?;
        let (out, _) = observed_out_in(&g);
        let (loops, multi) = count_self_loops_and_multi(&g);
        println!("Scenario 2: skewed power-law-like, n = {}", seq.len());
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

    // Scenario 3: directed graph with mixed in/out degrees.
    {
        let out_seq: Vec<u32> = vec![3, 2, 2, 1, 1, 1, 1, 1];
        let in_seq: Vec<u32> = vec![2, 2, 1, 2, 1, 1, 1, 2];
        let sum_out: u64 = out_seq.iter().map(|&d| u64::from(d)).sum();
        let sum_in: u64 = in_seq.iter().map(|&d| u64::from(d)).sum();
        assert_eq!(sum_out, sum_in);
        let g = degree_sequence_game_fast_heur_simple(&out_seq, Some(&in_seq), 0xBEEF_BABE_u64)?;
        let (out, inv) = observed_out_in(&g);
        let (loops, multi) = count_self_loops_and_multi(&g);
        println!("Scenario 3: directed mixed, n = {}", out_seq.len());
        println!("  Σout = {sum_out}, |E| = {} (must equal Σout)", g.ecount());
        println!("  observed out  = {out:?}");
        println!("  expected out  = {out_seq:?}");
        println!("  observed in   = {inv:?}");
        println!("  expected in   = {in_seq:?}");
        println!("  preserved exactly = {}", out == out_seq && inv == in_seq);
        println!("  self-loops = {loops}, multi-arcs = {multi}  (both must be 0)");
        println!();
    }

    // Scenario 4: degenerate singleton (isolated vertex).
    {
        let seq: Vec<u32> = vec![0];
        let g = degree_sequence_game_fast_heur_simple(&seq, None, 0x0000_0001_u64)?;
        let (out, _) = observed_out_in(&g);
        println!("Scenario 4: isolated singleton, n = 1");
        println!("  |E| = {} (expected 0)", g.ecount());
        println!("  observed degrees = {out:?}");
        println!();
    }

    println!(
        "FAST_HEUR_SIMPLE guarantees exact degree preservation and simplicity but NOT connectivity;"
    );
    println!("for connected simple realisations use degree_sequence_game_vl (ALGO-GN-025), or");
    println!("degree_sequence_game_configuration (ALGO-GN-024) for fast multigraph sampling.");
    Ok(())
}
