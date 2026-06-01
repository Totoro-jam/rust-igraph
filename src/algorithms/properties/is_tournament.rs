//! Tournament graph predicate (ALGO-PR-084).
//!
//! A tournament is a directed graph in which every pair of distinct
//! vertices is connected by exactly one directed edge (either u→v or
//! v→u, but not both).
//!
//! Equivalently: directed, simple, no mutual edges, and exactly
//! n(n-1)/2 edges.

use crate::algorithms::properties::is_simple::is_simple;
use crate::algorithms::properties::mutual::has_mutual;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a tournament.
///
/// A tournament is a complete asymmetric directed graph: for every
/// pair of distinct vertices, exactly one directed edge exists.
///
/// Returns `false` for undirected graphs.
/// Returns `true` for the empty graph and single-vertex graph (vacuously).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_tournament};
///
/// // Directed 3-cycle: 0→1, 1→2, 2→0 — a tournament
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_tournament(&g).unwrap());
///
/// // Undirected triangle is NOT a tournament
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(!is_tournament(&g).unwrap());
/// ```
pub fn is_tournament(graph: &Graph) -> IgraphResult<bool> {
    if !graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n <= 1 {
        return Ok(true);
    }

    // n(n-1)/2 edges required
    let n_u64 = u64::from(n);
    let expected = n_u64.saturating_mul(n_u64.saturating_sub(1)) / 2;
    if graph.ecount() as u64 != expected {
        return Ok(false);
    }

    if !is_simple(graph)? {
        return Ok(false);
    }

    // No mutual (reciprocal) edges allowed
    if has_mutual(graph, false)? {
        return Ok(false);
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::new(0, true).unwrap();
        assert!(is_tournament(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::new(1, true).unwrap();
        assert!(is_tournament(&g).unwrap());
    }

    #[test]
    fn single_arc() {
        // 0→1: tournament on 2 vertices
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(is_tournament(&g).unwrap());
    }

    #[test]
    fn directed_3cycle() {
        // 0→1→2→0: tournament
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_tournament(&g).unwrap());
    }

    #[test]
    fn transitive_tournament_3() {
        // 0→1, 0→2, 1→2: transitive tournament on 3 vertices
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_tournament(&g).unwrap());
    }

    #[test]
    fn transitive_tournament_4() {
        // Total order: 0→1, 0→2, 0→3, 1→2, 1→3, 2→3
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_tournament(&g).unwrap());
    }

    #[test]
    fn undirected_returns_false() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_tournament(&g).unwrap());
    }

    #[test]
    fn mutual_edge_not_tournament() {
        // 0↔1, 0→2, 1→2: has mutual edge → not tournament
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        // 4 edges, expected 3 → fails edge count
        assert!(!is_tournament(&g).unwrap());
    }

    #[test]
    fn missing_edge_not_tournament() {
        // 0→1, 1→2 but no edge between 0 and 2: only 2 of 3 required
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_tournament(&g).unwrap());
    }

    #[test]
    fn self_loop_not_tournament() {
        // 0→1, 1→0→0 (self-loop): not simple → not tournament
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 0).unwrap();
        // 2 edges but expected 1 → fails
        assert!(!is_tournament(&g).unwrap());
    }

    #[test]
    fn complete_directed_not_tournament() {
        // Both directions for every pair: n(n-1) edges, not n(n-1)/2
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 1).unwrap();
        assert!(!is_tournament(&g).unwrap());
    }

    #[test]
    fn two_vertices_no_edge() {
        let g = Graph::new(2, true).unwrap();
        assert!(!is_tournament(&g).unwrap());
    }

    #[test]
    fn tournament_5_vertices() {
        // 5-vertex tournament: 0→1, 0→2, 0→3, 0→4, 1→2, 2→3, 3→4, 4→1, 1→3, 4→2
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 1).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(4, 2).unwrap();
        assert!(is_tournament(&g).unwrap());
    }
}
