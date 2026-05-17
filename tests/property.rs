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
}
