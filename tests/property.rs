//! Property-based invariants.
//!
//! Run with: `cargo test --features proptest-harness --test property`.
//! Without the feature this file is empty so plain `cargo test` stays fast.

#![cfg(feature = "proptest-harness")]

use proptest::prelude::*;
use rust_igraph::Graph;

/// Build an arbitrary undirected graph on `0..n` vertices with each candidate
/// edge appearing independently with probability ~0.3.
fn arb_graph(max_n: u32) -> impl Strategy<Value = Graph> {
    (1u32..=max_n)
        .prop_flat_map(|n| {
            let edges = proptest::collection::vec((0u32..n, 0u32..n), 0..=(n as usize * 2));
            (Just(n), edges)
        })
        .prop_map(|(n, edges)| {
            let mut g = Graph::with_vertices(n);
            for (u, v) in edges {
                g.add_edge(u, v).expect("indices in range");
            }
            g
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// BFS visits each vertex at most once, and every visited vertex is in range.
    #[test]
    fn bfs_visits_each_vertex_at_most_once(g in arb_graph(20)) {
        let order = rust_igraph::bfs(&g, 0).expect("root 0 is valid for n>=1");
        let mut sorted = order.clone();
        sorted.sort_unstable();
        sorted.dedup();
        prop_assert_eq!(sorted.len(), order.len(), "duplicate vertex in BFS order");
        for &v in &order {
            prop_assert!(v < g.vcount(), "BFS produced an out-of-range vertex");
        }
    }

    /// BFS-reachable set from `root` is symmetric on undirected graphs:
    /// `v` reachable from `0` ⇔ `0` reachable from `v`.
    #[test]
    fn bfs_reachability_is_symmetric(g in arb_graph(15)) {
        let from_zero: std::collections::HashSet<u32> =
            rust_igraph::bfs(&g, 0).unwrap().into_iter().collect();
        for v in 1..g.vcount() {
            let from_v: std::collections::HashSet<u32> =
                rust_igraph::bfs(&g, v).unwrap().into_iter().collect();
            let v_in_zero = from_zero.contains(&v);
            let zero_in_v = from_v.contains(&0);
            prop_assert_eq!(
                v_in_zero, zero_in_v,
                "asymmetric reachability between 0 and {}", v
            );
        }
    }
}
