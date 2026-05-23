//! ALGO-GN-009 example: Watts–Strogatz 1-D small-world diagnostics.
//!
//! Sweeps the rewire probability p across {0.0, 0.05, 0.3, 1.0} on a
//! 32-vertex ring lattice with 2 neighbours per side and prints, per
//! sample, the edge count (which is invariant under rewire) and the
//! count of "long-range" edges — edges whose endpoints are more than
//! `nei` positions apart on the ring. At p = 0 the count is 0 (pure
//! lattice); at p = 1 essentially every endpoint is rewired and the
//! count approaches the edge total. The intermediate p values trace
//! out the small-world regime where short-range structure is preserved
//! but a handful of long-range shortcuts dramatically shrink the
//! graph's diameter.
//!
//! Run: `cargo run --example watts_demo`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::similar_names
)]

use rust_igraph::{Graph, watts_strogatz_game};

fn count_long_range_edges(g: &Graph, size: u32, nei: u32) -> usize {
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    let mut count = 0usize;
    for eid in 0..m {
        let (a, b) = g.edge(eid).expect("edge id in bounds for example");
        // Ring distance: min(|a - b|, size - |a - b|).
        let diff = a.abs_diff(b);
        let ring_dist = std::cmp::min(diff, size - diff);
        if ring_dist > nei {
            count += 1;
        }
    }
    count
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let size = 32u32;
    let nei = 2u32;
    println!(
        "Watts–Strogatz small-world sweep on a 1-D ring lattice (size = {size}, nei = {nei}):"
    );
    println!();
    println!("  p      edges   long-range   ratio (long / edges)   |  interpretation");
    println!("  -----  ------  ----------  --------------------   |  ----------------");

    let seed_base: u64 = 0x0155_5717;
    let total_edges = size * nei;

    for (i, &p) in [0.0_f64, 0.05, 0.3, 1.0].iter().enumerate() {
        let seed = seed_base.wrapping_add(i as u64);
        let g = watts_strogatz_game(size, nei, p, false, false, seed)?;
        assert_eq!(g.vcount(), size);
        assert_eq!(
            g.ecount() as u32,
            total_edges,
            "rewire is endpoint-preserving"
        );

        let long_range = count_long_range_edges(&g, size, nei);
        #[allow(clippy::cast_precision_loss)]
        let ratio = (long_range as f64) / f64::from(total_edges);

        let label = match i {
            0 => "pure ring lattice (no rewires)",
            1 => "small-world: few shortcuts, regular locally",
            2 => "intermediate: shortcuts proliferate",
            3 => "fully rewired: almost a random regular-ish graph",
            _ => "",
        };

        println!(
            "  {p:>4.2}  {:>6}  {long_range:>10}  {ratio:>20.3}   |  {label}",
            g.ecount(),
        );
    }

    println!();
    println!("Note: edge count = size * nei = {total_edges} for every p — the rewire");
    println!("operation moves one endpoint of an edge but never creates or deletes");
    println!("edges. Only the *placement* of the endpoints changes with p.");
    Ok(())
}
