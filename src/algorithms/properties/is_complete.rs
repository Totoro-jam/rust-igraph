//! `is_complete` predicate (ALGO-PR-016).
//!
//! Counterpart of `igraph_is_complete()` from
//! `references/igraph/src/properties/complete.c:43`. A graph is *complete*
//! if all pairs of distinct vertices are adjacent. The null graph and the
//! singleton graph are considered complete by convention (matches upstream).
//!
//! Algorithm: cardinality short-circuit then either a simple-graph
//! fast path or a per-vertex unique-neighbour scan.
//!
//! 1. `n <= 1` → `true`.
//! 2. Compute the target edge count for a complete graph on `n`
//!    vertices: `n*(n-1)` (directed) or `n*(n-1)/2` (undirected).
//!    Fall back to `false` immediately if the multiplication would
//!    overflow `u64`.
//! 3. If `ecount < target` → `false`.
//! 4. If the graph is simple (no loops, no parallel edges) → return
//!    `ecount == target` (must hold with equality once the lower
//!    bound is met).
//! 5. Otherwise the graph has loops / multi-edges that pad `ecount`;
//!    walk every vertex and require its set of unique non-self
//!    neighbours to have size at least `n - 1`.
//!
//! Time complexity: `O(V + E)` worst case (the unique-neighbour scan
//! visits every incidence at most once).

use std::collections::HashSet;

use crate::core::Graph;
use crate::core::IgraphResult;
use crate::core::graph::VertexId;

use super::is_simple::{SimpleMode, is_simple_with_mode};

/// Returns `true` iff every pair of distinct vertices is adjacent.
///
/// The null graph (`n == 0`) and the singleton graph (`n == 1`) are
/// considered complete (matches `igraph_is_complete`).
///
/// On directed graphs both directions must be present for every pair
/// `(u, v)` with `u != v` — this matches upstream's
/// `igraph_neighbors(_, _, _, IGRAPH_OUT, NO_LOOPS, NO_MULTIPLE)`
/// counting and `igraph_is_simple(_, _, IGRAPH_DIRECTED)` short-circuit.
///
/// Counterpart of `igraph_is_complete` from
/// `references/igraph/src/properties/complete.c:43`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_complete};
///
/// // Null graph — vacuously complete.
/// let g = Graph::with_vertices(0);
/// assert!(is_complete(&g).unwrap());
///
/// // K_3 — every pair adjacent.
/// let mut g = Graph::with_vertices(3);
/// g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 2)]).unwrap();
/// assert!(is_complete(&g).unwrap());
///
/// // Path P_3 (0-1-2) — 0 and 2 not adjacent.
/// let mut g = Graph::with_vertices(3);
/// g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
/// assert!(!is_complete(&g).unwrap());
/// ```
pub fn is_complete(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    if n <= 1 {
        return Ok(true);
    }

    let n64 = u64::from(n);
    let m64 = graph.ecount() as u64;
    let directed = graph.is_directed();

    // n*(n-1) cannot overflow u64 since n is u32 (so ≤ 2^32).
    let Some(target_undir_x2) = n64.checked_mul(n64 - 1) else {
        return Ok(false);
    };
    let target = if directed {
        target_undir_x2
    } else {
        target_undir_x2 / 2
    };

    if m64 < target {
        return Ok(false);
    }

    // Fast path: a simple graph has no padding, so equality is exact.
    if is_simple_with_mode(graph, SimpleMode::DirectedAsDirected)? {
        return Ok(m64 == target);
    }

    // Slow path: graph has loops or multi-edges. For every vertex v,
    // count distinct non-self out-neighbours (matches `IGRAPH_OUT`
    // semantics with NO_LOOPS / NO_MULTIPLE in the C reference). On
    // an undirected graph `Graph::neighbors` already returns all
    // neighbours; the directed branch needs the out-direction only.
    let n_us = n as usize;
    let mut seen: HashSet<VertexId> = HashSet::with_capacity(n_us);
    for v in 0..n {
        seen.clear();
        if directed {
            let eids = graph.incident(v)?;
            for eid in eids {
                let nei = graph.edge_other(eid, v)?;
                if nei != v {
                    seen.insert(nei);
                }
            }
        } else {
            let neis = graph.neighbors(v)?;
            for nei in neis {
                if nei != v {
                    seen.insert(nei);
                }
            }
        }
        // n - 1 is required (every other vertex must appear once).
        if seen.len() + 1 < n_us {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Graph;

    #[test]
    fn null_graph_is_complete() {
        let g = Graph::with_vertices(0);
        assert!(is_complete(&g).unwrap());
    }

    #[test]
    fn null_directed_is_complete() {
        let g = Graph::new(0, true).unwrap();
        assert!(is_complete(&g).unwrap());
    }

    #[test]
    fn singleton_undirected_is_complete() {
        let g = Graph::with_vertices(1);
        assert!(is_complete(&g).unwrap());
    }

    #[test]
    fn singleton_directed_is_complete() {
        let g = Graph::new(1, true).unwrap();
        assert!(is_complete(&g).unwrap());
    }

    #[test]
    fn singleton_with_self_loop_is_complete() {
        // n == 1 short-circuits — the loop doesn't matter.
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();
        assert!(is_complete(&g).unwrap());
    }

    #[test]
    fn k2_undirected_is_complete() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_complete(&g).unwrap());
    }

    #[test]
    fn k3_undirected_is_complete() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 2)]).unwrap();
        assert!(is_complete(&g).unwrap());
    }

    #[test]
    fn k4_undirected_is_complete() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)])
            .unwrap();
        assert!(is_complete(&g).unwrap());
    }

    #[test]
    fn path_is_not_complete() {
        // P_3: 0-1-2 — 0 and 2 not adjacent.
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
        assert!(!is_complete(&g).unwrap());
    }

    #[test]
    fn missing_one_edge_is_not_complete() {
        // K_4 minus the edge (2,3).
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (0, 2), (0, 3), (1, 2), (1, 3)])
            .unwrap();
        assert!(!is_complete(&g).unwrap());
    }

    #[test]
    fn k3_directed_needs_both_directions() {
        // Three directed cycle 0→1→2→0 has the right edge count
        // for a complete *undirected* K_3 (3 edges) but only one
        // direction per pair, so it is NOT a complete directed graph.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        assert!(!is_complete(&g).unwrap());
    }

    #[test]
    fn k3_directed_complete() {
        // n*(n-1) = 6 directed edges for K_3.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 0), (0, 2), (2, 0), (1, 2), (2, 1)])
            .unwrap();
        assert!(is_complete(&g).unwrap());
    }

    #[test]
    fn k2_directed_not_complete_with_only_one_arc() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_complete(&g).unwrap());
    }

    #[test]
    fn k2_directed_complete_with_both_arcs() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 0)]).unwrap();
        assert!(is_complete(&g).unwrap());
    }

    #[test]
    fn complete_with_extra_self_loops_still_complete() {
        // K_3 plus a self-loop at vertex 0: ecount > target but every
        // vertex still has the other two as unique non-self neighbours.
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 2), (0, 0)])
            .unwrap();
        assert!(is_complete(&g).unwrap());
    }

    #[test]
    fn complete_with_parallel_edges_still_complete() {
        // K_3 with a duplicate (0,1) edge: ecount > target but unique
        // neighbour set per vertex still equals the other two.
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 2), (0, 1)])
            .unwrap();
        assert!(is_complete(&g).unwrap());
    }

    #[test]
    fn parallel_edges_padding_does_not_help_missing_pair() {
        // 4 vertices, edges (0,1) (0,2) (0,3) (1,2) (1,2) (1,2)
        // ecount = 6 = K_4 target, but {2,3} and {3,1}? Let's check:
        // vertex 3 only has 0 as neighbour — incomplete.
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (0, 2), (0, 3), (1, 2), (1, 2), (1, 2)])
            .unwrap();
        assert!(!is_complete(&g).unwrap());
    }

    #[test]
    fn ecount_too_low_short_circuit() {
        // Graph deliberately too sparse — short-circuits on cardinality.
        let mut g = Graph::with_vertices(10);
        g.add_edge(0, 1).unwrap();
        assert!(!is_complete(&g).unwrap());
    }

    #[test]
    fn empty_n2_is_not_complete() {
        // Two isolated vertices — needs 1 undirected edge to be K_2.
        let g = Graph::with_vertices(2);
        assert!(!is_complete(&g).unwrap());
    }

    #[test]
    fn multi_edge_padding_to_target_but_missing_pair() {
        // n=3, target_undir = 3 edges. Three parallel (0,1) edges
        // hit the cardinality bound but vertex 2 is isolated.
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0u32, 1u32), (0, 1), (0, 1)]).unwrap();
        assert!(!is_complete(&g).unwrap());
    }
}
