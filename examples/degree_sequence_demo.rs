//! ALGO-GN-024 example: `degree_sequence_game_configuration` — sample
//! a random multigraph realising a given degree sequence by stub
//! matching (the configuration model).
//!
//! The demo runs three scenarios that exercise the algorithm's three
//! interesting regimes:
//!
//! 1. **Uniform 4-regular undirected** at `n = 200`. The realised graph
//!    has exactly `n · d / 2 = 400` edges and every vertex sits at
//!    degree 4 by construction. We report the expected count of
//!    self-loops `E[L] = n · d² / (2 · Σd) = n · d / (2(2m)) · n · d²
//!    ≈ d² / (2 · 2) = 2.0` for `d = 4` and compare to the observed
//!    count.
//! 2. **Skewed (Zipf-like) undirected** at `n = 100` with degrees
//!    `floor(n / (r + 1))` for `r = 0..n`. The first vertex hoards
//!    ~50% of stubs, so we expect a heavy self-loop and multi-edge
//!    load concentrated on vertex 0.
//! 3. **Balanced directed** at `n = 50` with `out = [1,2,3,4,5] * 10`
//!    and `in = out`. The directed branch uses two independent stub
//!    bags; degrees are preserved exactly.
//!
//! Run: `cargo run --example degree_sequence_demo --release`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::collections::HashMap;

use rust_igraph::{Graph, degree_sequence_game_configuration};

fn observed_degrees(g: &Graph) -> (Vec<u32>, Vec<u32>) {
    let vcount = g.vcount() as usize;
    let mut out_deg = vec![0u32; vcount];
    let mut in_deg = vec![0u32; vcount];
    let ecount = u32::try_from(g.ecount()).expect("ecount fits u32 for demo");
    for eid in 0..ecount {
        let (src, dst) = g.edge(eid).expect("edge id in bounds for demo");
        if g.is_directed() {
            out_deg[src as usize] = out_deg[src as usize].saturating_add(1);
            in_deg[dst as usize] = in_deg[dst as usize].saturating_add(1);
        } else {
            out_deg[src as usize] = out_deg[src as usize].saturating_add(1);
            out_deg[dst as usize] = out_deg[dst as usize].saturating_add(1);
        }
    }
    (out_deg, in_deg)
}

fn count_self_loops(g: &Graph) -> u32 {
    let ecount = u32::try_from(g.ecount()).expect("ecount fits u32 for demo");
    let mut loops = 0u32;
    for eid in 0..ecount {
        let (src, dst) = g.edge(eid).expect("edge id in bounds for demo");
        if src == dst {
            loops += 1;
        }
    }
    loops
}

fn count_multi_edges(g: &Graph) -> u32 {
    let ecount = u32::try_from(g.ecount()).expect("ecount fits u32 for demo");
    let mut bag: HashMap<(u32, u32), u32> = HashMap::new();
    for eid in 0..ecount {
        let (src, dst) = g.edge(eid).expect("edge id in bounds for demo");
        let key = if g.is_directed() || src <= dst {
            (src, dst)
        } else {
            (dst, src)
        };
        *bag.entry(key).or_insert(0) += 1;
    }
    bag.values().map(|c| c.saturating_sub(1)).sum()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Scenario 1: uniform 4-regular undirected.
    {
        let n: usize = 200;
        let d: u32 = 4;
        let seq: Vec<u32> = vec![d; n];
        let g = degree_sequence_game_configuration(&seq, None, 0x1234_5678_u64)?;
        let (deg, _) = observed_degrees(&g);
        let loops = count_self_loops(&g);
        let multi = count_multi_edges(&g);
        println!("Scenario 1: uniform 4-regular undirected, n = {n}");
        println!(
            "  |E| = {} (expected n·d/2 = {})",
            g.ecount(),
            n * d as usize / 2
        );
        println!(
            "  degree range = [{}, {}] (expected exact {})",
            deg.iter().min().unwrap(),
            deg.iter().max().unwrap(),
            d,
        );
        println!(
            "  self-loops = {loops}, multi-edges = {multi}  (expected ≈ d/2 = {:.1} loops + a few collisions)",
            f64::from(d) / 2.0
        );
        println!();
    }

    // Scenario 2: skewed undirected (Zipf-like).
    {
        let n: usize = 100;
        let seq: Vec<u32> = (0..n)
            .map(|r| u32::try_from(n / (r + 1)).expect("term fits u32"))
            .map(|d| if d == 0 { 1 } else { d })
            .collect();
        // Ensure even sum.
        let sum: u64 = seq.iter().map(|&d| u64::from(d)).sum();
        let mut seq = seq;
        if sum % 2 != 0 {
            seq[0] = seq[0].saturating_add(1);
        }
        let g = degree_sequence_game_configuration(&seq, None, 0xBEEF_F00D_u64)?;
        let (deg, _) = observed_degrees(&g);
        let loops = count_self_loops(&g);
        let multi = count_multi_edges(&g);
        println!("Scenario 2: Zipf-like skewed undirected, n = {n}");
        println!(
            "  Σd_input = {}",
            seq.iter().map(|&d| u64::from(d)).sum::<u64>()
        );
        println!("  |E| = {}", g.ecount());
        println!(
            "  hottest vertex (v=0): input deg = {}, observed deg = {}",
            seq[0], deg[0]
        );
        println!("  self-loops = {loops}, multi-edges = {multi}  (concentrated on the hot vertex)");
        println!();
    }

    // Scenario 3: balanced directed.
    {
        let n_blocks: usize = 10;
        let out_pattern: [u32; 5] = [1, 2, 3, 4, 5];
        let mut out_seq: Vec<u32> = Vec::with_capacity(n_blocks * 5);
        for _ in 0..n_blocks {
            out_seq.extend_from_slice(&out_pattern);
        }
        let in_seq = out_seq.clone();
        let g = degree_sequence_game_configuration(&out_seq, Some(&in_seq), 0xDEAD_BEEF_u64)?;
        let (obs_out, obs_in) = observed_degrees(&g);
        println!("Scenario 3: balanced directed, n = {}", out_seq.len());
        println!(
            "  Σout = Σin = {}",
            out_seq.iter().map(|&d| u64::from(d)).sum::<u64>()
        );
        println!("  |E| = {} (expected = Σout)", g.ecount());
        println!(
            "  out-degree match = {}, in-degree match = {}",
            obs_out == out_seq,
            obs_in == in_seq
        );
        println!();
    }

    println!(
        "Multigraphs are expected: stub matching admits self-loops and multi-edges with positive probability."
    );
    println!(
        "For *simple*-graph samples from a fixed degree sequence, use the VL or EDGE_SWITCHING_SIMPLE methods (follow-up AWUs GN-025+)."
    );
    Ok(())
}
