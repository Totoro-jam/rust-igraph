//! Star graph predicate (ALGO-PR-091).
//!
//! A star graph `K_{1,n-1}` (or `S_n`) has one central vertex of
//! degree n-1 connected to n-1 leaf vertices, each of degree 1.
//! Equivalently, it is a complete bipartite graph with one part of
//! size 1.
//!
//! For directed graphs, the function returns `false`.

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::algorithms::properties::is_tree::is_tree;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a star graph.
///
/// A star graph has one center vertex connected to all other vertices,
/// with no other edges. `K_{1,0}` (single vertex) and `K_{1,1}`
/// (single edge) are considered stars.
///
/// Returns `false` for directed graphs and the empty graph (no vertices).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_star};
///
/// // K_{1,4}: center 0 with 4 leaves
/// let mut g = Graph::with_vertices(5);
/// for i in 1..5u32 {
///     g.add_edge(0, i).unwrap();
/// }
/// assert!(is_star(&g).unwrap());
///
/// // Path P_3 is NOT a star (middle vertex has degree 2, not n-1=2... wait
/// // P_3 has 3 vertices, center degree should be 2. Actually P_3 IS K_{1,2}!)
/// // Use P_4 as negative example instead.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(!is_star(&g).unwrap());
/// ```
pub fn is_star(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n == 0 {
        return Ok(false);
    }
    if n == 1 {
        return Ok(true);
    }

    if is_tree(graph, DijkstraMode::All)?.is_none() {
        return Ok(false);
    }

    // A star has exactly one vertex of degree n-1 and all others of degree 1.
    // Equivalently, a tree with exactly n-1 edges where one vertex has degree n-1.
    let n_minus_1 = n - 1;
    let mut found_center = false;
    for v in 0..n {
        let d = graph.degree(v)?;
        if d == n_minus_1 as usize {
            found_center = true;
        } else if d != 1 {
            return Ok(false);
        }
    }

    Ok(found_center)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(!is_star(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_star(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_star(&g).unwrap());
    }

    #[test]
    fn k13() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(is_star(&g).unwrap());
    }

    #[test]
    fn k14() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_star(&g).unwrap());
    }

    #[test]
    fn k17_center_not_zero() {
        // Center is vertex 3, not vertex 0
        let mut g = Graph::with_vertices(8);
        for i in 0..8u32 {
            if i != 3 {
                g.add_edge(3, i).unwrap();
            }
        }
        assert!(is_star(&g).unwrap());
    }

    #[test]
    fn path_p4_not_star() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_star(&g).unwrap());
    }

    #[test]
    fn triangle_not_star() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_star(&g).unwrap());
    }

    #[test]
    fn caterpillar_not_star() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(!is_star(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(!is_star(&g).unwrap());
    }

    #[test]
    fn disconnected_not_star() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        assert!(!is_star(&g).unwrap());
    }

    #[test]
    fn two_isolated_not_star() {
        let g = Graph::with_vertices(2);
        assert!(!is_star(&g).unwrap());
    }
}
