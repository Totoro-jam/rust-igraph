//! Ring / path / cycle constructors (ALGO-CN-001).
//!
//! Counterpart of `igraph_ring()`, `igraph_path_graph()` and
//! `igraph_cycle_graph()` in
//! `references/igraph/src/constructors/regular.c:495-604`.
//!
//! The model lays `n` vertices in a sequence `0, 1, …, n-1` and joins
//! consecutive vertices in order. Three boolean flags shape the result:
//!
//! * `directed` — emit a digraph. Edges always run in the forward
//!   direction `(i, i+1)`. Ignored only by the parity of `mutual`.
//! * `mutual` — *only* applied when `directed` is `true`; emits the
//!   back-arc `(i+1, i)` alongside every forward arc. Undirected
//!   construction ignores `mutual` entirely (matching upstream).
//! * `circular` — close the sequence with the wrap-around edge
//!   `(n-1, 0)`. With `circular = false` the result is the path
//!   `P_n`; with `circular = true` it is the cycle `C_n`.
//!
//! Edge count is `n - 1` for a path and `n` for a cycle, doubled when
//! `directed && mutual`. The two convenience wrappers
//! [`path_graph`] (alias for `circular = false`) and
//! [`cycle_graph`] (alias for `circular = true`) mirror the upstream
//! one-line wrappers exactly.
//!
//! Degenerate cases mirror upstream behaviour verbatim:
//!
//! * `n == 0` — the empty graph.
//! * `n == 1`, `circular == false` — a single isolated vertex.
//! * `n == 1`, `circular == true` — a self-loop `(0, 0)`. **Not simple.**
//! * `n == 2`, `circular == true`, `directed == false` — two parallel
//!   edges between `0` and `1`. **Not simple.**
//!
//! Time complexity: O(|V|).

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Lay `n` vertices in a sequence and join them; close the loop iff
/// `circular`.
///
/// See the module-level docs for the precise role of each flag.
///
/// # Errors
///
/// Returns [`IgraphError::Internal`] if the implied edge count would
/// overflow `usize` — only reachable on 32-bit targets with absurdly
/// large `n`.
///
/// # Examples
///
/// ```
/// use rust_igraph::ring_graph;
///
/// // Undirected cycle on 5 vertices.
/// let c5 = ring_graph(5, false, false, true).unwrap();
/// assert_eq!(c5.vcount(), 5);
/// assert_eq!(c5.ecount(), 5);
///
/// // Open path on 4 vertices, no wrap-around.
/// let p4 = ring_graph(4, false, false, false).unwrap();
/// assert_eq!(p4.ecount(), 3);
/// ```
pub fn ring_graph(n: u32, directed: bool, mutual: bool, circular: bool) -> IgraphResult<Graph> {
    if n == 0 {
        return Graph::new(0, directed);
    }

    let n_usize = n as usize;
    let forward_edges = if circular { n_usize } else { n_usize - 1 };
    let edges_per_link = if directed && mutual { 2 } else { 1 };
    let total_edges = forward_edges
        .checked_mul(edges_per_link)
        .ok_or(IgraphError::Internal("ring_graph edge count overflow"))?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(total_edges);

    for i in 0..n.saturating_sub(1) {
        let next = i + 1;
        edges.push((i, next));
        if directed && mutual {
            edges.push((next, i));
        }
    }
    if circular {
        let last = n - 1;
        edges.push((last, 0));
        if directed && mutual {
            edges.push((0, last));
        }
    }

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

/// Path graph `P_n`: convenience wrapper for `ring_graph(n, directed,
/// mutual, false)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::path_graph;
///
/// let p3 = path_graph(3, false, false).unwrap();
/// assert_eq!(p3.ecount(), 2);
/// ```
pub fn path_graph(n: u32, directed: bool, mutual: bool) -> IgraphResult<Graph> {
    ring_graph(n, directed, mutual, false)
}

/// Cycle graph `C_n`: convenience wrapper for `ring_graph(n, directed,
/// mutual, true)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::cycle_graph;
///
/// let c4 = cycle_graph(4, false, false).unwrap();
/// assert_eq!(c4.ecount(), 4);
/// ```
pub fn cycle_graph(n: u32, directed: bool, mutual: bool) -> IgraphResult<Graph> {
    ring_graph(n, directed, mutual, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..m)
            .map(|eid| g.edge(eid).expect("edge id in bounds"))
            .collect()
    }

    #[test]
    fn empty_graph() {
        for &directed in &[false, true] {
            for &mutual in &[false, true] {
                for &circular in &[false, true] {
                    let g = ring_graph(0, directed, mutual, circular).expect("ring n=0");
                    assert_eq!(g.vcount(), 0);
                    assert_eq!(g.ecount(), 0);
                    assert_eq!(g.is_directed(), directed);
                }
            }
        }
    }

    #[test]
    fn singleton_path_is_isolated_vertex() {
        let g = ring_graph(1, false, false, false).expect("ring n=1 path");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn singleton_cycle_is_self_loop() {
        let g = ring_graph(1, false, false, true).expect("ring n=1 cycle");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 1);
        assert_eq!(collect_edges(&g), vec![(0, 0)]);
    }

    #[test]
    fn two_vertex_undirected_cycle_has_parallel_edges() {
        let g = ring_graph(2, false, false, true).expect("ring n=2 undirected cycle");
        // Undirected canonicalisation stores both as (0, 1) — two parallel edges.
        assert_eq!(g.ecount(), 2);
        assert_eq!(collect_edges(&g), vec![(0, 1), (0, 1)]);
    }

    #[test]
    fn undirected_path_p5_canonical_edges() {
        let g = ring_graph(5, false, false, false).expect("ring P5");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 4);
        assert!(!g.is_directed());
        assert_eq!(collect_edges(&g), vec![(0, 1), (1, 2), (2, 3), (3, 4)]);
    }

    #[test]
    fn undirected_cycle_c5_closes_with_wraparound() {
        let g = ring_graph(5, false, false, true).expect("ring C5");
        // Undirected canonicalisation stores the wrap-around as (0, 4), not (4, 0).
        assert_eq!(g.ecount(), 5);
        assert_eq!(
            collect_edges(&g),
            vec![(0, 1), (1, 2), (2, 3), (3, 4), (0, 4)]
        );
    }

    #[test]
    fn directed_path_p4_forward_only() {
        let g = ring_graph(4, true, false, false).expect("ring directed P4");
        assert!(g.is_directed());
        assert_eq!(collect_edges(&g), vec![(0, 1), (1, 2), (2, 3)]);
    }

    #[test]
    fn directed_cycle_c4_forward_only() {
        let g = ring_graph(4, true, false, true).expect("ring directed C4");
        assert_eq!(collect_edges(&g), vec![(0, 1), (1, 2), (2, 3), (3, 0)]);
    }

    #[test]
    fn directed_mutual_path_emits_back_arcs_in_order() {
        let g = ring_graph(4, true, true, false).expect("ring directed mutual P4");
        assert_eq!(g.ecount(), 6);
        assert_eq!(
            collect_edges(&g),
            vec![(0, 1), (1, 0), (1, 2), (2, 1), (2, 3), (3, 2)]
        );
    }

    #[test]
    fn directed_mutual_cycle_emits_wraparound_back_arc_last() {
        let g = ring_graph(4, true, true, true).expect("ring directed mutual C4");
        assert_eq!(g.ecount(), 8);
        assert_eq!(
            collect_edges(&g),
            vec![
                (0, 1),
                (1, 0),
                (1, 2),
                (2, 1),
                (2, 3),
                (3, 2),
                (3, 0),
                (0, 3),
            ]
        );
    }

    #[test]
    fn undirected_ignores_mutual_flag() {
        // mutual must be IGNORED when directed == false. Both shapes
        // must agree edge-for-edge.
        for &circular in &[false, true] {
            let g_no_mutual = ring_graph(6, false, false, circular).expect("undirected no mutual");
            let g_mutual = ring_graph(6, false, true, circular).expect("undirected w/ mutual");
            assert_eq!(collect_edges(&g_no_mutual), collect_edges(&g_mutual));
        }
    }

    #[test]
    fn path_wrapper_matches_ring_circular_false() {
        for &directed in &[false, true] {
            for &mutual in &[false, true] {
                let a = path_graph(7, directed, mutual).expect("path_graph");
                let b = ring_graph(7, directed, mutual, false).expect("ring circular=false");
                assert_eq!(collect_edges(&a), collect_edges(&b));
                assert_eq!(a.is_directed(), b.is_directed());
            }
        }
    }

    #[test]
    fn cycle_wrapper_matches_ring_circular_true() {
        for &directed in &[false, true] {
            for &mutual in &[false, true] {
                let a = cycle_graph(8, directed, mutual).expect("cycle_graph");
                let b = ring_graph(8, directed, mutual, true).expect("ring circular=true");
                assert_eq!(collect_edges(&a), collect_edges(&b));
                assert_eq!(a.is_directed(), b.is_directed());
            }
        }
    }

    #[test]
    fn vertex_degrees_path_undirected() {
        let g = ring_graph(10, false, false, false).expect("undirected P10");
        // Interior vertices have degree 2; endpoints have degree 1.
        for v in 0..10u32 {
            let deg = g.neighbors(v).expect("neighbors").len();
            let expected = if v == 0 || v == 9 { 1 } else { 2 };
            assert_eq!(deg, expected, "vertex {v}");
        }
    }

    #[test]
    fn vertex_degrees_cycle_undirected() {
        let g = ring_graph(10, false, false, true).expect("undirected C10");
        for v in 0..10u32 {
            let deg = g.neighbors(v).expect("neighbors").len();
            assert_eq!(deg, 2, "vertex {v} (cycle)");
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn ring_edge_count_matches_formula(
            n in 0u32..40,
            directed in any::<bool>(),
            mutual in any::<bool>(),
            circular in any::<bool>(),
        ) {
            let g = ring_graph(n, directed, mutual, circular).unwrap();
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(g.is_directed(), directed);

            let expected = if n == 0 {
                0
            } else {
                let base = if circular { n } else { n - 1 };
                let factor = if directed && mutual { 2 } else { 1 };
                base * factor
            };
            prop_assert_eq!(u32::try_from(g.ecount()).unwrap(), expected);
        }

        #[test]
        fn undirected_ignores_mutual_property(
            n in 0u32..40,
            circular in any::<bool>(),
        ) {
            let a = ring_graph(n, false, false, circular).unwrap();
            let b = ring_graph(n, false, true, circular).unwrap();
            let m_a = u32::try_from(a.ecount()).unwrap();
            let m_b = u32::try_from(b.ecount()).unwrap();
            let edges_a: Vec<_> = (0..m_a).map(|e| a.edge(e).unwrap()).collect();
            let edges_b: Vec<_> = (0..m_b).map(|e| b.edge(e).unwrap()).collect();
            prop_assert_eq!(edges_a, edges_b);
        }

        #[test]
        fn path_cycle_wrappers_agree_with_ring(
            n in 0u32..40,
            directed in any::<bool>(),
            mutual in any::<bool>(),
        ) {
            let p_wrap = path_graph(n, directed, mutual).unwrap();
            let p_ring = ring_graph(n, directed, mutual, false).unwrap();
            let c_wrap = cycle_graph(n, directed, mutual).unwrap();
            let c_ring = ring_graph(n, directed, mutual, true).unwrap();

            let collect = |g: &Graph| -> Vec<(VertexId, VertexId)> {
                let m = u32::try_from(g.ecount()).unwrap();
                (0..m).map(|e| g.edge(e).unwrap()).collect()
            };

            prop_assert_eq!(collect(&p_wrap), collect(&p_ring));
            prop_assert_eq!(collect(&c_wrap), collect(&c_ring));
        }
    }
}
