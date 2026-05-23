//! ALGO-GN-002 example: Barabási–Albert preferential-attachment
//! random graph (BAG variant).
//!
//! Each new vertex attaches `m` outgoing edges to existing vertices
//! chosen with probability proportional to their current degree. This
//! is the classical "rich-get-richer" mechanism that produces
//! scale-free degree distributions.
//!
//! Run: `cargo run --example barabasi_demo`.

use rust_igraph::barabasi_game_bag;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // n = 200 vertices, m = 2 new edges per step, directed.
    // Edge count is deterministic: (n - 1) · m = 199 · 2 = 398.
    let n: u32 = 200;
    let m: u32 = 2;
    let g = barabasi_game_bag(n, m, false, true, 0xBABA_0001)?;
    println!(
        "BA(n={}, m={}, directed): {} vertices, {} edges",
        n,
        m,
        g.vcount(),
        g.ecount(),
    );
    assert_eq!(u32::try_from(g.ecount())?, (n - 1) * m);

    // Per-vertex degree distribution — show the top-10 hubs.
    let mut deg = vec![0u32; g.vcount() as usize];
    let n_edges = u32::try_from(g.ecount())?;
    for eid in 0..n_edges {
        let (src, dst) = g.edge(eid)?;
        deg[src as usize] += 1;
        deg[dst as usize] += 1;
    }
    let mut indexed: Vec<(usize, u32)> = deg.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| b.1.cmp(&a.1));
    println!("Top-10 hubs (vertex_id: total_degree):");
    for (v, d) in indexed.iter().take(10) {
        println!("  {v:>3}: {d}");
    }

    // The undirected variant forces outpref = true.
    let g_und = barabasi_game_bag(n, m, false, false, 0xBABA_0002)?;
    println!(
        "BA(n={n}, m={m}, undirected) — outpref auto-promoted: {} edges, directed={}",
        g_und.ecount(),
        g_und.is_directed(),
    );

    Ok(())
}
