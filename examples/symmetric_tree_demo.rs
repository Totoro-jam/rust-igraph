//! ALGO-CN-005 example: symmetric tree constructor
//! (`igraph_symmetric_tree`).
//!
//! Whereas `kary_tree` uses a single branching factor, `symmetric_tree`
//! takes a slice of branching factors and applies one per BFS depth:
//! `branches[d]` children at depth `d`. Vertex layout is BFS-ordered
//! (root = 0, then all depth-1 in arrival order, then all depth-2, …)
//! and edge direction is governed by the shared [`TreeMode`] enum.
//!
//! Truth table for `symmetric_tree(branches, mode)`:
//!
//! | branches      | mode        | directed | vcount | ecount | shape           |
//! |---------------|-------------|----------|--------|--------|-----------------|
//! | `[2, 2]`      | Out         | true     | 7      | 6      | perfect binary  |
//! | `[2, 2]`      | In          | true     | 7      | 6      | reversed arcs   |
//! | `[2, 2]`      | Undirected  | false    | 7      | 6      | canonical pairs |
//! | `[3, 2]`      | Out         | true     | 10     | 9      | 3-then-2 mixed  |
//! | `[3, 2, 1]`   | Out         | true     | 16     | 15     | 3-level mixed   |
//! | `[1, 1, 1]`   | Undirected  | false    | 4      | 3      | linear chain    |
//! | `[3]`         | Out         | true     | 4      | 3      | star K1,3       |
//! | `[]`          | Out         | true     | 1      | 0      | singleton root  |
//!
//! Degenerate cases:
//!
//! * `branches` empty → singleton root (1 vertex, 0 edges).
//! * any `branches[d] == 0` → `InvalidArgument` (zero branching collapses
//!   the next level to nothing and is rejected upstream).
//! * vertex count overflow `u32` → `InvalidArgument` (e.g. `[2; 32]`).
//!
//! Run: `cargo run --example symmetric_tree_demo`.

#![allow(clippy::cast_possible_truncation)]

use rust_igraph::{Graph, TreeMode, symmetric_tree};

fn dump_edges(g: &Graph) -> Vec<(u32, u32)> {
    let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for example");
    (0..m)
        .map(|e| g.edge(e).expect("edge id in bounds for example"))
        .collect()
}

fn print_summary(label: &str, g: &Graph) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
    println!("  edges    = {:?}", dump_edges(g));
}

fn main() {
    // Three modes on the canonical [2,2] binary tree.
    let out = symmetric_tree(&[2, 2], TreeMode::Out).expect("binary out");
    print_summary("Out  symmetric tree (branches=[2,2])", &out);

    let inn = symmetric_tree(&[2, 2], TreeMode::In).expect("binary in");
    print_summary("In   symmetric tree (branches=[2,2])", &inn);

    let und = symmetric_tree(&[2, 2], TreeMode::Undirected).expect("binary undirected");
    print_summary("Undirected symmetric tree (branches=[2,2])", &und);

    // Mixed branching: root has 3 kids, each has 2.
    let mixed = symmetric_tree(&[3, 2], TreeMode::Out).expect("3-then-2");
    print_summary("Mixed symmetric tree (branches=[3,2])", &mixed);

    // Deeper mixed branching: depth-3 tree with shrinking fan-out.
    let deep = symmetric_tree(&[3, 2, 1], TreeMode::Out).expect("3-level");
    print_summary("Deep symmetric tree (branches=[3,2,1])", &deep);

    // Linear chain: one child per level produces a path.
    let chain = symmetric_tree(&[1, 1, 1], TreeMode::Undirected).expect("chain");
    print_summary("Chain symmetric tree (branches=[1,1,1])", &chain);

    // Single-level: equivalent to a star K1,3 (root + 3 leaves).
    let star = symmetric_tree(&[3], TreeMode::Out).expect("star K1,3");
    print_summary("Star-like symmetric tree (branches=[3])", &star);

    // Degenerate: empty branches collapses to singleton root.
    let singleton = symmetric_tree(&[], TreeMode::Out).expect("singleton");
    print_summary("Singleton symmetric tree (branches=[])", &singleton);
}
