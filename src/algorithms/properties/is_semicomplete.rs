//! Semicomplete digraph predicate (ALGO-PR-095).
//!
//! A semicomplete digraph is a directed graph where for every pair of
//! distinct vertices u and v, at least one of the arcs (u,v) or (v,u)
//! exists (possibly both). Tournaments are the special case where
//! exactly one arc exists per pair.
//!
//! For undirected graphs, the function returns `false` (use
//! `is_complete` instead).

use crate::core::{Graph, IgraphResult};

/// Check whether a directed graph is semicomplete.
///
/// A semicomplete digraph has at least one arc between every pair
/// of distinct vertices. This generalizes tournaments (which forbid
/// bidirectional edges).
///
/// Returns `false` for undirected graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_semicomplete};
///
/// // Tournament on 3 vertices: 0→1, 1→2, 0→2
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 2).unwrap();
/// assert!(is_semicomplete(&g).unwrap());
///
/// // Missing arc between 0 and 2
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert!(!is_semicomplete(&g).unwrap());
/// ```
pub fn is_semicomplete(graph: &Graph) -> IgraphResult<bool> {
    if !graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n <= 1 {
        return Ok(true);
    }

    // Build adjacency matrix for fast pair lookup
    let n_usize = n as usize;
    let mut has_arc = vec![vec![false; n_usize]; n_usize];

    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            has_arc[v as usize][w as usize] = true;
        }
    }

    // Check every pair
    for (u, row_u) in has_arc.iter().enumerate().take(n_usize) {
        for (v, &u_to_v) in row_u.iter().enumerate().take(n_usize).skip(u + 1) {
            if !u_to_v && !has_arc[v][u] {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::new(0, true).unwrap();
        assert!(is_semicomplete(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::new(1, true).unwrap();
        assert!(is_semicomplete(&g).unwrap());
    }

    #[test]
    fn tournament_3() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        assert!(is_semicomplete(&g).unwrap());
    }

    #[test]
    fn bidirectional_pair() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        assert!(is_semicomplete(&g).unwrap());
    }

    #[test]
    fn semicomplete_not_tournament() {
        // All arcs present (complete digraph = semicomplete)
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 1).unwrap();
        assert!(is_semicomplete(&g).unwrap());
    }

    #[test]
    fn missing_arc() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // Missing: no arc between 0 and 2
        assert!(!is_semicomplete(&g).unwrap());
    }

    #[test]
    fn undirected_returns_false() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_semicomplete(&g).unwrap());
    }

    #[test]
    fn single_arc() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(is_semicomplete(&g).unwrap());
    }

    #[test]
    fn four_vertex_tournament() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_semicomplete(&g).unwrap());
    }

    #[test]
    fn four_vertex_not_semicomplete() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        // Path: missing arcs between non-consecutive vertices
        assert!(!is_semicomplete(&g).unwrap());
    }

    #[test]
    fn no_edges() {
        let g = Graph::new(3, true).unwrap();
        assert!(!is_semicomplete(&g).unwrap());
    }

    #[test]
    fn self_loops_dont_help() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(1, 1).unwrap();
        // Self-loops don't satisfy the pair requirement
        assert!(!is_semicomplete(&g).unwrap());
    }
}
