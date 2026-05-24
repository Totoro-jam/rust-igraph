//! ALGO-GN-025 example: `degree_sequence_game_vl` — sample a uniformly
//! random **simple connected** graph realising a given degree sequence
//! via the Viger–Latapy edge-switch MCMC (2005).
//!
//! Unlike the configuration-model generator (ALGO-GN-024), VL guarantees:
//!
//! - exact degree preservation (no self-loops, no multi-edges)
//! - weak connectivity (every positive-degree vertex shares a single component)
//!
//! The demo walks three scenarios:
//!
//! 1. **3-regular undirected** at `n = 30`. Even-Σd cubic sequences are
//!    always graphical *and* connected-graphical. We confirm degree and
//!    simplicity invariants are preserved.
//! 2. **Skewed power-law-like** at `n = 12` with degrees
//!    `[5,4,4,3,3,3,2,2,2,2]` (Σ=30, |E|=15). High-degree hubs survive
//!    the MCMC swaps because every accepted swap preserves the multiset
//!    of degrees exactly.
//! 3. **Isolated singleton** at `n = 1`, degree `[0]`. Edge-free graphs
//!    are accepted and trivially connected.
//!
//! Run: `cargo run --example degree_sequence_vl_demo --release`.

#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use std::collections::HashSet;

use rust_igraph::{Graph, degree_sequence_game_vl};

fn observed_degree_sequence(g: &Graph) -> Vec<u32> {
    let vcount = g.vcount() as usize;
    let mut deg = vec![0u32; vcount];
    let ecount = u32::try_from(g.ecount()).expect("ecount fits u32 for demo");
    for eid in 0..ecount {
        let (src, dst) = g.edge(eid).expect("edge id in bounds for demo");
        deg[src as usize] = deg[src as usize].saturating_add(1);
        deg[dst as usize] = deg[dst as usize].saturating_add(1);
    }
    deg
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
        let key = if a < b { (a, b) } else { (b, a) };
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
        let g = degree_sequence_game_vl(&seq, 0xC0DE_C0DE_u64)?;
        let observed = observed_degree_sequence(&g);
        let (loops, multi) = count_self_loops_and_multi(&g);
        println!("Scenario 1: 3-regular undirected, n = {n}");
        println!(
            "  |E| = {} (expected n·d/2 = {})",
            g.ecount(),
            n * d as usize / 2
        );
        println!("  degree sequence preserved exactly = {}", observed == seq);
        println!("  self-loops = {loops}, multi-edges = {multi}  (both must be 0)");
        println!();
    }

    // Scenario 2: skewed power-law-like sequence.
    {
        let seq: Vec<u32> = vec![5, 4, 4, 3, 3, 3, 2, 2, 2, 2];
        let sum: u64 = seq.iter().map(|&d| u64::from(d)).sum();
        let g = degree_sequence_game_vl(&seq, 0xFEED_FACE_u64)?;
        let observed = observed_degree_sequence(&g);
        let (loops, multi) = count_self_loops_and_multi(&g);
        println!("Scenario 2: skewed power-law-like, n = {}", seq.len());
        println!(
            "  Σd = {sum}, |E| = {} (expected Σd/2 = {})",
            g.ecount(),
            sum / 2
        );
        println!("  observed degrees   = {observed:?}");
        println!("  expected degrees   = {seq:?}");
        println!("  preserved exactly  = {}", observed == seq);
        println!("  self-loops = {loops}, multi-edges = {multi}  (both must be 0)");
        println!();
    }

    // Scenario 3: degenerate singleton (vacuous connectivity).
    {
        let seq: Vec<u32> = vec![0];
        let g = degree_sequence_game_vl(&seq, 0x0000_0001_u64)?;
        let observed = observed_degree_sequence(&g);
        println!("Scenario 3: isolated singleton, n = 1");
        println!("  |E| = {} (expected 0)", g.ecount());
        println!("  observed degrees = {observed:?}");
        println!();
    }

    println!("VL guarantees exact degree preservation, simplicity, and weak connectivity;");
    println!(
        "for fast multigraph realisations (no MCMC) use degree_sequence_game_configuration (ALGO-GN-024)."
    );
    Ok(())
}
