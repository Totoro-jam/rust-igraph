//! Demonstrates the method-style API on `Graph`.
//!
//! All operations below use methods directly on `Graph` instead of
//! free functions — the ergonomic API that wraps the full algorithm
//! library.
//!
//! Run: `cargo run --example method_api_demo`

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. Construction ---
    let g = rust_igraph::Graph::erdos_renyi(50, 0.1, 42)?;
    println!("=== Random Graph (Erdos-Renyi n=50, p=0.1) ===");
    println!("{g}");
    println!();

    // --- 2. Structural queries ---
    println!("Connected: {}", g.is_connected()?);
    println!(
        "Diameter:  {}",
        g.diameter()?.map_or("N/A".to_string(), |d| d.to_string())
    );
    println!(
        "Girth:     {}",
        g.girth()?.map_or("N/A".to_string(), |d| d.to_string())
    );
    println!("Triangles: {}", g.count_triangles()?);
    println!("Density:   {:.4}", g.density()?.unwrap_or(0.0));
    println!();

    // --- 3. Centrality ---
    let pr = g.pagerank()?;
    let bc = g.betweenness()?;
    let hc = g.harmonic_centrality()?;

    let mut ranked: Vec<(usize, f64)> = pr.iter().copied().enumerate().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    println!("Top-5 vertices by PageRank:");
    for (v, score) in ranked.iter().take(5) {
        println!(
            "  v{v:>2}: PR={score:.4}  betweenness={:.1}  harmonic={:.4}",
            bc[*v], hc[*v],
        );
    }
    println!();

    // --- 4. Community detection ---
    let communities = g.louvain()?;
    let k = communities
        .membership
        .iter()
        .copied()
        .max()
        .map_or(0, |m| m + 1);
    println!(
        "Louvain: {k} communities, modularity = {:.4}",
        communities.modularity
    );
    println!();

    // --- 5. Shortest paths ---
    let dists = g.distances(0)?;
    let reachable: Vec<f64> = dists.iter().filter_map(|d| d.map(f64::from)).collect();
    #[allow(clippy::cast_precision_loss)]
    let avg = reachable.iter().sum::<f64>() / reachable.len().max(1) as f64;
    println!("Avg distance from vertex 0: {avg:.2}");
    println!();

    // --- 6. Graph generators ---
    let ba = rust_igraph::Graph::barabasi_albert(100, 3, 42)?;
    println!(
        "Barabasi-Albert (n=100, m=3): {} vertices, {} edges",
        ba.vcount(),
        ba.ecount()
    );

    let ws = rust_igraph::Graph::watts_strogatz(100, 4, 0.1, 42)?;
    println!(
        "Watts-Strogatz (n=100, k=4, p=0.1): {} vertices, {} edges",
        ws.vcount(),
        ws.ecount()
    );
    println!();

    // --- 7. Isomorphism ---
    let k4_first = rust_igraph::full_graph(4, false, false)?;
    let k4_second = rust_igraph::full_graph(4, false, false)?;
    println!("K4 isomorphic to K4: {}", k4_first.isomorphic(&k4_second)?);
    println!(
        "K4 automorphisms: {}",
        k4_first.count_isomorphisms_vf2(&k4_first)?
    );

    // --- 8. Graph operations ---
    let a = rust_igraph::Graph::from_edges(&[(0, 1), (1, 2)], false, None)?;
    let b = rust_igraph::Graph::from_edges(&[(1, 2), (2, 3)], false, None)?;
    let union = a.union(&b)?;
    println!(
        "Union of two paths: {} vertices, {} edges",
        union.vcount(),
        union.ecount()
    );
    println!();

    println!("Done — method-style API demonstrated across 8 categories.");
    Ok(())
}
