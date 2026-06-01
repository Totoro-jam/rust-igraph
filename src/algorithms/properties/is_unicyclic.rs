//! Unicyclic graph predicate (ALGO-PR-093).
//!
//! A unicyclic graph is a connected graph containing exactly one
//! cycle. Equivalently, a connected graph with n vertices and n edges.
//!
//! For directed graphs, the function returns `false`.

use crate::algorithms::connectivity::{ConnectednessMode, is_connected};
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is unicyclic.
///
/// A unicyclic graph is connected and has exactly as many edges as
/// vertices (n vertices, n edges), which means it contains exactly
/// one cycle.
///
/// Returns `false` for directed graphs, disconnected graphs, trees
/// (n-1 edges), and graphs with more than one cycle.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_unicyclic};
///
/// // Triangle with a tail: 0-1-2-0, 2-3
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_unicyclic(&g).unwrap());
///
/// // Path is NOT unicyclic (no cycle)
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert!(!is_unicyclic(&g).unwrap());
/// ```
pub fn is_unicyclic(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n == 0 {
        return Ok(false);
    }

    // A connected graph with n edges has exactly one cycle.
    if graph.ecount() != n as usize {
        return Ok(false);
    }

    is_connected(graph, ConnectednessMode::Weak)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(!is_unicyclic(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(!is_unicyclic(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(!is_unicyclic(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_unicyclic(&g).unwrap());
    }

    #[test]
    fn c5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_unicyclic(&g).unwrap());
    }

    #[test]
    fn triangle_with_tail() {
        // 0-1-2-0, 2-3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_unicyclic(&g).unwrap());
    }

    #[test]
    fn path_not_unicyclic() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_unicyclic(&g).unwrap());
    }

    #[test]
    fn k4_not_unicyclic() {
        // K4 has 6 edges, 4 vertices — multiple cycles
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_unicyclic(&g).unwrap());
    }

    #[test]
    fn two_triangles_disconnected() {
        // Two triangles, each unicyclic, but disconnected
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        assert!(!is_unicyclic(&g).unwrap());
    }

    #[test]
    fn theta_graph_not_unicyclic() {
        // Theta: two paths between same endpoints → 2 cycles
        // 0-1-2-3, 0-4-3: 5 vertices, 5 edges, connected
        // Wait: 5 vertices, 5 edges, connected → exactly 1 cycle!
        // Actually no: 0-1-2-3, 0-4-3, 0-3 would be theta.
        // Let me make a proper graph with 2 cycles:
        // Triangle 0-1-2 + triangle 0-2-3: 4 vertices, 5 edges
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(3, 2).unwrap();
        // 4 vertices, 5 edges → not unicyclic
        assert!(!is_unicyclic(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_unicyclic(&g).unwrap());
    }

    #[test]
    fn star_not_unicyclic() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(!is_unicyclic(&g).unwrap());
    }

    #[test]
    fn lollipop() {
        // Triangle 0-1-2-0 with a path tail 2-3-4
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_unicyclic(&g).unwrap());
    }
}
