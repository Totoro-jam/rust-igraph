//! ALGO-CN-026 example: full multipartite graph (`igraph_full_multipartite`).
//!
//! Walks the four invariant flavours of the constructor and asserts the
//! structural facts the upstream C unit test (`igraph_full_multipartite.c`)
//! and rustdoc advertise:
//!
//! * `partitions == &[]` → empty graph, empty `types` (both directed
//!   settings).
//! * `partitions == &[n]` → `n` isolated vertices, no edges,
//!   `types == [0; n]`.
//! * `partitions == &[2, 0, 3]` → vertex 2 is skipped at the type level
//!   (the empty middle partition is preserved in the type labels).
//! * `partitions == &[2, 3, 4, 2]` → undirected `ALL` matches the
//!   directed `OUT` edge multiset under canonical (min, max), and
//!   directed `ALL` arc count equals `2 ·` directed `OUT` arc count.
//!
//! Run: `cargo run --example full_multipartite_demo`.

use rust_igraph::{Graph, MultipartiteMode, full_multipartite};
use std::collections::BTreeSet;

fn dump_edges(g: &Graph) -> Vec<(u32, u32)> {
    let m = u32::try_from(g.ecount()).expect("ecount fits u32 in example");
    (0..m)
        .map(|e| g.edge(e).expect("edge id in bounds for example"))
        .collect()
}

fn canonical_undirected(g: &Graph) -> BTreeSet<(u32, u32)> {
    dump_edges(g)
        .into_iter()
        .map(|(u, v)| if u <= v { (u, v) } else { (v, u) })
        .collect()
}

fn print_summary(label: &str, g: &Graph, types: &[u32]) {
    println!("--- {label} ---");
    println!("  vcount   = {}", g.vcount());
    println!("  ecount   = {}", g.ecount());
    println!("  directed = {}", g.is_directed());
    println!("  types    = {types:?}");
    println!("  edges    = {:?}", dump_edges(g));
}

fn main() {
    // Degenerate guards.
    let empty = full_multipartite(&[], true, MultipartiteMode::All).expect("empty ok");
    print_summary("empty partitions — directed", &empty.graph, &empty.types);
    assert_eq!(empty.graph.vcount(), 0);
    assert!(empty.types.is_empty());

    let singleton = full_multipartite(&[4], false, MultipartiteMode::All).expect("singleton ok");
    print_summary(
        "partitions=[4] — undirected",
        &singleton.graph,
        &singleton.types,
    );
    assert_eq!(singleton.graph.vcount(), 4);
    assert_eq!(singleton.graph.ecount(), 0);
    assert_eq!(singleton.types, vec![0, 0, 0, 0]);

    // Empty middle partition keeps the type label gap.
    let skipped = full_multipartite(&[2, 0, 3], true, MultipartiteMode::All).expect("ok");
    print_summary(
        "partitions=[2,0,3] — directed ALL (empty middle partition)",
        &skipped.graph,
        &skipped.types,
    );
    assert_eq!(skipped.graph.vcount(), 5);
    assert_eq!(skipped.graph.ecount(), 12); // 2·6 mutual arcs across K_{2,3}
    assert_eq!(skipped.types, vec![0, 0, 2, 2, 2]);

    // The richer K_{2,3,4,2} fixture: cross-mode consistency.
    let parts = [2u32, 3, 4, 2];
    let undirected = full_multipartite(&parts, false, MultipartiteMode::All).expect("ok");
    let out = full_multipartite(&parts, true, MultipartiteMode::Out).expect("ok");
    let in_ = full_multipartite(&parts, true, MultipartiteMode::In).expect("ok");
    let all = full_multipartite(&parts, true, MultipartiteMode::All).expect("ok");
    print_summary(
        "partitions=[2,3,4,2] — undirected ALL",
        &undirected.graph,
        &undirected.types,
    );
    println!(
        "  (directed OUT ecount = {}, directed IN ecount = {}, directed ALL ecount = {})",
        out.graph.ecount(),
        in_.graph.ecount(),
        all.graph.ecount(),
    );

    // Invariant 1: every edge connects two different partitions.
    for (u, v) in dump_edges(&undirected.graph) {
        assert_ne!(
            undirected.types[u as usize], undirected.types[v as usize],
            "edge ({u}, {v}) crosses a partition boundary"
        );
    }

    // Invariant 2: undirected canonical multiset == directed-OUT multiset.
    assert_eq!(
        canonical_undirected(&undirected.graph),
        canonical_undirected(&out.graph),
        "undirected and directed-OUT should yield the same canonical multiset"
    );

    // Invariant 3: directed ALL = 2 · directed OUT in arc count.
    assert_eq!(all.graph.ecount(), 2 * out.graph.ecount());

    // Invariant 4: types vector is partition-major (monotone non-decreasing).
    for w in all.types.windows(2) {
        assert!(w[0] <= w[1], "types must be monotone, got {:?}", all.types);
    }

    // Invariant 5: types vector length equals the vertex count.
    assert_eq!(
        u32::try_from(all.types.len()).expect("types length fits u32 in example"),
        all.graph.vcount()
    );

    println!("\nall structural invariants OK ✓");
}
