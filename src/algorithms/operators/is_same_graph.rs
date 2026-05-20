//! Structural graph equality (ALGO-CORE-001e, `is_same_graph` slice).
//!
//! Counterpart of `igraph_is_same_graph()` from
//! `references/igraph/src/graph/type_indexededgelist.c:1947`.
//! Returns `true` iff two graphs have the same vertex count, the
//! same directedness, and the same edge multiset. Edge ordering
//! and the orientation of undirected edges are normalised away —
//! only set-equality of edges matters.
//!
//! This is **not** isomorphism: `0-1, 1-2` and `0-2, 1-2` are
//! isomorphic but not "the same graph" because they use different
//! edges. The vertex labels (0..vcount) are taken as ground truth.

use crate::core::Graph;

/// Returns `true` iff `g1` and `g2` are identical as labelled
/// graphs: same vertex count, same directedness, same edge
/// multiset. Edges differing only in storage order (or, for
/// undirected, in endpoint orientation) are treated as identical.
///
/// Time complexity: `O(E log E)` due to the canonical sort of the
/// two edge lists — slightly worse than upstream's `O(E)` walk over
/// pre-sorted `ii` indices, but our `Graph` keeps that sort as a
/// private detail so we re-sort here. Edge counts up to a few
/// million sort in well under a second.
///
/// Note this is **not** isomorphism: vertex labels matter. Use a
/// dedicated isomorphism checker (future AWU) if you need to ignore
/// relabelling.
///
/// Counterpart of `igraph_is_same_graph` from
/// `references/igraph/src/graph/type_indexededgelist.c:1947`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_same_graph};
///
/// // Same edge set, different insertion order ⇒ same graph.
/// let mut g1 = Graph::with_vertices(3);
/// g1.add_edge(0, 1).unwrap();
/// g1.add_edge(1, 2).unwrap();
/// let mut g2 = Graph::with_vertices(3);
/// g2.add_edge(1, 2).unwrap();
/// g2.add_edge(0, 1).unwrap();
/// assert!(is_same_graph(&g1, &g2));
///
/// // Same vcount + ecount but different edge set ⇒ not the same.
/// let mut g3 = Graph::with_vertices(3);
/// g3.add_edge(0, 2).unwrap();
/// g3.add_edge(1, 2).unwrap();
/// assert!(!is_same_graph(&g1, &g3));
/// ```
#[must_use]
pub fn is_same_graph(g1: &Graph, g2: &Graph) -> bool {
    if g1.vcount() != g2.vcount() {
        return false;
    }
    if g1.ecount() != g2.ecount() {
        return false;
    }
    if g1.is_directed() != g2.is_directed() {
        return false;
    }

    let m = g1.ecount();
    if m == 0 {
        return true;
    }

    // Canonical form: for undirected, `Graph::add_edge` already
    // stores `from <= to`. For directed, the (from, to) tuple is
    // already canonical. So lex-sorting the (from, to) pair lists
    // and comparing element-wise is enough.
    // Edge count exceeding u32::MAX is impossible given EdgeId is
    // u32, but if it ever happened fall back to "not the same".
    let Ok(m_u32) = u32::try_from(m) else {
        return false;
    };
    let mut e1: Vec<(u32, u32)> = (0..m_u32)
        .map(|e| g1.edge(e).expect("valid edge id"))
        .collect();
    let mut e2: Vec<(u32, u32)> = (0..m_u32)
        .map(|e| g2.edge(e).expect("valid edge id"))
        .collect();
    // `(u32, u32)` derives `Ord` with lex ordering already; no
    // custom comparator needed.
    e1.sort_unstable();
    e2.sort_unstable();
    e1 == e2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Graph;

    #[test]
    fn same_graph_identical_construction() {
        let mut g1 = Graph::with_vertices(3);
        g1.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
        let mut g2 = Graph::with_vertices(3);
        g2.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
        assert!(is_same_graph(&g1, &g2));
    }

    #[test]
    fn same_graph_edge_order_does_not_matter() {
        let mut g1 = Graph::with_vertices(3);
        g1.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
        let mut g2 = Graph::with_vertices(3);
        g2.add_edges(vec![(1u32, 2u32), (0, 1)]).unwrap();
        assert!(is_same_graph(&g1, &g2));
    }

    #[test]
    fn same_graph_undirected_endpoint_orientation_does_not_matter() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::with_vertices(2);
        g2.add_edge(1, 0).unwrap();
        assert!(is_same_graph(&g1, &g2));
    }

    #[test]
    fn different_vcount_not_same() {
        let mut g1 = Graph::with_vertices(3);
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::with_vertices(4);
        g2.add_edge(0, 1).unwrap();
        assert!(!is_same_graph(&g1, &g2));
    }

    #[test]
    fn different_ecount_not_same() {
        let mut g1 = Graph::with_vertices(3);
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::with_vertices(3);
        g2.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
        assert!(!is_same_graph(&g1, &g2));
    }

    #[test]
    fn directed_vs_undirected_not_same() {
        let mut g1 = Graph::new(3, true).unwrap();
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::with_vertices(3);
        g2.add_edge(0, 1).unwrap();
        assert!(!is_same_graph(&g1, &g2));
    }

    #[test]
    fn directed_edge_direction_matters() {
        // 0→1 and 1→0 are distinct directed edges.
        let mut g1 = Graph::new(2, true).unwrap();
        g1.add_edge(0, 1).unwrap();
        let mut g2 = Graph::new(2, true).unwrap();
        g2.add_edge(1, 0).unwrap();
        assert!(!is_same_graph(&g1, &g2));
    }

    #[test]
    fn parallel_edges_multiplicity_matters() {
        // {0-1, 0-1} ≠ {0-1} even though vertex/edge sets look similar.
        let mut g1 = Graph::with_vertices(2);
        g1.add_edges(vec![(0u32, 1u32), (0, 1)]).unwrap();
        let mut g2 = Graph::with_vertices(2);
        g2.add_edges(vec![(0u32, 1u32), (0, 1)]).unwrap();
        assert!(is_same_graph(&g1, &g2));
        // Now drop one parallel edge in g2.
        let mut g3 = Graph::with_vertices(2);
        g3.add_edge(0, 1).unwrap();
        assert!(!is_same_graph(&g1, &g3));
    }

    #[test]
    fn self_loop_recognised_as_same() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edges(vec![(0u32, 0u32), (0, 1)]).unwrap();
        let mut g2 = Graph::with_vertices(2);
        g2.add_edges(vec![(0u32, 1u32), (0, 0)]).unwrap();
        assert!(is_same_graph(&g1, &g2));
    }

    #[test]
    fn empty_graphs_are_same_when_vcount_matches() {
        let g1 = Graph::with_vertices(0);
        let g2 = Graph::with_vertices(0);
        assert!(is_same_graph(&g1, &g2));
        let g3 = Graph::with_vertices(5);
        let g4 = Graph::with_vertices(5);
        assert!(is_same_graph(&g3, &g4));
    }

    #[test]
    fn isomorphic_but_different_edge_sets_not_same() {
        // Both are paths of length 2 but on different vertex labellings:
        // g1: 0-1-2; g2: 0-2-1. Isomorphic, not "same".
        let mut g1 = Graph::with_vertices(3);
        g1.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
        let mut g2 = Graph::with_vertices(3);
        g2.add_edges(vec![(0u32, 2u32), (1, 2)]).unwrap();
        assert!(!is_same_graph(&g1, &g2));
    }

    #[test]
    fn reflexivity() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 3)]).unwrap();
        let g2 = g.clone();
        assert!(is_same_graph(&g, &g2));
    }
}
