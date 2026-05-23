//! ALGO-GN-005 example: geometric random graph diagnostics.
//!
//! Drops 500 random points on the unit square and connects every pair
//! within radius `r = 0.08`. Prints the mean degree and how it tracks
//! the bulk expectation `E[deg] = (n - 1) · π · r²` (interior),
//! contrasts plane vs. torus mode (the torus has no boundary, so its
//! mean degree matches the interior prediction more closely), and
//! reports the size of the largest connected component.
//!
//! Run: `cargo run --example grg_demo`.

#![allow(clippy::cast_precision_loss)]

use rust_igraph::{Graph, connected_components, grg_game};

fn mean_degree(g: &Graph) -> f64 {
    // Sum of degrees = 2 · |E| for an undirected simple graph.
    2.0 * (g.ecount() as f64) / f64::from(g.vcount())
}

fn largest_component_size(g: &Graph) -> usize {
    let cc = connected_components(g).expect("connected_components on undirected graph");
    let n_components = cc.membership.iter().copied().max().unwrap_or(0) as usize + 1;
    let mut sizes = vec![0usize; n_components];
    for &c in &cc.membership {
        sizes[c as usize] += 1;
    }
    sizes.into_iter().max().unwrap_or(0)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n: u32 = 500;
    let r: f64 = 0.08;
    let expected_deg = f64::from(n - 1) * std::f64::consts::PI * r * r;

    let plane = grg_game(n, r, false, 0x6E0_1057)?;
    let torus = grg_game(n, r, true, 0x6E0_1057)?;

    println!("geometric random graph: n = {n}, r = {r}");
    println!("  E[deg] (interior bulk)   = {expected_deg:.2}");
    println!(
        "  plane mean degree        = {:.2} (edges = {})",
        mean_degree(&plane),
        plane.ecount(),
    );
    println!(
        "  torus mean degree        = {:.2} (edges = {})",
        mean_degree(&torus),
        torus.ecount(),
    );
    println!(
        "  largest CC: plane = {} / {n}, torus = {} / {n}",
        largest_component_size(&plane),
        largest_component_size(&torus),
    );

    Ok(())
}
