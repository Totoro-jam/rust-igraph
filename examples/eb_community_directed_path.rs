//! ALGO-CO-006c demo: directed Girvan-Newman edge-betweenness on a
//! directed 6-path 0->1->2->3->4->5. The middle directed edge (2,3) is
//! the unique maximum-betweenness edge so the dendrogram cuts it first
//! and yields the clean {0,1,2} | {3,4,5} split with directed Q = 8/25.
//!
//! Same demo through the weighted entry point with unit weights — it
//! reproduces the unweighted result via Brandes-Dijkstra, with directed
//! weighted modularity dispatched through `modularity_weighted_directed`.
//!
//! Run from the repo root: `cargo run --example eb_community_directed_path`.

#![allow(clippy::many_single_char_names)]

use rust_igraph::{
    EdgeBetweennessResult, Graph, edge_betweenness_community, edge_betweenness_community_weighted,
    modularity_directed, modularity_weighted_directed,
};

fn print_partition(label: &str, g: &Graph, r: &EdgeBetweennessResult, best_q: f64) {
    println!("{label}");
    println!("  best partition: k = {}, Q = {best_q:.6}", r.nb_clusters);
    let mut by_community: Vec<Vec<u32>> = vec![Vec::new(); r.nb_clusters as usize];
    for (v, &c) in r.membership.iter().enumerate() {
        by_community[c as usize].push(u32::try_from(v).expect("vertex id fits u32"));
    }
    for (cid, members) in by_community.iter().enumerate() {
        println!("    c{cid}: {members:?}");
    }
    let first_eid = r.removed_edges[0];
    let first_eb = r.edge_betweenness[0];
    let (u, v) = g.edge(first_eid).expect("removed edge id is valid");
    println!("  first removal: eid={first_eid} ({u}->{v})  eb = {first_eb:.4}");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = Graph::new(6, true)?;
    for i in 0..5u32 {
        g.add_edge(i, i + 1)?;
    }
    println!(
        "directed 6-path: {} vertices, {} edges, directed = {}",
        g.vcount(),
        g.ecount(),
        g.is_directed()
    );

    let r_unweighted = edge_betweenness_community(&g)?;
    let best_q_unweighted = modularity_directed(&g, &r_unweighted.membership, 1.0)?.unwrap_or(0.0);
    print_partition("unweighted:", &g, &r_unweighted, best_q_unweighted);

    let weights = vec![1.0_f64; g.ecount()];
    let r_weighted = edge_betweenness_community_weighted(&g, &weights)?;
    let best_q_weighted =
        modularity_weighted_directed(&g, &r_weighted.membership, 1.0, &weights)?.unwrap_or(0.0);
    print_partition("weighted (unit weights):", &g, &r_weighted, best_q_weighted);

    assert_eq!(r_unweighted.membership, r_weighted.membership);
    println!("\nunweighted and weighted-unit produce the same partition (as expected).");

    Ok(())
}
