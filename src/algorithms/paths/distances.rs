//! Unweighted single-source shortest-path distances (ALGO-SP-006).
//!
//! Counterpart of `igraph_distances(_, NULL, _, single_from, all_to, IGRAPH_OUT)`
//! from `references/igraph/src/paths/unweighted.c:273-325` (reduced single-source
//! single-mode-OUT slice). The full multi-source / multi-mode / cutoff matrix
//! variant ships in a follow-up AWU.
//!
//! Algorithm: BFS from `source`; the first time a vertex `v` is dequeued
//! its distance is the shortest-path length from `source` (because BFS
//! discovers vertices in non-decreasing distance order). Vertices never
//! reached get `None`.
//!
//! Result type is `Vec<Option<u32>>`, mirroring upstream's
//! `IGRAPH_INFINITY` sentinel. Conversion to a `f64` matrix with `inf` for
//! unreachable is the caller's responsibility.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphResult, VertexId};

/// Unweighted shortest-path lengths from `source` to every vertex.
///
/// `result[v] == Some(d)` if `v` is reachable from `source` in `d`
/// edges (`d == 0` for the source itself); `None` if `v` is in a
/// different connected component (undirected) or has no out-path from
/// `source` (directed).
///
/// Counterpart of `igraph_distances(_, NULL, _, single_from, all_to,
/// IGRAPH_OUT)`. For directed graphs traversal follows out-edges
/// (matching `IGRAPH_OUT`); for undirected graphs every edge counts
/// both ways.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distances};
///
/// // Undirected path 0-1-2-3, plus an isolated vertex 4.
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let d = distances(&g, 0).unwrap();
/// assert_eq!(d, vec![Some(0), Some(1), Some(2), Some(3), None]);
/// ```
pub fn distances(graph: &Graph, source: VertexId) -> IgraphResult<Vec<Option<u32>>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }
    // `neighbors()` will reject an out-of-range `source`.
    graph.neighbors(source)?;

    let n_us = n as usize;
    let mut dist: Vec<Option<u32>> = vec![None; n_us];
    let mut queue: VecDeque<VertexId> = VecDeque::new();

    dist[source as usize] = Some(0);
    queue.push_back(source);

    while let Some(v) = queue.pop_front() {
        let next = dist[v as usize]
            .expect("dequeued vertex has been visited")
            .checked_add(1)
            .ok_or(crate::core::IgraphError::Internal(
                "distance overflow (graph diameter exceeds u32::MAX)",
            ))?;
        for w in graph.neighbors(v)? {
            if dist[w as usize].is_none() {
                dist[w as usize] = Some(next);
                queue.push_back(w);
            }
        }
    }

    Ok(dist)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_yields_empty_distances() {
        let g = Graph::with_vertices(0);
        let d = distances(&g, 0);
        // No source can be valid in an empty graph, but per the contract
        // we short-circuit and return an empty Vec without consulting
        // `source`. (Matches the BFS Phase-0 behaviour.)
        assert_eq!(d.unwrap(), Vec::<Option<u32>>::new());
    }

    #[test]
    fn single_vertex_distance_to_self_is_zero() {
        let g = Graph::with_vertices(1);
        let d = distances(&g, 0).unwrap();
        assert_eq!(d, vec![Some(0)]);
    }

    #[test]
    fn invalid_source_errors() {
        let g = Graph::with_vertices(3);
        let err = distances(&g, 5);
        assert!(err.is_err(), "expected error for source >= vcount");
    }

    #[test]
    fn path_distances_increase_by_one() {
        // 0 - 1 - 2 - 3 - 4 (undirected path).
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        let d = distances(&g, 0).unwrap();
        assert_eq!(d, vec![Some(0), Some(1), Some(2), Some(3), Some(4)]);
    }

    #[test]
    fn isolated_components_are_unreachable() {
        // {0,1,2} - {3,4}.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        let d = distances(&g, 0).unwrap();
        assert_eq!(d, vec![Some(0), Some(1), Some(2), None, None]);
    }

    #[test]
    fn self_loop_does_not_affect_distance() {
        // Self-loop on the source plus 0-1.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let d = distances(&g, 0).unwrap();
        assert_eq!(d, vec![Some(0), Some(1)]);
    }

    #[test]
    fn parallel_edges_do_not_affect_distance() {
        // Three copies of (0,1).
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let d = distances(&g, 0).unwrap();
        assert_eq!(d, vec![Some(0), Some(1)]);
    }

    #[test]
    fn directed_graph_uses_out_edges() {
        // 0 -> 1 -> 2: forward distances 0,1,2; from 2 only 2 reachable.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d_from_zero = distances(&g, 0).unwrap();
        assert_eq!(d_from_zero, vec![Some(0), Some(1), Some(2)]);
        let d_from_two = distances(&g, 2).unwrap();
        assert_eq!(d_from_two, vec![None, None, Some(0)]);
    }

    #[test]
    fn cycle_distances_are_minimum_of_two_directions() {
        // Undirected 6-cycle: 0-1-2-3-4-5-0. From 0:
        //   1 → 1,  2 → 2,  3 → 3,  4 → 2,  5 → 1.
        let mut g = Graph::with_vertices(6);
        for i in 0..5 {
            g.add_edge(i, i + 1).unwrap();
        }
        g.add_edge(5, 0).unwrap();
        let d = distances(&g, 0).unwrap();
        assert_eq!(
            d,
            vec![Some(0), Some(1), Some(2), Some(3), Some(2), Some(1)]
        );
    }
}
