//! Transitive closure (ALGO-CC-022).
//!
//! Counterpart of `igraph_transitive_closure()` from
//! `references/igraph/src/connectivity/reachability.c:225-257`. Returns a
//! new graph with an edge `u -> v` (or `u - v` for undirected) for every
//! ordered (resp. unordered) reachable pair with `u != v`.
//!
//! Built on top of the
//! [`super::reachability_matrix::reachability_matrix`] primitive.
//! Phase-1 cost: `O(|V|² + |V|*|V|*log|E|)` from BFS-from-each-vertex
//! plus the closure construction. Upstream's SCC-condensation
//! optimisation could shave this in a future perf pass.

use crate::core::{Graph, IgraphResult};

/// Transitive closure of `graph`. The returned graph shares the same
/// vertex count and direction as the input.
///
/// For directed graphs every reachable ordered pair `(u, v)` (`u != v`)
/// becomes an edge. For undirected graphs each unordered reachable pair
/// `{u, v}` is added once.
///
/// Counterpart of `igraph_transitive_closure(_, _)` from
/// `references/igraph/src/connectivity/reachability.c:225`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, transitive_closure};
///
/// // Directed path 0 -> 1 -> 2: closure adds 0 -> 2.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let tc = transitive_closure(&g).unwrap();
/// assert_eq!(tc.vcount(), 3);
/// assert_eq!(tc.ecount(), 3); // 0->1, 0->2, 1->2
/// assert!(tc.find_eid(0, 2).unwrap().is_some());
/// ```
pub fn transitive_closure(graph: &Graph) -> IgraphResult<Graph> {
    let n = graph.vcount();
    let directed = graph.is_directed();
    let m = super::reachability_matrix::reachability_matrix(graph)?;
    let mut closure = Graph::new(n, directed)?;
    for (u, row) in m.iter().enumerate() {
        let start_j = if directed { 0 } else { u + 1 };
        let u_id = u32::try_from(u).map_err(|_| {
            crate::core::IgraphError::Internal("vcount overflows u32 in transitive_closure")
        })?;
        for (v, &reachable) in row.iter().enumerate().skip(start_j) {
            if u != v && reachable {
                let v_id = u32::try_from(v).map_err(|_| {
                    crate::core::IgraphError::Internal("vcount overflows u32 in transitive_closure")
                })?;
                closure.add_edge(u_id, v_id)?;
            }
        }
    }
    Ok(closure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_yields_empty_closure() {
        let g = Graph::with_vertices(0);
        let tc = transitive_closure(&g).unwrap();
        assert_eq!(tc.vcount(), 0);
        assert_eq!(tc.ecount(), 0);
    }

    #[test]
    fn isolated_vertices_no_edges_in_closure() {
        let g = Graph::with_vertices(5);
        let tc = transitive_closure(&g).unwrap();
        assert_eq!(tc.vcount(), 5);
        assert_eq!(tc.ecount(), 0);
    }

    #[test]
    fn directed_path_3_closure_has_three_edges() {
        // 0 -> 1 -> 2: closure adds 0->1, 0->2, 1->2.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let tc = transitive_closure(&g).unwrap();
        assert!(tc.is_directed());
        assert_eq!(tc.ecount(), 3);
        for &(u, v) in &[(0u32, 1u32), (0, 2), (1, 2)] {
            assert!(tc.find_eid(u, v).unwrap().is_some(), "missing {u}->{v}");
        }
        // No reverse edges in directed closure.
        assert!(tc.find_eid(1, 0).unwrap().is_none());
        assert!(tc.find_eid(2, 0).unwrap().is_none());
    }

    #[test]
    fn undirected_path_3_closure_is_complete_graph() {
        // 0-1-2 undirected: closure is K3 (3 edges: 01, 02, 12).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let tc = transitive_closure(&g).unwrap();
        assert!(!tc.is_directed());
        assert_eq!(tc.ecount(), 3);
        for &(u, v) in &[(0u32, 1u32), (0, 2), (1, 2)] {
            assert!(tc.find_eid(u, v).unwrap().is_some(), "missing {u}-{v}");
        }
    }

    #[test]
    fn directed_cycle_closure_is_complete_directed() {
        // 0->1->2->0: closure has all 6 directed pairs.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let tc = transitive_closure(&g).unwrap();
        assert_eq!(tc.ecount(), 6); // 3*2 ordered pairs (no self-loops).
        for u in 0..3u32 {
            for v in 0..3u32 {
                if u != v {
                    assert!(tc.find_eid(u, v).unwrap().is_some());
                }
            }
        }
    }

    #[test]
    fn disconnected_components_isolate_in_closure() {
        // {0-1-2} and {3-4} undirected: closure has edges within each component
        // only. Component sizes 3 and 2 → 3 + 1 = 4 edges.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        let tc = transitive_closure(&g).unwrap();
        assert_eq!(tc.ecount(), 4);
        // Cross-component edges absent.
        for u in 0..3u32 {
            for v in 3..5u32 {
                assert!(tc.find_eid(u, v).unwrap().is_none());
            }
        }
    }

    #[test]
    fn closure_of_complete_graph_is_itself() {
        // K3: every pair already connected.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let tc = transitive_closure(&g).unwrap();
        assert_eq!(tc.vcount(), 3);
        assert_eq!(tc.ecount(), 3); // 3 unordered pairs.
    }
}
