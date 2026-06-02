//! Spider graph predicate (ALGO-PR-117).
//!
//! A spider graph (also called a subdivided star or caterpillar star)
//! is a tree with exactly one vertex of degree ≥ 3 (the body), and
//! all other vertices have degree ≤ 2. Equivalently, a spider is a
//! tree formed by joining paths at a common endpoint.
//!
//! For n ≤ 2, any tree is a spider (including a single edge or vertex).
//! A star `K_{1,k}` is a spider where all legs have length 1.
//! A path is a spider with exactly 2 legs (or 1 leg for `P_2`).
//!
//! Directed graphs are treated as undirected.

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::algorithms::properties::is_tree::is_tree;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a spider graph.
///
/// A spider is a tree with at most one vertex of degree ≥ 3.
/// Every star and every path is a spider.
///
/// Directed graphs are treated as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_spider};
///
/// // Star `S_3` is a spider
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// assert!(is_spider(&g).unwrap());
///
/// // Two adjacent high-degree vertices → NOT a spider
/// let mut g = Graph::with_vertices(7);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(3, 5).unwrap();
/// g.add_edge(3, 6).unwrap();
/// assert!(!is_spider(&g).unwrap());
/// ```
pub fn is_spider(graph: &Graph) -> IgraphResult<bool> {
    if is_tree(graph, DijkstraMode::All)?.is_none() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n <= 3 {
        return Ok(true);
    }

    let mut high_degree_count = 0u32;
    for v in 0..n {
        let deg = graph.degree(v)?;
        if deg >= 3 {
            high_degree_count = high_degree_count.saturating_add(1);
            if high_degree_count > 1 {
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
        // Null graph is not a tree → not a spider
        let g = Graph::with_vertices(0);
        assert!(!is_spider(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_spider(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_spider(&g).unwrap());
    }

    #[test]
    fn path_p3() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_spider(&g).unwrap());
    }

    #[test]
    fn path_p5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_spider(&g).unwrap());
    }

    #[test]
    fn star_s3() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(is_spider(&g).unwrap());
    }

    #[test]
    fn star_s5() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        assert!(is_spider(&g).unwrap());
    }

    #[test]
    fn spider_with_legs() {
        // Body at 0, three legs of lengths 1, 2, 3
        // 0-1, 0-2-3, 0-4-5-6
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 6).unwrap();
        assert!(is_spider(&g).unwrap());
    }

    #[test]
    fn two_branching_vertices_not_spider() {
        // 0-1-2, 1-3, 2-4, 2-5 → both 1 and 2 have degree 3
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        assert!(!is_spider(&g).unwrap());
    }

    #[test]
    fn triangle_not_tree() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_spider(&g).unwrap());
    }

    #[test]
    fn disconnected_not_spider() {
        let g = Graph::with_vertices(3);
        // 3 isolated vertices → forest but not tree
        assert!(!is_spider(&g).unwrap());
    }

    #[test]
    fn caterpillar_not_spider() {
        // 0-1-2-3, 1-4, 2-5 → both 1 and 2 have degree 3 → not spider
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        assert!(!is_spider(&g).unwrap());
    }

    #[test]
    fn directed_treated_as_undirected() {
        // Directed star
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(is_spider(&g).unwrap());
    }

    #[test]
    fn two_vertices_no_edge() {
        // Not connected → not a tree → not a spider
        let g = Graph::with_vertices(2);
        assert!(!is_spider(&g).unwrap());
    }
}
