//! ALGO-GN-007 example: simple interconnected islands diagnostics.
//!
//! Builds a 6-island lattice (size 25 each, 150 vertices total) with a
//! moderate within-island density and three bipartite edges between
//! every island pair, then prints per-island vertex counts, per-island
//! edge counts, and the bipartite slice profile so the block structure
//! is visible in plain text.
//!
//! Run: `cargo run --example islands_demo`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use rust_igraph::simple_interconnected_islands_game;

fn island_of(v: u32, islands_size: u32) -> u32 {
    v / islands_size
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let islands_n: u32 = 6;
    let islands_size: u32 = 25;
    let islands_pin = 0.20;
    let n_inter: u32 = 3;

    let g = simple_interconnected_islands_game(
        islands_n,
        islands_size,
        islands_pin,
        n_inter,
        0x15_1A4D_5EED,
    )?;

    let total = islands_n * islands_size;
    println!(
        "islands lattice: islands_n = {islands_n}, islands_size = {islands_size}, \
         pin = {islands_pin}, n_inter = {n_inter}"
    );
    println!("  vertices  = {total}");
    println!("  edges     = {}", g.ecount());
    println!("  directed? = {}", g.is_directed());

    // Per-island intra-edge counts and a flat (islands_n x islands_n)
    // tally of inter-island edges. Diagonals are the intra counts.
    let mut block: Vec<Vec<u32>> = vec![vec![0u32; islands_n as usize]; islands_n as usize];
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    for eid in 0..m {
        let (a, b) = g.edge(eid).expect("edge id in bounds for example");
        let ia = island_of(a, islands_size);
        let ib = island_of(b, islands_size);
        let (lo, hi) = if ia <= ib { (ia, ib) } else { (ib, ia) };
        block[lo as usize][hi as usize] += 1;
    }

    // Per-island intra summary.
    println!("  per-island intra-edges (diagonal of the block table):");
    let max_intra = (islands_size * (islands_size - 1)) / 2;
    for i in 0..islands_n {
        let e = block[i as usize][i as usize];
        let pct = 100.0 * f64::from(e) / f64::from(max_intra);
        println!(
            "    island {i}  intra-edges = {e:>4}  (out of {max_intra} possible, {pct:>5.1}% saturated)"
        );
    }

    // Off-diagonal: every (i, j) with i < j should hold exactly n_inter
    // bipartite edges. Verifying this is one of the model's defining
    // invariants and the demo prints it so the user can eyeball it.
    println!("  bipartite slices (should all equal n_inter = {n_inter}):");
    for i in 0..islands_n {
        for j in (i + 1)..islands_n {
            let e = block[i as usize][j as usize];
            let marker = if e == n_inter { " ok " } else { "MISS" };
            println!("    [{i},{j}]  inter-edges = {e}  {marker}");
        }
    }

    Ok(())
}
