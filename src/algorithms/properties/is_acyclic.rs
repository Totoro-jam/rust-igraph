//! Acyclic predicate (ALGO-PR-022).
//!
//! Counterpart of `igraph_is_acyclic()` from
//! `references/igraph/src/properties/trees.c:753`. Returns `true`
//! iff `graph` contains no cycle (of any length). For directed
//! graphs this delegates to [`crate::is_dag`]; for undirected
//! graphs we run union-find over the edges — a cycle is the first
//! edge whose endpoints are already in the same connected
//! component. Self-loops and parallel edges count as cycles.
//!
//! Time complexity: `O(V + E · α(E))` where `α` is the inverse
//! Ackermann function (effectively `O(V + E)`).

use crate::algorithms::properties::is_dag::is_dag;
use crate::core::Graph;

/// Returns `true` iff `graph` contains no cycle.
///
/// For directed graphs this is equivalent to [`crate::is_dag`].
/// For undirected graphs, a cycle is any non-trivial closed walk —
/// in particular self-loops and parallel undirected edges count
/// as cycles.
///
/// Algorithm:
/// - Directed: delegate to `is_dag` (matches upstream).
/// - Undirected: walk every edge; for each `(u, v)` use union-find
///   to check whether `u` and `v` are already connected. If yes,
///   we just closed a cycle ⇒ return false. Otherwise union and
///   continue.
///
/// Counterpart of `igraph_is_acyclic` from
/// `references/igraph/src/properties/trees.c:753`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_acyclic};
///
/// // Undirected tree 0-1-2-3: acyclic.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_acyclic(&g));
///
/// // Triangle 0-1-2-0: cyclic.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(!is_acyclic(&g));
/// ```
#[must_use]
pub fn is_acyclic(graph: &Graph) -> bool {
    if graph.is_directed() {
        return is_dag(graph);
    }

    let n = graph.vcount();
    let m = graph.ecount();
    let n_us = n as usize;

    // Trivial early outs: no vertices or no edges ⇒ no cycle.
    if n == 0 || m == 0 {
        return true;
    }

    // Union-find with path compression + union-by-rank-ish sizes.
    let mut parent: Vec<u32> = (0..n).collect();
    let mut size: Vec<u32> = vec![1; n_us];

    let Ok(m_u32) = u32::try_from(m) else {
        // Edge count beyond u32::MAX is impossible (EdgeId is u32),
        // but if it did happen we'd lose precision: treat as cyclic
        // to be safe.
        return false;
    };
    for eid in 0..m_u32 {
        // Should not happen on a well-formed Graph; fall back to
        // "cyclic" (conservative).
        let Ok((u, v)) = graph.edge(eid) else {
            return false;
        };
        if u == v {
            // Self-loop: instant cycle.
            return false;
        }
        let mut ru = u as usize;
        while parent[ru] as usize != ru {
            parent[ru] = parent[parent[ru] as usize];
            ru = parent[ru] as usize;
        }
        let mut rv = v as usize;
        while parent[rv] as usize != rv {
            parent[rv] = parent[parent[rv] as usize];
            rv = parent[rv] as usize;
        }
        if ru == rv {
            // Already in the same component ⇒ this edge closes a
            // cycle. Parallel undirected edges land here on the
            // second occurrence.
            return false;
        }
        // Union by size: attach smaller under larger.
        let (parent_idx, child_idx) = if size[ru] < size[rv] {
            (rv, ru)
        } else {
            (ru, rv)
        };
        let Ok(parent_u32) = u32::try_from(parent_idx) else {
            return false;
        };
        parent[child_idx] = parent_u32;
        size[parent_idx] = size[parent_idx].saturating_add(size[child_idx]);
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Graph;

    // -------- Undirected cases --------

    #[test]
    fn empty_undirected_is_acyclic() {
        let g = Graph::with_vertices(0);
        assert!(is_acyclic(&g));
    }

    #[test]
    fn isolated_vertices_only_is_acyclic() {
        let g = Graph::with_vertices(5);
        assert!(is_acyclic(&g));
    }

    #[test]
    fn undirected_tree_is_acyclic() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3)]).unwrap();
        assert!(is_acyclic(&g));
    }

    #[test]
    fn undirected_forest_is_acyclic() {
        // Two disjoint trees: 0-1-2 and 3-4.
        let mut g = Graph::with_vertices(5);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (3, 4)]).unwrap();
        assert!(is_acyclic(&g));
    }

    #[test]
    fn undirected_triangle_is_not_acyclic() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        assert!(!is_acyclic(&g));
    }

    #[test]
    fn undirected_self_loop_is_not_acyclic() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_acyclic(&g));
    }

    #[test]
    fn undirected_parallel_edge_is_not_acyclic() {
        // Two parallel edges between 0 and 1 form a 2-cycle.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_acyclic(&g));
    }

    #[test]
    fn undirected_star_is_acyclic() {
        // Star around vertex 0.
        let mut g = Graph::with_vertices(5);
        g.add_edges(vec![(0u32, 1u32), (0, 2), (0, 3), (0, 4)])
            .unwrap();
        assert!(is_acyclic(&g));
    }

    // -------- Directed cases (delegated to is_dag) --------

    #[test]
    fn directed_dag_chain_is_acyclic() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
        assert!(is_acyclic(&g));
    }

    #[test]
    fn directed_cycle_is_not_acyclic() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        assert!(!is_acyclic(&g));
    }

    #[test]
    fn directed_self_loop_is_not_acyclic() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_acyclic(&g));
    }

    #[test]
    fn directed_diamond_dag_is_acyclic() {
        // 0→1, 0→2, 1→3, 2→3.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 3), (2, 3)])
            .unwrap();
        assert!(is_acyclic(&g));
    }

    #[test]
    fn undirected_no_edges_with_many_vertices_is_acyclic() {
        let g = Graph::with_vertices(100);
        assert!(is_acyclic(&g));
    }
}
