//! ALGO-CN-004 example: k-ary tree constructor (`igraph_kary_tree`).
//!
//! Builds a perfect binary tree on 7 vertices, a partial ternary tree on
//! 8 vertices, and a linear chain (children = 1) on 6 vertices under
//! each `TreeMode` variant where applicable. The generator emits exactly
//! `n - 1` edges in BFS order: at each `parent` index, up to `children`
//! `(parent, to)` pairs are pushed and `to` advances by one each push;
//! the last parent may emit a short batch.
//!
//! Truth table for `kary_tree(n, children, mode)`:
//!
//! | n | children | mode        | directed | ecount | first edge | last edge   |
//! |---|----------|-------------|----------|--------|------------|-------------|
//! | 7 | 2        | Out         | true     | 6      | (0, 1)     | (2, 6)      |
//! | 7 | 2        | In          | true     | 6      | (1, 0)     | (6, 2)      |
//! | 7 | 2        | Undirected  | false    | 6      | (0, 1)*    | (2, 6)*     |
//! | 8 | 3        | Undirected  | false    | 7      | (0, 1)*    | (2, 7)*     |
//! | 6 | 1        | Undirected  | false    | 5      | (0, 1)*    | (4, 5)*     |
//!
//! `*` Undirected storage canonicalises endpoints as `(min, max)`.
//!
//! Degenerate shapes:
//!
//! * `n = 0` — empty graph (no vertices, no edges).
//! * `n = 1` — singleton (one vertex, no edges).
//! * `children >= n - 1` — collapses to a star: vertex 0 connects to
//!   every other vertex.
//!
//! Run: `cargo run --example kary_tree_demo`.

#![allow(clippy::cast_possible_truncation)]

use rust_igraph::{Graph, TreeMode, kary_tree};

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
    let out = kary_tree(7, 2, TreeMode::Out).expect("binary out tree");
    print_summary("Out  binary tree (n=7, children=2)", &out);

    let inn = kary_tree(7, 2, TreeMode::In).expect("binary in tree");
    print_summary("In   binary tree (n=7, children=2)", &inn);

    let undirected = kary_tree(7, 2, TreeMode::Undirected).expect("binary undirected tree");
    print_summary("Undirected binary tree (n=7, children=2)", &undirected);

    // Partial ternary tree — the last parent emits only one child
    // (8 = 1 + 3 + 3 + 1) so the BFS sweep terminates mid-batch.
    let ternary = kary_tree(8, 3, TreeMode::Undirected).expect("partial ternary tree");
    print_summary(
        "Undirected ternary tree (n=8, children=3, partial)",
        &ternary,
    );

    // Linear chain — children = 1 produces a path.
    let chain = kary_tree(6, 1, TreeMode::Undirected).expect("chain tree");
    print_summary("Chain tree (n=6, children=1)", &chain);

    // Degenerate: children >= n - 1 collapses to a star.
    let star_like = kary_tree(5, 4, TreeMode::Undirected).expect("star-like tree");
    print_summary("Star-like tree (n=5, children=4)", &star_like);

    let singleton = kary_tree(1, 2, TreeMode::Undirected).expect("singleton");
    print_summary("Singleton tree (n=1)", &singleton);
}
