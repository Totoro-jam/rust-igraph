//! Complementer (ALGO-OP-003).
//!
//! Counterpart of `igraph_complementer()` from
//! `references/igraph/src/operators/complementer.c`. The complementer
//! graph contains every edge `(u, v)` that the input graph does NOT
//! contain. For undirected graphs, only unordered pairs `u < v` are
//! considered; for directed graphs, ordered pairs `(u, v)` are
//! considered for all `u, v`. The `loops` flag toggles whether self-
//! loops `(v, v)` may appear in the output.
//!
//! Phase-1 minimal slice: no attribute copy. Vertex / edge attributes
//! are dropped (ALGO-AT-*).

use std::collections::BTreeSet;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphResult, VertexId};

/// Returns the complementer of `graph`.
///
/// If `loops == true`, self-loops `(v, v)` are added for every vertex
/// that does not already have one. If `loops == false`, no self-loops
/// appear in the output regardless of the input.
///
/// The output's vertex count equals the input's; the output's edge
/// count is `n*(n-1) − ecount` (directed) or `n*(n-1)/2 − ecount`
/// (undirected) when `loops == false`, plus `n` extra slots if
/// `loops == true` for self-loops not in the input.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, complementer};
///
/// // Path 0-1-2 → complementer is the single edge (0, 2).
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let c = complementer(&g, false).unwrap();
/// assert_eq!(c.ecount(), 1);
/// assert_eq!(c.edge(0).unwrap(), (0, 2));
/// ```
pub fn complementer(graph: &Graph, loops: bool) -> IgraphResult<Graph> {
    let n = graph.vcount();
    let directed = graph.is_directed();
    let mut g = Graph::new(n, directed)?;
    if n == 0 {
        return Ok(g);
    }

    // Snapshot original edges as a sorted-set of canonical pairs.
    // Storage already canonicalises undirected (from <= to), so a single
    // sort is enough. We also count parallel originals as a single
    // existing edge — complementer ignores multiplicity.
    let m = u32::try_from(graph.ecount())
        .map_err(|_| crate::IgraphError::Internal("ecount exceeds u32::MAX"))?;
    let mut existing: BTreeSet<(VertexId, VertexId)> = BTreeSet::new();
    for e in 0..m {
        existing.insert(graph.edge(e as EdgeId)?);
    }

    let mut new_edges: Vec<(VertexId, VertexId)> = Vec::new();
    if directed {
        for u in 0..n {
            for v in 0..n {
                if u == v && !loops {
                    continue;
                }
                if !existing.contains(&(u, v)) {
                    new_edges.push((u, v));
                }
            }
        }
    } else {
        // Undirected: only unordered pairs (u, v) with u <= v. Self-loop
        // appears only when loops == true.
        for u in 0..n {
            let lo = if loops { u } else { u + 1 };
            for v in lo..n {
                if !existing.contains(&(u, v)) {
                    new_edges.push((u, v));
                }
            }
        }
    }

    g.add_edges(new_edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let m = u32::try_from(g.ecount()).unwrap();
        let mut v: Vec<_> = (0..m).map(|e| g.edge(e).unwrap()).collect();
        v.sort_unstable();
        v
    }

    #[test]
    fn empty_graph_yields_empty() {
        let g = Graph::with_vertices(0);
        let c = complementer(&g, false).unwrap();
        assert_eq!(c.vcount(), 0);
        assert_eq!(c.ecount(), 0);
    }

    #[test]
    fn no_edges_no_loops_is_complete_graph() {
        // 3 isolated vertices → complementer (loops=false) = K3.
        let g = Graph::with_vertices(3);
        let c = complementer(&g, false).unwrap();
        assert_eq!(sorted_edges(&c), vec![(0, 1), (0, 2), (1, 2)]);
    }

    #[test]
    fn no_edges_with_loops_adds_self_loops() {
        let g = Graph::with_vertices(2);
        let c = complementer(&g, true).unwrap();
        // K2 with both self-loops = (0,0)(0,1)(1,1).
        assert_eq!(sorted_edges(&c), vec![(0, 0), (0, 1), (1, 1)]);
    }

    #[test]
    fn complete_graph_complementer_is_empty() {
        // K3 → complementer (no loops) is empty.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let c = complementer(&g, false).unwrap();
        assert_eq!(c.ecount(), 0);
    }

    #[test]
    fn path_complementer_is_single_chord() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let c = complementer(&g, false).unwrap();
        assert_eq!(c.ecount(), 1);
        assert_eq!(c.edge(0).unwrap(), (0, 2));
    }

    #[test]
    fn directed_complementer_includes_reverses() {
        // Directed (0,1): complementer (no loops) should add
        // (1,0), (0,2), (2,0), (1,2), (2,1) on 3 vertices.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let c = complementer(&g, false).unwrap();
        assert!(c.is_directed());
        assert_eq!(
            sorted_edges(&c),
            vec![(0, 2), (1, 0), (1, 2), (2, 0), (2, 1)]
        );
    }

    #[test]
    fn directed_with_loops_adds_self_loops() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let c = complementer(&g, true).unwrap();
        assert_eq!(sorted_edges(&c), vec![(0, 0), (1, 0), (1, 1)]);
    }

    #[test]
    fn double_complement_recovers_original_for_simple_undirected() {
        // For a simple undirected graph, complement(complement(g)) == g.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let c = complementer(&g, false).unwrap();
        let cc = complementer(&c, false).unwrap();
        assert_eq!(sorted_edges(&cc), sorted_edges(&g));
    }

    #[test]
    fn singleton_with_loops() {
        let g = Graph::with_vertices(1);
        let c = complementer(&g, true).unwrap();
        assert_eq!(sorted_edges(&c), vec![(0, 0)]);
    }

    #[test]
    fn singleton_without_loops() {
        let g = Graph::with_vertices(1);
        let c = complementer(&g, false).unwrap();
        assert_eq!(c.ecount(), 0);
    }

    #[test]
    fn parallel_edges_in_input_collapse() {
        // Multi-edges in input shouldn't change the complementer beyond
        // suppressing the (0,1) pair once.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let c = complementer(&g, false).unwrap();
        // Surviving: (0,2), (1,2).
        assert_eq!(sorted_edges(&c), vec![(0, 2), (1, 2)]);
    }
}
