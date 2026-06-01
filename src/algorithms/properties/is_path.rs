//! Path graph predicate (ALGO-PR-088).
//!
//! A path graph `P_n` is a tree on n vertices where every vertex has
//! degree at most 2 (equivalently, exactly two vertices have degree 1
//! and the rest have degree 2, or n ≤ 1).
//!
//! For directed graphs, the function returns `false`.

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::algorithms::properties::is_tree::is_tree;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a path graph.
///
/// A path graph `P_n` has n vertices connected in a single line:
/// 0-1-2-..-(n-1). Every vertex has degree ≤ 2, and the graph is a
/// tree (connected, acyclic).
///
/// Returns `false` for directed graphs.
/// Returns `true` for the empty graph and single vertex (`P_0`, `P_1`).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_path};
///
/// // P_4: 0-1-2-3
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_path(&g).unwrap());
///
/// // Star K_{1,3} is NOT a path (center has degree 3)
/// let mut g = Graph::with_vertices(4);
/// for i in 1..4u32 {
///     g.add_edge(0, i).unwrap();
/// }
/// assert!(!is_path(&g).unwrap());
/// ```
pub fn is_path(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n == 0 {
        return Ok(true);
    }

    if is_tree(graph, DijkstraMode::All)?.is_none() {
        return Ok(false);
    }

    for v in 0..n {
        if graph.degree(v)? > 2 {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_path(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_path(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_path(&g).unwrap());
    }

    #[test]
    fn p3() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_path(&g).unwrap());
    }

    #[test]
    fn p5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_path(&g).unwrap());
    }

    #[test]
    fn star_not_path() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(!is_path(&g).unwrap());
    }

    #[test]
    fn cycle_not_path() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_path(&g).unwrap());
    }

    #[test]
    fn triangle_not_path() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_path(&g).unwrap());
    }

    #[test]
    fn caterpillar_not_path() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(!is_path(&g).unwrap());
    }

    #[test]
    fn disconnected_not_path() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_path(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_path(&g).unwrap());
    }

    #[test]
    fn two_isolated_vertices_not_path() {
        let g = Graph::with_vertices(2);
        assert!(!is_path(&g).unwrap());
    }

    #[test]
    fn path_non_sequential_edges() {
        // Path but edges added in non-order: 3-1, 1-0, 0-2
        let mut g = Graph::with_vertices(4);
        g.add_edge(3, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(0, 2).unwrap();
        assert!(is_path(&g).unwrap());
    }
}
