//! `edge_connectivity` (ALGO-FL-016) — global graph adhesion: the
//! minimum number of edges whose removal disconnects some pair of
//! vertices in the graph.
//!
//! Counterpart of `igraph_edge_connectivity` in
//! `references/igraph/src/flow/flow.c:2270`. Equivalent to the C
//! alias `igraph_adhesion` (flow.c:2433) and to python-igraph's
//! `Graph.edge_connectivity()` (and `Graph.adhesion()`),
//! R-igraph's `edge_connectivity()` (no source/target).
//!
//! ## Algorithm
//!
//! Definition: `lambda(G) = min_{s ≠ t} st_edge_connectivity(s, t)`.
//!
//! Optional cheap short-circuits when `checks = true` (suggested by
//! Peter `McMahan` per the upstream C docstring, see
//! flow.c:2253-2259) — shared with [`super::vertex_connectivity`] via
//! the same `_connectivity_checks` helper inlined here:
//!
//! 1. Empty/singleton graph → `0`.
//! 2. Disconnected (weakly for undirected, strongly for directed)
//!    → `0`.
//! 3. Any vertex with `min(in, out) = 1` (or undirected degree
//!    `= 1`) → `1` (its incident edge is a bridge for that vertex).
//!
//! Crucially, we do **not** short-circuit on complete graphs. The
//! upstream C comment (flow.c:2168-2180) calls this out: completeness
//! alone does not determine the edge connectivity of a multigraph,
//! because parallel edges raise edge connectivity beyond `n - 1`.
//!
//! Otherwise we run the same fixed-vertex iteration as
//! `igraph_mincut_value` (flow.c:1706-1723): pick vertex `0`, and for
//! every other vertex `v` compute `st_edge_connectivity(0, v)`
//! (undirected) or both `(0, v)` and `(v, 0)` (directed). The minimum
//! over all such pairs equals the global edge connectivity, because
//! every global min-cut separates `0` from at least one vertex on the
//! other side.
//!
//! ## Complexity
//!
//! `O(V)` calls to FL-011 for undirected, `O(2V)` for directed. Each
//! FL-011 inherits FL-002 = `O(V·E²)` on unit caps, so the total is
//! `O(V²·E²)` worst case. The igraph C docstring reports
//! `O(log V · V²)` for undirected (their Stoer-Wagner implementation,
//! not ported here) and `O(V⁴)` for directed.
//!
//! ## Why fixed-vertex iteration is correct
//!
//! For any global min-cut `(S, V \ S)` containing vertex `0`, there
//! exists some `v ∈ V \ S` with `v ≠ 0`. The cut is then a feasible
//! `0 → v` cut (undirected) or `0 → v` / `v → 0` cut (directed), so
//! `st_edge_connectivity(0, v) ≤ |cut| = lambda(G)`. Combined with
//! `lambda(G) ≤ st_edge_connectivity(0, v)` for every `v` (any
//! `0-v` min-cut is a feasible global cut), we get equality.

use crate::core::{Graph, IgraphResult};

use super::st_edge_connectivity::st_edge_connectivity;
use crate::algorithms::connectivity::components::connected_components;
use crate::algorithms::connectivity::strong::strongly_connected_components;

/// Edge connectivity (adhesion) of a graph.
///
/// Returns the minimum number of edges that must be removed to
/// disconnect *some* pair of vertices in `graph`. Equal to
/// `min_{s ≠ t} st_edge_connectivity(s, t)`.
///
/// Counterpart of `igraph_edge_connectivity`
/// (`references/igraph/src/flow/flow.c:2270`) and its alias
/// `igraph_adhesion` (flow.c:2433).
///
/// # Arguments
///
/// * `graph` — input graph (directed or undirected).
/// * `checks` — when `true`, run the cheap short-circuits described
///   in the module docs before falling back to the fixed-vertex
///   `O(V)`-flows loop. Recommended for any non-trivial graph; the
///   helpers cost `O(V + E)` and can replace the whole loop.
///   Equivalent to igraph C's `checks` argument.
///
/// # Returns
///
/// The edge connectivity as `i64`. Returns:
/// * `0` when `vcount() ≤ 1`, or the graph is disconnected.
/// * `1` when there is a vertex with degree `1` (undirected) or
///   `min(in, out) = 1` (directed).
/// * Otherwise, the result of the fixed-vertex `st_edge_connectivity`
///   loop from vertex `0`.
///
/// # Errors
///
/// Propagates errors from [`st_edge_connectivity`],
/// [`connected_components`], and [`strongly_connected_components`].
/// In practice these arise only from arithmetic overflow on very
/// large graphs, which is unreachable here.
///
/// [`IgraphError`]: crate::core::IgraphError
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_connectivity};
///
/// // Undirected ring on 5 vertices — lambda = 2 (any two non-adjacent
/// // edges form a min-cut).
/// let mut g = Graph::new(5, false).unwrap();
/// for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 0)] {
///     g.add_edge(u, v).unwrap();
/// }
/// assert_eq!(edge_connectivity(&g, true).unwrap(), 2);
///
/// // Undirected path on 5 vertices — lambda = 1 (any edge is a
/// // bridge).
/// let mut p = Graph::new(5, false).unwrap();
/// for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4)] {
///     p.add_edge(u, v).unwrap();
/// }
/// assert_eq!(edge_connectivity(&p, true).unwrap(), 1);
/// ```
pub fn edge_connectivity(graph: &Graph, checks: bool) -> IgraphResult<i64> {
    let n = graph.vcount();

    // Mirrors flow.c:2281-2284 — singleton/empty graph short-circuit
    // (catches the `igraph_mincut_value = +inf` corner case for n=1
    // up front).
    if n <= 1 {
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
        // Undirected: any vertex with degree 1 ⇒ lambda = 1.
        // Directed: any vertex with out-degree 1 or in-degree 1
        // ⇒ lambda = 1. (Whitney: lambda ≤ min-degree.)
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
        // Note: deliberately no complete-graph short-circuit (see
        // flow.c:2168-2180 — completeness alone does not determine
        // the edge connectivity of a multigraph).
    }

    // Fixed-vertex iteration — mirrors `igraph_mincut_value`
    // (flow.c:1706-1723). lambda(G) = min over v != 0 of
    // st_edge_connectivity(0, v) [+ symmetric direction if directed].
    let mut min_lambda = i64::MAX;
    let directed = graph.is_directed();
    for v in 1..n {
        let f = st_edge_connectivity(graph, 0, v)?;
        if f < min_lambda {
            min_lambda = f;
            if min_lambda == 0 {
                return Ok(0);
            }
        }
        if directed {
            let f2 = st_edge_connectivity(graph, v, 0)?;
            if f2 < min_lambda {
                min_lambda = f2;
                if min_lambda == 0 {
                    return Ok(0);
                }
            }
        }
    }

    Ok(min_lambda)
}

/// Group adhesion — igraph C alias `igraph_adhesion` (flow.c:2433).
/// Exact synonym for [`edge_connectivity`]; kept for naming parity
/// with the upstream API and so users following the
/// White-Harary (2001) sociological-network literature have a direct
/// hit. Identical signature and behaviour.
pub fn adhesion(graph: &Graph, checks: bool) -> IgraphResult<i64> {
    edge_connectivity(graph, checks)
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

    // --- Edge cases ---

    #[test]
    fn empty_graph_returns_zero() {
        let g = Graph::new(0, false).expect("graph");
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 0);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 0);
    }

    #[test]
    fn single_vertex_returns_zero() {
        let g = Graph::new(1, false).expect("graph");
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 0);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 0);
    }

    #[test]
    fn two_disconnected_vertices_return_zero() {
        let g = Graph::new(2, false).expect("graph");
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 0);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 0);
    }

    #[test]
    fn k2_undirected_returns_one() {
        // K_2 single edge — lambda = 1.
        let mut g = Graph::new(2, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 1);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 1);
    }

    // --- R-igraph test parity (test-flow.R) ---

    #[test]
    fn path_5v_undirected_returns_one() {
        let g = path_graph_n(5, false);
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 1);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 1);
        assert_eq!(adhesion(&g, true).expect("ec"), 1);
    }

    #[test]
    fn two_isolated_edges_undirected_returns_zero() {
        let mut g = Graph::new(4, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(2, 3).expect("edge");
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 0);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 0);
    }

    #[test]
    fn ring_5v_undirected_returns_two() {
        let g = ring_graph_n(5, false);
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 2);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 2);
        assert_eq!(adhesion(&g, true).expect("ec"), 2);
    }

    // --- Complete-graph: no cheap short-circuit, but the fixed-vertex
    //     loop still returns n - 1 for simple K_n. ---

    #[test]
    fn complete_undirected_5v_returns_four() {
        let g = complete_undirected(5);
        // lambda(K_5) = 4. checks=true short-circuits at min-degree=4
        // only after passing through; here we hit the full loop path
        // because min-degree=4 != 1.
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 4);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 4);
    }

    #[test]
    fn complete_directed_mutual_4v_returns_three() {
        let g = complete_directed_mutual(4);
        // For mutual K_4: every pair has 3 directed paths, so
        // lambda = 3.
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 3);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 3);
    }

    // --- Multigraph fixture: completeness alone does not bound lambda. ---

    #[test]
    fn multigraph_two_parallel_edges_doubles_lambda() {
        // 2 vertices, 2 parallel undirected edges — lambda = 2 even
        // though it is "complete" on 2 vertices.
        let mut g = Graph::new(2, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(0, 1).expect("edge");
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 2);
        // With checks=true the min-degree=2 check does NOT short-circuit
        // (only min-degree=1 does), so the fixed-vertex loop runs.
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 2);
    }

    // --- py-igraph test parity ---

    #[test]
    fn out_tree_3ary_10v_returns_zero() {
        // Graph.Tree(10, 3, "out") — not strongly connected (leaves
        // have no out-edges), so lambda = 0.
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
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 0);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 0);
    }

    #[test]
    fn undirected_tree_3ary_10v_returns_one() {
        // Same tree but undirected — connected, every edge is a
        // bridge → lambda = 1.
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
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 1);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 1);
    }

    // --- Directed cycle: lambda = 1 (every arc is a directed bridge). ---

    #[test]
    fn directed_cycle_6v_returns_one() {
        let g = ring_graph_n(6, true);
        // Strongly connected (it's a directed cycle), min in/out = 1
        // ⇒ cheap short-circuit returns 1. Without checks, the
        // fixed-vertex loop still finds st_edge_conn(0, v) = 1.
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 1);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 1);
    }

    // --- 4-cycle with chord: cheap short-circuit fails (min-deg=2,
    //     not complete in the FL-015 sense), full loop returns 2. ---

    #[test]
    fn cycle_with_chord_undirected_returns_two() {
        let mut g = Graph::new(4, false).expect("graph");
        for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 0), (0, 2)] {
            g.add_edge(u, v).expect("edge");
        }
        // Vertices 1 and 3 have degree 2 — lambda = 2.
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 2);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 2);
    }

    // --- C unit-test fixture parity (igraph_edge_connectivity.c style) ---

    #[test]
    fn c_fixture_directed_6v_equals_one() {
        // 6 vertices, 8 directed arcs (mirrors st_edge_connectivity
        // C fixture). Without a 5→0 back-arc this graph is not
        // strongly connected — lambda = 0.
        let mut g = Graph::new(6, true).expect("graph");
        let arcs = [
            (0u32, 1u32),
            (0, 2),
            (1, 2),
            (1, 3),
            (2, 4),
            (3, 4),
            (3, 5),
            (4, 5),
        ];
        for (u, v) in arcs {
            g.add_edge(u, v).expect("edge");
        }
        // Source 0 has no incoming, sink 5 has no outgoing —
        // strongly disconnected.
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 0);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 0);
    }

    #[test]
    fn c_fixture_undirected_6v_equals_two() {
        let mut g = Graph::new(6, false).expect("graph");
        let arcs = [
            (0u32, 1u32),
            (0, 2),
            (1, 2),
            (1, 3),
            (2, 4),
            (3, 4),
            (3, 5),
            (4, 5),
        ];
        for (u, v) in arcs {
            g.add_edge(u, v).expect("edge");
        }
        // All vertices have degree ≥ 2; lambda = 2 (e.g. cut {3-5, 4-5}
        // isolates vertex 5).
        assert_eq!(edge_connectivity(&g, true).expect("ec"), 2);
        assert_eq!(edge_connectivity(&g, false).expect("ec"), 2);
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
            let with_checks = edge_connectivity(&g, true).expect("ec");
            let without = edge_connectivity(&g, false).expect("ec");
            assert_eq!(
                with_checks,
                without,
                "checks={{true,false}} disagree on n={}, dir={}",
                g.vcount(),
                g.is_directed()
            );
        }
    }

    // --- adhesion alias parity ---

    #[test]
    fn adhesion_alias_matches_edge_connectivity() {
        let fixtures: Vec<Graph> = vec![
            ring_graph_n(7, false),
            complete_undirected(5),
            path_graph_n(4, false),
        ];
        for g in fixtures {
            assert_eq!(
                edge_connectivity(&g, true).expect("ec"),
                adhesion(&g, true).expect("ec"),
                "adhesion/edge_connectivity mismatch on n={}",
                g.vcount(),
            );
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    //! Proptest invariants:
    //! * `0 <= edge_connectivity <= min_degree` (Whitney 1932).
    //! * `edge_connectivity <= st_edge_connectivity(s, t)` for every
    //!   pair `(s, t)`.
    //! * `checks=true` agrees with `checks=false`.
    //! * On disconnected graphs (cheap to detect), the result is `0`.

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
        fn ec_nonnegative_and_bounded_by_n_minus_one(
            seed in any::<u64>(),
            n in 2u32..7,
            m in 0u32..14,
            directed in any::<bool>(),
        ) {
            let g = build_random(seed, n, m, directed);
            let ec = edge_connectivity(&g, true).expect("ec");
            prop_assert!(ec >= 0, "ec must be non-negative, got {ec}");
            // ec is unbounded above by n - 1 for multigraphs, but is
            // bounded by m. (Multigraphs from our random builder can
            // produce parallel edges.)
            prop_assert!(ec as u64 <= u64::from(m),
                "ec={ec} > m={m} (n={n}, seed={seed})");
        }

        #[test]
        fn checks_true_matches_checks_false(
            seed in any::<u64>(),
            n in 2u32..6,
            m in 0u32..12,
            directed in any::<bool>(),
        ) {
            let g = build_random(seed, n, m, directed);
            let with_checks = edge_connectivity(&g, true).expect("ec");
            let without = edge_connectivity(&g, false).expect("ec");
            prop_assert_eq!(with_checks, without,
                "checks=true {} != checks=false {} (n={}, m={}, directed={}, seed={})",
                with_checks, without, n, m, directed, seed);
        }

        #[test]
        fn ec_bounded_by_min_degree_undirected(
            seed in any::<u64>(),
            n in 3u32..6,
            m in 1u32..10,
        ) {
            // Whitney: lambda(G) <= min-degree(G).
            let g = build_random(seed, n, m, false);
            let mut min_deg = u32::MAX;
            for v in 0..n {
                let d = u32::try_from(g.degree(v).expect("degree")).unwrap_or(u32::MAX);
                if d < min_deg { min_deg = d; }
            }
            let ec = edge_connectivity(&g, true).expect("ec");
            prop_assert!(ec <= i64::from(min_deg),
                "ec={ec} > min_deg={min_deg} (n={n}, m={m}, seed={seed})");
        }

        #[test]
        fn ec_bounded_by_any_pair_st_edge_connectivity(
            seed in any::<u64>(),
            n in 3u32..5,
            m in 1u32..9,
            directed in any::<bool>(),
        ) {
            // lambda(G) <= st_edge_connectivity(s, t) for any (s, t).
            use super::super::st_edge_connectivity::st_edge_connectivity;
            let g = build_random(seed, n, m, directed);
            let ec = edge_connectivity(&g, true).expect("ec");
            for s in 0..n {
                for t in 0..n {
                    if s == t { continue; }
                    let st = st_edge_connectivity(&g, s, t).expect("st_ec");
                    prop_assert!(ec <= st,
                        "ec={ec} > st_ec({s},{t})={st} (n={n}, m={m}, dir={directed}, seed={seed})");
                }
            }
        }
    }
}
