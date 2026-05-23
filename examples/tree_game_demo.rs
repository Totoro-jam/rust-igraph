//! ALGO-GN-004 example: uniform random labelled tree via Wilson LERW.
//!
//! Wilson's loop-erased random walk on the complete graph `K_n` samples
//! spanning trees *uniformly* — every one of the `n^{n-2}` labelled trees
//! has identical probability (Cayley's formula). This demo confirms the
//! basic invariants and shows the BFS-depth distribution from the root
//! to illustrate that the sampled trees tend to be reasonably balanced
//! (depth grows as O(sqrt(n)) on average).
//!
//! Run: `cargo run --example tree_game_demo`.
//!
//! Directed mode emits an out-rooted tree at the random initial vertex.
//! Setting `directed = false` returns the same edges with undirected
//! semantics.

use std::collections::VecDeque;

use rust_igraph::tree_game_lerw;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n: u32 = 200;

    let tree = tree_game_lerw(n, true, 0x7EE0_0001)?;
    println!(
        "tree_game_lerw(n={n}, directed): {} vertices, {} edges",
        tree.vcount(),
        tree.ecount(),
    );
    assert_eq!(u32::try_from(tree.ecount())?, n - 1);

    // For a directed Wilson tree, the root is the unique vertex with
    // in-degree 0; every other vertex has in-degree exactly 1.
    let mut indeg = vec![0u32; n as usize];
    for eid in 0..u32::try_from(tree.ecount())? {
        let (_src, dst) = tree.edge(eid)?;
        indeg[dst as usize] += 1;
    }
    let roots: Vec<u32> = (0..n).filter(|&v| indeg[v as usize] == 0).collect();
    assert_eq!(roots.len(), 1);
    let root = roots[0];
    println!("  root = {root}");

    // BFS from the root over the directed edges to read off the depth
    // histogram. (The tree is out-rooted, so following outgoing edges
    // traverses parent -> child.)
    let mut depth = vec![u32::MAX; n as usize];
    depth[root as usize] = 0;
    let mut frontier = VecDeque::from([root]);
    while let Some(u) = frontier.pop_front() {
        let d = depth[u as usize] + 1;
        for v in tree.neighbors(u)? {
            if depth[v as usize] == u32::MAX {
                depth[v as usize] = d;
                frontier.push_back(v);
            }
        }
    }

    let max_depth = depth.iter().copied().max().unwrap_or(0);
    let mut hist = vec![0u32; (max_depth + 1) as usize];
    for &d in &depth {
        hist[d as usize] += 1;
    }
    let mean_depth = depth.iter().map(|&d| f64::from(d)).sum::<f64>() / f64::from(n);
    println!("  max depth = {max_depth}, mean depth = {mean_depth:.2}");
    print!("  depth histogram:");
    for (d, c) in hist.iter().enumerate() {
        print!(" {d}:{c}");
    }
    println!();

    Ok(())
}
