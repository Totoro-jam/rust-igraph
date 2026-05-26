//! `vertex_connectivity` (ALGO-FL-015) — global graph cohesion: the
//! minimum number of internal vertices whose removal disconnects some
//! pair of vertices in the graph.
//!
//! Counterpart of `igraph_vertex_connectivity` in
//! `references/igraph/src/flow/flow.c:2158`. Equivalent to the C
//! alias `igraph_cohesion` (flow.c:2470) and to python-igraph's
//! `Graph.vertex_connectivity()` (and `Graph.cohesion()`),
//! R-igraph's `vertex_connectivity()` (no source/target).
//!
//! ## Algorithm
//!
//! Definition: `vc(G) = min_{s ≠ t} kappa(s, t)` where `kappa(s, t)`
//! is the s-t vertex connectivity (ALGO-FL-013 in
//! `IGRAPH_VCONN_NEI_NUMBER_OF_NODES` mode, so that a direct
//! `s → t` edge contributes the "infinity" sentinel and never
//! lowers the running minimum).
//!
//! Optional cheap short-circuits when `checks = true` (suggested by
//! Peter `McMahan` per the upstream C docstring, see
//! flow.c:2147-2149):
//!
//! 1. Empty graph → `0` (no pair exists).
//! 2. Disconnected (weakly for undirected, strongly for directed)
//!    → `0` (some pair of vertices is already separated).
//! 3. Any vertex with `min(in, out) = 1` (or undirected degree
//!    `= 1`) → `1` (removing its single neighbour separates it).
//! 4. Complete graph (`K_n` or its mutual directed counterpart)
//!    → `n - 1` (every pair is internally adjacent).
//!
//! Otherwise iterate FL-013 over all unordered pairs (undirected) or
//! ordered pairs (directed), tracking the running minimum and
//! exiting early once it hits `0`.
//!
//! ## Complexity
//!
//! `O(V^2)` calls to FL-013, each `O(V·E^2)` on the split-graph
//! max-flow → `O(V^3·E^2)` worst case. The igraph C docstring
//! reports `O(|V|^5)` (their bound assumes a denser graph).
//!
//! ## Direct-edge handling in the inner loop
//!
//! The inner call uses [`VconnNei::NumberOfNodes`] so a direct edge
//! `s → t` yields `vcount()` (≥ `vcount - 1`). Comparing
//! `conn < min_conn` (with `min_conn` initialised to `vcount - 1`)
//! is therefore false for such pairs — they leave `min_conn`
//! unchanged. This mirrors the upstream C loop at flow.c:1969-2037.

use crate::core::{Graph, IgraphResult};

use super::st_vertex_connectivity::{VconnNei, st_vertex_connectivity};
use crate::algorithms::connectivity::components::connected_components;
use crate::algorithms::connectivity::strong::strongly_connected_components;
use crate::algorithms::properties::is_complete::is_complete;

/// Vertex connectivity (cohesion) of a graph.
///
/// Returns the minimum number of internal vertices that must be
/// removed to disconnect *some* pair of vertices in `graph`. Equal
/// to `min_{s ≠ t} st_vertex_connectivity(s, t,
/// VconnNei::NumberOfNodes)`.
///
/// Counterpart of `igraph_vertex_connectivity`
/// (`references/igraph/src/flow/flow.c:2158`) and its alias
/// `igraph_cohesion` (flow.c:2470).
///
/// # Arguments
///
/// * `graph` — input graph (directed or undirected).
/// * `checks` — when `true`, run the cheap short-circuits described
///   in the module docs before falling back to the `O(V^2)` pairwise
///   loop. Recommended for any non-trivial graph; the helpers cost
///   `O(V + E)` and can replace the whole pairwise loop. Equivalent
///   to igraph C's `checks` argument.
///
/// # Returns
///
/// The vertex connectivity as `i64`. Returns:
/// * `0` when `vcount() < 2`, or the graph is disconnected (some
///   pair is already separated).
/// * `1` when there is a vertex with degree `1` (undirected) or
///   `min(in, out) = 1` (directed).
/// * `vcount - 1` when the graph is complete.
/// * Otherwise, the result of the pairwise FL-013 loop.
///
/// # Errors
///
/// Propagates errors from [`st_vertex_connectivity`],
/// [`connected_components`], [`strongly_connected_components`], and
/// [`is_complete`]. In practice these arise only from arithmetic
/// overflow on very large graphs, which is unreachable here.
///
/// [`IgraphError`]: crate::core::IgraphError
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, vertex_connectivity};
///
/// // Undirected ring on 5 vertices — vc = 2 (any single vertex
/// // removal leaves the rest connected).
/// let mut g = Graph::new(5, false).unwrap();
/// for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 0)] {
///     g.add_edge(u, v).unwrap();
/// }
/// assert_eq!(vertex_connectivity(&g, true).unwrap(), 2);
///
/// // Undirected path on 5 vertices — vc = 1 (the two endpoints
/// // each have degree 1).
/// let mut p = Graph::new(5, false).unwrap();
/// for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4)] {
///     p.add_edge(u, v).unwrap();
/// }
/// assert_eq!(vertex_connectivity(&p, true).unwrap(), 1);
/// ```
pub fn vertex_connectivity(graph: &Graph, checks: bool) -> IgraphResult<i64> {
    let n = graph.vcount();

    // Empty graph or single vertex: no pair to disconnect.
    // Mirrors flow.c:2084-2087 (handled inside _connectivity_checks).
    if n < 2 {
        return Ok(0);
    }

    if checks {
        // (1) Connectedness — disconnected ⇒ some pair separated ⇒ 0.
        let connected = if graph.is_directed() {
            strongly_connected_components(graph)?.count == 1
        } else {
            connected_components(graph)?.count == 1
        };
        if !connected {
            return Ok(0);
        }

        // (2) Min-degree check (suggested by McMahan, flow.c:2069-2122).
        // Undirected: any vertex with degree 1 ⇒ vc = 1.
        // Directed: any vertex with out-degree 1 or in-degree 1 ⇒ vc = 1.
        let min_one = if graph.is_directed() {
            let mut hit = false;
            for v in 0..n {
                let out = graph.out_neighbors_vec(v)?.len();
                let in_ = graph.in_neighbors_vec(v)?.len();
                if out == 1 || in_ == 1 {
                    hit = true;
                    break;
                }
            }
            hit
        } else {
            let mut hit = false;
            for v in 0..n {
                if graph.degree(v)? == 1 {
                    hit = true;
                    break;
                }
            }
            hit
        };
        if min_one {
            return Ok(1);
        }

        // (3) Complete graph ⇒ vc = n - 1. The C version splits this
        // out from _connectivity_checks because that helper is reused
        // for edge connectivity where the complete-graph short-circuit
        // does not apply to multigraphs (flow.c:2168-2180).
        if is_complete(graph)? {
            return Ok(i64::from(n) - 1);
        }
    }

    // Pairwise FL-013 loop.
    //
    // Initial min_conn = n - 1, matching the C upper bound at
    // flow.c:1950. NumberOfNodes mode returns `n` for direct-edge
    // pairs (vs C's `n - 1`); either way `n < n - 1` is false so
    // direct-edge pairs never lower min_conn — same end result.
    let mut min_conn = i64::from(n) - 1;
    let directed = graph.is_directed();
    for i in 0..n {
        // Undirected: j > i (all pairs are unordered; vc(i,j) = vc(j,i)
        // after the implicit IGRAPH_TO_DIRECTED_MUTUAL).
        // Directed: j != i (vc(i,j) need not equal vc(j,i)).
        let start = if directed { 0 } else { i + 1 };
        for j in start..n {
            if i == j {
                continue;
            }
            let conn = st_vertex_connectivity(graph, i, j, VconnNei::NumberOfNodes)?;
            if conn < min_conn {
                min_conn = conn;
                if min_conn == 0 {
                    return Ok(0);
                }
            }
        }
    }

    Ok(min_conn)
}

/// Group cohesion — igraph C alias `igraph_cohesion` (flow.c:2470).
/// Exact synonym for [`vertex_connectivity`]; kept for naming parity
/// with the upstream API and so users following the
/// White-Harary (2001) sociological-network literature have a direct
/// hit. Identical signature and behaviour.
pub fn cohesion(graph: &Graph, checks: bool) -> IgraphResult<i64> {
    vertex_connectivity(graph, checks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring_graph_n(n: u32, directed: bool) -> Graph {
        let mut g = Graph::new(n, directed).expect("graph");
        for i in 0..n {
            let j = (i + 1) % n;
            g.add_edge(i, j).expect("edge");
        }
        g
    }

    fn path_graph_n(n: u32, directed: bool) -> Graph {
        let mut g = Graph::new(n, directed).expect("graph");
        for i in 0..(n - 1) {
            g.add_edge(i, i + 1).expect("edge");
        }
        g
    }

    fn complete_undirected(n: u32) -> Graph {
        let mut g = Graph::new(n, false).expect("graph");
        for i in 0..n {
            for j in (i + 1)..n {
                g.add_edge(i, j).expect("edge");
            }
        }
        g
    }

    fn complete_directed_mutual(n: u32) -> Graph {
        let mut g = Graph::new(n, true).expect("graph");
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    g.add_edge(i, j).expect("edge");
                }
            }
        }
        g
    }

    // --- C unit-test fixtures (igraph_cohesion.c) ---

    #[test]
    fn cohesion_c_fixture_directed_7v_equals_one() {
        // edges: 0-1 0-2 1-2 1-3 2-4 3-4 3-5 4-5 1-6 6-3 5-0  (DIRECTED)
        // Expected vc = 1.
        let mut g = Graph::new(7, true).expect("graph");
        for (u, v) in [
            (0u32, 1u32),
            (0, 2),
            (1, 2),
            (1, 3),
            (2, 4),
            (3, 4),
            (3, 5),
            (4, 5),
            (1, 6),
            (6, 3),
            (5, 0),
        ] {
            g.add_edge(u, v).expect("edge");
        }
        assert_eq!(vertex_connectivity(&g, true).expect("vc"), 1);
        assert_eq!(cohesion(&g, true).expect("vc"), 1);
    }

    #[test]
    fn cohesion_c_fixture_undirected_7v_equals_two() {
        // edges: 0-1 0-2 1-2 1-3 2-4 3-4 3-5 4-5 1-6 6-3  (UNDIRECTED)
        // Expected vc = 2.
        let mut g = Graph::new(7, false).expect("graph");
        for (u, v) in [
            (0u32, 1u32),
            (0, 2),
            (1, 2),
            (1, 3),
            (2, 4),
            (3, 4),
            (3, 5),
            (4, 5),
            (1, 6),
            (6, 3),
        ] {
            g.add_edge(u, v).expect("edge");
        }
        assert_eq!(vertex_connectivity(&g, true).expect("vc"), 2);
        assert_eq!(cohesion(&g, true).expect("vc"), 2);
    }

    // --- Edge cases ---

    #[test]
    fn empty_graph_returns_zero() {
        let g = Graph::new(0, false).expect("graph");
        assert_eq!(vertex_connectivity(&g, true).expect("vc"), 0);
        assert_eq!(vertex_connectivity(&g, false).expect("vc"), 0);
    }

    #[test]
    fn single_vertex_returns_zero() {
        let g = Graph::new(1, false).expect("graph");
        assert_eq!(vertex_connectivity(&g, true).expect("vc"), 0);
    }

    #[test]
    fn two_disconnected_vertices_return_zero() {
        let g = Graph::new(2, false).expect("graph");
        assert_eq!(vertex_connectivity(&g, true).expect("vc"), 0);
        assert_eq!(vertex_connectivity(&g, false).expect("vc"), 0);
    }

    #[test]
    fn k2_returns_one() {
        // K_2 is complete: vc = n - 1 = 1.
        let mut g = Graph::new(2, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        assert_eq!(vertex_connectivity(&g, true).expect("vc"), 1);
        assert_eq!(vertex_connectivity(&g, false).expect("vc"), 1);
    }

    // --- R-igraph test parity (test-flow.R:131-138) ---

    #[test]
    fn path_5v_undirected_returns_one() {
        let g = path_graph_n(5, false);
        assert_eq!(vertex_connectivity(&g, true).expect("vc"), 1);
        assert_eq!(vertex_connectivity(&g, false).expect("vc"), 1);
    }

    #[test]
    fn two_isolated_edges_undirected_returns_zero() {
        // make_graph(edges = c(1, 2, 3, 4)) — two disconnected edges
        // 0-1 and 2-3.
        let mut g = Graph::new(4, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(2, 3).expect("edge");
        assert_eq!(vertex_connectivity(&g, true).expect("vc"), 0);
    }

    #[test]
    fn ring_5v_undirected_returns_two() {
        let g = ring_graph_n(5, false);
        assert_eq!(vertex_connectivity(&g, true).expect("vc"), 2);
        assert_eq!(vertex_connectivity(&g, false).expect("vc"), 2);
    }

    // --- Complete-graph short-circuit ---

    #[test]
    fn complete_undirected_6v_returns_five() {
        let g = complete_undirected(6);
        assert_eq!(vertex_connectivity(&g, true).expect("vc"), 5);
        assert_eq!(vertex_connectivity(&g, false).expect("vc"), 5);
    }

    #[test]
    fn complete_directed_5v_returns_four() {
        let g = complete_directed_mutual(5);
        assert_eq!(vertex_connectivity(&g, true).expect("vc"), 4);
        assert_eq!(vertex_connectivity(&g, false).expect("vc"), 4);
    }

    // --- py-igraph test parity (test_flow.py:27,29-30) ---

    #[test]
    fn out_tree_3ary_10v_returns_zero() {
        // Graph.Tree(10, 3, "out") is a directed rooted out-tree (root
        // → children only). Not strongly connected (leaves have no
        // out-edges), so vc = 0.
        let edges: &[(u32, u32)] = &[
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 4),
            (1, 5),
            (1, 6),
            (2, 7),
            (2, 8),
            (2, 9),
        ];
        let mut g = Graph::new(10, true).expect("graph");
        for &(u, v) in edges {
            g.add_edge(u, v).expect("edge");
        }
        assert_eq!(vertex_connectivity(&g, true).expect("vc"), 0);
    }

    #[test]
    fn undirected_tree_3ary_10v_returns_one() {
        // Same tree but undirected — connected, leaves have degree 1
        // → cheap min-degree short-circuit gives 1.
        let edges: &[(u32, u32)] = &[
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 4),
            (1, 5),
            (1, 6),
            (2, 7),
            (2, 8),
            (2, 9),
        ];
        let mut g = Graph::new(10, false).expect("graph");
        for &(u, v) in edges {
            g.add_edge(u, v).expect("edge");
        }
        assert_eq!(vertex_connectivity(&g, true).expect("vc"), 1);
        assert_eq!(vertex_connectivity(&g, false).expect("vc"), 1);
    }

    // --- checks=false vs checks=true agreement ---

    #[test]
    fn checks_false_matches_checks_true_on_small_graphs() {
        let fixtures: Vec<Graph> = vec![
            ring_graph_n(6, false),
            ring_graph_n(6, true),
            path_graph_n(5, false),
            complete_undirected(4),
            complete_directed_mutual(4),
        ];
        for g in fixtures {
            let with_checks = vertex_connectivity(&g, true).expect("vc");
            let without = vertex_connectivity(&g, false).expect("vc");
            assert_eq!(
                with_checks,
                without,
                "checks={{true,false}} disagree on n={}, dir={}",
                g.vcount(),
                g.is_directed()
            );
        }
    }

    // --- Sanity: 2 internally vertex-disjoint paths between every pair ---

    #[test]
    fn two_disjoint_paths_giving_vc_two() {
        // Wheel-like: 0-1-2-3-0 cycle plus chord 0-2 turning a triangle
        // shape. Min degree = 2 (vertex 1 and 3); cycles ensure vc = 2.
        let mut g = Graph::new(4, false).expect("graph");
        for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 0), (0, 2)] {
            g.add_edge(u, v).expect("edge");
        }
        assert_eq!(vertex_connectivity(&g, false).expect("vc"), 2);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    //! Proptest invariants:
    //! * `vertex_connectivity` is bounded above by `n - 1`.
    //! * `vertex_connectivity` is bounded above by the minimum degree
    //!   (Whitney).
    //! * Disconnected graphs return `0`.
    //! * `checks=false` agrees with `checks=true`.

    use super::*;
    use crate::core::Graph;
    use proptest::prelude::*;

    fn xorshift(mut r: u64) -> u64 {
        r ^= r << 13;
        r ^= r >> 7;
        r ^= r << 17;
        r
    }

    fn build_random(seed: u64, n: u32, m_max: u32, directed: bool) -> Graph {
        let mut g = Graph::new(n, directed).expect("graph");
        let mut state = seed | 1;
        for _ in 0..m_max {
            state = xorshift(state);
            let u = u32::try_from(state % u64::from(n)).expect("modulo fits");
            state = xorshift(state);
            let v = u32::try_from(state % u64::from(n)).expect("modulo fits");
            if u == v {
                continue;
            }
            g.add_edge(u, v).expect("edge");
        }
        g
    }

    proptest! {
        #[test]
        fn vc_bounded_by_n_minus_one(
            seed in any::<u64>(),
            n in 2u32..7,
            m in 0u32..14,
            directed in any::<bool>(),
        ) {
            let g = build_random(seed, n, m, directed);
            let vc = vertex_connectivity(&g, true).expect("vc");
            prop_assert!(vc >= 0, "vc must be non-negative, got {vc}");
            prop_assert!(vc <= i64::from(n) - 1,
                "vc={vc} exceeds n-1={} (n={n})", i64::from(n) - 1);
        }

        #[test]
        fn checks_true_matches_checks_false(
            seed in any::<u64>(),
            n in 2u32..6,
            m in 0u32..12,
            directed in any::<bool>(),
        ) {
            let g = build_random(seed, n, m, directed);
            let with_checks = vertex_connectivity(&g, true).expect("vc");
            let without = vertex_connectivity(&g, false).expect("vc");
            prop_assert_eq!(with_checks, without,
                "checks=true {} != checks=false {} (n={}, m={}, directed={}, seed={})",
                with_checks, without, n, m, directed, seed);
        }

        #[test]
        fn vc_bounded_by_min_degree_undirected(
            seed in any::<u64>(),
            n in 3u32..6,
            m in 1u32..10,
        ) {
            // For undirected simple graphs, vc <= min degree (Whitney).
            // Our random builder may produce parallel edges; vc still
            // <= min-degree because each parallel edge contributes to
            // degree but not to the connectivity beyond 1.
            let g = build_random(seed, n, m, false);
            let mut min_deg = u32::MAX;
            for v in 0..n {
                let d = u32::try_from(g.degree(v).expect("degree")).unwrap_or(u32::MAX);
                if d < min_deg { min_deg = d; }
            }
            let vc = vertex_connectivity(&g, true).expect("vc");
            // vc <= min_deg only meaningful when graph has no isolated
            // vertices; when there are isolated vertices, vc = 0 = min_deg.
            prop_assert!(vc <= i64::from(min_deg),
                "vc={vc} > min_deg={min_deg} (n={n}, m={m}, seed={seed})");
        }
    }
}
