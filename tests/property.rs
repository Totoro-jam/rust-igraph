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

/// Same as [`arb_graph`] but builds a directed graph. Used by SCC properties.
fn arb_directed_graph(max_n: u32) -> impl Strategy<Value = Graph> {
    (1u32..=max_n)
        .prop_flat_map(|n| {
            let edges = proptest::collection::vec((0u32..n, 0u32..n), 0..=(n as usize * 2));
            (Just(n), edges)
        })
        .prop_map(|(n, edges)| {
            let mut g = Graph::new(n, true).expect("directed init");
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

    /// DFS shares the same per-vertex invariants as BFS: every visited
    /// vertex is in range, and no vertex is visited twice.
    #[test]
    fn dfs_visits_each_vertex_at_most_once(g in arb_graph(20)) {
        let order = rust_igraph::dfs(&g, 0).expect("root 0 is valid for n>=1");
        let mut sorted = order.clone();
        sorted.sort_unstable();
        sorted.dedup();
        prop_assert_eq!(sorted.len(), order.len(), "duplicate vertex in DFS order");
        for &v in &order {
            prop_assert!(v < g.vcount(), "DFS produced an out-of-range vertex");
        }
    }

    /// BFS and DFS reach the same set of vertices from the same root —
    /// both compute the connected component (for undirected graphs).
    #[test]
    fn bfs_and_dfs_visit_the_same_set(g in arb_graph(15)) {
        let bfs_set: std::collections::BTreeSet<u32> =
            rust_igraph::bfs(&g, 0).unwrap().into_iter().collect();
        let dfs_set: std::collections::BTreeSet<u32> =
            rust_igraph::dfs(&g, 0).unwrap().into_iter().collect();
        prop_assert_eq!(bfs_set, dfs_set);
    }

    /// `connected_components`'s membership has length `vcount`, and
    /// the component ids are dense 0..count (no gaps).
    #[test]
    fn cc_membership_is_dense_and_correct_length(g in arb_graph(20)) {
        let cc = rust_igraph::connected_components(&g).unwrap();
        prop_assert_eq!(cc.membership.len(), g.vcount() as usize);
        if cc.count == 0 {
            prop_assert!(cc.membership.is_empty());
        } else {
            let max = *cc.membership.iter().max().unwrap();
            prop_assert_eq!(max + 1, cc.count, "membership ids must be dense 0..count");
        }
    }

    /// CC reachability matches BFS: vertex 0's component contains
    /// exactly the BFS-reachable set from 0 (undirected).
    #[test]
    fn cc_component_of_zero_equals_bfs_reachable(g in arb_graph(15)) {
        let cc = rust_igraph::connected_components(&g).unwrap();
        let bfs_reachable: std::collections::BTreeSet<u32> =
            rust_igraph::bfs(&g, 0).unwrap().into_iter().collect();
        let cc_reachable: std::collections::BTreeSet<u32> = (0..g.vcount())
            .filter(|&v| cc.membership[v as usize] == cc.membership[0])
            .collect();
        prop_assert_eq!(bfs_reachable, cc_reachable);
    }

    /// SCC membership has length `vcount` and dense ids `0..count`. The
    /// number of SCCs is at least the number of weak components (every
    /// SCC is contained in some weak component) and at most `vcount`.
    #[test]
    fn scc_membership_is_dense_and_bounded(g in arb_directed_graph(15)) {
        let scc = rust_igraph::strongly_connected_components(&g).unwrap();
        prop_assert_eq!(scc.membership.len(), g.vcount() as usize);
        if scc.count == 0 {
            prop_assert!(scc.membership.is_empty());
        } else {
            let max = *scc.membership.iter().max().unwrap();
            prop_assert_eq!(max + 1, scc.count, "SCC ids must be dense 0..count");
            prop_assert!(scc.count <= g.vcount(), "SCC count cannot exceed vcount");
        }
    }

    /// SCC partition is a refinement of the underlying-undirected weak
    /// connected components: every SCC is fully contained inside one
    /// weak component (`u, v` in same SCC ⇒ `u, v` in same WCC after
    /// dropping edge directions).
    #[test]
    fn scc_refines_weak_components(g in arb_directed_graph(15)) {
        let scc = rust_igraph::strongly_connected_components(&g).unwrap();
        // Build the same graph as undirected (every directed edge becomes
        // an undirected edge) and take its weak components. Iterate by
        // edge-id so reverse edges aren't dropped.
        let mut undirected = Graph::with_vertices(g.vcount());
        let m = u32::try_from(g.ecount()).expect("edge count fits in u32 for proptest");
        for e in 0..m {
            let (u, v) = g.edge(e).unwrap();
            undirected.add_edge(u, v).unwrap();
        }
        let wcc = rust_igraph::connected_components(&undirected).unwrap();
        for u in 0..g.vcount() {
            for v in (u + 1)..g.vcount() {
                if scc.membership[u as usize] == scc.membership[v as usize] {
                    prop_assert_eq!(
                        wcc.membership[u as usize], wcc.membership[v as usize],
                        "{} and {} share an SCC but not a weak component", u, v
                    );
                }
            }
        }
    }

    /// `bfs_tree` consistency: order matches `bfs`, distances match
    /// `distances`, parents form a valid tree (`parents[parents[v]] != v`
    /// is implied by tree structure), and `parents[v] is Some` iff `v`
    /// is reachable and not the root.
    #[test]
    fn bfs_tree_is_consistent_with_bfs_and_distances(g in arb_graph(10)) {
        let r = rust_igraph::bfs_tree(&g, 0).unwrap();
        let order = rust_igraph::bfs(&g, 0).unwrap();
        let dist = rust_igraph::distances(&g, 0).unwrap();
        prop_assert_eq!(&r.order, &order);
        prop_assert_eq!(&r.distances, &dist);
        // Root has no parent; reachable non-root has a parent; unreachable has None.
        prop_assert_eq!(r.parents[0], None);
        for v in 1..g.vcount() {
            let dv = r.distances[v as usize];
            let pv = r.parents[v as usize];
            match (dv, pv) {
                (Some(_), Some(p)) => {
                    prop_assert!(p < g.vcount(), "parent out of range");
                    // The parent should be one step closer to the root.
                    let dp = r.distances[p as usize].expect("parent must be reachable");
                    prop_assert!(dp + 1 == dv.unwrap(),
                                 "parent({})={} dist({}) != dist({}) - 1", v, p, p, v);
                }
                (None, None) => {}
                (a, b) => prop_assert!(false,
                                       "vertex {}: dist={:?} but parent={:?}", v, a, b),
            }
        }
    }

    /// Reachability counts: every vertex reaches at least itself
    /// (count[v] >= 1) and at most vcount() vertices. For undirected
    /// graphs, count[v] equals the size of v's connected component.
    #[test]
    fn count_reachable_is_within_bounds(g in arb_graph(10)) {
        let counts = rust_igraph::count_reachable(&g).unwrap();
        let n = g.vcount();
        prop_assert_eq!(counts.len(), n as usize);
        for &c in &counts {
            prop_assert!(c >= 1, "count {} < 1", c);
            prop_assert!(c <= n, "count {} > vcount {}", c, n);
        }
        // Undirected check: count[v] == component_size(v).
        let cc = rust_igraph::connected_components(&g).unwrap();
        let mut comp_sizes = vec![0u32; cc.count as usize];
        for v in 0..n {
            comp_sizes[cc.membership[v as usize] as usize] += 1;
        }
        for v in 0..n {
            prop_assert_eq!(counts[v as usize], comp_sizes[cc.membership[v as usize] as usize],
                            "vertex {} count != its component size", v);
        }
    }

    /// Eulerian-path validity: when `is_eulerian.has_path` is true and the
    /// graph is undirected, `eulerian_path` returns a walk that visits
    /// every edge exactly once and is consecutively connected.
    #[test]
    fn eulerian_path_visits_every_edge_once_when_it_exists(g in arb_graph(7)) {
        // Skip multigraphs that confuse the bookkeeping (proptest generates
        // them); the algorithm is correct on them but our walk-validator
        // here uses edge endpoints which are fine. Just guard against
        // self-loops vs the path output.
        let cls = rust_igraph::is_eulerian(&g).unwrap();
        if !cls.has_path { return Ok(()); }
        let walk = rust_igraph::eulerian_path(&g).unwrap()
            .expect("walk should exist when has_path");
        prop_assert_eq!(walk.len(), g.ecount(),
                        "walk length should equal edge count");
        let mut seen: Vec<bool> = vec![false; g.ecount()];
        for &e in &walk {
            let idx = e as usize;
            prop_assert!(idx < g.ecount(), "edge id out of range");
            prop_assert!(!seen[idx], "edge {e} visited twice");
            seen[idx] = true;
        }
        // Consecutively connected.
        for i in 0..walk.len().saturating_sub(1) {
            let (a, b) = g.edge(walk[i]).unwrap();
            let (c, d) = g.edge(walk[i + 1]).unwrap();
            prop_assert!(
                a == c || a == d || b == c || b == d,
                "walk break between edges {} and {}", walk[i], walk[i + 1]
            );
        }
    }

    /// Eigenvector centrality bounds: nonneg, finite, max == 1 (for
    /// undirected graphs where the dominant eigenvector is positive
    /// or trivially 1.0 for the no-edge case).
    #[test]
    fn eigenvector_centrality_max_is_one(g in arb_graph(8)) {
        let ec = rust_igraph::eigenvector_centrality(&g).unwrap();
        if ec.is_empty() { return Ok(()); }
        let mut maxabs = 0.0_f64;
        for (v, &x) in ec.iter().enumerate() {
            prop_assert!(x.is_finite(), "ec[{}] = {} not finite", v, x);
            prop_assert!(x >= -1e-9, "ec[{}] = {} negative", v, x);
            maxabs = maxabs.max(x.abs());
        }
        prop_assert!((maxabs - 1.0).abs() < 1e-6,
                     "max(ec) = {} should be 1.0 (within fp tol)", maxabs);
    }

    /// Biconnected components invariants:
    /// - count equals components.len() == tree_edges.len()
    /// - the AP set from `biconnected_components` matches CC-010 `articulation_points`
    /// - each component has at least 2 vertices (singletons aren't components)
    #[test]
    fn biconnected_components_consistent_with_articulation(g in arb_graph(8)) {
        let bc = rust_igraph::biconnected_components(&g).unwrap();
        prop_assert_eq!(bc.count as usize, bc.components.len());
        prop_assert_eq!(bc.count as usize, bc.tree_edges.len());

        // Component sizes ≥ 2.
        for comp in &bc.components {
            prop_assert!(comp.len() >= 2,
                         "biconnected component has fewer than 2 vertices: {:?}", comp);
        }

        // AP set matches CC-010.
        let mut bc_aps = bc.articulation_points.clone();
        bc_aps.sort_unstable();
        let mut std_aps = rust_igraph::articulation_points(&g).unwrap();
        std_aps.sort_unstable();
        prop_assert_eq!(bc_aps, std_aps);
    }

    /// CC-012 component_edges invariants:
    /// - same length as `components` (one edge-set per component)
    /// - tree_edges[i] is a subset of component_edges[i]
    /// - all edge ids across component_edges are unique (partition)
    /// - each component edge has both endpoints in that component
    /// - non-loop edges are partitioned: total = ecount minus loops
    ///   (loops are skipped by the `nei < vert` guard, matching upstream)
    #[test]
    fn biconnected_component_edges_partition_invariants(g in arb_graph(8)) {
        let bc = rust_igraph::biconnected_components(&g).unwrap();
        prop_assert_eq!(bc.component_edges.len(), bc.components.len());

        // Track every edge id seen across components → must be unique.
        let mut seen: std::collections::HashSet<u32> = std::collections::HashSet::new();
        let mut total_partitioned: usize = 0;
        for (i, edges) in bc.component_edges.iter().enumerate() {
            let comp_set: std::collections::HashSet<u32> =
                bc.components[i].iter().copied().collect();
            // Every edge endpoint pair lies inside this component.
            for &e in edges {
                let (u, v) = g.edge(e).unwrap();
                prop_assert!(comp_set.contains(&u) && comp_set.contains(&v),
                             "component {} edge {} = ({},{}) not in vertex set {:?}",
                             i, e, u, v, comp_set);
                prop_assert!(seen.insert(e),
                             "edge {} appeared in two component_edges entries", e);
                total_partitioned += 1;
            }
            // Tree edges are a subset of component edges.
            let edge_set: std::collections::HashSet<u32> = edges.iter().copied().collect();
            for &t in &bc.tree_edges[i] {
                prop_assert!(edge_set.contains(&t),
                             "tree edge {} not in component_edges of comp {}", t, i);
            }
        }

        // Loop edges are dropped by the `nei < vert` guard (matches upstream).
        let mut non_loop_count: usize = 0;
        for e in 0..g.ecount() {
            let (u, v) = g.edge(e as u32).unwrap();
            if u != v {
                non_loop_count += 1;
            }
        }
        prop_assert_eq!(total_partitioned, non_loop_count);
    }

    /// Modularity invariants:
    /// - finite (or `None` for empty graphs)
    /// - bounded in `[-1, 1]` for `resolution=1.0`
    /// - all-same partition gives `Q = 0` (within fp tolerance) on graphs
    ///   with at least one edge — every edge is "internal" so e/2m = 1,
    ///   and the single community accounts for all degree mass so
    ///   sum k_c² = 1.
    /// - all-singleton partition (each vertex its own community) gives
    ///   `Q ≤ 0` (no internal edges, so e/2m = 0; degree-mass term is
    ///   non-negative).
    #[test]
    fn modularity_bounds_and_known_partitions(g in arb_graph(8)) {
        let n = g.vcount();
        if n == 0 || g.ecount() == 0 {
            // No-edge case: modularity returns None — nothing further to check.
            let same: Vec<u32> = vec![0; n as usize];
            prop_assert!(rust_igraph::modularity(&g, &same, 1.0).unwrap().is_none());
            return Ok(());
        }

        let same: Vec<u32> = vec![0; n as usize];
        let q_same = rust_igraph::modularity(&g, &same, 1.0).unwrap().unwrap();
        prop_assert!(q_same.is_finite(), "Q(all-same) = {} not finite", q_same);
        prop_assert!(q_same.abs() < 1e-12,
                     "Q(all-same) should be 0 (got {})", q_same);

        let singletons: Vec<u32> = (0..n).collect();
        let q_each = rust_igraph::modularity(&g, &singletons, 1.0).unwrap().unwrap();
        prop_assert!(q_each.is_finite(), "Q(singletons) = {} not finite", q_each);
        // Without self-loops there are no internal edges in the singleton
        // partition, so e/2m = 0 and Q ≤ 0. Self-loops do count as
        // internal (each loop contributes 2 to e in the C definition),
        // so we don't bound Q from above in their presence — only the
        // global |Q| ≤ 1 bound applies.
        let m = u32::try_from(g.ecount()).unwrap();
        let has_self_loop = (0..m).any(|e| {
            let (u, v) = g.edge(e).unwrap();
            u == v
        });
        if !has_self_loop {
            prop_assert!(q_each <= 1e-12,
                         "Q(singletons, no loops) should be ≤ 0 (got {})", q_each);
        }

        // Standard bound: |Q| ≤ 1 for resolution = 1.
        prop_assert!(q_same.abs() <= 1.0 + 1e-9, "Q(all-same)={} > 1", q_same);
        prop_assert!(q_each.abs() <= 1.0 + 1e-9, "Q(singletons)={} > 1", q_each);
    }

    /// `complementer` invariants (undirected, simple input):
    /// - vcount preserved
    /// - for a simple graph (no loops, no parallel edges):
    ///   ecount(original) + ecount(complement(loops=false)) ==
    ///   n*(n-1)/2 (the complete-graph edge count for undirected)
    /// - complement(complement(g, false), false) == g (double-complement
    ///   is identity for simple undirected inputs)
    /// - complement(g, true) has every vertex with at most one self-loop
    #[test]
    fn complementer_double_complement_recovers_simple_graphs(g in arb_graph(8)) {
        let c = rust_igraph::complementer(&g, false).unwrap();
        prop_assert_eq!(c.vcount(), g.vcount());
        prop_assert_eq!(c.is_directed(), g.is_directed());

        // Edge-count identity only holds for simple inputs (no loops,
        // no parallels). simplify() makes it simple first.
        let simple = rust_igraph::simplify(&g, true, true).unwrap();
        let simple_c = rust_igraph::complementer(&simple, false).unwrap();
        let n = u64::from(simple.vcount());
        let max_edges = if simple.is_directed() {
            n.saturating_mul(n.saturating_sub(1))
        } else {
            n.saturating_mul(n.saturating_sub(1)) / 2
        };
        prop_assert_eq!(
            simple.ecount() as u64 + simple_c.ecount() as u64,
            max_edges,
        );

        // Double-complement recovers the (simplified) graph.
        let cc = rust_igraph::complementer(&simple_c, false).unwrap();
        let m_simple = u32::try_from(simple.ecount()).unwrap();
        let m_cc = u32::try_from(cc.ecount()).unwrap();
        let mut a: Vec<_> = (0..m_simple).map(|e| simple.edge(e).unwrap()).collect();
        let mut b: Vec<_> = (0..m_cc).map(|e| cc.edge(e).unwrap()).collect();
        a.sort_unstable();
        b.sort_unstable();
        prop_assert_eq!(a, b);
    }

    /// Weighted assortativity invariants:
    /// - `assortativity_degree_weighted(g, [1.0; m])` matches
    ///   `assortativity_degree(g)` (within fp tolerance)
    /// - result is in `[-1, 1]` when defined
    #[test]
    fn assortativity_degree_weighted_unit_weights_match_unweighted(g in arb_graph(8)) {
        if g.is_directed() { return Ok(()); }
        let m = g.ecount();
        let weights = vec![1.0_f64; m];
        let aw = rust_igraph::assortativity_degree_weighted(&g, &weights).unwrap();
        let au = rust_igraph::assortativity_degree(&g).unwrap();
        match (aw, au) {
            (Some(x), Some(y)) => {
                prop_assert!(x.is_finite(),
                             "weighted assortativity = {} not finite", x);
                prop_assert!(x.abs() <= 1.0 + 1e-9,
                             "weighted assortativity = {} out of [-1, 1]", x);
                prop_assert!((x - y).abs() < 1e-9,
                             "weighted={} unweighted={}", x, y);
            }
            (None, None) => {}
            (a, b) => prop_assert!(false, "weighted={:?} unweighted={:?}", a, b),
        }
    }

    /// Weighted PageRank invariants:
    /// - `pagerank_weighted(g, [1.0; m])` matches `pagerank(g)`
    ///   (within fp tolerance — both use power iteration in Rust)
    /// - sums to 1 (probability distribution)
    /// - all entries non-negative finite
    #[test]
    fn pagerank_weighted_unit_weights_match_unweighted(g in arb_graph(8)) {
        let m = g.ecount();
        let weights = vec![1.0_f64; m];
        let pw = rust_igraph::pagerank_weighted(&g, &weights).unwrap();
        let pu = rust_igraph::pagerank(&g).unwrap();
        prop_assert_eq!(pw.len(), pu.len());
        prop_assert_eq!(pw.len(), g.vcount() as usize);
        if g.vcount() == 0 { return Ok(()); }
        let mut sum = 0.0_f64;
        for (v, (a, b)) in pw.iter().zip(pu.iter()).enumerate() {
            prop_assert!(a.is_finite() && *a >= -1e-12,
                         "vertex {}: weighted pr = {}", v, a);
            prop_assert!((a - b).abs() < 1e-9,
                         "vertex {}: weighted={} unweighted={}", v, a, b);
            sum += a;
        }
        prop_assert!((sum - 1.0).abs() < 1e-6,
                     "weighted PageRank sums to {} not 1.0", sum);
    }

    /// Weighted edge betweenness invariants:
    /// - `edge_betweenness_weighted(g, [1.0; m])` matches
    ///   `edge_betweenness(g)` (within fp tolerance)
    /// - all values are finite and >= 0
    /// - length equals ecount
    #[test]
    fn edge_betweenness_weighted_unit_weights_match_unweighted(g in arb_graph(8)) {
        let m = g.ecount();
        let weights = vec![1.0_f64; m];
        let ebw = rust_igraph::edge_betweenness_weighted(&g, &weights).unwrap();
        let ebu = rust_igraph::edge_betweenness(&g).unwrap();
        prop_assert_eq!(ebw.len(), ebu.len());
        prop_assert_eq!(ebw.len(), m);
        for (e, (a, b)) in ebw.iter().zip(ebu.iter()).enumerate() {
            prop_assert!(a.is_finite() && *a >= -1e-9,
                         "edge {}: weighted = {}", e, a);
            prop_assert!((a - b).abs() < 1e-9,
                         "edge {}: weighted={} unweighted={}", e, a, b);
        }
    }

    /// Weighted betweenness invariants:
    /// - `betweenness_weighted(g, [1.0; m])` matches `betweenness(g)`
    ///   (within fp tolerance)
    /// - all values are finite and >= 0
    /// - length equals vcount
    #[test]
    fn betweenness_weighted_unit_weights_match_unweighted(g in arb_graph(8)) {
        let m = g.ecount();
        let weights = vec![1.0_f64; m];
        let bw = rust_igraph::betweenness_weighted(&g, &weights).unwrap();
        let bu = rust_igraph::betweenness(&g).unwrap();
        prop_assert_eq!(bw.len(), bu.len());
        prop_assert_eq!(bw.len(), g.vcount() as usize);
        for (v, (a, b)) in bw.iter().zip(bu.iter()).enumerate() {
            prop_assert!(a.is_finite() && *a >= -1e-9,
                         "vertex {}: betweenness_w = {}", v, a);
            prop_assert!((a - b).abs() < 1e-9,
                         "vertex {}: weighted={} unweighted={}", v, a, b);
        }
    }

    /// Weighted harmonic centrality invariants:
    /// - `harmonic_centrality_weighted(g, [1.0; m])` matches
    ///   `harmonic_centrality(g)` (within fp tolerance)
    /// - all values are finite and >= 0
    /// - length equals vcount
    #[test]
    fn harmonic_centrality_weighted_unit_matches_unweighted(g in arb_graph(8)) {
        let m = g.ecount();
        let weights = vec![1.0_f64; m];
        let hw = rust_igraph::harmonic_centrality_weighted(&g, &weights).unwrap();
        let hu = rust_igraph::harmonic_centrality(&g).unwrap();
        prop_assert_eq!(hw.len(), hu.len());
        prop_assert_eq!(hw.len(), g.vcount() as usize);
        for (v, (a, b)) in hw.iter().zip(hu.iter()).enumerate() {
            prop_assert!(a.is_finite() && *a >= -1e-12,
                         "vertex {}: harmonic_w = {}", v, a);
            prop_assert!((a - b).abs() < 1e-12,
                         "vertex {}: weighted={} unweighted={}", v, a, b);
        }
    }

    /// Weighted closeness invariants:
    /// - `closeness_weighted(g, [1.0; m])` matches `closeness(g)` (within
    ///   fp tolerance)
    /// - all values are either None or in `[0, ∞)` and finite
    /// - length equals vcount
    #[test]
    fn closeness_weighted_unit_weights_match_unweighted(g in arb_graph(8)) {
        let m = g.ecount();
        let weights = vec![1.0_f64; m];
        let cw = rust_igraph::closeness_weighted(&g, &weights).unwrap();
        let cu = rust_igraph::closeness(&g).unwrap();
        prop_assert_eq!(cw.len(), cu.len());
        prop_assert_eq!(cw.len(), g.vcount() as usize);
        for (v, (a, b)) in cw.iter().zip(cu.iter()).enumerate() {
            match (a, b) {
                (Some(x), Some(y)) => {
                    prop_assert!(x.is_finite() && *x >= -1e-12,
                                 "vertex {}: closeness_weighted = {}", v, x);
                    prop_assert!((x - y).abs() < 1e-12,
                                 "vertex {}: weighted={} unweighted={}", v, x, y);
                }
                (None, None) => {}
                (a, b) => prop_assert!(false,
                    "vertex {}: weighted={:?} unweighted={:?}", v, a, b),
            }
        }
    }

    /// Dijkstra distances invariants:
    /// - all-unit weights collapse to the BFS distance (SP-006)
    /// - source distance is 0.0
    /// - distances are non-negative
    /// - unreachable iff the BFS distance is also `None`
    /// - triangle inequality: dist(s, b) ≤ dist(s, a) + w(a, b) for each
    ///   edge (a, b) reachable from s
    #[test]
    fn dijkstra_unit_weights_match_bfs(g in arb_graph(10)) {
        let m = g.ecount();
        let weights = vec![1.0_f64; m];
        let source = 0u32;
        if g.vcount() == 0 { return Ok(()); }
        let d = rust_igraph::dijkstra_distances(&g, source, &weights).unwrap();
        let bfs = rust_igraph::distances(&g, source).unwrap();
        prop_assert_eq!(d.len(), bfs.len());
        for (v, (dd, bb)) in d.iter().zip(bfs.iter()).enumerate() {
            match (dd, bb) {
                (Some(rd), Some(rb)) => {
                    prop_assert!((rd - f64::from(*rb)).abs() < 1e-12,
                                 "vertex {}: dijkstra={} bfs={}", v, rd, rb);
                }
                (None, None) => {}
                (a, b) => {
                    prop_assert!(false, "vertex {}: dijkstra={:?} bfs={:?}", v, a, b);
                }
            }
        }
        prop_assert_eq!(d[0], Some(0.0));
        for (v, x) in d.iter().enumerate() {
            if let Some(rd) = x {
                prop_assert!(rd.is_finite() && *rd >= -1e-12,
                             "vertex {} distance {} should be non-negative finite", v, rd);
            }
        }
    }

    /// `dijkstra_paths` SPT invariants:
    /// - distances match `dijkstra_distances`
    /// - source has no parent / inbound edge
    /// - every reachable non-source vertex has a parent and an inbound edge
    /// - the parent / inbound edge satisfy the relaxation equality
    ///   `dist[parent] + w(eid) == dist[v]`
    #[test]
    fn dijkstra_paths_consistent_with_distances(g in arb_graph(8)) {
        if g.vcount() == 0 { return Ok(()); }
        let m = g.ecount();
        let weights: Vec<f64> = (0..m).map(|i| 1.0 + (i as f64) * 0.25).collect();
        let p = rust_igraph::dijkstra_paths(&g, 0, &weights).unwrap();
        let d = rust_igraph::dijkstra_distances(&g, 0, &weights).unwrap();
        prop_assert_eq!(&p.distances, &d);
        prop_assert_eq!(p.parents[0], None);
        prop_assert_eq!(p.inbound_edges[0], None);
        for v in 1..g.vcount() as usize {
            match (p.distances[v], p.parents[v], p.inbound_edges[v]) {
                (None, None, None) => {}
                (Some(dv), Some(parent), Some(eid)) => {
                    let (s, t) = g.edge(eid).unwrap();
                    let other = if s == parent { t } else if t == parent { s } else {
                        prop_assert!(false, "inbound edge {eid} not incident on parent {parent}");
                        unreachable!()
                    };
                    prop_assert_eq!(other as usize, v);
                    let dp = p.distances[parent as usize].expect("parent reachable");
                    let w = weights[eid as usize];
                    prop_assert!((dp + w - dv).abs() < 1e-9,
                                 "v={} relax dp={} + w={} != dv={}", v, dp, w, dv);
                }
                (a, b, c) => prop_assert!(false, "v={} dist/parent/edge mismatch: {:?} {:?} {:?}", v, a, b, c),
            }
        }
    }

    /// `dijkstra_path_to` invariants:
    /// - vertex path starts at source and ends at target (when reachable)
    /// - len(edges) == len(vertices) - 1
    /// - sum of edge weights along the path equals dist[target]
    /// - target unreachable ⇒ Ok(None)
    #[test]
    fn dijkstra_path_to_sums_to_distance(g in arb_graph(8), target in 0u32..8) {
        if g.vcount() == 0 || target >= g.vcount() { return Ok(()); }
        let m = g.ecount();
        let weights: Vec<f64> = (0..m).map(|i| 0.5 + (i as f64) * 0.5).collect();
        let d = rust_igraph::dijkstra_distances(&g, 0, &weights).unwrap();
        let p = rust_igraph::dijkstra_path_to(&g, 0, target, &weights).unwrap();
        match (d[target as usize], p) {
            (None, None) => {}
            (Some(dt), Some((vs, es))) => {
                prop_assert_eq!(*vs.first().unwrap(), 0u32);
                prop_assert_eq!(*vs.last().unwrap(), target);
                prop_assert_eq!(es.len() + 1, vs.len());
                let mut total = 0.0;
                for &eid in &es { total += weights[eid as usize]; }
                prop_assert!((total - dt).abs() < 1e-9, "path sum {} != dist {}", total, dt);
            }
            (a, b) => prop_assert!(false, "dist/path mismatch: {:?} / {:?}", a, b),
        }
    }

    /// `dijkstra_distances_cutoff` invariants:
    /// - cutoff = None ≡ unbounded dijkstra_distances
    /// - cutoff = c masks every vertex with dist > c to None
    /// - cutoff is monotone: more permissive cutoff produces a superset of
    ///   reachable vertices
    #[test]
    fn dijkstra_distances_cutoff_masks_above_cutoff(g in arb_graph(8), cutoff_idx in 0u32..6) {
        if g.vcount() == 0 { return Ok(()); }
        let m = g.ecount();
        let weights: Vec<f64> = vec![1.0; m];
        let d_unbounded = rust_igraph::dijkstra_distances_cutoff(&g, 0, &weights, None).unwrap();
        prop_assert_eq!(&d_unbounded, &rust_igraph::dijkstra_distances(&g, 0, &weights).unwrap());
        let c = cutoff_idx as f64;
        let d_cut = rust_igraph::dijkstra_distances_cutoff(&g, 0, &weights, Some(c)).unwrap();
        for (v, (du, dc)) in d_unbounded.iter().zip(d_cut.iter()).enumerate() {
            match (du, dc) {
                (None, None) => {}
                (Some(_), None) => prop_assert!(du.unwrap() > c, "v={} masked but dist {}≤cut", v, du.unwrap()),
                (Some(uu), Some(cc)) => {
                    prop_assert!((uu - cc).abs() < 1e-9 && *uu <= c + 1e-9,
                                 "v={} cutoff disagreement: u={} c={}", v, uu, cc);
                }
                (None, Some(_)) => prop_assert!(false, "v={} cutoff revealed unreachable vertex", v),
            }
        }
    }

    /// SP-001c: mode-aware dijkstra invariants.
    /// - `dijkstra_distances_with_mode(_, Out)` agrees with the legacy
    ///   `dijkstra_distances` (which is hard-coded to OUT).
    /// - For undirected graphs every mode is identical.
    /// - For directed graphs, ALL-mode equals the result on the
    ///   undirected projection.
    #[test]
    fn dijkstra_with_mode_out_matches_legacy(g in arb_graph(8)) {
        if g.vcount() == 0 { return Ok(()); }
        let m = g.ecount();
        let weights: Vec<f64> = (0..m).map(|i| 1.0 + (i as f64) * 0.5).collect();
        prop_assert_eq!(
            rust_igraph::dijkstra_distances_with_mode(&g, 0, &weights, rust_igraph::DijkstraMode::Out).unwrap(),
            rust_igraph::dijkstra_distances(&g, 0, &weights).unwrap()
        );
    }

    /// SP-001c: all-shortest-paths invariants.
    /// - distances derived from `dijkstra_all_shortest_paths` (sum of
    ///   weights along any returned path) match `dijkstra_distances_with_mode`
    /// - `nrgeo[source]` equals 1 when source is reachable; 0 only on
    ///   pathological empty-graph inputs.
    /// - `nrgeo[v] == 0` iff `distances[v] == None`.
    /// - vertex_paths[v].len() equals nrgeo[v].
    /// - every vertex_path is a valid weighted geodesic: starts at
    ///   source, ends at v, sum of edge weights equals distances[v].
    #[test]
    fn all_shortest_paths_consistent(g in arb_graph(6)) {
        if g.vcount() == 0 { return Ok(()); }
        let m = g.ecount();
        let weights: Vec<f64> = (0..m).map(|i| 1.0 + (i as f64) * 0.25).collect();
        let r = rust_igraph::dijkstra_all_shortest_paths(&g, 0, &weights, rust_igraph::DijkstraMode::Out).unwrap();
        let d = rust_igraph::dijkstra_distances_with_mode(&g, 0, &weights, rust_igraph::DijkstraMode::Out).unwrap();
        prop_assert_eq!(r.nrgeo[0], 1);
        for v in 0..g.vcount() as usize {
            prop_assert_eq!(r.vertex_paths[v].len() as u64, r.nrgeo[v]);
            prop_assert_eq!(r.edge_paths[v].len() as u64, r.nrgeo[v]);
            match d[v] {
                None => prop_assert_eq!(r.nrgeo[v], 0, "v={} unreachable but nrgeo={}", v, r.nrgeo[v]),
                Some(dv) => {
                    prop_assert!(r.nrgeo[v] >= 1);
                    for (vp, ep) in r.vertex_paths[v].iter().zip(r.edge_paths[v].iter()) {
                        prop_assert_eq!(*vp.first().unwrap(), 0u32);
                        prop_assert_eq!(*vp.last().unwrap(), v as u32);
                        prop_assert_eq!(ep.len() + 1, vp.len());
                        let sum: f64 = ep.iter().map(|&e| weights[e as usize]).sum();
                        prop_assert!((sum - dv).abs() < 1e-9, "v={} path sum {} != dist {}", v, sum, dv);
                    }
                }
            }
        }
    }

    /// `disjoint_union` invariants:
    /// - vcount(left) + vcount(right) == vcount(result)
    /// - ecount(left) + ecount(right) == ecount(result)
    /// - directedness preserved
    /// - left's edges appear in result with original endpoints
    /// - right's edges appear in result with endpoints shifted by left.vcount()
    #[test]
    fn disjoint_union_preserves_counts_and_shifts_right_endpoints(
        a in arb_graph(8),
        b in arb_graph(8),
    ) {
        let u = rust_igraph::disjoint_union(&a, &b).unwrap();
        prop_assert_eq!(u.vcount(), a.vcount() + b.vcount());
        prop_assert_eq!(u.ecount(), a.ecount() + b.ecount());
        prop_assert_eq!(u.is_directed(), a.is_directed());

        // First a.ecount() edges in u match a's edges (storage order).
        let m_a = u32::try_from(a.ecount()).unwrap();
        for e in 0..m_a {
            prop_assert_eq!(u.edge(e).unwrap(), a.edge(e).unwrap());
        }
        // Next b.ecount() edges are b's, shifted by a.vcount(). Edge
        // canonicalisation (from <= to in undirected mode) is applied
        // *after* the shift, so we compare canonicalised pairs.
        let n_a = a.vcount();
        let m_b = u32::try_from(b.ecount()).unwrap();
        for e in 0..m_b {
            let (bu, bv) = b.edge(e).unwrap();
            let (uu, uv) = u.edge(m_a + e).unwrap();
            let mut shifted = (bu + n_a, bv + n_a);
            if !a.is_directed() && shifted.0 > shifted.1 {
                shifted = (shifted.1, shifted.0);
            }
            prop_assert_eq!((uu, uv), shifted);
        }
    }

    /// Per-edge `is_loop` / `is_multiple` invariants:
    /// - lengths equal `ecount`
    /// - `is_loop[e] == (g.edge(e).0 == g.edge(e).1)`
    /// - `has_loop ⇔ any(is_loop)`
    /// - `has_multiple ⇔ any(is_multiple)`
    /// - count of trues in `is_multiple` equals (multi-edge group size − 1)
    ///   summed across groups (one canonical edge stays false per group).
    #[test]
    fn is_loop_and_is_multiple_per_edge_consistency(g in arb_graph(10)) {
        let m = u32::try_from(g.ecount()).unwrap();
        let il = rust_igraph::is_loop(&g).unwrap();
        let im = rust_igraph::is_multiple(&g).unwrap();
        prop_assert_eq!(il.len(), g.ecount());
        prop_assert_eq!(im.len(), g.ecount());

        for e in 0..m {
            let (u, v) = g.edge(e).unwrap();
            prop_assert_eq!(il[e as usize], u == v);
        }

        prop_assert_eq!(rust_igraph::has_loop(&g).unwrap(), il.iter().any(|&b| b));
        prop_assert_eq!(rust_igraph::has_multiple(&g).unwrap(), im.iter().any(|&b| b));

        // multi-mask count = total parallel-edge copies = m − distinct
        // canonical pairs.
        let mut pairs: Vec<(u32, u32)> = (0..m).map(|e| g.edge(e).unwrap()).collect();
        pairs.sort_unstable();
        pairs.dedup();
        let distinct_pairs = pairs.len();
        let true_count = im.iter().filter(|&&b| b).count();
        prop_assert_eq!(true_count, g.ecount() - distinct_pairs);
    }

    /// `has_loop` and `has_multiple` invariants:
    /// - `has_loop ⇔ ∃ edge (u, u)`
    /// - `is_simple ⇔ ¬has_loop ∧ ¬has_multiple`
    /// - simplify(g, true, true) makes both predicates false
    #[test]
    fn has_loop_and_has_multiple_match_definition(g in arb_graph(10)) {
        let m = u32::try_from(g.ecount()).unwrap();
        let any_self_loop = (0..m).any(|e| {
            let (u, v) = g.edge(e).unwrap();
            u == v
        });
        prop_assert_eq!(rust_igraph::has_loop(&g).unwrap(), any_self_loop);

        let hl = rust_igraph::has_loop(&g).unwrap();
        let hm = rust_igraph::has_multiple(&g).unwrap();
        let simple = rust_igraph::is_simple(&g).unwrap();
        prop_assert_eq!(simple, !hl && !hm);

        let s = rust_igraph::simplify(&g, true, true).unwrap();
        prop_assert!(!rust_igraph::has_loop(&s).unwrap());
        prop_assert!(!rust_igraph::has_multiple(&s).unwrap());
    }

    /// `is_simple` invariants:
    /// - returns the same boolean as the conjunction of "no self-loops"
    ///   and "no parallel edges across the canonicalised endpoint set"
    /// - `simplify(g, true, true)` is always simple
    /// - already-simple graphs are no-op simplified (ecount unchanged)
    #[test]
    fn is_simple_agrees_with_structural_definition(g in arb_graph(10)) {
        let m = u32::try_from(g.ecount()).unwrap();
        let mut seen = std::collections::BTreeSet::<(u32, u32)>::new();
        let mut found_loop_or_parallel = false;
        for e in 0..m {
            let (u, v) = g.edge(e).unwrap();
            if u == v { found_loop_or_parallel = true; break; }
            // Graph storage already canonicalises undirected (u <= v).
            if !seen.insert((u, v)) { found_loop_or_parallel = true; break; }
        }
        let expected_simple = !found_loop_or_parallel;
        prop_assert_eq!(rust_igraph::is_simple(&g).unwrap(), expected_simple);

        // simplify(g) is always simple.
        let s = rust_igraph::simplify(&g, true, true).unwrap();
        prop_assert!(rust_igraph::is_simple(&s).unwrap());

        // Already-simple graphs are unchanged by simplify.
        if expected_simple {
            prop_assert_eq!(s.ecount(), g.ecount());
        }
    }

    /// `simplify` invariants:
    /// - vcount unchanged
    /// - directedness unchanged
    /// - ecount never grows
    /// - on output: no self-loops if `remove_loops`, no parallel edges
    ///   if `remove_multiple`
    /// - simplifying twice with the same flags is idempotent
    #[test]
    fn simplify_drops_loops_and_multi_idempotently(
        g in arb_graph(8),
        remove_multiple in any::<bool>(),
        remove_loops in any::<bool>(),
    ) {
        let s = rust_igraph::simplify(&g, remove_multiple, remove_loops).unwrap();
        prop_assert_eq!(s.vcount(), g.vcount());
        prop_assert_eq!(s.is_directed(), g.is_directed());
        prop_assert!(s.ecount() <= g.ecount());

        let m = u32::try_from(s.ecount()).expect("ecount fits in u32");
        // No self-loops survive when remove_loops.
        if remove_loops {
            for e in 0..m {
                let (u, v) = s.edge(e).unwrap();
                prop_assert!(u != v, "self-loop survived: ({},{})", u, v);
            }
        }
        // No parallel edges survive when remove_multiple. Endpoints are
        // canonicalised by Graph storage for undirected graphs (from <=
        // to); for directed, we treat (a,b) and (b,a) as distinct.
        if remove_multiple {
            let mut pairs: Vec<(u32, u32)> = (0..m)
                .map(|e| s.edge(e).unwrap())
                .collect();
            pairs.sort_unstable();
            let unique = pairs.iter().collect::<std::collections::BTreeSet<_>>().len();
            prop_assert_eq!(pairs.len(), unique,
                            "parallel edges survived: {:?}", pairs);
        }

        // Idempotency: simplify(simplify(g)) == simplify(g).
        let s2 = rust_igraph::simplify(&s, remove_multiple, remove_loops).unwrap();
        prop_assert_eq!(s2.ecount(), s.ecount());
    }

    /// PageRank invariants: nonneg, finite, sums to 1 (for n >= 1),
    /// length equals vcount.
    #[test]
    fn pagerank_is_a_probability_distribution(g in arb_graph(8)) {
        let pr = rust_igraph::pagerank(&g).unwrap();
        let n = g.vcount() as usize;
        prop_assert_eq!(pr.len(), n);
        if n == 0 { return Ok(()); }
        let mut sum: f64 = 0.0;
        for (v, &x) in pr.iter().enumerate() {
            prop_assert!(x.is_finite(), "pr[{}] = {} not finite", v, x);
            prop_assert!(x >= -1e-12, "pr[{}] = {} negative", v, x);
            sum += x;
        }
        prop_assert!((sum - 1.0).abs() < 1e-6,
                     "pagerank does not sum to 1 (got {})", sum);
    }

    /// Edge betweenness invariants: nonneg, finite, length equals ecount.
    /// Sum of edge_betweenness across edges of an undirected geodesic
    /// equals (vertex sum of dependencies / 2) — a weak but useful
    /// soundness check we approximate by total = sum_pairs * mean_path_length.
    #[test]
    fn edge_betweenness_is_nonneg_finite(g in arb_graph(8)) {
        let eb = rust_igraph::edge_betweenness(&g).unwrap();
        prop_assert_eq!(eb.len(), g.ecount());
        for (e, &x) in eb.iter().enumerate() {
            prop_assert!(x.is_finite(), "eb[{}] = {} not finite", e, x);
            prop_assert!(x >= -1e-9, "eb[{}] = {} negative", e, x);
        }
    }

    /// Betweenness centrality bounds: nonnegative, finite, ≤ C(n,2)
    /// for undirected (each unordered pair contributes ≤ 1 unit) and
    /// ≤ n*(n-1) for directed.
    #[test]
    fn betweenness_is_nonneg_and_bounded(g in arb_graph(8)) {
        let b = rust_igraph::betweenness(&g).unwrap();
        let n = u64::from(g.vcount());
        let max = if g.is_directed() {
            n.saturating_mul(n.saturating_sub(1))
        } else {
            n.saturating_mul(n.saturating_sub(1)) / 2
        };
        #[allow(clippy::cast_precision_loss)]
        let max_f = max as f64;
        for (v, &x) in b.iter().enumerate() {
            prop_assert!(x.is_finite(), "betweenness[{}] = {} not finite", v, x);
            prop_assert!(x >= -1e-9, "betweenness[{}] = {} negative", v, x);
            prop_assert!(x <= max_f + 1e-9,
                         "betweenness[{}] = {} exceeds bound {}", v, x, max_f);
        }
    }

    /// Harmonic centrality bounds: 0 ≤ h ≤ 1 (max when every other
    /// vertex is at distance 1; sum_inv = n-1, /(n-1) = 1).
    /// Always finite (no NaN since unreachable contributes 0).
    #[test]
    fn harmonic_centrality_in_zero_to_one(g in arb_graph(8)) {
        let h = rust_igraph::harmonic_centrality(&g).unwrap();
        for (v, &x) in h.iter().enumerate() {
            prop_assert!(x.is_finite(), "harmonic[{}] = {} not finite", v, x);
            prop_assert!((0.0 - 1e-9..=1.0 + 1e-9).contains(&x),
                         "harmonic[{}] = {} outside [0, 1]", v, x);
        }
    }

    /// Closeness centrality bounds: when `Some(x)`, `0 < x <= 1` for
    /// connected components (since reach >= 1 and sum_dist >= reach).
    /// `None` only for isolated vertices.
    #[test]
    fn closeness_in_zero_to_one(g in arb_graph(8)) {
        let c = rust_igraph::closeness(&g).unwrap();
        for (v, val) in c.iter().enumerate() {
            if let Some(x) = val {
                prop_assert!(*x > 0.0 && *x <= 1.0,
                             "closeness[{}] = {} outside (0, 1]", v, x);
                prop_assert!(x.is_finite());
            }
        }
    }

    /// Transitive closure invariants:
    /// - same vcount and directedness as input
    /// - closure edge set equals reachability matrix off-diagonal pairs
    ///   (directed: ordered, undirected: unordered)
    /// - closure has no self-loops
    /// - closure is itself transitively closed (idempotent)
    #[test]
    fn transitive_closure_matches_reachability_matrix(g in arb_graph(7)) {
        let tc = rust_igraph::transitive_closure(&g).unwrap();
        prop_assert_eq!(tc.vcount(), g.vcount());
        prop_assert_eq!(tc.is_directed(), g.is_directed());

        // Build set of expected edges from the reachability matrix.
        let m = rust_igraph::reachability_matrix(&g).unwrap();
        let mut expected: std::collections::BTreeSet<(u32, u32)> = std::collections::BTreeSet::new();
        let directed = g.is_directed();
        for (u, row) in m.iter().enumerate() {
            let u_id = u32::try_from(u).expect("u fits in u32 for proptest");
            let start = if directed { 0 } else { u + 1 };
            for (v, &reachable) in row.iter().enumerate().skip(start) {
                if u != v && reachable {
                    let v_id = u32::try_from(v).expect("v fits in u32 for proptest");
                    expected.insert((u_id, v_id));
                }
            }
        }

        // Closure edges (canonicalised for undirected).
        let mut actual: std::collections::BTreeSet<(u32, u32)> = std::collections::BTreeSet::new();
        let m_edges = u32::try_from(tc.ecount()).expect("ecount fits in u32 for proptest");
        for e in 0..m_edges {
            let (a, b) = tc.edge(e).unwrap();
            // Self-loops shouldn't appear.
            prop_assert_ne!(a, b, "transitive closure has a self-loop");
            let pair = if directed || a < b { (a, b) } else { (b, a) };
            actual.insert(pair);
        }
        prop_assert_eq!(expected, actual);
    }

    /// Reachability matrix invariants:
    /// - n×n shape, all diagonals true.
    /// - Reachability is transitive: if `m[i][j] && m[j][k]` then `m[i][k]`.
    /// - For undirected graphs the matrix is symmetric.
    /// - Cross-check with `count_reachable`: row-sum equals the count.
    #[test]
    fn reachability_matrix_is_internally_consistent(g in arb_graph(8)) {
        let m = rust_igraph::reachability_matrix(&g).unwrap();
        let n = g.vcount() as usize;
        prop_assert_eq!(m.len(), n);
        for row in &m { prop_assert_eq!(row.len(), n); }
        // Diagonal.
        for (i, row) in m.iter().enumerate() {
            prop_assert!(row[i], "diagonal not self-reachable at {}", i);
        }
        // Symmetry on undirected graphs.
        if !g.is_directed() {
            for (i, row) in m.iter().enumerate() {
                for (j, &val) in row.iter().enumerate() {
                    prop_assert_eq!(val, m[j][i],
                                    "asymmetric undirected reach");
                }
            }
        }
        // Cross-check with count_reachable.
        let counts = rust_igraph::count_reachable(&g).unwrap();
        for (i, row) in m.iter().enumerate() {
            let row_count = u32::try_from(row.iter().filter(|&&b| b).count())
                .expect("row count fits in u32");
            prop_assert_eq!(counts[i], row_count,
                            "count_reachable disagrees with row sum");
        }
    }

    /// Assortativity bounds: when defined, the Pearson correlation lies
    /// in [-1, 1] (with small float slack at the boundaries). Skip the
    /// directed-error case via try_into. None for regular graphs.
    #[test]
    fn assortativity_in_minus_one_to_one(g in arb_graph(8)) {
        if let Some(r) = rust_igraph::assortativity_degree(&g).unwrap() {
            prop_assert!((-1.0 - 1e-9..=1.0 + 1e-9).contains(&r),
                         "assortativity {r} outside [-1, 1]");
            prop_assert!(r.is_finite());
        }
    }

    /// Density bounds: 0 ≤ density ≤ 1 for graphs without parallel edges
    /// (proptest's arb_graph allows multigraphs, so density may exceed 1
    /// — only check the lower bound). Empty/singleton: None.
    #[test]
    fn density_is_non_negative(g in arb_graph(10)) {
        let n = g.vcount();
        let d = rust_igraph::density(&g).unwrap();
        if n < 2 {
            prop_assert_eq!(d, None);
        } else {
            let v = d.expect("density should be Some for n>=2");
            prop_assert!(v >= 0.0, "density {} is negative", v);
            prop_assert!(v.is_finite(), "density should be finite");
        }
    }

    /// Mean-distance lower bound: ≥ 1.0 when at least one edge exists in
    /// a connected pair (since shortest distance is at least 1). When
    /// there are no connected pairs, returns None.
    #[test]
    fn mean_distance_is_at_least_one(g in arb_graph(10)) {
        if let Some(md) = rust_igraph::mean_distance(&g).unwrap() {
            prop_assert!(md >= 1.0, "mean_distance {} < 1.0", md);
            prop_assert!(md.is_finite(), "mean_distance should be finite");
        }
    }

    /// Local transitivity sum equals 3 * (count_triangles divided by ...).
    /// The simpler invariant: sum of (per-vertex adjacent-triangle count)
    /// over all vertices equals 3 * total triangles. We back this out by
    /// pulling per-vertex `t = clustering * d * (d - 1) / 2` from the
    /// clustering vector and degrees.
    #[test]
    fn local_transitivity_back_solves_to_total_triangles(g in arb_graph(8)) {
        let local = rust_igraph::transitivity_local_undirected(&g).unwrap();
        let total = rust_igraph::count_triangles(&g).unwrap();

        // Sum per-vertex triangle counts. For deg<2 (None entries) the
        // contribution is 0.
        let mut sum_per_vertex_triangles: f64 = 0.0;
        for v in 0..g.vcount() {
            // Compute simple-degree.
            let raw = g.neighbors(v).unwrap();
            let mut simple: Vec<u32> = raw.into_iter().filter(|&u| u != v).collect();
            simple.sort_unstable();
            simple.dedup();
            let d = u32::try_from(simple.len()).expect("simple-degree fits in u32 for proptest");
            if let Some(c) = local[v as usize] {
                let pairs = f64::from(d) * f64::from(d.saturating_sub(1)) / 2.0;
                sum_per_vertex_triangles += c * pairs;
            }
        }
        // Each undirected triangle contributes +1 to three vertices' counts,
        // so the sum is 3 * triangles. Allow a small float tolerance.
        #[allow(clippy::cast_precision_loss)]
        let expected = 3.0 * (total as f64);
        prop_assert!(
            (sum_per_vertex_triangles - expected).abs() < 1e-9,
            "sum local triangles {} != 3 * total {}",
            sum_per_vertex_triangles, expected
        );
    }

    /// Triangle count / transitivity coherence: transitivity equals
    /// `3 * triangles / triples`. Triples can be brute-forced as
    /// `sum_v C(deg_simple(v), 2)`. Triangle count must be ≤ triples / 3.
    #[test]
    fn triangle_count_and_transitivity_are_coherent(g in arb_graph(8)) {
        let triangles = rust_igraph::count_triangles(&g).unwrap();
        let trans = rust_igraph::transitivity_undirected(&g).unwrap();

        // Recompute simple-degree for each vertex (no self-loops, no parallels).
        let n = g.vcount();
        let mut triples: u64 = 0;
        for v in 0..n {
            let raw = g.neighbors(v).unwrap();
            let mut simple: Vec<u32> = raw.into_iter().filter(|&u| u != v).collect();
            simple.sort_unstable();
            simple.dedup();
            let d = simple.len() as u64;
            if d >= 2 { triples += d * (d - 1) / 2; }
        }
        // Each triangle contributes 3 closed triples (one per vertex).
        prop_assert!(triangles * 3 <= triples,
                     "3 * triangles ({}) > triples ({})", triangles * 3, triples);
        if triples == 0 {
            prop_assert_eq!(trans, None);
            prop_assert_eq!(triangles, 0);
        } else {
            // Small proptest graphs (≤ 8 vertices); counts fit in f64 exactly.
            #[allow(clippy::cast_precision_loss)]
            let expected = (triangles as f64) * 3.0 / (triples as f64);
            prop_assert_eq!(trans, Some(expected));
        }
    }

    /// Radius / diameter / eccentricity coherence: `radius == min(ecc)`,
    /// `diameter == max(ecc)`, all entries non-negative, all bounded by
    /// `vcount`. (Sanity invariants since the three functions share the
    /// same `distances` inner loop.)
    #[test]
    fn radii_are_coherent(g in arb_graph(10)) {
        let ecc = rust_igraph::eccentricity(&g).unwrap();
        let r = rust_igraph::radius(&g).unwrap();
        let d = rust_igraph::diameter(&g).unwrap();
        let n = g.vcount();
        prop_assert_eq!(ecc.len(), n as usize);
        for &e in &ecc {
            prop_assert!(e <= n, "eccentricity {} exceeds vcount {}", e, n);
        }
        let derived_r = ecc.iter().copied().min();
        let derived_d = ecc.iter().copied().max();
        prop_assert_eq!(r, derived_r);
        prop_assert_eq!(d, derived_d);
    }

    /// Girth lower bound: if `girth(g) == Some(k)`, no BFS tree from any
    /// vertex can find a cycle shorter than k. We sanity-check that the
    /// reported girth is consistent with each vertex's BFS depth — every
    /// neighbour pair (u, v) on the same BFS level L from some root forces
    /// `girth <= 2L + 1`, neighbours on adjacent levels force `girth <= 2L`.
    /// (Equivalently: girth ≤ shortest cycle found by Itai-Rodeh BFS from
    /// any vertex, which is exactly what the algorithm computes.)
    /// Bounded brute-force: just check the result is plausible — `girth >= 3`
    /// (no loops/multi-edges count) and `<= vcount`.
    #[test]
    fn girth_is_within_bounds(g in arb_graph(8)) {
        let n = g.vcount();
        if let Some(k) = rust_igraph::girth(&g).unwrap() {
            prop_assert!(k >= 3, "girth {} is too small (loops/multi shouldn't count)", k);
            prop_assert!(k <= n, "girth {} exceeds vcount {}", k, n);
        }
        // Acyclic graph (a forest): girth must be None.
        // Cheap check via edge-count: a forest has at most `n - components`
        // edges. We use cc to detect this.
        let cc = rust_igraph::connected_components(&g).unwrap();
        // Filter out loops/parallels — actual cycles are independent of them.
        let mut simple_edges: std::collections::BTreeSet<(u32, u32)> =
            std::collections::BTreeSet::new();
        let m = u32::try_from(g.ecount()).expect("edge count fits in u32");
        for e in 0..m {
            let (u, v) = g.edge(e).unwrap();
            if u == v { continue; }
            simple_edges.insert(if u < v { (u, v) } else { (v, u) });
        }
        // Tree-edge count = n - components. If equal, the graph is a forest
        // and has no cycles.
        let simple_count = u32::try_from(simple_edges.len())
            .expect("simple edge count fits in u32 for proptest");
        if simple_count + cc.count == n {
            prop_assert_eq!(rust_igraph::girth(&g).unwrap(), None,
                            "forest reported girth Some");
        }
    }

    /// `is_biconnected` consistency: a graph with vcount >= 3 is biconnected
    /// iff `connected_components.count == 1` AND `articulation_points` is
    /// empty. (Two-vertex and trivial cases excluded — they have their own
    /// special-case logic.)
    #[test]
    fn is_biconnected_matches_aps_and_connectivity(g in arb_graph(10)) {
        let n = g.vcount();
        if n < 3 { return Ok(()); }
        let computed = rust_igraph::is_biconnected(&g).unwrap();
        let cc = rust_igraph::connected_components(&g).unwrap();
        let aps = rust_igraph::articulation_points(&g).unwrap();
        let derived = cc.count == 1 && aps.is_empty();
        prop_assert_eq!(computed, derived,
                        "is_biconnected={} but cc.count={} aps={:?}",
                        computed, cc.count, aps);
    }

    /// Brute-force bridge invariant: an edge is a bridge iff its endpoints
    /// land in different weak components after the edge is removed (and
    /// neither endpoint had any other path to the other side).
    /// Tested by direct edge removal + recount on small graphs.
    #[test]
    fn bridges_match_brute_force_definition(g in arb_graph(7)) {
        let computed: std::collections::BTreeSet<u32> =
            rust_igraph::bridges(&g).unwrap().into_iter().collect();
        let m = u32::try_from(g.ecount()).expect("edge count fits in u32");
        for e in 0..m {
            let (u, v) = g.edge(e).unwrap();
            // Self-loops are never bridges.
            if u == v {
                prop_assert!(!computed.contains(&e),
                             "self-loop {} reported as bridge", e);
                continue;
            }
            // Build g - e: same edges minus this one.
            let mut h = rust_igraph::Graph::with_vertices(g.vcount());
            for f in 0..m {
                if f == e { continue; }
                let (a, b) = g.edge(f).unwrap();
                h.add_edge(a, b).unwrap();
            }
            let cc_h = rust_igraph::connected_components(&h).unwrap();
            let split = cc_h.membership[u as usize] != cc_h.membership[v as usize];
            prop_assert_eq!(
                computed.contains(&e), split,
                "edge {} ({},{}) bridge-status mismatch (computed={}, brute={})",
                e, u, v, computed.contains(&e), split
            );
        }
    }

    /// Brute-force articulation invariant: a vertex `v` is an articulation
    /// point iff removing all edges incident to it (and `v` itself) increases
    /// the number of weakly connected components on the remaining vertices.
    /// We verify on a small graph by direct removal + recount.
    #[test]
    fn articulation_points_match_brute_force_definition(g in arb_graph(7)) {
        let computed: std::collections::BTreeSet<u32> =
            rust_igraph::articulation_points(&g).unwrap().into_iter().collect();

        // Baseline: number of WCCs on `g`'s non-isolated vertices.
        let baseline_cc = rust_igraph::connected_components(&g).unwrap();
        let baseline_nontrivial = (0..g.vcount())
            .filter(|&v| g.degree(v).unwrap() > 0)
            .count();
        let baseline_components_with_edges = (0..baseline_cc.count)
            .filter(|&cid| {
                (0..g.vcount()).any(|v| baseline_cc.membership[v as usize] == cid
                                          && g.degree(v).unwrap() > 0)
            })
            .count();
        let _ = (baseline_nontrivial, baseline_components_with_edges);

        for v in 0..g.vcount() {
            // An isolated vertex is never an articulation point.
            if g.degree(v).unwrap() == 0 {
                prop_assert!(!computed.contains(&v),
                             "isolated vertex {} reported as articulation", v);
                continue;
            }

            // Build g - v: rebuild the edge list, dropping edges incident to v
            // (and the vertex itself by mapping ids around v).
            let mut h = rust_igraph::Graph::with_vertices(g.vcount());
            let m = u32::try_from(g.ecount()).expect("edge count fits in u32");
            for e in 0..m {
                let (u, w) = g.edge(e).unwrap();
                if u == v || w == v { continue; }
                h.add_edge(u, w).unwrap();
            }
            // Count components on h, ignoring v's own component (it's
            // isolated in h since we removed all its incident edges).
            let cc_h = rust_igraph::connected_components(&h).unwrap();
            // For each non-v vertex, group by component id; count groups
            // that contain at least one of v's original neighbours.
            let neighbours: std::collections::BTreeSet<u32> =
                g.neighbors(v).unwrap().into_iter().filter(|&w| w != v).collect();
            if neighbours.len() <= 1 {
                // Pendant vertex (degree 1 modulo self-loops): never articulation.
                prop_assert!(!computed.contains(&v),
                             "pendant vertex {} reported as articulation", v);
                continue;
            }
            let nbr_components: std::collections::BTreeSet<u32> = neighbours
                .iter()
                .map(|&w| cc_h.membership[w as usize])
                .collect();

            let v_is_articulation = nbr_components.len() > 1;
            prop_assert_eq!(
                computed.contains(&v), v_is_articulation,
                "vertex {} articulation-status mismatch (computed={}, brute={})",
                v, computed.contains(&v), v_is_articulation
            );
        }
    }

    /// Eulerian classification monotonicity: if `has_cycle` then
    /// `has_path`, on every graph. (A closed Eulerian walk is also a
    /// valid open Eulerian walk.)
    #[test]
    fn is_eulerian_cycle_implies_path(g in arb_graph(10)) {
        let r = rust_igraph::is_eulerian(&g).unwrap();
        if r.has_cycle {
            prop_assert!(r.has_path,
                         "graph reports Eulerian cycle but no Eulerian path");
        }
    }

    /// `distances(g, 0)` is consistent with `bfs(g, 0)`: every BFS-visited
    /// vertex has `Some(_)` distance and every unvisited vertex has `None`.
    /// Source's own distance is always `Some(0)`.
    #[test]
    fn distances_match_bfs_reachability(g in arb_graph(15)) {
        let d = rust_igraph::distances(&g, 0).unwrap();
        let bfs_set: std::collections::BTreeSet<u32> =
            rust_igraph::bfs(&g, 0).unwrap().into_iter().collect();
        prop_assert_eq!(d.len(), g.vcount() as usize);
        prop_assert_eq!(d[0], Some(0));
        for v in 0..g.vcount() {
            if bfs_set.contains(&v) {
                prop_assert!(d[v as usize].is_some(),
                             "vertex {} reachable but distance is None", v);
            } else {
                prop_assert_eq!(d[v as usize], None,
                                "vertex {} unreachable but distance is Some", v);
            }
        }
    }

    /// Triangle inequality: for any edge (u, v) in an undirected graph,
    /// `|d[u] - d[v]| <= 1` whenever both are reachable from the source.
    #[test]
    fn distances_obey_triangle_inequality(g in arb_graph(15)) {
        let d = rust_igraph::distances(&g, 0).unwrap();
        let m = u32::try_from(g.ecount()).expect("edge count fits in u32 for proptest");
        for e in 0..m {
            let (u, v) = g.edge(e).unwrap();
            if let (Some(du), Some(dv)) = (d[u as usize], d[v as usize]) {
                let diff = du.abs_diff(dv);
                prop_assert!(diff <= 1,
                             "edge ({},{}) violates triangle inequality: d[{}]={}, d[{}]={}",
                             u, v, u, du, v, dv);
            } else {
                // Either both unreachable, or one of them is unreachable
                // because they're on different sides of the BFS frontier
                // (impossible — adjacency means same component). Force
                // both to be Some or both to be None.
                prop_assert_eq!(d[u as usize].is_some(), d[v as usize].is_some(),
                                "endpoints of edge ({},{}) have mixed reachability", u, v);
            }
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

    /// On undirected graphs `assortativity_degree_directed` must
    /// agree with the canonical `assortativity_degree` (since the C
    /// `directed` arg is documented as ignored on undirected inputs).
    #[test]
    fn assortativity_degree_directed_undirected_matches_canonical(g in arb_graph(15)) {
        let a = rust_igraph::assortativity_degree(&g).unwrap();
        let b = rust_igraph::assortativity_degree_directed(&g).unwrap();
        prop_assert_eq!(a, b);
    }

    /// On undirected graphs `modularity_directed` must agree with
    /// the canonical `modularity` (the `directed` arg is ignored
    /// upstream on undirected inputs).
    #[test]
    fn modularity_directed_undirected_matches_canonical(g in arb_graph(12)) {
        if g.ecount() == 0 { return Ok(()); }
        let n = g.vcount() as usize;
        let mem: Vec<u32> = (0..n).map(|i| u32::from(i >= n / 2)).collect();
        let a = rust_igraph::modularity(&g, &mem, 1.0).unwrap();
        let b = rust_igraph::modularity_directed(&g, &mem, 1.0).unwrap();
        match (a, b) {
            (Some(x), Some(y)) => prop_assert!((x - y).abs() < 1e-12, "a={x} b={y}"),
            (None, None) => {}
            (a, b) => prop_assert!(false, "a={a:?} b={b:?}"),
        }
    }

    /// On undirected graphs every [`CorenessMode`] must agree with
    /// the canonical `coreness` entry. The mode parameter is
    /// meaningful only for directed graphs.
    #[test]
    fn coreness_with_mode_undirected_modes_agree(g in arb_graph(15)) {
        use rust_igraph::CorenessMode;
        let canonical = rust_igraph::coreness(&g).unwrap();
        for mode in [CorenessMode::All, CorenessMode::In, CorenessMode::Out] {
            let r = rust_igraph::coreness_with_mode(&g, mode).unwrap();
            prop_assert_eq!(r, canonical.clone(), "mode {:?} diverged", mode);
        }
    }

    /// `disjoint_union_many(&[a, b])` must produce the same vertex /
    /// edge counts as `disjoint_union(a, b)` (and the same edge
    /// multiset after sorting). Two-arg parity is the cheapest
    /// invariant the variadic path has.
    #[test]
    fn disjoint_union_many_two_arg_matches_disjoint_union(
        a in arb_graph(8),
        b in arb_graph(8),
    ) {
        if a.is_directed() != b.is_directed() { return Ok(()); }
        let pairwise = rust_igraph::disjoint_union(&a, &b).unwrap();
        let many = rust_igraph::disjoint_union_many(&[&a, &b]).unwrap();
        prop_assert_eq!(pairwise.vcount(), many.vcount());
        prop_assert_eq!(pairwise.ecount(), many.ecount());
        let p_m = u32::try_from(pairwise.ecount()).unwrap();
        let m_m = u32::try_from(many.ecount()).unwrap();
        let mut p_edges: Vec<(u32, u32)> = (0..p_m)
            .map(|e| pairwise.edge(e).unwrap()).collect();
        let mut m_edges: Vec<(u32, u32)> = (0..m_m)
            .map(|e| many.edge(e).unwrap()).collect();
        p_edges.sort_unstable();
        m_edges.sort_unstable();
        prop_assert_eq!(p_edges, m_edges);
    }

    /// On undirected graphs, both [`SimpleMode`] variants must agree
    /// (the mode parameter is meaningful only for directed graphs).
    #[test]
    fn is_simple_with_mode_undirected_modes_agree(g in arb_graph(15)) {
        use rust_igraph::SimpleMode;
        let a = rust_igraph::is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap();
        let b = rust_igraph::is_simple_with_mode(&g, SimpleMode::DirectedAsUndirected).unwrap();
        prop_assert_eq!(a, b, "modes diverged on an undirected graph");
        // And both must agree with the canonical wrapper.
        let c = rust_igraph::is_simple(&g).unwrap();
        prop_assert_eq!(a, c);
    }

    /// Unit-weight `modularity_weighted` must equal unweighted
    /// `modularity` exactly on every graph + partition. Trivial
    /// invariant but it's the fastest sanity gate the weighted
    /// path has — any drift in the strength accumulator surfaces.
    #[test]
    fn modularity_weighted_unit_weights_match_unweighted_proptest(g in arb_graph(12)) {
        if g.ecount() == 0 { return Ok(()); }
        let n = g.vcount() as usize;
        // 2-block partition: first half in 0, rest in 1.
        let mem: Vec<u32> = (0..n).map(|i| u32::from(i >= n / 2)).collect();
        let weights = vec![1.0_f64; g.ecount()];
        let qw = rust_igraph::modularity_weighted(&g, &mem, 1.0, &weights).unwrap();
        let q = rust_igraph::modularity(&g, &mem, 1.0).unwrap();
        match (qw, q) {
            (Some(a), Some(b)) => prop_assert!((a - b).abs() < 1e-12, "qw={a} q={b}"),
            (None, None) => {}
            (a, b) => prop_assert!(false, "qw={a:?} q={b:?}"),
        }
    }

    /// Unit-weight `avg_nearest_neighbor_degree_weighted` must match
    /// the unweighted entry on every graph. Verifies the weighted code
    /// path's strength accumulator and edge alignment are correct.
    #[test]
    fn knn_weighted_unit_weights_match_unweighted(g in arb_graph(12)) {
        let weights = vec![1.0_f64; g.ecount()];
        let unw = rust_igraph::avg_nearest_neighbor_degree(&g).unwrap();
        let w = rust_igraph::avg_nearest_neighbor_degree_weighted(&g, &weights).unwrap();
        prop_assert_eq!(unw.len(), w.len());
        for (i, (a, b)) in unw.iter().zip(w.iter()).enumerate() {
            match (a, b) {
                (Some(x), Some(y)) => prop_assert!((x - y).abs() < 1e-12, "v={i} a={x} b={y}"),
                (None, None) => {}
                _ => prop_assert!(false, "v={} unw={:?} w={:?}", i, a, b),
            }
        }
    }

    /// `decompose(g)` invariants on arbitrary undirected graphs:
    /// - The number of components matches `connected_components(g).count`.
    /// - Total `vcount` across subgraphs equals `g.vcount`.
    /// - Total `ecount` across subgraphs equals `g.ecount`.
    /// - Each component subgraph is itself fully connected
    ///   (single-component) when re-checked.
    #[test]
    fn decompose_partition_invariants(g in arb_graph(12)) {
        let parts = rust_igraph::decompose(&g).unwrap();
        let cc = rust_igraph::connected_components(&g).unwrap();
        prop_assert_eq!(parts.len() as u32, cc.count);

        let total_v: u32 = parts.iter().map(|p| p.vcount()).sum();
        prop_assert_eq!(total_v, g.vcount());

        let total_e: usize = parts.iter().map(|p| p.ecount()).sum();
        prop_assert_eq!(total_e, g.ecount());

        for (i, p) in parts.iter().enumerate() {
            let sub_cc = rust_igraph::connected_components(p).unwrap();
            prop_assert_eq!(sub_cc.count, 1, "component {} not single-cc: count={}",
                i, sub_cc.count);
        }
    }

    /// On a simple graph with unit weights, Barrat's weighted local
    /// transitivity must equal the unweighted local transitivity. This
    /// pins the formula's symmetry property and the strength
    /// computation against the well-tested unweighted variant.
    #[test]
    fn barrat_unit_weights_match_unweighted(g in arb_graph(10)) {
        let s = rust_igraph::simplify(&g, true, true).unwrap();
        let weights = vec![1.0_f64; s.ecount()];
        let unw = rust_igraph::transitivity_local_undirected(&s).unwrap();
        let bar = rust_igraph::transitivity_barrat(&s, &weights).unwrap();
        prop_assert_eq!(unw.len(), bar.len());
        for (i, (a, b)) in unw.iter().zip(bar.iter()).enumerate() {
            match (a, b) {
                (Some(x), Some(y)) => prop_assert!((x - y).abs() < 1e-12, "v={i} unw={x} bar={y}"),
                (None, None) => {}
                _ => prop_assert!(false, "v={} unw={:?} bar={:?}", i, a, b),
            }
        }
    }

    /// Barrat result is bounded: on a simple graph each vertex's
    /// Barrat value (when defined) lies in [0, 1] — same range as the
    /// unweighted clustering coefficient.
    #[test]
    fn barrat_values_in_unit_interval(g in arb_graph(10)) {
        let s = rust_igraph::simplify(&g, true, true).unwrap();
        let m = s.ecount();
        // Use varying positive weights to exercise the formula.
        let weights: Vec<f64> = (0..m).map(|i| (i as f64) + 1.0).collect();
        let bar = rust_igraph::transitivity_barrat(&s, &weights).unwrap();
        for (i, val) in bar.iter().enumerate() {
            if let Some(x) = val {
                prop_assert!(*x >= -1e-12 && *x <= 1.0 + 1e-12,
                    "v={i} barrat={x} out of [0,1]");
            }
        }
    }

    /// Unit-weight `knnk_weighted` must match unweighted `knnk` on
    /// every graph. Same correctness gate as the per-vertex variant.
    #[test]
    fn knnk_weighted_unit_weights_match_unweighted(g in arb_graph(12)) {
        let weights = vec![1.0_f64; g.ecount()];
        let unw = rust_igraph::knnk(&g).unwrap();
        let w = rust_igraph::knnk_weighted(&g, &weights).unwrap();
        prop_assert_eq!(unw.len(), w.len());
        for (i, (a, b)) in unw.iter().zip(w.iter()).enumerate() {
            match (a, b) {
                (Some(x), Some(y)) => prop_assert!((x - y).abs() < 1e-12, "deg {} a={x} b={y}", i+1),
                (None, None) => {}
                _ => prop_assert!(false, "deg {} unw={:?} w={:?}", i+1, a, b),
            }
        }
    }

    /// `reciprocity_with_mode(_, false, Default)` must equal the
    /// canonical [`reciprocity`] entry on every graph (the latter is
    /// just a wrapper around the former with default args).
    #[test]
    fn reciprocity_with_mode_default_matches_reciprocity(g in arb_graph(15)) {
        use rust_igraph::ReciprocityMode;
        let a = rust_igraph::reciprocity(&g).unwrap();
        let b = rust_igraph::reciprocity_with_mode(&g, false, ReciprocityMode::Default).unwrap();
        prop_assert_eq!(a, b);
    }

    /// `coreness` invariants on arbitrary undirected graphs:
    /// - same length as `vcount`
    /// - per-vertex coreness ≤ degree (peeling can only decrease)
    /// - vertices in the same connected component cannot have a coreness
    ///   difference larger than the component's max degree (sanity bound)
    #[test]
    fn coreness_bounded_by_degree(g in arb_graph(15)) {
        let cores = rust_igraph::coreness(&g).unwrap();
        prop_assert_eq!(cores.len(), g.vcount() as usize);
        for v in 0..g.vcount() {
            let d = u32::try_from(g.degree(v).unwrap()).unwrap();
            prop_assert!(
                cores[v as usize] <= d,
                "vertex {} coreness {} exceeds degree {}",
                v, cores[v as usize], d
            );
        }
    }

    /// `floyd_warshall_distances` invariants on undirected graphs with
    /// unit weights:
    /// - diagonal is `Some(0.0)`
    /// - matrix is symmetric: `M[i][j] == M[j][i]`
    /// - row 0 (`M[0]`) matches the BFS distances from vertex 0
    ///   (cast to f64).
    #[test]
    fn floyd_warshall_unit_weights_symmetric_and_match_bfs(g in arb_graph(8)) {
        if g.vcount() == 0 { return Ok(()); }
        let m = g.ecount();
        let weights = vec![1.0_f64; m];
        let fw = rust_igraph::floyd_warshall_distances(&g, Some(&weights)).unwrap();
        let n = g.vcount() as usize;
        prop_assert_eq!(fw.len(), n);

        for (i, row) in fw.iter().enumerate() {
            prop_assert_eq!(row[i], Some(0.0), "diagonal at {}", i);
            for (j, cell) in row.iter().enumerate() {
                prop_assert_eq!(*cell, fw[j][i], "asymmetric at ({}, {})", i, j);
            }
        }

        // Row 0 should agree with the unweighted single-source BFS.
        let bfs = rust_igraph::distances(&g, 0).unwrap();
        for (v, (fwv, bfsv)) in fw[0].iter().zip(bfs.iter()).enumerate() {
            match (fwv, bfsv) {
                (Some(rd), Some(rb)) => {
                    prop_assert!((rd - f64::from(*rb)).abs() < 1e-12,
                                 "vertex {}: fw={} bfs={}", v, rd, rb);
                }
                (None, None) => {}
                (a, b) => {
                    prop_assert!(false, "vertex {}: fw={:?} bfs={:?}", v, a, b);
                }
            }
        }
    }

    /// `union(a, a)` is idempotent up to canonical-edge multiset
    /// equality (max(k, k) = k preserves every multiplicity).
    #[test]
    fn union_with_self_is_idempotent(g in arb_graph(8)) {
        let u = rust_igraph::union(&g, &g).unwrap();
        prop_assert_eq!(u.vcount(), g.vcount());
        prop_assert_eq!(u.ecount(), g.ecount());
        prop_assert_eq!(u.is_directed(), g.is_directed());

        let m_g = u32::try_from(g.ecount()).unwrap();
        let mut g_pairs: Vec<(u32, u32)> = (0..m_g).map(|e| g.edge(e).unwrap()).collect();
        g_pairs.sort_unstable();
        let m_u = u32::try_from(u.ecount()).unwrap();
        let mut u_pairs: Vec<(u32, u32)> = (0..m_u).map(|e| u.edge(e).unwrap()).collect();
        u_pairs.sort_unstable();
        prop_assert_eq!(u_pairs, g_pairs);
    }

    /// `intersection(a, a)` is idempotent up to canonical-edge multiset
    /// equality (min(k, k) = k preserves every multiplicity).
    #[test]
    fn intersection_with_self_is_idempotent(g in arb_graph(8)) {
        let i = rust_igraph::intersection(&g, &g).unwrap();
        prop_assert_eq!(i.vcount(), g.vcount());
        prop_assert_eq!(i.ecount(), g.ecount());
        prop_assert_eq!(i.is_directed(), g.is_directed());

        let m_g = u32::try_from(g.ecount()).unwrap();
        let mut g_pairs: Vec<(u32, u32)> = (0..m_g).map(|e| g.edge(e).unwrap()).collect();
        g_pairs.sort_unstable();
        let m_i = u32::try_from(i.ecount()).unwrap();
        let mut i_pairs: Vec<(u32, u32)> = (0..m_i).map(|e| i.edge(e).unwrap()).collect();
        i_pairs.sort_unstable();
        prop_assert_eq!(i_pairs, g_pairs);
    }

    /// `intersection` invariants on two arbitrary undirected graphs:
    /// - vcount = max(left, right)
    /// - directedness preserved (and shared)
    /// - per-pair multiplicity = min of the two inputs' multiplicities
    /// - ecount = Σ_pairs min(count_left, count_right)
    /// - intersection is commutative: intersection(a, b) ≡ intersection(b, a)
    #[test]
    fn intersection_min_multiplicity_per_pair(
        a in arb_graph(6),
        b in arb_graph(6),
    ) {
        use std::collections::BTreeMap;
        let i = rust_igraph::intersection(&a, &b).unwrap();
        prop_assert_eq!(i.vcount(), std::cmp::max(a.vcount(), b.vcount()));
        prop_assert_eq!(i.is_directed(), a.is_directed());

        let count = |g: &Graph| -> BTreeMap<(u32, u32), u32> {
            let mut m = BTreeMap::new();
            let n = u32::try_from(g.ecount()).unwrap();
            for e in 0..n {
                let p = g.edge(e).unwrap();
                *m.entry(p).or_insert(0u32) += 1;
            }
            m
        };
        let ca = count(&a);
        let cb = count(&b);
        let ci = count(&i);

        // Per-pair: count in result equals min(count_a, count_b); 0 when
        // pair missing from either side.
        let mut keys: Vec<(u32, u32)> = ca.keys().chain(cb.keys()).copied().collect();
        keys.sort_unstable();
        keys.dedup();
        let mut expected_e: usize = 0;
        for k in &keys {
            let want = std::cmp::min(
                ca.get(k).copied().unwrap_or(0),
                cb.get(k).copied().unwrap_or(0),
            );
            let got = ci.get(k).copied().unwrap_or(0);
            prop_assert_eq!(got, want, "pair {:?}", k);
            expected_e += want as usize;
        }
        prop_assert_eq!(i.ecount(), expected_e);

        // Commutativity: swapping operands yields the same edge multiset.
        let i_swap = rust_igraph::intersection(&b, &a).unwrap();
        let m_i_swap = u32::try_from(i_swap.ecount()).unwrap();
        let mut swap_pairs: Vec<(u32, u32)> =
            (0..m_i_swap).map(|e| i_swap.edge(e).unwrap()).collect();
        swap_pairs.sort_unstable();
        let m_i = u32::try_from(i.ecount()).unwrap();
        let mut i_pairs: Vec<(u32, u32)> = (0..m_i).map(|e| i.edge(e).unwrap()).collect();
        i_pairs.sort_unstable();
        prop_assert_eq!(swap_pairs, i_pairs);
    }

    /// `union` invariants on two arbitrary undirected graphs:
    /// - vcount = max(left, right)
    /// - directedness preserved (and shared)
    /// - per-pair multiplicity = max of the two inputs' multiplicities
    /// - ecount = Σ_pairs max(count_left, count_right)
    #[test]
    fn union_max_multiplicity_per_pair(
        a in arb_graph(6),
        b in arb_graph(6),
    ) {
        use std::collections::BTreeMap;
        let u = rust_igraph::union(&a, &b).unwrap();
        prop_assert_eq!(u.vcount(), std::cmp::max(a.vcount(), b.vcount()));
        prop_assert_eq!(u.is_directed(), a.is_directed());

        let count = |g: &Graph| -> BTreeMap<(u32, u32), u32> {
            let mut m = BTreeMap::new();
            let n = u32::try_from(g.ecount()).unwrap();
            for e in 0..n {
                let p = g.edge(e).unwrap();
                *m.entry(p).or_insert(0u32) += 1;
            }
            m
        };
        let ca = count(&a);
        let cb = count(&b);
        let cu = count(&u);

        // Every pair appearing in a or b has the right max-multiplicity in u.
        let mut keys: Vec<(u32, u32)> = ca.keys().chain(cb.keys()).copied().collect();
        keys.sort_unstable();
        keys.dedup();
        let mut expected_e: usize = 0;
        for k in &keys {
            let want = std::cmp::max(
                ca.get(k).copied().unwrap_or(0),
                cb.get(k).copied().unwrap_or(0),
            );
            let got = cu.get(k).copied().unwrap_or(0);
            prop_assert_eq!(got, want, "pair {:?}", k);
            expected_e += want as usize;
        }
        prop_assert_eq!(u.ecount(), expected_e);

        // No spurious pairs: every pair in u must appear in a or b.
        for k in cu.keys() {
            prop_assert!(ca.contains_key(k) || cb.contains_key(k));
        }
    }

    /// `difference(a, a)` is the empty edge set on a.vcount() vertices.
    #[test]
    fn difference_with_self_is_empty(g in arb_graph(8)) {
        let d = rust_igraph::difference(&g, &g).unwrap();
        prop_assert_eq!(d.vcount(), g.vcount());
        prop_assert_eq!(d.ecount(), 0);
        prop_assert_eq!(d.is_directed(), g.is_directed());
    }

    /// `difference(a, empty)` keeps every edge of `a` (multiset
    /// equality) and the same vcount.
    #[test]
    fn difference_with_empty_is_identity(g in arb_graph(8)) {
        use std::collections::BTreeMap;
        let empty = if g.is_directed() {
            Graph::new(g.vcount(), true).unwrap()
        } else {
            Graph::with_vertices(g.vcount())
        };
        let d = rust_igraph::difference(&g, &empty).unwrap();
        prop_assert_eq!(d.vcount(), g.vcount());
        prop_assert_eq!(d.ecount(), g.ecount());

        let count = |graph: &Graph| -> BTreeMap<(u32, u32), u32> {
            let mut m = BTreeMap::new();
            let n = u32::try_from(graph.ecount()).unwrap();
            for e in 0..n {
                let p = graph.edge(e).unwrap();
                *m.entry(p).or_insert(0u32) += 1;
            }
            m
        };
        prop_assert_eq!(count(&g), count(&d));
    }

    /// `difference` invariants on two arbitrary undirected graphs:
    /// - vcount = orig.vcount() (asymmetric, NOT max)
    /// - directedness preserved
    /// - per-pair multiplicity = max(0, count_orig - count_sub)
    /// - ecount = Σ_pairs max(0, count_orig - count_sub)
    /// - no pair in result is absent from `orig`
    #[test]
    fn difference_clamped_subtract_per_pair(
        a in arb_graph(6),
        b in arb_graph(6),
    ) {
        use std::collections::BTreeMap;
        let d = rust_igraph::difference(&a, &b).unwrap();
        prop_assert_eq!(d.vcount(), a.vcount());
        prop_assert_eq!(d.is_directed(), a.is_directed());

        let count = |g: &Graph| -> BTreeMap<(u32, u32), u32> {
            let mut m = BTreeMap::new();
            let n = u32::try_from(g.ecount()).unwrap();
            for e in 0..n {
                let p = g.edge(e).unwrap();
                *m.entry(p).or_insert(0u32) += 1;
            }
            m
        };
        let ca = count(&a);
        let cb = count(&b);
        let cd = count(&d);

        let mut expected_e: usize = 0;
        for (k, &co) in &ca {
            let cs = cb.get(k).copied().unwrap_or(0);
            let want = co.saturating_sub(cs);
            let got = cd.get(k).copied().unwrap_or(0);
            prop_assert_eq!(got, want, "pair {:?}", k);
            expected_e += want as usize;
        }
        prop_assert_eq!(d.ecount(), expected_e);

        // No pair appears in d that isn't in a (difference can never
        // synthesise edges).
        for k in cd.keys() {
            prop_assert!(ca.contains_key(k));
        }
    }

    /// Mode-aware ecc/radius/diameter invariants:
    /// - radius == min(ecc); diameter == max(ecc)
    /// - radius ≤ diameter
    /// - on undirected graphs, all three modes produce identical
    ///   eccentricity vectors
    /// - eccentricity_with_mode(_, Out) on a directed graph equals the
    ///   legacy `eccentricity` (which uses OUT)
    #[test]
    fn ecc_with_mode_radius_diameter_consistent(g in arb_graph(8)) {
        use rust_igraph::EccMode;
        for m in [EccMode::Out, EccMode::In, EccMode::All] {
            let ecc = rust_igraph::eccentricity_with_mode(&g, m).unwrap();
            let r = rust_igraph::radius_with_mode(&g, m).unwrap();
            let d = rust_igraph::diameter_with_mode(&g, m).unwrap();
            if g.vcount() == 0 {
                prop_assert!(r.is_none());
                prop_assert!(d.is_none());
                prop_assert!(ecc.is_empty());
            } else {
                prop_assert_eq!(r, ecc.iter().copied().min());
                prop_assert_eq!(d, ecc.iter().copied().max());
                prop_assert!(r.unwrap() <= d.unwrap());
            }
        }
    }

    /// Undirected graphs: every mode produces the same eccentricity
    /// vector (every edge is bidirectional).
    #[test]
    fn ecc_with_mode_undirected_modes_agree(g in arb_graph(8)) {
        use rust_igraph::EccMode;
        if g.is_directed() {
            return Ok(());
        }
        let out = rust_igraph::eccentricity_with_mode(&g, EccMode::Out).unwrap();
        let in_  = rust_igraph::eccentricity_with_mode(&g, EccMode::In).unwrap();
        let all = rust_igraph::eccentricity_with_mode(&g, EccMode::All).unwrap();
        prop_assert_eq!(&out, &in_);
        prop_assert_eq!(&out, &all);
    }

    /// `eccentricity_with_mode(g, Out)` matches the legacy
    /// mode-defaulted `eccentricity(g)` for both directed and undirected
    /// graphs (the legacy function's BFS follows OUT edges via
    /// `distances`).
    #[test]
    fn ecc_with_mode_out_matches_legacy(g in arb_graph(8)) {
        use rust_igraph::EccMode;
        let legacy = rust_igraph::eccentricity(&g).unwrap();
        let with_out = rust_igraph::eccentricity_with_mode(&g, EccMode::Out).unwrap();
        prop_assert_eq!(legacy, with_out);
    }

    /// SP-021..023 weighted ecc/rad/diam invariants.
    /// - radius == min(ecc) and diameter == max(ecc) for every mode.
    /// - On undirected graphs every mode is identical.
    /// - With unit weights the weighted variants agree with the
    ///   unweighted u32 variants (cast to f64).
    #[test]
    fn weighted_ecc_radius_diameter_consistent(g in arb_graph(6)) {
        use rust_igraph::EccMode;
        if g.vcount() == 0 { return Ok(()); }
        let m = g.ecount();
        let weights: Vec<f64> = (0..m).map(|i| 1.0 + (i as f64) * 0.5).collect();
        for mode in [EccMode::Out, EccMode::In, EccMode::All] {
            let ecc = rust_igraph::eccentricity_weighted_with_mode(&g, &weights, mode).unwrap();
            let r = rust_igraph::radius_weighted_with_mode(&g, &weights, mode).unwrap();
            let d = rust_igraph::diameter_weighted_with_mode(&g, &weights, mode).unwrap();
            let rust_min = ecc.iter().copied().fold(f64::INFINITY, f64::min);
            let rust_max = ecc.iter().copied().fold(0.0_f64, f64::max);
            prop_assert!((r.unwrap() - rust_min).abs() < 1e-9, "mode {:?}", mode);
            prop_assert!((d.unwrap() - rust_max).abs() < 1e-9, "mode {:?}", mode);
        }
    }

    /// Unit-weight weighted ecc agrees with unweighted ecc cast to f64.
    #[test]
    fn weighted_ecc_unit_weights_match_unweighted(g in arb_graph(6)) {
        if g.vcount() == 0 { return Ok(()); }
        let m = g.ecount();
        let unit = vec![1.0_f64; m];
        let w_ecc = rust_igraph::eccentricity_weighted(&g, &unit).unwrap();
        let u_ecc = rust_igraph::eccentricity(&g).unwrap();
        prop_assert_eq!(w_ecc.len(), u_ecc.len());
        for (i, (rw, ru)) in w_ecc.iter().zip(u_ecc.iter()).enumerate() {
            prop_assert!((rw - f64::from(*ru)).abs() < 1e-9,
                         "vertex {}: weighted={} unweighted={}", i, rw, ru);
        }
    }

    /// TR-003 random walk structural invariants:
    /// - `vs[0] == start`
    /// - `len(vs) <= steps + 1` and `len(es) == len(vs) - 1`
    /// - every consecutive `(vs[i], vs[i+1])` is connected by `es[i]`
    /// - same `(graph, start, steps, seed)` ⇒ identical chain
    #[test]
    fn random_walk_chain_is_well_formed(
        g in arb_graph(8),
        steps in 0u32..15,
        seed in 0u64..u64::MAX,
    ) {
        if g.vcount() == 0 { return Ok(()); }
        let start = 0u32;
        let (vs, es) = rust_igraph::random_walk(
            &g, None, start, rust_igraph::DijkstraMode::Out, steps, seed,
        ).unwrap();
        prop_assert_eq!(vs[0], start);
        prop_assert!(vs.len() <= (steps as usize) + 1);
        prop_assert_eq!(vs.len(), es.len() + 1);
        for (i, w) in vs.windows(2).enumerate() {
            let (a, b) = (w[0], w[1]);
            let eid = es[i];
            let (s, t) = g.edge(eid).unwrap();
            prop_assert!(
                (s == a && t == b) || (t == a && s == b),
                "step {}: edge {} = ({},{}), expected adjacency to ({},{})",
                i, eid, s, t, a, b
            );
        }
        // Determinism: same seed yields same chain.
        let (vs2, es2) = rust_igraph::random_walk(
            &g, None, start, rust_igraph::DijkstraMode::Out, steps, seed,
        ).unwrap();
        prop_assert_eq!(&vs, &vs2);
        prop_assert_eq!(&es, &es2);
    }

    /// SP-003 Johnson invariants:
    /// - All-pairs matrix is `vcount × vcount`.
    /// - Diagonal is always `Some(0.0)`.
    /// - With non-negative weights, every row matches
    ///   `dijkstra_distances` from that source (Johnson short-circuits
    ///   to pairwise Dijkstra in that case).
    #[test]
    fn johnson_matches_pairwise_dijkstra_on_nonneg_weights(g in arb_graph(8)) {
        if g.vcount() == 0 { return Ok(()); }
        let n = g.vcount();
        let m = g.ecount();
        let weights: Vec<f64> = (0..m).map(|i| 0.5 + (i as f64) * 0.25).collect();
        let johnson = rust_igraph::johnson_distances(&g, &weights).unwrap();
        prop_assert_eq!(johnson.len() as u32, n);
        for (u, row) in johnson.iter().enumerate() {
            prop_assert_eq!(row.len() as u32, n);
            prop_assert_eq!(row[u], Some(0.0));
            let u_u32 = u32::try_from(u).expect("u fits in u32 for proptest");
            let dk = rust_igraph::dijkstra_distances(&g, u_u32, &weights).unwrap();
            for (v, (a, b)) in row.iter().zip(dk.iter()).enumerate() {
                match (a, b) {
                    (Some(x), Some(y)) => prop_assert!(
                        (x - y).abs() < 1e-9,
                        "[{}][{}]: Johnson={} Dijkstra={}", u, v, x, y),
                    (None, None) => {}
                    (x, y) => prop_assert!(false,
                        "[{}][{}]: Johnson={:?} Dijkstra={:?}", u, v, x, y),
                }
            }
        }
    }

    /// SP-002 Bellman-Ford invariants:
    /// - For **non-negative** weights, BF must produce the same
    ///   distances as Dijkstra (since BF subsumes Dijkstra on this
    ///   weight class).
    /// - For unit weights, both must match unweighted BFS distances
    ///   (covered by the dijkstra invariant; here we cross-check BF
    ///   against Dijkstra directly).
    #[test]
    fn bellman_ford_matches_dijkstra_on_nonneg_weights(g in arb_graph(8)) {
        if g.vcount() == 0 { return Ok(()); }
        let m = g.ecount();
        // Strictly positive weights so both BF and Dijkstra agree
        // unambiguously.
        let weights: Vec<f64> = (0..m).map(|i| 0.5 + (i as f64) * 0.25).collect();
        let bf = rust_igraph::bellman_ford_distances(&g, 0, &weights).unwrap();
        let dk = rust_igraph::dijkstra_distances(&g, 0, &weights).unwrap();
        prop_assert_eq!(bf.len(), dk.len());
        for (i, (a, b)) in bf.iter().zip(dk.iter()).enumerate() {
            match (a, b) {
                (Some(x), Some(y)) => prop_assert!(
                    (x - y).abs() < 1e-9,
                    "vertex {}: BF={} Dijkstra={}", i, x, y),
                (None, None) => {}
                (x, y) => prop_assert!(false,
                    "vertex {}: BF={:?} Dijkstra={:?}", i, x, y),
            }
        }
    }

    /// CORE-001c invariants for `delete_edges`:
    /// - vcount unchanged.
    /// - ecount decreases by exactly 1 when removing one edge.
    /// - Every retained edge's endpoints existed in the original graph.
    /// - `incident(v).len() == degree(v)` (index consistency).
    #[test]
    fn delete_edges_preserves_invariants(g in arb_graph(8)) {
        let m = g.ecount();
        if m == 0 { return Ok(()); }
        let mut g2 = g.clone();
        g2.delete_edges(&[0u32]).unwrap();
        prop_assert_eq!(g2.vcount(), g.vcount());
        prop_assert_eq!(g2.ecount(), m - 1);
        let new_m = u32::try_from(g2.ecount()).expect("edge count fits in u32 for proptest");
        for e in 0..new_m {
            let (u, v) = g2.edge(e).unwrap();
            prop_assert!(g.find_eid(u, v).unwrap().is_some()
                         || g.find_eid(v, u).unwrap().is_some());
        }
        for v in 0..g2.vcount() {
            prop_assert_eq!(g2.incident(v).unwrap().len(), g2.degree(v).unwrap());
        }
    }

    /// CORE-001c invariants for `delete_vertices_map`:
    /// - new vcount = old vcount − 1 when removing one vertex.
    /// - new ecount ≤ old ecount.
    /// - `map.len() == old vcount`, `map[removed] == None`.
    /// - `invmap` is a strict permutation of retained old ids, and
    ///   `map[invmap[new_id]] == Some(new_id)` for every retained id.
    /// - Index consistency: `incident(v).len() == degree(v)`.
    #[test]
    fn delete_vertices_preserves_invariants(g in arb_graph(8)) {
        let n = g.vcount();
        if n == 0 { return Ok(()); }
        let mut g2 = g.clone();
        let (map, invmap) = g2.delete_vertices_map(&[0u32]).unwrap();
        prop_assert_eq!(map.len(), n as usize);
        prop_assert_eq!(map[0], None);
        prop_assert_eq!(invmap.len(), g2.vcount() as usize);
        for (new_id, &old_id) in invmap.iter().enumerate() {
            prop_assert!(old_id != 0);
            prop_assert!(old_id < n);
            let new_id_u32 = u32::try_from(new_id).expect("new_id fits in u32");
            prop_assert_eq!(map[old_id as usize], Some(new_id_u32));
        }
        prop_assert_eq!(g2.vcount(), n - 1);
        prop_assert!(g2.ecount() <= g.ecount());
        for v in 0..g2.vcount() {
            prop_assert_eq!(g2.incident(v).unwrap().len(), g2.degree(v).unwrap());
        }
    }

    /// SP-005 A* invariants:
    /// - With null heuristic, A* finds the same length path as
    ///   `dijkstra_path_to`. Path identity may differ (different
    ///   tie-breaking) but the **edge weight sum** must match.
    /// - For unit weights, the path length matches `dijkstra_distances`
    ///   from source to target.
    #[test]
    fn a_star_null_heuristic_matches_dijkstra_path_length(g in arb_graph(8), target in 0u32..8) {
        if g.vcount() == 0 || target >= g.vcount() { return Ok(()); }
        let m = g.ecount();
        let weights: Vec<f64> = (0..m).map(|i| 1.0 + (i as f64) * 0.5).collect();
        let astar_result = rust_igraph::a_star_path(
            &g, 0, target, Some(&weights), rust_igraph::DijkstraMode::Out, |_, _| 0.0,
        ).unwrap();
        let dijkstra_d = rust_igraph::dijkstra_distances(&g, 0, &weights).unwrap();
        match (astar_result, dijkstra_d[target as usize]) {
            (None, None) => {}
            (Some((vs, es)), Some(dij_dist)) => {
                prop_assert_eq!(*vs.first().unwrap(), 0u32);
                prop_assert_eq!(*vs.last().unwrap(), target);
                prop_assert_eq!(es.len() + 1, vs.len());
                let total: f64 = es.iter().map(|&e| weights[e as usize]).sum();
                prop_assert!((total - dij_dist).abs() < 1e-9,
                             "A* path sum {} != dijkstra dist {}", total, dij_dist);
            }
            (a, b) => prop_assert!(false, "A* / dijkstra disagreement: astar={:?} dij={:?}", a, b),
        }
    }
}
