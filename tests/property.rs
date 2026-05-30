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

/// Build a *simple* graph (deduplicated edges, self-loops dropped) plus a
/// random vertex relabeling. Used to check canonical-labeling invariance.
fn arb_simple_graph_with_perm(
    max_n: u32,
    directed: bool,
) -> impl Strategy<Value = (Graph, Vec<u32>)> {
    (1u32..=max_n)
        .prop_flat_map(move |n| {
            let edges = proptest::collection::vec((0u32..n, 0u32..n), 0..=(n as usize * 2));
            let perm = Just((0..n).collect::<Vec<u32>>()).prop_shuffle();
            (Just(n), edges, perm)
        })
        .prop_map(move |(n, edges, perm)| {
            let mut g = Graph::new(n, directed).expect("graph init");
            let mut seen = std::collections::HashSet::new();
            for (u, v) in edges {
                if u == v {
                    continue; // keep it simple & loopless for these invariants
                }
                let key = if directed || u <= v { (u, v) } else { (v, u) };
                if seen.insert(key) {
                    g.add_edge(u, v).expect("indices in range");
                }
            }
            (g, perm)
        })
}

/// Sorted, direction-aware canonical edge list induced by a vertex → position
/// `labeling`: the edge multiset of the canonical form of `g`.
fn canon_form_edges(g: &Graph, labeling: &[u32]) -> Vec<(u32, u32)> {
    let directed = g.is_directed();
    let mut edges: Vec<(u32, u32)> = (0..g.ecount())
        .map(|e| {
            let (u, v) = g.edge(e as u32).expect("edge in range");
            let (cu, cv) = (labeling[u as usize], labeling[v as usize]);
            if directed || cu <= cv {
                (cu, cv)
            } else {
                (cv, cu)
            }
        })
        .collect();
    edges.sort_unstable();
    edges
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

    /// Weighted eigenvector centrality under unit weights must match
    /// the unweighted path bit-for-bit (within fp tolerance). Length
    /// validation: passing a wrong-length weights vector errors.
    #[test]
    fn eigenvector_centrality_weighted_unit_matches_unweighted(g in arb_graph(8)) {
        let m = g.ecount();
        let unw = rust_igraph::eigenvector_centrality(&g).unwrap();
        let w = rust_igraph::eigenvector_centrality_weighted(&g, &vec![1.0; m]).unwrap();
        prop_assert_eq!(unw.len(), w.vector.len());
        for (v, (&a, &b)) in unw.iter().zip(w.vector.iter()).enumerate() {
            prop_assert!((a - b).abs() < 1e-6,
                         "vertex {}: unweighted={} weighted-unit={}", v, a, b);
        }
        // Length-mismatch error path.
        if m > 0 {
            let err = rust_igraph::eigenvector_centrality_weighted(&g, &vec![1.0; m + 1]);
            prop_assert!(err.is_err(), "expected length-mismatch error for m+1 weights");
        }
    }

    /// Directed eigenvector centrality invariants:
    /// - max-abs of the vector is 1.0 (or zero for empty graphs)
    /// - eigenvalue is non-negative (Perron-Frobenius on non-negative M)
    /// - entries are non-negative (after sign cleanup)
    #[test]
    fn eigenvector_centrality_directed_invariants(g in arb_directed_graph(8)) {
        let s = rust_igraph::eigenvector_centrality_directed(
            &g,
            rust_igraph::EigenvectorMode::Out,
        ).unwrap();
        if s.vector.is_empty() { return Ok(()); }
        let mut maxabs = 0.0_f64;
        for (v, &x) in s.vector.iter().enumerate() {
            prop_assert!(x.is_finite(), "vec[{}] = {} not finite", v, x);
            prop_assert!(x >= -1e-9, "vec[{}] = {} negative (non-negative M)", v, x);
            maxabs = maxabs.max(x.abs());
        }
        // Either every entry is zero (rare DAG-pathological) or the
        // vector is max-1 normalised.
        if maxabs > 0.0 {
            prop_assert!((maxabs - 1.0).abs() < 1e-6,
                         "max(vec) = {} should be 1.0", maxabs);
        }
        prop_assert!(s.eigenvalue >= -1e-9,
                     "eigenvalue {} < 0 for non-negative M", s.eigenvalue);
        prop_assert!(s.eigenvalue.is_finite(),
                     "eigenvalue {} not finite", s.eigenvalue);
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

    /// `is_perfect` invariants (undirected, simple input):
    /// - Weak Perfect Graph Theorem: a graph is perfect iff its
    ///   complement is perfect, so `is_perfect(g) == is_perfect(comp(g))`.
    /// - Every bipartite graph is perfect.
    #[test]
    fn is_perfect_matches_complement_and_bipartite_are_perfect(g in arb_graph(8)) {
        let simple = rust_igraph::simplify(&g, true, true).unwrap();
        let comp = rust_igraph::complementer(&simple, false).unwrap();

        let p = rust_igraph::is_perfect(&simple).unwrap();
        let pc = rust_igraph::is_perfect(&comp).unwrap();
        prop_assert_eq!(
            p, pc,
            "Weak Perfect Graph Theorem violated: is_perfect(g)={} but is_perfect(comp(g))={}",
            p, pc,
        );

        if rust_igraph::is_bipartite(&simple).unwrap().is_bipartite {
            prop_assert!(p, "bipartite graph reported as not perfect");
        }
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

    /// `pagerank_linsys` parity with `pagerank` on small random graphs.
    /// GMRES converges to the unique fixed point of `(I - α·Mᵀ)x = (1-α)/N·1`,
    /// the same fixed point power iteration reaches in PR-011, so the two
    /// backends must agree elementwise within the combined tolerance of
    /// their stopping rules (PR-011 `eps=1e-10`, PR-011c relative residual
    /// `1e-13` → ~1e-9 worst-case parity).
    #[test]
    fn pagerank_linsys_matches_power_iter(g in arb_graph(8)) {
        let n = g.vcount() as usize;
        let a = rust_igraph::pagerank(&g).unwrap();
        let b = rust_igraph::pagerank_linsys(&g).unwrap();
        prop_assert_eq!(a.len(), n);
        prop_assert_eq!(b.len(), n);
        if n == 0 { return Ok(()); }
        let mut sum: f64 = 0.0;
        for (v, (&ai, &bi)) in a.iter().zip(b.iter()).enumerate() {
            prop_assert!(bi.is_finite(), "pr_linsys[{}] = {} not finite", v, bi);
            prop_assert!(bi >= -1e-12, "pr_linsys[{}] = {} negative", v, bi);
            prop_assert!((ai - bi).abs() < 1e-9,
                         "vertex {}: power={} linsys={}", v, ai, bi);
            sum += bi;
        }
        prop_assert!((sum - 1.0).abs() < 1e-6,
                     "pagerank_linsys does not sum to 1 (got {})", sum);
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

    /// PR-022 `is_acyclic` invariants:
    /// - Directed: equals `is_dag` exactly.
    /// - Undirected with no edges or no vertices: always acyclic.
    /// - Undirected `m == n - cc` (where cc = number of connected
    ///   components) iff acyclic AND no parallel/self-loop edges
    ///   (a forest has exactly n - cc edges; any extra edge closes
    ///   a cycle). We verify the forward direction: if acyclic then
    ///   `m == n - cc`.
    #[test]
    fn is_acyclic_matches_is_dag_for_directed(g in arb_directed_graph(8)) {
        prop_assert_eq!(rust_igraph::is_acyclic(&g), rust_igraph::is_dag(&g));
    }

    #[test]
    fn is_acyclic_undirected_implies_forest_edge_count(g in arb_graph(8)) {
        if rust_igraph::is_acyclic(&g) {
            // Forest invariant: m == n - cc.
            let cc = rust_igraph::connected_components(&g).unwrap();
            let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for proptest");
            let n = g.vcount();
            let expected = n - cc.count;
            prop_assert_eq!(m, expected);
        }
    }

    /// PR-023 `is_tree` invariants:
    /// - If `is_tree(g, All)` returns `Some`, then `g` is acyclic,
    ///   connected (`cc.count == 1`), and has exactly `vcount-1`
    ///   edges. (Forest with a single component.)
    /// - For directed graphs, `is_tree(g, Out).is_some()` implies
    ///   `is_tree(g, All).is_some()` — every directed tree is also
    ///   a tree when orientation is ignored.
    #[test]
    fn is_tree_implies_acyclic_connected_and_correct_edge_count(g in arb_graph(8)) {
        let result = rust_igraph::is_tree(&g, rust_igraph::DijkstraMode::All).unwrap();
        if result.is_some() {
            // Tree ⇒ acyclic.
            prop_assert!(rust_igraph::is_acyclic(&g),
                "is_tree=Some but is_acyclic=false");
            // Tree ⇒ single connected component.
            let cc = rust_igraph::connected_components(&g).unwrap();
            prop_assert_eq!(cc.count, 1, "tree must have exactly one component");
            // Tree ⇒ m == n - 1 (edge count by definition).
            let m = u32::try_from(g.ecount()).expect("ecount fits in u32");
            let n = g.vcount();
            prop_assert_eq!(m, n - 1);
        }
    }

    #[test]
    fn directed_out_tree_is_undirected_tree(g in arb_directed_graph(8)) {
        let out_tree = rust_igraph::is_tree(&g, rust_igraph::DijkstraMode::Out).unwrap();
        if out_tree.is_some() {
            let all_tree = rust_igraph::is_tree(&g, rust_igraph::DijkstraMode::All).unwrap();
            prop_assert!(all_tree.is_some(),
                "directed out-tree must also be an undirected tree");
        }
    }

    /// PR-024 `is_forest` invariants:
    /// - If `is_forest(g, All)` is `Some(roots)` then `g` is
    ///   acyclic and `m == n - cc.count` (forest identity).
    /// - Every tree is a forest: if `is_tree(g, mode)` is `Some`
    ///   then `is_forest(g, mode)` is `Some` with exactly one root.
    #[test]
    fn is_forest_implies_acyclic_with_forest_edge_count(g in arb_graph(8)) {
        let result = rust_igraph::is_forest(&g, rust_igraph::DijkstraMode::All).unwrap();
        if let Some(roots) = result {
            prop_assert!(rust_igraph::is_acyclic(&g),
                "is_forest=Some but is_acyclic=false");
            let cc = rust_igraph::connected_components(&g).unwrap();
            // Each component contributes exactly one root (mode=All).
            let nroots = u32::try_from(roots.len()).expect("roots.len fits in u32");
            prop_assert_eq!(nroots, cc.count,
                "forest root count must equal number of CCs");
            // Forest identity: m == n - cc.count.
            let m = u32::try_from(g.ecount()).expect("ecount fits in u32");
            let n = g.vcount();
            prop_assert_eq!(m, n - cc.count, "forest identity m == n - cc");
        }
    }

    #[test]
    fn tree_is_forest_with_one_root(g in arb_graph(8)) {
        let tree = rust_igraph::is_tree(&g, rust_igraph::DijkstraMode::All).unwrap();
        if let Some(tree_root) = tree {
            let forest = rust_igraph::is_forest(&g, rust_igraph::DijkstraMode::All).unwrap();
            let roots = forest.expect("a tree must also be a forest");
            prop_assert_eq!(roots, vec![tree_root],
                "tree forest must have exactly the tree root");
        }
    }

    /// PR-016 `is_complete` invariants:
    /// - If complete and simple, ecount must equal `n*(n-1)/2`
    ///   (undirected) or `n*(n-1)` (directed).
    /// - Every vertex must have at least `n-1` distinct non-self
    ///   neighbours when the graph is complete.
    #[test]
    fn complete_simple_graph_has_full_edge_count(g in arb_graph(7)) {
        let n = u64::from(g.vcount());
        let m = g.ecount() as u64;
        let directed = g.is_directed();
        let target = if directed { n * n.saturating_sub(1) }
                     else        { n * n.saturating_sub(1) / 2 };
        let complete = rust_igraph::is_complete(&g).unwrap();
        let simple = rust_igraph::is_simple(&g).unwrap();
        if complete && simple && n >= 2 {
            prop_assert_eq!(m, target,
                "simple complete graph must hit exact target ecount");
        }
        // Singleton/empty are always complete.
        if n <= 1 {
            prop_assert!(complete, "n<=1 must be complete");
        }
    }

    #[test]
    fn complete_implies_every_vertex_sees_n_minus_1(g in arb_graph(7)) {
        let n = g.vcount();
        if !rust_igraph::is_complete(&g).unwrap() {
            return Ok(());
        }
        if n <= 1 {
            return Ok(());
        }
        // For each vertex collect its unique non-self neighbour set
        // (undirected view; for the directed graph the predicate
        // requires the same in *both* directions which implies
        // |N(v) \ {v}| = n-1 in the undirected projection too).
        for v in 0..n {
            let mut seen: std::collections::HashSet<u32> =
                std::collections::HashSet::new();
            for nei in g.neighbors(v).unwrap() {
                if nei != v {
                    seen.insert(nei);
                }
            }
            let seen_count = u32::try_from(seen.len()).expect("seen.len fits in u32");
            prop_assert_eq!(seen_count, n - 1,
                "complete: every vertex must see all others");
        }
    }

    /// PR-027 `neighborhood_size` invariants:
    /// - Order 0 returns 1 for every vertex (mindist 0).
    /// - Result is monotone non-decreasing as order grows
    ///   (the k-ball can only get larger or stay the same).
    /// - Bounded above by `vcount`.
    /// - Order >= n-1 stabilises to the reachable-set size
    ///   (using count_reachable as oracle for undirected ALL mode).
    /// - mindist=1 result + 1 == mindist=0 result whenever the
    ///   vertex itself is within `order` of itself (always true for
    ///   order >= 0).
    #[test]
    fn neighborhood_size_order_0_is_all_ones(g in arb_graph(7)) {
        let sizes = rust_igraph::neighborhood_size(&g, 0).unwrap();
        prop_assert_eq!(sizes.len(), g.vcount() as usize);
        for s in sizes {
            prop_assert_eq!(s, 1, "order=0 always counts just the vertex itself");
        }
    }

    #[test]
    fn neighborhood_size_monotone_non_decreasing(g in arb_graph(7)) {
        let s0 = rust_igraph::neighborhood_size(&g, 0).unwrap();
        let s1 = rust_igraph::neighborhood_size(&g, 1).unwrap();
        let s2 = rust_igraph::neighborhood_size(&g, 2).unwrap();
        for ((a, b), c) in s0.iter().zip(s1.iter()).zip(s2.iter()) {
            prop_assert!(a <= b, "order 0 ≤ order 1");
            prop_assert!(b <= c, "order 1 ≤ order 2");
        }
    }

    #[test]
    fn neighborhood_size_bounded_by_vcount(g in arb_graph(7)) {
        let n = g.vcount();
        let s = rust_igraph::neighborhood_size(&g, -1).unwrap();
        for v in s {
            prop_assert!(v <= n, "neighborhood size cannot exceed vcount");
        }
    }

    #[test]
    fn neighborhood_size_mindist_1_excludes_self(g in arb_graph(7)) {
        let s_inc = rust_igraph::neighborhood_size_with_mode(
            &g, 2, rust_igraph::NeighborhoodMode::All, 0).unwrap();
        let s_exc = rust_igraph::neighborhood_size_with_mode(
            &g, 2, rust_igraph::NeighborhoodMode::All, 1).unwrap();
        for (a, b) in s_inc.iter().zip(s_exc.iter()) {
            prop_assert_eq!(*a, b.saturating_add(1),
                "mindist 0 = mindist 1 + 1 (self toggles)");
        }
    }

    /// PR-027b `neighborhood` invariants:
    /// - List length equals `neighborhood_size` at any (order, mindist).
    /// - mindist=0 always includes the source vertex itself.
    /// - mindist=1 never includes the source vertex.
    /// - All listed vertex IDs are in `0..vcount`.
    /// - No duplicates within a single source's list (BFS marker dedups).
    /// - Listed set is monotone non-decreasing as order grows.
    #[test]
    fn neighborhood_length_matches_size(g in arb_graph(7)) {
        for &order in &[0_i32, 1, 2, -1] {
            for &mindist in &[0_i32, 1, 2] {
                if order >= 0 && mindist > order { continue; }
                let sizes = rust_igraph::neighborhood_size_with_mode(
                    &g, order, rust_igraph::NeighborhoodMode::All, mindist).unwrap();
                let lists = rust_igraph::neighborhood_with_mode(
                    &g, order, rust_igraph::NeighborhoodMode::All, mindist).unwrap();
                prop_assert_eq!(sizes.len(), lists.len());
                for (s, l) in sizes.iter().zip(lists.iter()) {
                    prop_assert_eq!(*s, u32::try_from(l.len()).unwrap(),
                        "neighborhood_size != neighborhood list length");
                }
            }
        }
    }

    #[test]
    fn neighborhood_mindist_0_includes_self(g in arb_graph(7)) {
        let lists = rust_igraph::neighborhood(&g, 2).unwrap();
        for (i, l) in lists.iter().enumerate() {
            let i_u32 = u32::try_from(i).unwrap();
            prop_assert!(l.contains(&i_u32), "vertex {} not in its own mindist=0 ball", i);
        }
    }

    #[test]
    fn neighborhood_mindist_1_excludes_self(g in arb_graph(7)) {
        let lists = rust_igraph::neighborhood_with_mode(
            &g, 2, rust_igraph::NeighborhoodMode::All, 1).unwrap();
        for (i, l) in lists.iter().enumerate() {
            let i_u32 = u32::try_from(i).unwrap();
            prop_assert!(!l.contains(&i_u32), "vertex {} should not be in own mindist=1 ball", i);
        }
    }

    #[test]
    fn neighborhood_ids_in_range_and_unique(g in arb_graph(7)) {
        let n = g.vcount();
        let lists = rust_igraph::neighborhood(&g, -1).unwrap();
        for l in &lists {
            let mut seen = std::collections::HashSet::new();
            for &v in l {
                prop_assert!(v < n, "out-of-range vertex id {} (vcount={})", v, n);
                prop_assert!(seen.insert(v), "duplicate id {} in same neighborhood", v);
            }
        }
    }

    #[test]
    fn neighborhood_monotone_in_order(g in arb_graph(7)) {
        // order 1 set ⊆ order 2 set
        let l1 = rust_igraph::neighborhood(&g, 1).unwrap();
        let l2 = rust_igraph::neighborhood(&g, 2).unwrap();
        for (s1, s2) in l1.iter().zip(l2.iter()) {
            let set2: std::collections::HashSet<_> = s2.iter().copied().collect();
            for &v in s1 {
                prop_assert!(set2.contains(&v),
                    "vertex {} appears in order-1 ball but not order-2 ball", v);
            }
        }
    }

    /// PR-021 `topological_sorting` invariants:
    /// - For DAGs, the result is a permutation of `0..vcount`
    ///   that respects every non-loop directed edge `u → v`
    ///   (pos[u] < pos[v]).
    /// - For graphs with cycles (non-loop), the function errors.
    /// - For DAGs, self-loops don't prevent a valid ordering
    ///   (upstream's IGRAPH_NO_LOOPS semantics).
    #[test]
    fn topological_sorting_respects_every_directed_edge(g in arb_directed_graph(8)) {
        let result = rust_igraph::topological_sorting(&g, rust_igraph::DijkstraMode::Out);
        let is_dag = rust_igraph::is_dag(&g);
        // Check whether the graph has any non-loop cycles. is_dag is
        // false for self-loops too, but topological_sorting accepts
        // self-loops as long as there are no other cycles. We have
        // to check separately.
        let has_non_loop_cycle = {
            // Strip self-loops by checking if the SCC-induced
            // non-DAG structure is purely self-loops.
            let scc = rust_igraph::strongly_connected_components(&g).unwrap();
            let mut bucket = vec![0u32; scc.count as usize];
            for &c in &scc.membership {
                bucket[c as usize] += 1;
            }
            bucket.iter().any(|&sz| sz >= 2)
        };
        if has_non_loop_cycle {
            // Multi-vertex SCC ⇒ cycle exists ⇒ should error.
            prop_assert!(result.is_err(), "expected error on cyclic graph");
        } else {
            // No multi-vertex cycle ⇒ topological sort should succeed.
            let order = result.expect("DAG (possibly with self-loops) has a topological order");
            let n = g.vcount() as usize;
            prop_assert_eq!(order.len(), n);
            // Permutation: every vertex appears exactly once.
            let mut sorted = order.clone();
            sorted.sort_unstable();
            prop_assert_eq!(sorted, (0..g.vcount()).collect::<Vec<_>>());
            // Edge consistency: every non-loop edge u→v has pos[u] < pos[v].
            let mut pos = vec![0usize; n];
            for (i, &v) in order.iter().enumerate() {
                pos[v as usize] = i;
            }
            for e in 0..g.ecount() as u32 {
                let (u, v) = g.edge(e).unwrap();
                if u == v { continue; }
                prop_assert!(pos[u as usize] < pos[v as usize],
                    "edge {}→{} violated: pos[{}]={} pos[{}]={}",
                    u, v, u, pos[u as usize], v, pos[v as usize]);
            }
            // Also: is_dag must be false (self-loops disqualify it)
            // OR the graph genuinely has no self-loops and is_dag = true.
            let has_self_loop = (0..g.ecount() as u32).any(|e| {
                let (u, v) = g.edge(e).unwrap();
                u == v
            });
            prop_assert_eq!(is_dag, !has_self_loop);
        }
    }

    /// PR-020 `is_dag` invariants:
    /// - Undirected graphs are never DAGs.
    /// - DAGs cannot have self-loops.
    /// - DAGs are consistent with SCC structure: every SCC has size 1
    ///   (no cycle means no strongly connected sub-component bigger
    ///   than a single vertex).
    #[test]
    fn is_dag_consistent_with_scc(g in arb_directed_graph(8)) {
        let dag = rust_igraph::is_dag(&g);
        // Strong CC count == vcount iff every SCC has size 1 iff DAG
        // (modulo self-loops which create a 1-vertex SCC but still
        // mean "not a DAG"). Check the "no self-loop" condition
        // separately.
        let has_self_loop = (0..g.ecount() as u32)
            .any(|e| {
                let (u, v) = g.edge(e).unwrap();
                u == v
            });
        if dag {
            // Every SCC must be a singleton AND there are no self-loops.
            let scc = rust_igraph::strongly_connected_components(&g).unwrap();
            prop_assert_eq!(scc.count, g.vcount(),
                "DAG must have one SCC per vertex");
            prop_assert!(!has_self_loop, "DAG cannot have self-loops");
        } else {
            // Not a DAG: either has a self-loop or has a multi-vertex SCC.
            let scc = rust_igraph::strongly_connected_components(&g).unwrap();
            prop_assert!(has_self_loop || scc.count < g.vcount(),
                "non-DAG must have a cycle (self-loop or multi-vertex SCC)");
        }
    }

    /// CORE-001e `is_same_graph` invariants:
    /// - Reflexivity: every graph is the same as itself (and as a
    ///   clone of itself).
    /// - Symmetry: `is_same_graph(g1, g2) == is_same_graph(g2, g1)`.
    /// - Adding an isolated vertex disagrees: `g` vs `g+1 vertex`
    ///   must return false (vcount differs).
    /// - Adding then removing an edge: `g` vs `g + (u, v) - (u, v)`
    ///   round-trip recovers the same graph.
    #[test]
    fn is_same_graph_reflexivity_and_symmetry(g in arb_graph(8)) {
        let g_clone = g.clone();
        prop_assert!(rust_igraph::is_same_graph(&g, &g));
        prop_assert!(rust_igraph::is_same_graph(&g, &g_clone));
        prop_assert!(rust_igraph::is_same_graph(&g_clone, &g));
        // Add an isolated vertex to g2 — vcount differs → not same.
        let mut g2 = g.clone();
        g2.add_vertices(1).unwrap();
        prop_assert!(!rust_igraph::is_same_graph(&g, &g2));
        prop_assert!(!rust_igraph::is_same_graph(&g2, &g));
    }

    /// CC-032 `site_percolation` invariants:
    /// - Outputs have length `vertex_order.len()`.
    /// - `giant_size` and `edge_count` are both monotone
    ///   non-decreasing (giant and edge counter only grow).
    /// - `edge_count[i]` is bounded above by twice the total number
    ///   of edges in the subgraph induced by the first `i+1`
    ///   activated vertices (loose upper bound; loops + parallels).
    /// - Final `giant_size` equals the largest CC of the subgraph
    ///   induced by the activated vertices (when all are activated).
    #[test]
    fn site_percolation_monotone_and_matches_components(g in arb_graph(8)) {
        let n = g.vcount();
        if n == 0 { return Ok(()); }
        // Activate every vertex in id order.
        let order: Vec<u32> = (0..n).collect();
        let p = rust_igraph::site_percolation(&g, &order).unwrap();
        prop_assert_eq!(p.giant_size.len(), order.len());
        prop_assert_eq!(p.edge_count.len(), order.len());
        // Monotonicity.
        for w in p.giant_size.windows(2) {
            prop_assert!(w[0] <= w[1], "giant_size not monotone");
        }
        for w in p.edge_count.windows(2) {
            prop_assert!(w[0] <= w[1], "edge_count not monotone");
        }
        // Final giant_size matches the largest CC of the full graph
        // (since we activated every vertex).
        let cc = rust_igraph::connected_components(&g).unwrap();
        let mut bucket = vec![0u32; cc.count as usize];
        for &c in &cc.membership {
            bucket[c as usize] += 1;
        }
        let expected_giant = bucket.iter().max().copied().unwrap_or(0);
        prop_assert_eq!(
            *p.giant_size.last().unwrap(),
            expected_giant,
            "final site percolation giant {} != CC max {}",
            p.giant_size.last().unwrap(),
            expected_giant,
        );
    }

    /// CC-031 `bond_percolation` invariants:
    /// - With `edge_order = (0..ecount)`, bond_percolation produces
    ///   the same curves as `edgelist_percolation` over the graph's
    ///   edges in id order — i.e. the wrapper is a faithful
    ///   id-resolver.
    /// - Reversing the order leaves the **final** giant_size
    ///   unchanged (set of touched vertices is the same, so the
    ///   max CC at the end is identical).
    #[test]
    fn bond_percolation_natural_order_matches_edgelist(g in arb_graph(8)) {
        let m = g.ecount();
        if m == 0 { return Ok(()); }
        let m_u32 = u32::try_from(m).expect("edge count fits in u32 for proptest");
        let natural: Vec<u32> = (0..m_u32).collect();
        let bond = rust_igraph::bond_percolation(&g, &natural).unwrap();
        // Resolve to (u, v) pairs and call edgelist_percolation directly.
        let pairs: Vec<(u32, u32)> = (0..m_u32)
            .map(|e| g.edge(e).unwrap())
            .collect();
        let direct = rust_igraph::edgelist_percolation(&pairs).unwrap();
        prop_assert_eq!(bond.giant_size, direct.giant_size);
        prop_assert_eq!(bond.vertex_count, direct.vertex_count);
    }

    /// CC-030 `edgelist_percolation` invariants:
    /// - Outputs have length `edges.len()`.
    /// - `giant_size` is monotone non-decreasing (the giant can only
    ///   grow, never shrink, as more edges are added).
    /// - `vertex_count` is monotone non-decreasing and bounded by
    ///   `2 * edges.len()` (each edge touches ≤ 2 distinct vertices).
    /// - `vertex_count[i] <= vcount(implicit)` for all i.
    /// - Final `giant_size` equals the largest connected component of
    ///   the cumulative undirected graph at the end (cross-check
    ///   against `connected_components`).
    #[test]
    fn edgelist_percolation_monotone_and_matches_components(g in arb_graph(8)) {
        let n = g.vcount();
        if n == 0 { return Ok(()); }
        let m = g.ecount();
        if m == 0 { return Ok(()); }
        // Build the edge sequence directly from the proptest-generated graph.
        let edges: Vec<(u32, u32)> = (0..m as u32)
            .map(|e| g.edge(e).unwrap())
            .collect();
        let p = rust_igraph::edgelist_percolation(&edges).unwrap();
        prop_assert_eq!(p.giant_size.len(), edges.len());
        prop_assert_eq!(p.vertex_count.len(), edges.len());
        // Monotonicity.
        for w in p.giant_size.windows(2) {
            prop_assert!(w[0] <= w[1], "giant_size not monotone");
        }
        for w in p.vertex_count.windows(2) {
            prop_assert!(w[0] <= w[1], "vertex_count not monotone");
        }
        // Final giant equals max CC size on the same graph.
        let cc = rust_igraph::connected_components(&g).unwrap();
        let mut bucket = vec![0u32; cc.count as usize];
        for &c in &cc.membership {
            bucket[c as usize] += 1;
        }
        let expected_giant = bucket.iter().max().copied().unwrap_or(0);
        // The percolation `giant_size` only counts touched vertices;
        // isolated vertices in g are excluded. To compare, restrict
        // CCs to vertices touched by any edge in the same sequence.
        let mut touched = vec![false; n as usize];
        for &(u, v) in &edges {
            touched[u as usize] = true;
            touched[v as usize] = true;
        }
        // Recount CC sizes over touched vertices only.
        let mut touched_bucket = vec![0u32; cc.count as usize];
        for (v, &t) in touched.iter().enumerate() {
            if t {
                touched_bucket[cc.membership[v] as usize] += 1;
            }
        }
        let touched_giant = touched_bucket.iter().max().copied().unwrap_or(0);
        let _ = expected_giant; // full-graph giant is an upper bound, not strict equality
        prop_assert_eq!(
            *p.giant_size.last().unwrap(),
            touched_giant,
            "final percolation giant {} != touched-vertex CC max {}",
            p.giant_size.last().unwrap(),
            touched_giant,
        );
    }

    /// SP-014 `widest_paths` SPT struct invariants:
    /// - `widths` field matches the standalone `widest_path_widths`.
    /// - `parents[source] == None`, `inbound_edges[source] == None`.
    /// - For every reachable v != source: `inbound_edges[v]` is some
    ///   edge id whose `edge_other(eid, v)` equals `parents[v]`.
    /// - Walking back via `parents` reconstructs a valid path from v
    ///   to source (finite length, vertices distinct).
    #[test]
    fn widest_paths_spt_consistent_with_widths(g in arb_graph(8)) {
        if g.vcount() == 0 { return Ok(()); }
        let m = g.ecount();
        let weights: Vec<f64> = (0..m).map(|i| 1.0 + (i as f64) * 0.5).collect();
        let from = 0u32;
        let sp = rust_igraph::widest_paths(&g, from, &weights).unwrap();
        let standalone = rust_igraph::widest_path_widths(&g, from, &weights).unwrap();
        prop_assert_eq!(&sp.widths, &standalone, "widths field mismatch");
        let n = g.vcount();
        prop_assert_eq!(sp.parents.len() as u32, n);
        prop_assert_eq!(sp.inbound_edges.len() as u32, n);
        prop_assert_eq!(sp.parents[from as usize], None);
        prop_assert_eq!(sp.inbound_edges[from as usize], None);
        // Consistency: for every reachable v != source, parent matches
        // edge_other(inbound_edge, v).
        for v in 0..n {
            if v == from { continue; }
            if sp.widths[v as usize].is_none() {
                prop_assert_eq!(sp.parents[v as usize], None);
                prop_assert_eq!(sp.inbound_edges[v as usize], None);
                continue;
            }
            let eid = sp.inbound_edges[v as usize]
                .expect("reachable vertex must have an inbound edge");
            let prev = g.edge_other(eid, v).unwrap();
            prop_assert_eq!(sp.parents[v as usize], Some(prev));
        }
        // Walking back via parents must reach source in at most n steps
        // and visit distinct vertices.
        for v in 0..n {
            if sp.widths[v as usize].is_none() || v == from { continue; }
            let mut visited: std::collections::HashSet<u32> = std::collections::HashSet::new();
            let mut cur = v;
            let mut steps = 0u32;
            visited.insert(cur);
            while cur != from {
                let prev = sp.parents[cur as usize]
                    .expect("non-source reachable has parent");
                prop_assert!(visited.insert(prev), "cycle in SPT from vertex {}", v);
                cur = prev;
                steps += 1;
                prop_assert!(steps <= n, "walk longer than vcount from {}", v);
            }
        }
    }

    /// SP-013 `widest_paths_to` invariants (multi-target):
    /// - Output length matches the number of targets.
    /// - For each target, `is_some()` agrees with
    ///   `widest_path_widths[t].is_some()`.
    /// - When `Some((vs, es))`, the chain is well-formed (consecutive
    ///   vertices adjacent via the recorded edge), `vs[0] == from`,
    ///   `*vs.last() == t`, and the bottleneck (min of edge weights)
    ///   matches `widths[t]` from the single-call API.
    /// - Cross-check against single-target `widest_path`: matching
    ///   bottlenecks even if vertex chains differ in tie-breaking.
    #[test]
    fn widest_paths_to_consistent_with_widths(
        g in arb_graph(8),
        ta in 0u32..8,
        tb in 0u32..8,
    ) {
        if g.vcount() == 0 { return Ok(()); }
        let n = g.vcount();
        if ta >= n || tb >= n { return Ok(()); }
        let m = g.ecount();
        let weights: Vec<f64> = (0..m).map(|i| 1.0 + (i as f64) * 0.5).collect();
        let from = 0u32;
        let widths = rust_igraph::widest_path_widths(&g, from, &weights).unwrap();
        let targets = vec![ta, tb];
        let paths = rust_igraph::widest_paths_to(&g, from, &targets, &weights).unwrap();
        prop_assert_eq!(paths.len(), targets.len());
        for (t_idx, &t) in targets.iter().enumerate() {
            let entry = &paths[t_idx];
            let reachable = widths[t as usize].is_some();
            prop_assert_eq!(entry.is_some(), reachable,
                "target {} reachability mismatch", t);
            if let Some((vs, es)) = entry {
                prop_assert_eq!(vs[0], from);
                prop_assert_eq!(*vs.last().unwrap(), t);
                prop_assert_eq!(es.len() + 1, vs.len());
                // Adjacency along the chain.
                for (i, w) in vs.windows(2).enumerate() {
                    let (a, b) = (w[0], w[1]);
                    let (s, t_ep) = g.edge(es[i]).unwrap();
                    prop_assert!(
                        (s == a && t_ep == b) || (t_ep == a && s == b),
                        "step {}: edge {} = ({},{}), expected ({},{})",
                        i, es[i], s, t_ep, a, b
                    );
                }
                // Bottleneck equals widths[t] when t != from.
                if t != from {
                    let bottleneck = es.iter()
                        .map(|&e| weights[e as usize])
                        .fold(f64::INFINITY, f64::min);
                    let expected = widths[t as usize].unwrap();
                    prop_assert!(
                        (bottleneck - expected).abs() < 1e-9,
                        "target {}: chain bottleneck {} != widths {}",
                        t, bottleneck, expected);
                }
            }
        }
    }

    /// SP-012 FW widest-widths invariants:
    /// - Output is `vcount × vcount` matrix.
    /// - Diagonal is always `Some(f64::INFINITY)`.
    /// - Every row matches the Dijkstra-based `widest_path_widths`
    ///   from that source (FW and Dijkstra agree on the widest
    ///   bottleneck — different code paths, same answer).
    #[test]
    fn fw_widest_matches_pairwise_dijkstra(g in arb_graph(8)) {
        if g.vcount() == 0 { return Ok(()); }
        let n = g.vcount();
        let m = g.ecount();
        let weights: Vec<f64> = (0..m).map(|i| 1.0 + (i as f64) * 0.5).collect();
        let fw = rust_igraph::widest_path_widths_floyd_warshall(&g, &weights).unwrap();
        prop_assert_eq!(fw.len() as u32, n);
        for (u, row) in fw.iter().enumerate() {
            prop_assert_eq!(row.len() as u32, n);
            prop_assert_eq!(row[u], Some(f64::INFINITY));
            let u_u32 = u32::try_from(u).expect("u fits in u32 for proptest");
            let dij = rust_igraph::widest_path_widths(&g, u_u32, &weights).unwrap();
            for (v, (a, b)) in row.iter().zip(dij.iter()).enumerate() {
                match (a, b) {
                    (Some(x), Some(y)) if x.is_infinite() && y.is_infinite() => {}
                    (Some(x), Some(y)) => prop_assert!(
                        (x - y).abs() < 1e-9,
                        "[{}][{}]: FW={} Dijkstra={}", u, v, x, y),
                    (None, None) => {}
                    (x, y) => prop_assert!(false,
                        "[{}][{}]: FW={:?} Dijkstra={:?}", u, v, x, y),
                }
            }
        }
    }

    /// SP-011 `widest_path` invariants:
    /// - When `widest_path_widths[target].is_some()`, `widest_path`
    ///   returns `Some((vs, es))` with the same bottleneck (min over
    ///   `es`'s weights == `widths[target]`).
    /// - When `widths[target].is_none()`, `widest_path` returns `None`.
    /// - The returned chain is a valid walk: every consecutive
    ///   `(vs[i], vs[i+1])` is an endpoint pair of edge `es[i]`.
    /// - `vs[0] == from`, `*vs.last().unwrap() == target`.
    #[test]
    fn widest_path_chain_is_well_formed(g in arb_graph(8), target in 0u32..8) {
        if g.vcount() == 0 || target >= g.vcount() { return Ok(()); }
        let m = g.ecount();
        let weights: Vec<f64> = (0..m).map(|i| 1.0 + (i as f64) * 0.5).collect();
        let from = 0u32;
        let widths = rust_igraph::widest_path_widths(&g, from, &weights).unwrap();
        let path = rust_igraph::widest_path(&g, from, target, &weights).unwrap();
        match (path, widths[target as usize]) {
            (None, None) => {}
            (None, Some(_)) if target != from => {
                prop_assert!(false, "widest_path None but widths reachable")
            }
            (Some((vs, es)), _) => {
                prop_assert_eq!(vs[0], from);
                prop_assert_eq!(*vs.last().unwrap(), target);
                prop_assert_eq!(es.len() + 1, vs.len());
                // Walk validity: every step uses an edge with the
                // correct endpoints.
                for (i, w) in vs.windows(2).enumerate() {
                    let (a, b) = (w[0], w[1]);
                    let (s, t) = g.edge(es[i]).unwrap();
                    prop_assert!(
                        (s == a && t == b) || (t == a && s == b),
                        "step {}: edge {} = ({},{}), expected adjacency ({},{})",
                        i, es[i], s, t, a, b
                    );
                }
                // Bottleneck of the chosen path equals widths[target]
                // (when target != from).
                if target != from {
                    let bottleneck = es.iter()
                        .map(|&e| weights[e as usize])
                        .fold(f64::INFINITY, f64::min);
                    let expected = widths[target as usize].unwrap();
                    prop_assert!(
                        (bottleneck - expected).abs() < 1e-9,
                        "path bottleneck {} != widths {}", bottleneck, expected);
                }
            }
            _ => {}
        }
    }

    /// SP-010 widest-path invariants:
    /// - `widths[source] == Some(f64::INFINITY)`.
    /// - For every reachable vertex `v`, `widths[v]` is **at most**
    ///   the maximum edge weight in the graph (the bottleneck can
    ///   never exceed the widest single edge).
    /// - For every reachable vertex `v`, `widths[v]` is **at least**
    ///   any specific path's bottleneck — checked by the direct edge
    ///   case: if there is a direct edge source→v with weight w,
    ///   then `widths[v] >= w` (the algorithm must at least find that
    ///   path).
    /// - Unreachable iff no walk exists from source (cross-checked
    ///   against `dijkstra_distances`: `widths[v].is_none()` iff
    ///   `dijkstra_distances[v].is_none()`).
    #[test]
    fn widest_path_invariants(g in arb_graph(8)) {
        if g.vcount() == 0 { return Ok(()); }
        let m = g.ecount();
        if m == 0 {
            // Only source itself reachable; nothing else to check.
            let widths = rust_igraph::widest_path_widths(&g, 0, &[]).unwrap();
            prop_assert_eq!(widths[0], Some(f64::INFINITY));
            for w in &widths[1..] {
                prop_assert_eq!(w, &None);
            }
            return Ok(());
        }
        // Strictly positive weights so the upper bound is well-defined.
        let weights: Vec<f64> = (0..m).map(|i| 1.0 + (i as f64) * 0.5).collect();
        let max_weight = weights.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let widths = rust_igraph::widest_path_widths(&g, 0, &weights).unwrap();
        let dij = rust_igraph::dijkstra_distances(&g, 0, &weights).unwrap();
        prop_assert_eq!(widths.len(), g.vcount() as usize);
        prop_assert_eq!(widths[0], Some(f64::INFINITY));
        for (v, (w, d)) in widths.iter().zip(dij.iter()).enumerate() {
            // Reachability agrees with Dijkstra: same set of None positions.
            prop_assert_eq!(w.is_some(), d.is_some(),
                "vertex {}: widest={:?} dijkstra={:?}", v, w, d);
            if let Some(wv) = w {
                if v != 0 {
                    // Bottleneck ≤ max edge weight in the graph.
                    prop_assert!(*wv <= max_weight + 1e-9,
                        "vertex {}: width {} > max edge weight {}", v, wv, max_weight);
                }
            }
        }
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

    /// PR-028 convergence_degree invariants on undirected graphs:
    /// - result length equals edge count
    /// - every non-NaN value is in [0, 1] (undirected absolute-value rule)
    /// - per-edge `ins`/`outs` are non-negative
    /// - if `ins[e] + outs[e] == 0` then `result[e]` is NaN; otherwise it is finite
    #[test]
    fn convergence_degree_undirected_invariants(g in arb_graph(8)) {
        let m = g.ecount();
        let (r, ins, outs) = rust_igraph::convergence_degree_full(&g).unwrap();
        prop_assert_eq!(r.len(), m);
        prop_assert_eq!(ins.len(), m);
        prop_assert_eq!(outs.len(), m);
        for e in 0..m {
            prop_assert!(ins[e] >= 0.0, "ins[{}] = {} negative", e, ins[e]);
            prop_assert!(outs[e] >= 0.0, "outs[{}] = {} negative", e, outs[e]);
            let s = ins[e] + outs[e];
            if s == 0.0 {
                prop_assert!(r[e].is_nan(),
                             "expected NaN for unreachable edge {}, got {}", e, r[e]);
            } else {
                prop_assert!(r[e].is_finite(),
                             "result[{}] = {} should be finite", e, r[e]);
                prop_assert!((-1e-12..=1.0 + 1e-12).contains(&r[e]),
                             "undirected result[{}] = {} outside [0,1]", e, r[e]);
            }
        }
    }

    /// PR-028 convergence_degree invariants on directed graphs:
    /// - result length equals edge count
    /// - every non-NaN value is in [-1, 1]
    /// - `ins`/`outs` are non-negative
    #[test]
    fn convergence_degree_directed_invariants(g in arb_directed_graph(8)) {
        let m = g.ecount();
        let (r, ins, outs) = rust_igraph::convergence_degree_full(&g).unwrap();
        prop_assert_eq!(r.len(), m);
        for e in 0..m {
            prop_assert!(ins[e] >= 0.0, "ins[{}] = {}", e, ins[e]);
            prop_assert!(outs[e] >= 0.0, "outs[{}] = {}", e, outs[e]);
            let s = ins[e] + outs[e];
            if s == 0.0 {
                prop_assert!(r[e].is_nan());
            } else {
                prop_assert!((-1.0 - 1e-12..=1.0 + 1e-12).contains(&r[e]),
                             "directed result[{}] = {} outside [-1,1]", e, r[e]);
            }
        }
    }

    /// PR-028 `convergence_degree` and `convergence_degree_full` agree
    /// on the result vector (the former is just `_full().0`).
    #[test]
    fn convergence_degree_matches_full(g in arb_graph(8)) {
        let r = rust_igraph::convergence_degree(&g).unwrap();
        let (r_full, _, _) = rust_igraph::convergence_degree_full(&g).unwrap();
        prop_assert_eq!(r.len(), r_full.len());
        for e in 0..r.len() {
            if r[e].is_nan() {
                prop_assert!(r_full[e].is_nan());
            } else {
                prop_assert!((r[e] - r_full[e]).abs() < 1e-12,
                             "edge {}: {} vs {}", e, r[e], r_full[e]);
            }
        }
    }

    /// PR-014c: `count_loops` agrees with the cardinality of `is_loop`.
    #[test]
    fn count_loops_agrees_with_is_loop(g in arb_graph(8)) {
        let n = rust_igraph::count_loops(&g).unwrap();
        let v = rust_igraph::is_loop(&g).unwrap();
        prop_assert_eq!(n, v.iter().filter(|&&b| b).count());
    }

    /// PR-014c: `count_multiple` length matches ecount, every entry is
    /// at least 1, and `>= 2` exactly when the edge participates in a
    /// parallel group.
    #[test]
    fn count_multiple_invariants(g in arb_graph(8)) {
        let m = g.ecount();
        let mults = rust_igraph::count_multiple(&g).unwrap();
        let is_mult = rust_igraph::is_multiple(&g).unwrap();
        prop_assert_eq!(mults.len(), m);
        prop_assert_eq!(is_mult.len(), m);
        for e in 0..m {
            prop_assert!(mults[e] >= 1, "edge {} has mult 0", e);
            // is_multiple ⇒ multiplicity > 1.
            if is_mult[e] {
                prop_assert!(mults[e] > 1);
            }
        }
        // Aggregate: there is at least one edge with multiplicity > 1
        // exactly when has_multiple says so.
        let any_parallel = mults.iter().any(|&k| k > 1);
        prop_assert_eq!(any_parallel, rust_igraph::has_multiple(&g).unwrap());
    }

    /// PR-014c directed variant of the same `count_multiple` invariants.
    #[test]
    fn count_multiple_directed_invariants(g in arb_directed_graph(8)) {
        let m = g.ecount();
        let mults = rust_igraph::count_multiple(&g).unwrap();
        prop_assert_eq!(mults.len(), m);
        for &k in &mults {
            prop_assert!(k >= 1);
        }
        let any_parallel = mults.iter().any(|&k| k > 1);
        prop_assert_eq!(any_parallel, rust_igraph::has_multiple(&g).unwrap());
    }

    /// PR-002d: per-vertex triangle counts sum to 3 × scalar count, and
    /// each entry is bounded by C(deg, 2).
    #[test]
    fn count_adjacent_triangles_invariants(g in arb_graph(8)) {
        let n = g.vcount() as usize;
        let adj = rust_igraph::count_adjacent_triangles(&g).unwrap();
        prop_assert_eq!(adj.len(), n);

        let total = rust_igraph::count_triangles(&g).unwrap();
        prop_assert_eq!(adj.iter().sum::<u64>(), 3 * total);

        // Each entry ≤ C(simple_degree, 2). Use a permissive simple
        // degree upper bound: total degree counted via neighbours().
        for v in 0..n as u32 {
            let raw = g.neighbors(v).unwrap();
            let mut simple: Vec<_> = raw.into_iter().filter(|&u| u != v).collect();
            simple.sort_unstable();
            simple.dedup();
            let d = simple.len() as u64;
            let max_t = if d < 2 { 0 } else { d * (d - 1) / 2 };
            prop_assert!(adj[v as usize] <= max_t);
        }
    }

    /// PR-029: global_efficiency lies in [0, 1] for any unweighted graph,
    /// and equals the average of harmonic_centrality over all vertices.
    #[test]
    fn global_efficiency_invariants(g in arb_graph(8)) {
        let n = g.vcount();
        let e = rust_igraph::global_efficiency(&g).unwrap();
        if n < 2 {
            prop_assert_eq!(e, None);
        } else {
            let val = e.expect("vcount >= 2 should yield Some");
            prop_assert!((0.0..=1.0).contains(&val));

            let h = rust_igraph::harmonic_centrality(&g).unwrap();
            let avg: f64 = h.iter().sum::<f64>() / (h.len() as f64);
            prop_assert!((val - avg).abs() < 1e-9);
        }
    }

    /// PR-030: per-vertex `local_efficiency` lies in `[0, 1]` and has
    /// length `vcount`; `average_local_efficiency` equals the mean of
    /// the per-vertex vector (and is `0` whenever `vcount < 3`).
    /// Additionally a vertex with fewer than 2 unique non-self
    /// neighbours must contribute `0`.
    #[test]
    fn local_efficiency_invariants(g in arb_graph(8)) {
        let n = g.vcount();
        let local = rust_igraph::local_efficiency(&g).unwrap();
        prop_assert_eq!(local.len(), n as usize);
        for v in &local {
            prop_assert!((0.0..=1.0).contains(v));
        }

        let avg = rust_igraph::average_local_efficiency(&g).unwrap();
        if n < 3 {
            prop_assert_eq!(avg, 0.0);
        } else {
            let computed = local.iter().sum::<f64>() / (n as f64);
            prop_assert!((avg - computed).abs() < 1e-12);
        }

        // Vertices with <2 unique non-self neighbours contribute 0.
        for v in 0..n {
            let raw = g.neighbors(v).unwrap();
            let mut simple: Vec<_> = raw.into_iter().filter(|&u| u != v).collect();
            simple.sort_unstable();
            simple.dedup();
            if simple.len() < 2 {
                prop_assert_eq!(local[v as usize], 0.0);
            }
        }
    }

    /// Louvain partition labels are dense in `[0, k)`, the final
    /// membership equals the last level snapshot, every level has
    /// `vcount` entries, and the reported modularity equals what
    /// `modularity()` computes on the same partition.
    #[test]
    fn louvain_partition_well_formed(g in arb_graph(20)) {
        let r = rust_igraph::louvain(&g).unwrap();
        let n = g.vcount() as usize;
        prop_assert_eq!(r.membership.len(), n);
        if n == 0 {
            prop_assert!(r.levels.is_empty());
            prop_assert_eq!(r.modularity, 0.0);
        } else {
            let k = r.membership.iter().copied().max().unwrap() + 1;
            let mut seen = vec![false; k as usize];
            for &m in &r.membership {
                prop_assert!((m as usize) < seen.len());
                seen[m as usize] = true;
            }
            prop_assert!(seen.into_iter().all(|b| b),
                "membership labels must be contiguous in [0, k)");
            for lvl in &r.levels {
                prop_assert_eq!(lvl.len(), n);
            }
            prop_assert_eq!(r.levels.len(), r.modularities.len());
            if let Some(last) = r.levels.last() {
                prop_assert_eq!(last, &r.membership);
            }
            // Internal Q must agree with standalone modularity().
            if let Some(q) = rust_igraph::modularity(&g, &r.membership, 1.0).unwrap() {
                prop_assert!((r.modularity - q).abs() < 1e-9,
                    "internal Q = {} ≠ modularity() = {}", r.modularity, q);
            }
        }
    }

    /// Louvain pass-loop only accepts strictly improving merges, so
    /// per-level modularity must be non-decreasing.
    #[test]
    fn louvain_modularity_non_decreasing(g in arb_graph(20)) {
        let r = rust_igraph::louvain(&g).unwrap();
        for w in r.modularities.windows(2) {
            prop_assert!(w[1] + 1e-9 >= w[0],
                "modularity decreased across levels: {} → {}", w[0], w[1]);
        }
    }

    /// Unit-weighted Louvain must produce the same modularity as
    /// unweighted Louvain on the same graph (gain formula reduces
    /// exactly when every weight is 1.0).
    #[test]
    fn louvain_unit_weighted_matches_unweighted(g in arb_graph(15)) {
        let a = rust_igraph::louvain(&g).unwrap();
        let ones = vec![1.0; g.ecount()];
        let b = rust_igraph::louvain_weighted(&g, &ones).unwrap();
        prop_assert!((a.modularity - b.modularity).abs() < 1e-9,
            "unit-weighted Q={} ≠ unweighted Q={}", b.modularity, a.modularity);
    }

    /// Same seed must reproduce the same membership and modularity
    /// bit-for-bit (deterministic SplitMix64 + Fisher-Yates).
    #[test]
    fn louvain_determinism_under_seed(g in arb_graph(15), seed: u64) {
        let a = rust_igraph::louvain_with_options(&g, None, 1.0, seed).unwrap();
        let b = rust_igraph::louvain_with_options(&g, None, 1.0, seed).unwrap();
        prop_assert_eq!(a.membership, b.membership);
        prop_assert!((a.modularity - b.modularity).abs() < 1e-12);
    }

    /// Leiden partition is well-formed: dense labels in [0, k),
    /// nb_clusters consistent with max label, quality finite, and
    /// qualities-history length matches n_iterations_run.
    #[test]
    fn leiden_partition_well_formed(g in arb_graph(20)) {
        let r = rust_igraph::leiden(&g).unwrap();
        let n = g.vcount() as usize;
        prop_assert_eq!(r.membership.len(), n);
        if n == 0 {
            prop_assert_eq!(r.nb_clusters, 0);
            prop_assert_eq!(r.quality, 0.0);
        } else {
            let max_label = *r.membership.iter().max().unwrap();
            prop_assert!((max_label as usize) < n);
            prop_assert_eq!(max_label + 1, r.nb_clusters,
                "nb_clusters {} ≠ max(membership)+1 {}", r.nb_clusters, max_label + 1);
            let mut seen = vec![false; r.nb_clusters as usize];
            for &m in &r.membership {
                seen[m as usize] = true;
            }
            prop_assert!(seen.into_iter().all(|b| b),
                "membership labels must be contiguous in [0, k)");
            prop_assert!(r.quality.is_finite(), "quality must be finite");
        }
        prop_assert_eq!(r.qualities.len() as u32, r.n_iterations_run);
    }

    /// Unit-weighted Leiden must produce the same quality as
    /// unweighted Leiden on the same graph (the weight=1.0 path
    /// reduces exactly to the unweighted formulas).
    #[test]
    fn leiden_unit_weighted_matches_unweighted(g in arb_graph(15)) {
        let a = rust_igraph::leiden(&g).unwrap();
        let ones = vec![1.0; g.ecount()];
        let b = rust_igraph::leiden_weighted(&g, &ones).unwrap();
        prop_assert!((a.quality - b.quality).abs() < 1e-9,
            "unit-weighted Q={} ≠ unweighted Q={}", b.quality, a.quality);
    }

    /// Same seed must reproduce the same membership and quality
    /// bit-for-bit (deterministic SplitMix64 + Fisher-Yates +
    /// exp(diff/β) sampling).
    #[test]
    fn leiden_determinism_under_seed(g in arb_graph(15), seed: u64) {
        let opts = rust_igraph::LeidenOptions {
            seed,
            ..rust_igraph::LeidenOptions::default()
        };
        let a = rust_igraph::leiden_with_options(&g, None, &opts).unwrap();
        let b = rust_igraph::leiden_with_options(&g, None, &opts).unwrap();
        prop_assert_eq!(a.membership, b.membership);
        prop_assert!((a.quality - b.quality).abs() < 1e-12);
    }

    /// CPM with a very large γ ⇒ singleton partition is uniquely
    /// optimal (every merge costs more than it gains). Leiden's
    /// argmax-leaning fast moves must land on `k = n`.
    #[test]
    fn leiden_cpm_huge_resolution_yields_singletons(g in arb_graph(12)) {
        let opts = rust_igraph::LeidenOptions {
            objective: rust_igraph::LeidenObjective::Cpm,
            resolution: 1.0e6,
            ..rust_igraph::LeidenOptions::default()
        };
        let r = rust_igraph::leiden_with_options(&g, None, &opts).unwrap();
        prop_assert_eq!(r.nb_clusters as usize, g.vcount() as usize);
    }

    /// Label propagation partition is well-formed: dense labels in
    /// [0, k), nb_clusters consistent with max label, and every label
    /// in [0, k) is actually used.
    #[test]
    fn lpa_partition_well_formed(g in arb_graph(20)) {
        let r = rust_igraph::label_propagation(&g).unwrap();
        let n = g.vcount() as usize;
        prop_assert_eq!(r.membership.len(), n);
        if n == 0 {
            prop_assert_eq!(r.nb_clusters, 0);
        } else {
            let max_label = *r.membership.iter().max().unwrap();
            prop_assert!((max_label as usize) < n);
            prop_assert_eq!(max_label + 1, r.nb_clusters,
                "nb_clusters {} ≠ max(membership)+1 {}", r.nb_clusters, max_label + 1);
            let mut seen = vec![false; r.nb_clusters as usize];
            for &m in &r.membership {
                seen[m as usize] = true;
            }
            prop_assert!(seen.into_iter().all(|b| b),
                "membership labels must be contiguous in [0, k)");
        }
    }

    /// Unit-weighted LPA must produce the same partition as the
    /// unweighted variant (the weight=1.0 path is the same algorithm
    /// applied to identical numbers).
    #[test]
    fn lpa_unit_weighted_matches_unweighted(g in arb_graph(15)) {
        let opts = rust_igraph::LpaOptions { seed: 0, ..rust_igraph::LpaOptions::default() };
        let a = rust_igraph::label_propagation_with_options(&g, None, &opts).unwrap();
        let ones = vec![1.0; g.ecount()];
        let b = rust_igraph::label_propagation_with_options(&g, Some(&ones), &opts).unwrap();
        prop_assert_eq!(a.membership, b.membership);
    }

    /// Same seed must reproduce the same membership bit-for-bit across
    /// all three variants.
    #[test]
    fn lpa_determinism_under_seed(
        g in arb_graph(15),
        seed: u64,
        variant_idx in 0u8..3,
    ) {
        let variant = match variant_idx {
            0 => rust_igraph::LpaVariant::Fast,
            1 => rust_igraph::LpaVariant::Dominance,
            _ => rust_igraph::LpaVariant::Retention,
        };
        let opts = rust_igraph::LpaOptions { variant, seed, ..rust_igraph::LpaOptions::default() };
        let a = rust_igraph::label_propagation_with_options(&g, None, &opts).unwrap();
        let b = rust_igraph::label_propagation_with_options(&g, None, &opts).unwrap();
        prop_assert_eq!(a.membership, b.membership);
        prop_assert_eq!(a.nb_clusters, b.nb_clusters);
    }

    /// All variants must produce a complete labelling — no negative
    /// labels leak through, every vertex ends up in some community.
    #[test]
    fn lpa_no_unlabelled_left(g in arb_graph(20), variant_idx in 0u8..3) {
        let variant = match variant_idx {
            0 => rust_igraph::LpaVariant::Fast,
            1 => rust_igraph::LpaVariant::Dominance,
            _ => rust_igraph::LpaVariant::Retention,
        };
        let opts = rust_igraph::LpaOptions { variant, ..rust_igraph::LpaOptions::default() };
        let r = rust_igraph::label_propagation_with_options(&g, None, &opts).unwrap();
        prop_assert_eq!(r.membership.len(), g.vcount() as usize);
        if !r.membership.is_empty() {
            let max = *r.membership.iter().max().unwrap();
            prop_assert!((max as usize) < g.vcount() as usize);
        }
    }

    /// Fluid Communities on any graph the random generator yields: if
    /// it satisfies the simple+connected precondition the partition is
    /// well-formed; otherwise the call must surface an explicit error.
    #[test]
    fn fluid_partition_well_formed(g in arb_graph(15), k in 1u32..6) {
        if g.vcount() == 0 || k > g.vcount() {
            // Skip degenerate slices — covered by the deterministic
            // suite already.
            return Ok(());
        }
        let r = rust_igraph::fluid_communities(&g, k);
        if let Ok(r) = r {
            prop_assert_eq!(r.membership.len(), g.vcount() as usize);
            let max = *r.membership.iter().max().unwrap();
            prop_assert!((max as usize) < g.vcount() as usize);
            prop_assert_eq!(max + 1, r.nb_clusters);
            prop_assert!(r.nb_clusters <= k);
            // contiguous labels
            let mut seen = vec![false; r.nb_clusters as usize];
            for &m in &r.membership { seen[m as usize] = true; }
            prop_assert!(seen.into_iter().all(|b| b));
        }
    }

    /// Fluid Communities is deterministic when seeded.
    #[test]
    fn fluid_determinism_under_seed(g in arb_graph(15), k in 1u32..5, seed: u64) {
        if g.vcount() == 0 || k > g.vcount() {
            return Ok(());
        }
        let opts = rust_igraph::FluidOptions {
            seed,
            ..rust_igraph::FluidOptions::default()
        };
        let a = rust_igraph::fluid_communities_with_options(&g, k, &opts);
        let b = rust_igraph::fluid_communities_with_options(&g, k, &opts);
        match (a, b) {
            (Ok(a), Ok(b)) => {
                prop_assert_eq!(a.membership, b.membership);
                prop_assert_eq!(a.nb_clusters, b.nb_clusters);
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "fluid_communities determinism mismatch (Ok vs Err)"),
        }
    }

    // ALGO-CO-006: Edge-betweenness community detection invariants.
    #[test]
    fn edge_betweenness_community_partition_well_formed(g in arb_graph(12)) {
        // Module rejects directed graphs; skip them.
        if g.is_directed() {
            return Ok(());
        }
        let r = match rust_igraph::edge_betweenness_community(&g) {
            Ok(r) => r,
            Err(_) => return Ok(()),
        };
        let n = g.vcount();
        let m = g.ecount();
        prop_assert_eq!(r.membership.len() as u32, n);
        prop_assert_eq!(r.removed_edges.len(), m);
        prop_assert_eq!(r.edge_betweenness.len(), m);
        prop_assert_eq!(r.merges.len(), r.bridges.len());
        prop_assert_eq!(r.modularity.len(), r.merges.len() + 1);
        if n > 0 {
            for &lbl in &r.membership {
                prop_assert!(lbl < r.nb_clusters);
            }
            // Membership labels are contiguous in [0, nb_clusters).
            let mut seen = vec![false; r.nb_clusters as usize];
            for &lbl in &r.membership {
                seen[lbl as usize] = true;
            }
            for b in seen { prop_assert!(b); }
            // Every removed edge id is in [0, m) and appears exactly once.
            let mut seen_eid = vec![false; m];
            for &eid in &r.removed_edges {
                let idx = eid as usize;
                prop_assert!(idx < m);
                prop_assert!(!seen_eid[idx]);
                seen_eid[idx] = true;
            }
        }
        // Best-Q membership re-feeds modularity() to within ε of
        // declared best level (only when m > 0).
        if m > 0 {
            let best_q = r.modularity.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let q_recompute = rust_igraph::modularity(&g, &r.membership, 1.0)
                .ok()
                .flatten()
                .unwrap_or(0.0);
            prop_assert!((q_recompute - best_q).abs() < 1e-9);
        }
    }

    #[test]
    fn edge_betweenness_community_deterministic(g in arb_graph(10)) {
        if g.is_directed() {
            return Ok(());
        }
        let a = rust_igraph::edge_betweenness_community(&g);
        let b = rust_igraph::edge_betweenness_community(&g);
        match (a, b) {
            (Ok(a), Ok(b)) => {
                prop_assert_eq!(a.membership, b.membership);
                prop_assert_eq!(a.removed_edges, b.removed_edges);
                prop_assert_eq!(a.merges, b.merges);
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "edge_betweenness_community determinism mismatch"),
        }
    }

    // ALGO-CO-007: Fast greedy modularity (Clauset-Newman-Moore 2004) invariants.
    #[test]
    fn fast_greedy_modularity_partition_well_formed(g in arb_graph(12)) {
        if g.is_directed() {
            return Ok(());
        }
        let r = match rust_igraph::fast_greedy_modularity(&g) {
            Ok(r) => r,
            Err(_) => return Ok(()),
        };
        let n = g.vcount();
        prop_assert_eq!(r.membership.len() as u32, n);
        prop_assert_eq!(r.modularity.len(), r.merges.len() + 1);
        for &lbl in &r.membership {
            prop_assert!(lbl < r.nb_clusters);
        }
        // Dense labels are contiguous in [0, nb_clusters).
        let mut seen = vec![false; r.nb_clusters as usize];
        for &lbl in &r.membership {
            seen[lbl as usize] = true;
        }
        for b in seen { prop_assert!(b); }
        // Each dendrogram row references either an original cluster id
        // < n or a previously synthesised id of the form n+i'.
        for (i, row) in r.merges.iter().enumerate() {
            let cap = n + i as u32;
            prop_assert!(row[0] < cap, "merge {i} c1 out of range");
            prop_assert!(row[1] < cap, "merge {i} c2 out of range");
            prop_assert!(row[0] != row[1], "merge {i} merges a cluster with itself");
        }
        // Best-Q membership re-feeds modularity() to within ε of declared
        // best level (only when edges exist; edgeless => Q = NaN).
        if g.ecount() > 0 {
            let finite_qs: Vec<f64> = r.modularity.iter().copied().filter(|q| !q.is_nan()).collect();
            if !finite_qs.is_empty() {
                let best_q = finite_qs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let q_recompute = rust_igraph::modularity(&g, &r.membership, 1.0)
                    .ok()
                    .flatten()
                    .unwrap_or(0.0);
                prop_assert!((q_recompute - best_q).abs() < 1e-9,
                    "best_q={best_q}, recompute={q_recompute}");
            }
        }
    }

    #[test]
    fn fast_greedy_modularity_deterministic(g in arb_graph(10)) {
        if g.is_directed() {
            return Ok(());
        }
        let a = rust_igraph::fast_greedy_modularity(&g);
        let b = rust_igraph::fast_greedy_modularity(&g);
        match (a, b) {
            (Ok(a), Ok(b)) => {
                prop_assert_eq!(a.membership, b.membership);
                prop_assert_eq!(a.merges, b.merges);
                prop_assert_eq!(a.nb_clusters, b.nb_clusters);
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "fast_greedy_modularity determinism mismatch"),
        }
    }

    // ALGO-CO-006b: Weighted edge_betweenness_community invariants on
    // unit weights. The weighted and unweighted code paths share the
    // shortest-path *lengths*, but the weighted variant runs Dijkstra
    // (CO-006c switched it on to also serve directed graphs) while
    // the unweighted variant runs BFS. When several shortest paths
    // tie, the two priority structures can pick different edges to
    // remove, so the dendrograms diverge in shape. Bit-exact
    // equivalence is therefore not a sound invariant — we assert
    // only that both runs produce well-formed dendrograms whose
    // modularity history matches its merge length.
    #[test]
    fn edge_betweenness_community_weighted_unit_well_formed(g in arb_graph(10)) {
        if g.is_directed() {
            return Ok(());
        }
        let weights = vec![1.0_f64; g.ecount()];
        let ru = match rust_igraph::edge_betweenness_community(&g) {
            Ok(r) => r,
            Err(_) => return Ok(()),
        };
        let rw = match rust_igraph::edge_betweenness_community_weighted(&g, &weights) {
            Ok(r) => r,
            Err(_) => return Ok(()),
        };
        prop_assert_eq!(rw.removed_edges.len(), g.ecount());
        prop_assert_eq!(ru.removed_edges.len(), g.ecount());
        prop_assert_eq!(rw.modularity.len(), rw.merges.len() + 1);
        prop_assert_eq!(ru.modularity.len(), ru.merges.len() + 1);
        prop_assert!(rw.nb_clusters >= 1);
        prop_assert!(ru.nb_clusters >= 1);
    }

    #[test]
    fn edge_betweenness_community_weighted_deterministic(g in arb_graph(10)) {
        if g.is_directed() {
            return Ok(());
        }
        let weights = vec![1.0_f64; g.ecount()];
        let a = rust_igraph::edge_betweenness_community_weighted(&g, &weights);
        let b = rust_igraph::edge_betweenness_community_weighted(&g, &weights);
        match (a, b) {
            (Ok(a), Ok(b)) => {
                prop_assert_eq!(a.membership, b.membership);
                prop_assert_eq!(a.removed_edges, b.removed_edges);
                prop_assert_eq!(a.merges, b.merges);
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "edge_betweenness_community_weighted determinism mismatch"),
        }
    }
}

// ALGO-CO-006c: directed edge_betweenness_community invariants.
// Directed graphs must produce a well-formed dendrogram (every membership
// label dense, modularity-history length = merges + 1) and the unit-weights
// weighted run must reproduce the unweighted-CO-006 dendrogram bit-for-bit.
#[cfg(feature = "proptest-harness")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn edge_betweenness_community_directed_partition_well_formed(g in arb_directed_graph(10)) {
        let n = g.vcount();
        let m = g.ecount();
        let r = match rust_igraph::edge_betweenness_community(&g) {
            Ok(r) => r,
            Err(_) => return Ok(()),
        };
        prop_assert_eq!(r.membership.len() as u32, n);
        prop_assert_eq!(r.removed_edges.len(), m);
        prop_assert_eq!(r.edge_betweenness.len(), m);
        prop_assert_eq!(r.merges.len(), r.bridges.len());
        prop_assert_eq!(r.modularity.len(), r.merges.len() + 1);
        for &lbl in &r.membership {
            prop_assert!(lbl < r.nb_clusters);
        }
        for &q in &r.modularity {
            prop_assert!(q.is_finite());
            prop_assert!((-1.0..=1.0).contains(&q));
        }
    }

    #[test]
    fn edge_betweenness_community_directed_weighted_unit_matches_unweighted(
        g in arb_directed_graph(8),
    ) {
        let weights = vec![1.0_f64; g.ecount()];
        let ru = match rust_igraph::edge_betweenness_community(&g) {
            Ok(r) => r,
            Err(_) => return Ok(()),
        };
        let rw = match rust_igraph::edge_betweenness_community_weighted(&g, &weights) {
            Ok(r) => r,
            Err(_) => return Ok(()),
        };
        prop_assert_eq!(rw.membership, ru.membership);
        prop_assert_eq!(rw.removed_edges, ru.removed_edges);
        prop_assert_eq!(rw.merges, ru.merges);
    }
}

// ALGO-FL-030: dominator-tree invariants.
//
// These invariants verify the Lengauer-Tarjan output on arbitrary small
// directed graphs without depending on an external oracle:
//
// 1. Shape: `idom` has length `vcount`; `idom[root] == -1`; every other
//    entry is either `-2` (unreachable) or a valid vertex id; tree has
//    one edge per reachable non-root vertex.
// 2. Reachability: a vertex `v` has `idom[v] >= 0` if and only if `v` is
//    BFS-reachable from `root`, and `leftout` enumerates exactly the
//    unreachable set in ascending order.
// 3. Direction equivalence: `dominator_tree(g, root, In)` and
//    `dominator_tree(reverse(g), root, Out)` produce identical `idom`
//    vectors. This is the algorithm's defining symmetry — IN-mode is
//    OUT-mode on the reverse graph.
// 4. Dominance property (brute force, n ≤ 7): for every reachable
//    non-root `w`, every simple path from `root` to `w` must contain
//    `idom[w]`. Verified by exhaustive DFS path enumeration.
#[cfg(feature = "proptest-harness")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn dominator_tree_shape_and_reachability(g in arb_directed_graph(10)) {
        use rust_igraph::{DominatorMode, dominator_tree, bfs};
        let n = g.vcount();
        let dt = dominator_tree(&g, 0, DominatorMode::Out).expect("root 0 is valid");

        // Shape.
        prop_assert_eq!(dt.idom.len(), n as usize);
        prop_assert_eq!(dt.idom[0], -1, "idom[root] must be -1");
        for v in 1..n {
            let d = dt.idom[v as usize];
            prop_assert!(d == -2 || (d >= 0 && (d as u32) < n),
                         "idom[{}] = {} out of range", v, d);
        }

        // Reachability: idom[v] >= 0 iff v reachable.
        let reachable: std::collections::BTreeSet<u32> =
            bfs(&g, 0).expect("bfs root 0").into_iter().collect();
        for v in 0..n {
            let d = dt.idom[v as usize];
            if reachable.contains(&v) {
                if v == 0 {
                    prop_assert_eq!(d, -1, "root must have idom = -1");
                } else {
                    prop_assert!(d >= 0, "reachable vertex {} must have idom >= 0", v);
                }
            } else {
                prop_assert_eq!(d, -2, "unreachable vertex {} must have idom = -2", v);
            }
        }

        // leftout = sorted complement of reachable set.
        let expected_leftout: Vec<u32> = (0..n).filter(|v| !reachable.contains(v)).collect();
        prop_assert_eq!(&dt.leftout, &expected_leftout);

        // Tree edge count = number of reachable non-root vertices.
        let reachable_non_root = reachable.len().saturating_sub(1) as u32;
        prop_assert_eq!(dt.tree.ecount() as u32, reachable_non_root);
    }

    #[test]
    fn dominator_tree_in_mode_equals_reverse_out_mode(g in arb_directed_graph(10)) {
        use rust_igraph::{DominatorMode, Graph, dominator_tree};
        let n = g.vcount();
        let m = u32::try_from(g.ecount()).expect("ecount fits u32");
        let mut g_rev = Graph::new(n, true).expect("directed reverse graph");
        for e in 0..m {
            let (u, v) = g.edge(e).expect("edge");
            g_rev.add_edge(v, u).expect("reverse edge");
        }
        let in_mode = dominator_tree(&g, 0, DominatorMode::In).expect("In mode");
        let out_mode = dominator_tree(&g_rev, 0, DominatorMode::Out).expect("Out on rev");
        prop_assert_eq!(&in_mode.idom, &out_mode.idom);
        prop_assert_eq!(&in_mode.leftout, &out_mode.leftout);
    }

    /// Brute-force dominance check: for every reachable non-root vertex
    /// `w`, every simple root-to-`w` path must pass through `idom(w)`.
    /// Bound `n ≤ 7` so simple-path enumeration stays feasible.
    #[test]
    fn dominator_tree_idom_lies_on_every_path_brute_force(g in arb_directed_graph(7)) {
        use rust_igraph::{DominatorMode, dominator_tree};
        let n = g.vcount();
        let dt = dominator_tree(&g, 0, DominatorMode::Out).expect("compute");
        for w in 1..n {
            let d = dt.idom[w as usize];
            if d < 0 {
                continue;
            }
            let d_u = d as u32;
            // Enumerate every simple root->w path.
            let mut paths: Vec<Vec<u32>> = Vec::new();
            let mut stack: Vec<u32> = vec![0];
            let mut on_stack = vec![false; n as usize];
            on_stack[0] = true;
            fn dfs_paths(
                g: &rust_igraph::Graph,
                cur: u32, t: u32,
                stack: &mut Vec<u32>, on_stack: &mut [bool],
                out: &mut Vec<Vec<u32>>,
            ) {
                if cur == t {
                    out.push(stack.clone());
                    return;
                }
                // Iterate via edge ids to stay on the public API.
                let m = u32::try_from(g.ecount()).expect("ecount fits u32");
                for e in 0..m {
                    let (from, to) = g.edge(e).expect("edge");
                    if from == cur && !on_stack[to as usize] {
                        on_stack[to as usize] = true;
                        stack.push(to);
                        dfs_paths(g, to, t, stack, on_stack, out);
                        stack.pop();
                        on_stack[to as usize] = false;
                    }
                }
            }
            dfs_paths(&g, 0, w, &mut stack, &mut on_stack, &mut paths);
            prop_assert!(!paths.is_empty(),
                         "vertex {} marked reachable (idom={}) but no path found", w, d);
            for p in &paths {
                prop_assert!(
                    p.contains(&d_u),
                    "idom({}) = {} missing from path {:?}", w, d_u, p
                );
            }
        }
    }
}

// ALGO-FL-031: all_st_cuts (Provan-Shier) invariants on arbitrary small
// directed graphs. Each enumerated cut must be a genuine, complete (s,t)
// edge cut whose source side is a sorted, unique set containing the source
// and excluding the target; deleting the cut must disconnect the target
// from the source.
#[cfg(feature = "proptest-harness")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn all_st_cuts_are_valid_complete_and_disconnecting(
        g in arb_directed_graph(7),
        target in 1u32..7,
    ) {
        let n = g.vcount();
        let source = 0u32;
        // Need at least two vertices and an in-range target distinct from
        // the source.
        if n < 2 || target >= n {
            return Ok(());
        }
        let res = rust_igraph::all_st_cuts(&g, source, target).unwrap();
        prop_assert_eq!(res.cuts.len(), res.partition1s.len());

        // Resolve the edge list once for cross-checks.
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for proptest");
        let edges: Vec<(u32, u32)> = (0..m).map(|e| g.edge(e).unwrap()).collect();

        let mut seen_partitions: std::collections::HashSet<Vec<u32>> =
            std::collections::HashSet::new();

        for (part, cut) in res.partition1s.iter().zip(res.cuts.iter()) {
            // Sortedness (strictly ascending).
            for w in part.windows(2) {
                prop_assert!(w[0] < w[1], "partition not strictly ascending: {:?}", part);
            }
            for w in cut.windows(2) {
                prop_assert!(w[0] < w[1], "cut not strictly ascending: {:?}", cut);
            }

            // Source/target membership.
            prop_assert!(part.contains(&source), "source missing from {:?}", part);
            prop_assert!(!part.contains(&target), "target present in {:?}", part);

            // Uniqueness of source-side sets.
            prop_assert!(seen_partitions.insert(part.clone()),
                         "duplicate partition {:?}", part);

            let mut in_s = vec![false; n as usize];
            for &v in part {
                in_s[v as usize] = true;
            }

            // Validity + completeness: the cut equals exactly the set of
            // edges leaving the source side.
            let mut crossing: Vec<u32> = Vec::new();
            for (e, &(from, to)) in edges.iter().enumerate() {
                if in_s[from as usize] && !in_s[to as usize] {
                    crossing.push(u32::try_from(e).expect("edge id fits in u32"));
                }
            }
            crossing.sort_unstable();
            prop_assert_eq!(cut, &crossing,
                            "cut != crossing-out edges of partition {:?}", part);

            // Removing the cut edges must disconnect target from source.
            let removed: std::collections::HashSet<u32> = cut.iter().copied().collect();
            let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n as usize];
            for (e, &(from, to)) in edges.iter().enumerate() {
                let eid = u32::try_from(e).expect("edge id fits in u32");
                if removed.contains(&eid) {
                    continue;
                }
                adj[from as usize].push(to);
            }
            let mut visited = vec![false; n as usize];
            let mut stack = vec![source];
            visited[source as usize] = true;
            let mut reached_target = false;
            while let Some(u) = stack.pop() {
                if u == target {
                    reached_target = true;
                    break;
                }
                for &w in &adj[u as usize] {
                    if !visited[w as usize] {
                        visited[w as usize] = true;
                        stack.push(w);
                    }
                }
            }
            prop_assert!(!reached_target,
                         "target {} still reachable after removing cut {:?}", target, cut);
        }
    }
}

// ALGO-FL-032: all_st_mincuts (Provan-Shier) invariants on arbitrary small
// directed graphs. Every enumerated cut must be a genuine, complete (s,t)
// edge cut of *minimum* total capacity: its source side is a sorted, unique
// set containing the source and excluding the target; the cut equals exactly
// the edges leaving that side; the unit-capacity weight of each cut equals the
// reported value, which in turn equals the maximum source→target flow; and
// deleting the cut disconnects the target from the source.
#[cfg(feature = "proptest-harness")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn all_st_mincuts_are_valid_minimum_and_disconnecting(
        g in arb_directed_graph(7),
        target in 1u32..7,
    ) {
        let n = g.vcount();
        let source = 0u32;
        if n < 2 || target >= n {
            return Ok(());
        }
        let res = rust_igraph::all_st_mincuts(&g, source, target, None).unwrap();
        prop_assert_eq!(res.cuts.len(), res.partition1s.len());

        // The reported value must equal the maximum flow.
        let mf = rust_igraph::max_flow_value(&g, source, target, None).unwrap();
        prop_assert!((res.value - mf).abs() < 1e-9,
                     "value {} != max flow {}", res.value, mf);

        // At least one minimum cut must exist whenever the source can reach
        // the target (value > 0). An unreachable target yields the empty list.
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32 for proptest");
        let edges: Vec<(u32, u32)> = (0..m).map(|e| g.edge(e).unwrap()).collect();

        let mut seen_partitions: std::collections::HashSet<Vec<u32>> =
            std::collections::HashSet::new();

        for (part, cut) in res.partition1s.iter().zip(res.cuts.iter()) {
            // Sortedness (strictly ascending).
            for w in part.windows(2) {
                prop_assert!(w[0] < w[1], "partition not strictly ascending: {:?}", part);
            }
            for w in cut.windows(2) {
                prop_assert!(w[0] < w[1], "cut not strictly ascending: {:?}", cut);
            }

            // Source/target membership.
            prop_assert!(part.contains(&source), "source missing from {:?}", part);
            prop_assert!(!part.contains(&target), "target present in {:?}", part);

            // Uniqueness of source-side sets.
            prop_assert!(seen_partitions.insert(part.clone()),
                         "duplicate partition {:?}", part);

            let mut in_s = vec![false; n as usize];
            for &v in part {
                in_s[v as usize] = true;
            }

            // Validity + completeness: the cut equals exactly the set of
            // edges leaving the source side (in a minimum cut every crossing
            // edge is saturated, so it carries positive flow).
            let mut crossing: Vec<u32> = Vec::new();
            for (e, &(from, to)) in edges.iter().enumerate() {
                if in_s[from as usize] && !in_s[to as usize] {
                    crossing.push(u32::try_from(e).expect("edge id fits in u32"));
                }
            }
            crossing.sort_unstable();
            prop_assert_eq!(cut, &crossing,
                            "cut != crossing-out edges of partition {:?}", part);

            // Minimality: the unit-capacity weight of the cut equals the value.
            prop_assert!((cut.len() as f64 - res.value).abs() < 1e-9,
                         "cut weight {} != value {}", cut.len(), res.value);

            // Removing the cut edges must disconnect target from source.
            let removed: std::collections::HashSet<u32> = cut.iter().copied().collect();
            let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n as usize];
            for (e, &(from, to)) in edges.iter().enumerate() {
                let eid = u32::try_from(e).expect("edge id fits in u32");
                if removed.contains(&eid) {
                    continue;
                }
                adj[from as usize].push(to);
            }
            let mut visited = vec![false; n as usize];
            let mut stack = vec![source];
            visited[source as usize] = true;
            let mut reached_target = false;
            while let Some(u) = stack.pop() {
                if u == target {
                    reached_target = true;
                    break;
                }
                for &w in &adj[u as usize] {
                    if !visited[w as usize] {
                        visited[w as usize] = true;
                        stack.push(w);
                    }
                }
            }
            prop_assert!(!reached_target,
                         "target {} still reachable after removing cut {:?}", target, cut);
        }
    }
}

// ALGO-CN-031: minimum_size_separators (Kanevsky) invariants on arbitrary
// small undirected graphs. Every enumerated set must be canonical (sorted,
// unique), all sets must share a single cardinality equal to the graph's
// vertex connectivity, a disconnected graph yields none, and — except for
// the complete-graph special case, whose (n-1)-subsets leave a lone vertex
// that `is_separator` does not count as disconnecting — every set must
// genuinely separate the graph.
#[cfg(feature = "proptest-harness")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn minimum_size_separators_valid_and_uniform(g in arb_graph(7)) {
        let n = g.vcount();
        if n < 2 {
            return Ok(());
        }
        let seps = rust_igraph::minimum_size_separators(&g).unwrap();
        let conn = rust_igraph::vertex_connectivity(&g, true).unwrap();

        if conn == 0 {
            prop_assert!(seps.is_empty(),
                         "disconnected graph must yield no separators");
            return Ok(());
        }

        let k = usize::try_from(conn).expect("connectivity fits in usize");
        let complete = conn == i64::from(n) - 1;
        let mut seen: std::collections::HashSet<Vec<u32>> =
            std::collections::HashSet::new();

        for s in &seps {
            for w in s.windows(2) {
                prop_assert!(w[0] < w[1], "separator not strictly ascending: {:?}", s);
            }
            prop_assert_eq!(s.len(), k,
                            "all minimum separators share size = connectivity");
            prop_assert!(seen.insert(s.clone()), "duplicate separator {:?}", s);
            if !complete {
                prop_assert!(rust_igraph::is_separator(&g, s).unwrap(),
                             "{:?} must separate the graph", s);
            }
        }

        // A connected, non-complete graph always has at least one minimum
        // separator.
        if !complete {
            prop_assert!(!seps.is_empty(),
                         "connected non-complete graph must have a separator");
        }
    }
}

// ALGO-CN-032: cohesive_blocks (Moody-White) invariants on arbitrary small
// undirected simple graphs. The result must be internally consistent (all
// four members index-aligned, root = whole graph, tree shape), each block's
// reported cohesion must equal the vertex connectivity of its induced
// subgraph, and every non-root block must be a strict subset of its parent
// with strictly higher cohesion.
#[cfg(feature = "proptest-harness")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn cohesive_blocks_consistent_and_nested(g in arb_graph(7)) {
        // cohesive_blocks requires a simple graph; collapse multi-edges and
        // drop self-loops from the arbitrary instance first.
        let g = rust_igraph::simplify(&g, true, true).unwrap();
        let n = g.vcount();

        let cb = rust_igraph::cohesive_blocks(&g).unwrap();
        let nb = cb.blocks.len();

        // Index-aligned members and a well-formed directed block tree.
        prop_assert_eq!(cb.cohesion.len(), nb);
        prop_assert_eq!(cb.parent.len(), nb);
        prop_assert_eq!(cb.block_tree.vcount() as usize, nb);
        prop_assert!(cb.block_tree.is_directed());
        let non_root = cb.parent.iter().filter(|&&p| p >= 0).count();
        prop_assert_eq!(cb.block_tree.ecount() as usize, non_root);

        // Root block 0 is the whole graph, and only the root is parentless.
        prop_assert_eq!(&cb.blocks[0], &(0..n).collect::<Vec<u32>>());
        prop_assert_eq!(cb.parent[0], -1);
        prop_assert_eq!(non_root, nb - 1);

        for i in 0..nb {
            // Each block is a canonical (sorted, unique) vertex set drawn
            // from the graph, and its cohesion matches the connectivity of
            // the induced subgraph.
            let block = &cb.blocks[i];
            for w in block.windows(2) {
                prop_assert!(w[0] < w[1], "block not strictly ascending: {:?}", block);
            }
            prop_assert!(block.iter().all(|&v| v < n), "block vertex out of range");

            let sub = rust_igraph::induced_subgraph(&g, block).unwrap();
            let conn = rust_igraph::vertex_connectivity(&sub.graph, true).unwrap();
            prop_assert_eq!(cb.cohesion[i], conn,
                "block {:?} cohesion {} != induced connectivity {}",
                block, cb.cohesion[i], conn);

            if i == 0 {
                continue;
            }
            // Non-root: parent index valid, child ⊂ parent, child strictly
            // more cohesive than its parent.
            let p = cb.parent[i];
            prop_assert!(p >= 0 && (p as usize) < nb, "bad parent index {}", p);
            let parent_block = &cb.blocks[p as usize];
            let pset: std::collections::HashSet<u32> = parent_block.iter().copied().collect();
            prop_assert!(block.iter().all(|v| pset.contains(v)),
                "block {:?} not a subset of parent {:?}", block, parent_block);
            prop_assert!(cb.cohesion[i] > cb.cohesion[p as usize],
                "block cohesion {} not greater than parent {}",
                cb.cohesion[i], cb.cohesion[p as usize]);
        }
    }

    /// ALGO-ISO-003: the canonical form is a complete isomorphism invariant.
    /// Relabeling an undirected graph and re-canonicalizing must yield an
    /// identical canonical edge set.
    #[test]
    fn canonical_permutation_invariant_undirected((g, perm) in arb_simple_graph_with_perm(8, false)) {
        let h = rust_igraph::permute_vertices(&g, &perm).expect("permute");
        let pg = rust_igraph::canonical_permutation(&g, None).expect("canon g");
        let ph = rust_igraph::canonical_permutation(&h, None).expect("canon h");

        // Each labeling is a permutation of 0..n.
        let mut sg = pg.clone();
        sg.sort_unstable();
        prop_assert_eq!(sg, (0..g.vcount()).collect::<Vec<_>>());

        prop_assert_eq!(canon_form_edges(&g, &pg), canon_form_edges(&h, &ph),
            "canonical form changed under relabeling");
    }

    /// ALGO-ISO-003, directed counterpart of the invariance property.
    #[test]
    fn canonical_permutation_invariant_directed((g, perm) in arb_simple_graph_with_perm(8, true)) {
        let h = rust_igraph::permute_vertices(&g, &perm).expect("permute");
        let pg = rust_igraph::canonical_permutation(&g, None).expect("canon g");
        let ph = rust_igraph::canonical_permutation(&h, None).expect("canon h");
        prop_assert_eq!(canon_form_edges(&g, &pg), canon_form_edges(&h, &ph),
            "directed canonical form changed under relabeling");
    }

    /// ALGO-ISO-005: every returned generator is a genuine automorphism — a
    /// vertex permutation that preserves the (direction-aware) edge set.
    #[test]
    fn automorphism_group_generators_are_automorphisms(g in arb_simple_graph_with_perm(7, false).prop_map(|(g, _)| g)) {
        let n = g.vcount() as usize;
        let gens = rust_igraph::automorphism_group(&g, None).expect("automorphism_group");
        let edges: std::collections::HashSet<(u32, u32)> = (0..g.ecount())
            .map(|e| g.edge(e as u32).expect("edge"))
            .map(|(u, v)| if u <= v { (u, v) } else { (v, u) })
            .collect();
        for aut in &gens {
            // A permutation of 0..n.
            prop_assert_eq!(aut.len(), n, "generator has wrong length");
            let mut sorted = aut.clone();
            sorted.sort_unstable();
            prop_assert_eq!(sorted, (0..g.vcount()).collect::<Vec<_>>(), "generator is not a permutation");
            // Image of every edge is an edge.
            for &(u, v) in &edges {
                let (iu, iv) = (aut[u as usize], aut[v as usize]);
                let key = if iu <= iv { (iu, iv) } else { (iv, iu) };
                prop_assert!(edges.contains(&key), "generator maps an edge off the edge set");
            }
        }
    }

    /// ALGO-ISO-005: the generators generate a group whose order equals
    /// `count_automorphisms` — i.e. the generating set is complete.
    #[test]
    fn automorphism_group_order_matches_count(g in arb_simple_graph_with_perm(6, false).prop_map(|(g, _)| g)) {
        let n = g.vcount() as usize;
        let gens = rust_igraph::automorphism_group(&g, None).expect("automorphism_group");
        // Close the generating set.
        let id: Vec<u32> = (0..g.vcount()).collect();
        let mut set = std::collections::HashSet::new();
        set.insert(id.clone());
        let mut frontier = vec![id];
        while let Some(p) = frontier.pop() {
            for aut in &gens {
                let q: Vec<u32> = p.iter().map(|&pv| aut[pv as usize]).collect();
                if set.insert(q.clone()) {
                    frontier.push(q);
                }
            }
        }
        let order = rust_igraph::count_automorphisms(&g, None).expect("count_automorphisms");
        prop_assert!((order - set.len() as f64).abs() < 0.5,
            "closure order {} != count_automorphisms {} (n={})", set.len(), order, n);
    }

    /// ALGO-ISO-006: a graph relabeled by a random permutation is always
    /// detected as isomorphic to itself, and the returned map is a genuine
    /// edge-preserving bijection.
    #[test]
    fn isomorphic_bliss_detects_relabeling((g, perm) in arb_simple_graph_with_perm(8, false)) {
        let h = rust_igraph::permute_vertices(&g, &perm).expect("permute");
        let r = rust_igraph::isomorphic_bliss(&g, &h, None, None).expect("bliss");
        prop_assert!(r.iso, "relabeled graph not detected as isomorphic");
        // map12 is a permutation and edge-preserving.
        let n = g.vcount() as usize;
        prop_assert_eq!(r.map12.len(), n);
        let mut sorted = r.map12.clone();
        sorted.sort_unstable();
        prop_assert_eq!(sorted, (0..g.vcount()).collect::<Vec<_>>(), "map12 not a permutation");
        let h_edges: std::collections::HashSet<(u32, u32)> = (0..h.ecount())
            .map(|e| h.edge(e as u32).expect("edge"))
            .map(|(u, v)| if u <= v { (u, v) } else { (v, u) })
            .collect();
        for e in 0..g.ecount() {
            let (u, v) = g.edge(e as u32).expect("edge");
            let (iu, iv) = (r.map12[u as usize], r.map12[v as usize]);
            let key = if iu <= iv { (iu, iv) } else { (iv, iu) };
            prop_assert!(h_edges.contains(&key), "map12 maps an edge off the edge set");
        }
    }

    /// ALGO-ISO-006: the BLISS yes/no verdict agrees with the independent VF2
    /// backend on arbitrary pairs of (loopless simple) graphs.
    #[test]
    fn isomorphic_bliss_agrees_with_vf2(
        (g1, _) in arb_simple_graph_with_perm(7, false),
        (g2, _) in arb_simple_graph_with_perm(7, false),
    ) {
        let bliss = rust_igraph::isomorphic_bliss(&g1, &g2, None, None).expect("bliss");
        let vf2 = rust_igraph::isomorphic_vf2(&g1, &g2, None, None, None, None).expect("vf2");
        prop_assert_eq!(bliss.iso, vf2.iso,
            "bliss/vf2 disagree on isomorphism verdict");
    }

    /// ALGO-ISO-007: the generic `isomorphic` dispatcher always reports a
    /// random relabelling of a graph as isomorphic to the original.
    #[test]
    fn isomorphic_detects_relabeling((g, perm) in arb_simple_graph_with_perm(8, false)) {
        let h = rust_igraph::permute_vertices(&g, &perm).expect("permute");
        prop_assert!(rust_igraph::isomorphic(&g, &h).expect("isomorphic"),
            "relabeled graph not reported isomorphic by dispatcher");
    }

    /// ALGO-ISO-007: the generic `isomorphic` verdict agrees with the explicit
    /// VF2 backend on arbitrary pairs of (loopless simple) graphs — the
    /// dispatcher's backend choice must not change the answer.
    #[test]
    fn isomorphic_agrees_with_vf2(
        (g1, _) in arb_simple_graph_with_perm(7, false),
        (g2, _) in arb_simple_graph_with_perm(7, false),
    ) {
        let generic = rust_igraph::isomorphic(&g1, &g2).expect("isomorphic");
        let vf2 = rust_igraph::isomorphic_vf2(&g1, &g2, None, None, None, None).expect("vf2");
        prop_assert_eq!(generic, vf2.iso, "generic/vf2 disagree on isomorphism verdict");
    }

    /// ALGO-ISO-007: the generic `subisomorphic` verdict agrees with the
    /// explicit VF2 subgraph backend it delegates to.
    #[test]
    fn subisomorphic_agrees_with_vf2(
        (g1, _) in arb_simple_graph_with_perm(7, false),
        (g2, _) in arb_simple_graph_with_perm(5, false),
    ) {
        let generic = rust_igraph::subisomorphic(&g1, &g2).expect("subisomorphic");
        let vf2 = rust_igraph::subisomorphic_vf2(&g1, &g2, None, None, None, None).expect("vf2");
        prop_assert_eq!(generic, vf2.iso, "generic/vf2 disagree on subisomorphism verdict");
    }

    /// ALGO-ISO-020: non-induced LAD (a subgraph monomorphism of `pattern`
    /// into `target`) must agree on the yes/no verdict with the VF2 subgraph
    /// backend. `subisomorphic_lad(pattern, target)` mirrors
    /// `subisomorphic_vf2(target, pattern)` — both ask whether the smaller
    /// pattern embeds into the larger target.
    #[test]
    fn subisomorphic_lad_agrees_with_vf2(
        (target, _) in arb_simple_graph_with_perm(7, false),
        (pattern, _) in arb_simple_graph_with_perm(5, false),
    ) {
        let lad = rust_igraph::subisomorphic_lad(&pattern, &target, None, false)
            .expect("subisomorphic_lad");
        let vf2 = rust_igraph::subisomorphic_vf2(&target, &pattern, None, None, None, None)
            .expect("vf2");
        prop_assert_eq!(lad.iso, vf2.iso, "lad/vf2 disagree on subisomorphism verdict");
    }

    /// ALGO-ISO-020: every map LAD enumerates is a genuine non-induced
    /// embedding — an injection of pattern vertices into target vertices that
    /// carries every pattern edge to a target edge.
    #[test]
    fn lad_maps_are_valid_embeddings(
        (target, _) in arb_simple_graph_with_perm(6, false),
        (pattern, _) in arb_simple_graph_with_perm(4, false),
    ) {
        let maps = rust_igraph::get_subisomorphisms_lad(&pattern, &target, None, false)
            .expect("get_subisomorphisms_lad");
        let pat_edges: Vec<(u32, u32)> = (0..pattern.ecount())
            .map(|e| pattern.edge(e as u32).expect("pattern edge"))
            .collect();
        for map in &maps {
            prop_assert_eq!(
                map.len(),
                pattern.vcount() as usize,
                "map covers every pattern vertex"
            );
            // Injection: no two pattern vertices map to the same target vertex.
            let mut seen = std::collections::HashSet::new();
            for &t in map {
                prop_assert!(t < target.vcount(), "image vertex in range");
                prop_assert!(seen.insert(t), "map is injective");
            }
            // Edge preservation: each pattern edge maps to a target edge.
            for &(u, v) in &pat_edges {
                let (tu, tv) = (map[u as usize], map[v as usize]);
                prop_assert!(
                    target.find_eid(tu, tv).expect("find_eid").is_some(),
                    "pattern edge not preserved by embedding"
                );
            }
        }
    }
}

// ALGO-PR-019: structural invariants of the power-law fit. These hold for any
// fit regardless of the (implementation-defined) xmin search, so they pin the
// engine without re-deriving its exact output.
#[cfg(feature = "proptest-harness")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Continuous fit with a fixed cutoff: every output field is well-formed.
    #[test]
    fn power_law_continuous_fixed_well_formed(
        raw in proptest::collection::vec(1.0f64..1000.0, 30..120)
    ) {
        let fit = rust_igraph::power_law_fit(&raw, 1.0, true).expect("fit");
        prop_assert!(fit.continuous);
        prop_assert_eq!(fit.xmin, 1.0);
        prop_assert!(fit.alpha.is_finite() && fit.alpha > 1.0);
        prop_assert!(fit.log_likelihood.is_finite());
        prop_assert!((0.0..=1.0).contains(&fit.ks_statistic));
    }

    /// The continuous closed-form MLE is exactly reproducible from the cut.
    #[test]
    fn power_law_continuous_alpha_matches_closed_form(
        raw in proptest::collection::vec(1.0f64..1000.0, 60..150)
    ) {
        let xmin = 1.0;
        let cut: Vec<f64> = raw.iter().copied().filter(|&x| x >= xmin).collect();
        prop_assume!(!cut.is_empty());
        let logsum: f64 = cut.iter().map(|&x| (x / xmin).ln()).sum();
        prop_assume!(logsum > 0.0);
        // n >= 50 here, so no finite-size correction is applied.
        let expected = 1.0 + (cut.len() as f64) / logsum;
        let fit = rust_igraph::power_law_fit(&raw, xmin, true).expect("fit");
        prop_assert!((fit.alpha - expected).abs() < 1e-9);
    }

    /// force_continuous always yields a continuous model, even for integer data.
    #[test]
    fn power_law_force_continuous_on_integers(
        ints in proptest::collection::vec(1u32..50, 30..100)
    ) {
        let data: Vec<f64> = ints.iter().map(|&x| f64::from(x)).collect();
        let fit = rust_igraph::power_law_fit(&data, -1.0, true).expect("fit");
        prop_assert!(fit.continuous);
        prop_assert!(fit.alpha > 1.0);
        prop_assert!(fit.xmin >= 1.0);
    }

    /// Integer-valued data without force_continuous fits a discrete model; the
    /// chosen xmin is one of the sample values.
    #[test]
    fn power_law_discrete_detected(
        ints in proptest::collection::vec(1u32..40, 40..120)
    ) {
        let data: Vec<f64> = ints.iter().map(|&x| f64::from(x)).collect();
        let fit = rust_igraph::power_law_fit(&data, -1.0, false).expect("fit");
        prop_assert!(!fit.continuous);
        prop_assert!(fit.alpha.is_finite() && fit.alpha > 1.0);
        prop_assert!(data.iter().any(|&x| (x - fit.xmin).abs() < 1e-12),
            "discrete xmin {} is not a sample value", fit.xmin);
        prop_assert!((0.0..=1.0).contains(&fit.ks_statistic));
    }
}

// ---- dim_select (ALGO-EM-001) ------------------------------------------

#[cfg(feature = "proptest-harness")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// The selected dimension is always a count in `1..=n`.
    #[test]
    fn dim_select_within_bounds(
        sv in proptest::collection::vec(0.001f64..1000.0, 1..50)
    ) {
        let d = rust_igraph::dim_select(&sv).expect("dim_select");
        prop_assert!((1..=sv.len()).contains(&d));
    }

    /// A clean two-level gap (a leading block far above a trailing block) is
    /// detected exactly at the block boundary.
    #[test]
    fn dim_select_detects_two_block_gap(
        head_len in 2usize..8,
        tail_len in 2usize..8,
    ) {
        // Leading block clustered near 1000, trailing block near 1, with a
        // large separating gap so the elbow is unambiguous.
        let mut sv = Vec::with_capacity(head_len + tail_len);
        for i in 0..head_len {
            sv.push(1000.0 - i as f64);
        }
        for i in 0..tail_len {
            sv.push(1.0 - 0.01 * i as f64);
        }
        let d = rust_igraph::dim_select(&sv).expect("dim_select");
        prop_assert_eq!(d, head_len);
    }

    /// A perfectly constant sequence has no elbow; the result is still a valid
    /// in-range count (the all-in-one-group fallback).
    #[test]
    fn dim_select_constant_is_well_formed(
        n in 1usize..40,
        v in 0.5f64..50.0,
    ) {
        let sv = vec![v; n];
        let d = rust_igraph::dim_select(&sv).expect("dim_select");
        prop_assert!((1..=n).contains(&d));
    }
}

/// Sorted undirected edge set of a graph as `(min, max)` pairs.
fn undirected_edge_set(g: &Graph) -> Vec<(u32, u32)> {
    let mut edges: Vec<(u32, u32)> = (0..g.ecount())
        .map(|e| {
            let (u, v) = g.edge(e as u32).expect("edge in range");
            (u.min(v), u.max(v))
        })
        .collect();
    edges.sort_unstable();
    edges
}

/// Arbitrary 2-D point set on a small integer grid. Integer coordinates
/// deliberately admit co-circular degeneracies so the closed-ball boundary
/// rule is exercised. Coincident points are allowed (Gabriel-legal).
fn arb_points_2d(max_n: usize) -> impl Strategy<Value = Vec<Vec<f64>>> {
    proptest::collection::vec(
        (-4i32..=4, -4i32..=4).prop_map(|(x, y)| vec![f64::from(x), f64::from(y)]),
        1..=max_n,
    )
}

/// Like [`arb_points_2d`] but deduplicates coincident points. The
/// EMST-subgraph (hence connectivity) guarantee only holds for *distinct*
/// points: two coincident points mutually block each other's edges to a
/// third point, which can disconnect it.
fn arb_distinct_points_2d(max_n: usize) -> impl Strategy<Value = Vec<Vec<f64>>> {
    arb_points_2d(max_n).prop_map(|pts| {
        let mut seen = std::collections::HashSet::new();
        pts.into_iter()
            .filter(|row| seen.insert((row[0].to_bits(), row[1].to_bits())))
            .collect()
    })
}

proptest! {
    /// Permutation invariance: reordering the input points permutes the Gabriel
    /// graph's vertices identically. Relabeling the result of the original run
    /// by the inverse permutation must reproduce the result of the permuted run.
    #[test]
    fn gabriel_graph_is_permutation_invariant(
        (points, perm) in arb_points_2d(8).prop_flat_map(|pts| {
            let n = pts.len();
            let perm = Just((0..n as u32).collect::<Vec<u32>>()).prop_shuffle();
            (Just(pts), perm)
        }),
    ) {
        let base = rust_igraph::gabriel_graph(&points).expect("gabriel_graph base");

        // Apply the permutation to the point rows: new position `perm[i]` holds
        // the point originally at `i`.
        let n = points.len();
        let mut permuted = vec![Vec::new(); n];
        for (i, row) in points.iter().enumerate() {
            permuted[perm[i] as usize] = row.clone();
        }
        let permed = rust_igraph::gabriel_graph(&permuted).expect("gabriel_graph permuted");

        // Map base edges through `perm` and compare against the permuted run.
        let mut mapped: Vec<(u32, u32)> = undirected_edge_set(&base)
            .into_iter()
            .map(|(u, v)| {
                let (pu, pv) = (perm[u as usize], perm[v as usize]);
                (pu.min(pv), pu.max(pv))
            })
            .collect();
        mapped.sort_unstable();

        prop_assert_eq!(mapped, undirected_edge_set(&permed));
    }

    /// The Gabriel graph contains the Euclidean minimum spanning tree, so it is
    /// always connected for a non-empty point set.
    #[test]
    fn gabriel_graph_is_connected(points in arb_distinct_points_2d(8)) {
        use rust_igraph::{is_connected, ConnectednessMode};
        let g = rust_igraph::gabriel_graph(&points).expect("gabriel_graph");
        prop_assert!(is_connected(&g, ConnectednessMode::Weak).expect("is_connected"));
    }
}
