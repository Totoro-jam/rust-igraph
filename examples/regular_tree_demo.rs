//! ALGO-CN-006 example: regular tree / Bethe lattice
//! (`igraph_regular_tree`).
//!
//! A regular tree of height `h` and degree `k` is a rooted tree where
//! every non-leaf vertex has total degree exactly `k`:
//!
//! * the root has `k` children (its total degree is `k`),
//! * internal non-root vertices have `k - 1` children + their parent,
//!   so total degree is also `k`,
//! * leaves sit at depth `h` and have degree 1.
//!
//! Built upstream as a thin wrapper over `symmetric_tree` with
//! `branches = [k, k-1, k-1, ..., k-1]` (length `h`).
//!
//! Truth table for `regular_tree(h, k, mode)`:
//!
//! | h | k | mode        | directed | vcount | ecount | shape                |
//! |---|---|-------------|----------|--------|--------|----------------------|
//! | 1 | 3 | Out         | true     | 4      | 3      | star K1,3            |
//! | 2 | 3 | Out         | true     | 10     | 9      | Bethe `[3, 2]`       |
//! | 2 | 3 | In          | true     | 10     | 9      | Bethe (reversed)     |
//! | 2 | 3 | Undirected  | false    | 10     | 9      | undirected Bethe     |
//! | 2 | 4 | Out         | true     | 17     | 16     | Bethe `[4, 3]`       |
//! | 3 | 2 | Out         | true     | 7      | 6      | degenerate `[2,1,1]` |
//!
//! Run: `cargo run --example regular_tree_demo`.

#![allow(clippy::cast_possible_truncation)]

use rust_igraph::{Graph, TreeMode, regular_tree};

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
    // h=1, k=3 — root with 3 leaves (star K1,3).
    let star = regular_tree(1, 3, TreeMode::Out).expect("h1 k3");
    print_summary("Out  regular_tree (h=1, k=3) — star K1,3", &star);

    // h=2, k=3 — canonical small Bethe lattice (10 vertices).
    let out = regular_tree(2, 3, TreeMode::Out).expect("h2 k3 out");
    print_summary("Out  regular_tree (h=2, k=3)", &out);

    let inn = regular_tree(2, 3, TreeMode::In).expect("h2 k3 in");
    print_summary("In   regular_tree (h=2, k=3)", &inn);

    let und = regular_tree(2, 3, TreeMode::Undirected).expect("h2 k3 und");
    print_summary("Undirected regular_tree (h=2, k=3)", &und);

    // h=2, k=4 — every non-leaf has total degree 4.
    let denser = regular_tree(2, 4, TreeMode::Out).expect("h2 k4");
    print_summary("Out  regular_tree (h=2, k=4)", &denser);

    // h=3, k=2 — degenerate case (branches=[2,1,1]); 7-vertex tree.
    let chainy = regular_tree(3, 2, TreeMode::Out).expect("h3 k2");
    print_summary("Out  regular_tree (h=3, k=2) — branches=[2,1,1]", &chainy);
}
