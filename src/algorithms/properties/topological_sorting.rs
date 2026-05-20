//! Topological sorting (ALGO-PR-021).
//!
//! Counterpart of `igraph_topological_sorting()` from
//! `references/igraph/src/properties/dag.c:54`. Produces a linear
//! ordering of a directed graph's vertices such that for every
//! edge `u → v`, `u` precedes `v` in the order (under
//! [`DijkstraMode::Out`]; flip for [`DijkstraMode::In`]).
//!
//! Algorithm: Kahn's algorithm — same peel loop as
//! [`crate::is_dag`] but recording each popped vertex in the
//! output rather than just counting. **Self-loops are ignored**
//! when computing degrees: a vertex's own self-loop does not
//! contribute to its in/out-degree, so a self-loop alone does not
//! block topological sorting (matches upstream).
//!
//! Time complexity: `O(V + E)`.

use std::collections::VecDeque;

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::core::Graph;
use crate::core::error::{IgraphError, IgraphResult};
use crate::core::graph::VertexId;

/// Returns a topological ordering of `graph`'s vertices.
///
/// `mode` controls the direction of "precedes":
/// - [`DijkstraMode::Out`] (most common): for every edge `u → v`,
///   `u` appears before `v` in the result. Vertices with no
///   incoming edges come first.
/// - [`DijkstraMode::In`]: reversed — for every edge `u → v`,
///   `v` appears before `u`. Vertices with no outgoing edges come
///   first.
/// - [`DijkstraMode::All`]: rejected with
///   [`IgraphError::InvalidArgument`] — topological sorting only
///   makes sense in a directional sense.
///
/// Self-loops are ignored when computing degrees (matches
/// upstream): a single self-loop with no other in/out edges
/// does not block sorting.
///
/// Errors:
/// - [`IgraphError::InvalidArgument`] if the graph is undirected.
/// - [`IgraphError::InvalidArgument`] if the graph contains a
///   non-trivial cycle (a topological ordering does not exist).
///
/// Counterpart of `igraph_topological_sorting(_, _, mode)` from
/// `references/igraph/src/properties/dag.c:54`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, topological_sorting, DijkstraMode};
///
/// // 0 → 1, 0 → 2, 1 → 3, 2 → 3: a DAG with two valid orderings.
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 3), (2, 3)]).unwrap();
/// let order = topological_sorting(&g, DijkstraMode::Out).unwrap();
/// // 0 must come first; 3 must come last.
/// assert_eq!(order[0], 0);
/// assert_eq!(order[3], 3);
/// ```
pub fn topological_sorting(graph: &Graph, mode: DijkstraMode) -> IgraphResult<Vec<VertexId>> {
    if !graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "topological_sorting requires a directed graph".to_string(),
        ));
    }
    if matches!(mode, DijkstraMode::All) {
        return Err(IgraphError::InvalidArgument(
            "topological_sorting does not accept DijkstraMode::All".to_string(),
        ));
    }
    let walk_forward = matches!(mode, DijkstraMode::Out);

    let n = graph.vcount();
    let n_us = n as usize;

    // `degrees[v]` = number of edges to be consumed before v is
    // ready to emit. For OUT mode this is the in-degree; for IN
    // mode the out-degree. Self-loops are NOT counted (upstream
    // passes IGRAPH_NO_LOOPS to igraph_degree).
    let mut degrees: Vec<u32> = vec![0; n_us];
    for v in 0..n {
        let incidents = if walk_forward {
            graph.incident_in(v)?
        } else {
            graph.incident(v)?
        };
        let mut count: u32 = 0;
        for eid in incidents {
            let other = graph.edge_other(eid, v)?;
            if other != v {
                count = count.saturating_add(1);
            }
        }
        degrees[v as usize] = count;
    }

    // Seed the queue with every zero-degree vertex.
    let mut sources: VecDeque<VertexId> = VecDeque::new();
    for v in 0..n {
        if degrees[v as usize] == 0 {
            sources.push_back(v);
        }
    }

    let mut order: Vec<VertexId> = Vec::with_capacity(n_us);
    while let Some(v) = sources.pop_front() {
        order.push(v);
        // Walk v's edges in the forward direction; for each
        // out-neighbour (in OUT mode) or in-neighbour (in IN mode),
        // decrement its degree counter.
        let outgoing = if walk_forward {
            graph.incident(v)?
        } else {
            graph.incident_in(v)?
        };
        for eid in outgoing {
            let nei = graph.edge_other(eid, v)?;
            if nei == v {
                // Self-loop: ignored.
                continue;
            }
            let nei_idx = nei as usize;
            degrees[nei_idx] = degrees[nei_idx].saturating_sub(1);
            if degrees[nei_idx] == 0 {
                sources.push_back(nei);
            }
        }
    }

    if order.len() != n_us {
        return Err(IgraphError::InvalidArgument(
            "topological_sorting: graph contains a cycle (no valid ordering exists)".to_string(),
        ));
    }
    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Graph;

    #[test]
    fn empty_directed_graph_returns_empty_order() {
        let g = Graph::new(0, true).unwrap();
        let order = topological_sorting(&g, DijkstraMode::Out).unwrap();
        assert!(order.is_empty());
    }

    #[test]
    fn isolated_vertices_only_returns_all_in_some_order() {
        let g = Graph::new(3, true).unwrap();
        let mut order = topological_sorting(&g, DijkstraMode::Out).unwrap();
        order.sort_unstable();
        assert_eq!(order, vec![0, 1, 2]);
    }

    #[test]
    fn linear_chain_out_mode_preserves_order() {
        // 0 → 1 → 2 → 3: only one valid topological order in OUT mode.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3)]).unwrap();
        let order = topological_sorting(&g, DijkstraMode::Out).unwrap();
        assert_eq!(order, vec![0, 1, 2, 3]);
    }

    #[test]
    fn linear_chain_in_mode_reverses_order() {
        // 0 → 1 → 2 → 3 with IN mode: sinks first.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3)]).unwrap();
        let order = topological_sorting(&g, DijkstraMode::In).unwrap();
        assert_eq!(order, vec![3, 2, 1, 0]);
    }

    #[test]
    fn diamond_dag_respects_edge_order() {
        // 0 → 1, 0 → 2, 1 → 3, 2 → 3. Valid orders: [0,1,2,3], [0,2,1,3].
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 3), (2, 3)])
            .unwrap();
        let order = topological_sorting(&g, DijkstraMode::Out).unwrap();
        // 0 first, 3 last; 1 and 2 in between.
        assert_eq!(order[0], 0);
        assert_eq!(order[3], 3);
        assert!(
            order == vec![0, 1, 2, 3] || order == vec![0, 2, 1, 3],
            "unexpected order: {order:?}"
        );
    }

    #[test]
    fn self_loop_does_not_block_sorting() {
        // 0 with self-loop, then 0 → 1. Order should be [0, 1].
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let order = topological_sorting(&g, DijkstraMode::Out).unwrap();
        assert_eq!(order, vec![0, 1]);
    }

    #[test]
    fn three_cycle_errors() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        let err = topological_sorting(&g, DijkstraMode::Out).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn undirected_graph_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = topological_sorting(&g, DijkstraMode::Out).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn all_mode_errors() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let err = topological_sorting(&g, DijkstraMode::All).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn parallel_edges_dont_block_ordering() {
        // 0 → 1 with parallel edges; still valid DAG, order = [0, 1].
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let order = topological_sorting(&g, DijkstraMode::Out).unwrap();
        assert_eq!(order, vec![0, 1]);
    }

    #[test]
    fn topological_order_respects_every_edge() {
        // 0 → 2, 1 → 2, 2 → 3, 2 → 4 — order must place 0,1 before 2,
        // and 2 before 3,4.
        let mut g = Graph::new(5, true).unwrap();
        g.add_edges(vec![(0u32, 2u32), (1, 2), (2, 3), (2, 4)])
            .unwrap();
        let order = topological_sorting(&g, DijkstraMode::Out).unwrap();
        let pos: Vec<usize> = {
            let mut p = vec![0usize; 5];
            for (i, &v) in order.iter().enumerate() {
                p[v as usize] = i;
            }
            p
        };
        // Every edge u → v must have pos[u] < pos[v].
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32 in test");
        for e in 0..m {
            let (u, v) = g.edge(e).unwrap();
            if u == v {
                continue;
            }
            assert!(
                pos[u as usize] < pos[v as usize],
                "edge {u}→{v} violates order"
            );
        }
    }

    #[test]
    fn cycle_with_tail_still_errors() {
        // 0 → 1 → 2 → 0 (cycle) + extra source 3 → 0. Tail 3 can be
        // peeled but the cycle remains → error.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0), (3, 0)])
            .unwrap();
        let err = topological_sorting(&g, DijkstraMode::Out).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }
}
