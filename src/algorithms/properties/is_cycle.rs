//! Cycle graph predicate (ALGO-PR-089).
//!
//! A cycle graph `C_n` is a connected 2-regular graph on n ≥ 3 vertices.
//! Every vertex has exactly degree 2, forming a single closed loop.
//!
//! For directed graphs, the function returns `false`.

use crate::algorithms::connectivity::{ConnectednessMode, is_connected};
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a cycle graph.
///
/// A cycle graph `C_n` has n ≥ 3 vertices, each with degree exactly 2,
/// forming a single Hamiltonian cycle. The graph must be connected
/// and simple.
///
/// Returns `false` for directed graphs, graphs with fewer than 3
/// vertices, disconnected graphs, and graphs with any vertex not
/// having degree 2.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_cycle};
///
/// // C_4: 0-1-2-3-0
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// assert!(is_cycle(&g).unwrap());
///
/// // P_4 is NOT a cycle (endpoints have degree 1)
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(!is_cycle(&g).unwrap());
/// ```
pub fn is_cycle(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n < 3 {
        return Ok(false);
    }

    // A cycle on n vertices has exactly n edges.
    if graph.ecount() != n as usize {
        return Ok(false);
    }

    // Every vertex must have degree exactly 2.
    for v in 0..n {
        if graph.degree(v)? != 2 {
            return Ok(false);
        }
    }

    // Connected + n edges + all degree 2 → single cycle.
    is_connected(graph, ConnectednessMode::Weak)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(!is_cycle(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(!is_cycle(&g).unwrap());
    }

    #[test]
    fn two_vertices_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(!is_cycle(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_cycle(&g).unwrap());
    }

    #[test]
    fn c4() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_cycle(&g).unwrap());
    }

    #[test]
    fn c5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_cycle(&g).unwrap());
    }

    #[test]
    fn path_not_cycle() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_cycle(&g).unwrap());
    }

    #[test]
    fn star_not_cycle() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(!is_cycle(&g).unwrap());
    }

    #[test]
    fn complete_k4_not_cycle() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_cycle(&g).unwrap());
    }

    #[test]
    fn two_disjoint_triangles_not_cycle() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        assert!(!is_cycle(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_cycle(&g).unwrap());
    }

    #[test]
    fn wheel_not_cycle() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 1).unwrap();
        assert!(!is_cycle(&g).unwrap());
    }

    #[test]
    fn c6_non_sequential() {
        // Cycle with non-sequential edge order
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 3).unwrap();
        g.add_edge(3, 1).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(4, 2).unwrap();
        g.add_edge(2, 5).unwrap();
        g.add_edge(5, 0).unwrap();
        assert!(is_cycle(&g).unwrap());
    }
}
