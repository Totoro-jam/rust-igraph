//! ALGO-GN-003 example: growing random graph.
//!
//! The graph starts as a single seed vertex and on every step a fresh
//! vertex arrives together with `m` new edges. The endpoint kernel is
//! *uniform* — every existing vertex is equally likely to receive an
//! incoming edge. Two modes are exposed:
//!
//! * **Citation**: edges originate at the freshly-added vertex and
//!   point to a uniformly chosen earlier vertex. The resulting graph
//!   is strictly time-ordered (`dst < src` for every edge), free of
//!   self-loops, and vertex 0 never appears as a source.
//! * **Free**: both endpoints are uniformly sampled within the current
//!   frontier — closer to a uniformly random graph snapshot than to a
//!   citation network.
//!
//! Run: `cargo run --example growing_random_demo`.

use rust_igraph::growing_random_game;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n: u32 = 200;
    let m: u32 = 3;

    let g_cite = growing_random_game(n, m, true, true, 0xC1A0_0001)?;
    println!(
        "growing_random(n={n}, m={m}, directed, citation): {} vertices, {} edges",
        g_cite.vcount(),
        g_cite.ecount(),
    );
    assert_eq!(u32::try_from(g_cite.ecount())?, (n - 1) * m);

    // In citation mode every edge points back in time, so the graph is
    // a DAG. Show the in-degree distribution — uniform attachment
    // produces a roughly geometric (not power-law) tail.
    let mut indeg = vec![0u32; g_cite.vcount() as usize];
    let n_edges = u32::try_from(g_cite.ecount())?;
    for eid in 0..n_edges {
        let (_src, dst) = g_cite.edge(eid)?;
        indeg[dst as usize] += 1;
    }
    let max_in = *indeg.iter().max().unwrap_or(&0);
    let mean_in = f64::from(n_edges) / f64::from(n);
    println!("  in-degree: max={max_in}, mean={mean_in:.2}");

    // Free mode: both endpoints uniform, no temporal ordering.
    let g_free = growing_random_game(n, m, true, false, 0xC1A0_0002)?;
    let mut backward = 0u32;
    for eid in 0..u32::try_from(g_free.ecount())? {
        let (src, dst) = g_free.edge(eid)?;
        if dst < src {
            backward += 1;
        }
    }
    println!(
        "growing_random(n={n}, m={m}, directed, free): {} edges, {} pointing backwards in time",
        g_free.ecount(),
        backward,
    );

    Ok(())
}
