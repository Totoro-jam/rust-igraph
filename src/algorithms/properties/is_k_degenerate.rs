//! k-degenerate graph predicate (ALGO-PR-116).
//!
//! A graph is k-degenerate if every subgraph has a vertex of degree
//! at most k. Equivalently, the degeneracy (maximum coreness) is at
//! most k. 0-degenerate = edgeless, 1-degenerate = forest,
//! 2-degenerate = every subgraph has a vertex of degree ≤ 2.
//!
//! Directed graphs are treated as undirected.

use crate::algorithms::properties::coreness::coreness;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is k-degenerate.
///
/// A graph is k-degenerate if every non-empty subgraph has a vertex
/// of degree at most k. Uses the coreness decomposition and checks
/// that the maximum core number does not exceed k.
///
/// Directed graphs are treated as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_k_degenerate};
///
/// // Tree is 1-degenerate
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// assert!(is_k_degenerate(&g, 1).unwrap());
/// assert!(!is_k_degenerate(&g, 0).unwrap());
///
/// // `K_4` is 3-degenerate but NOT 2-degenerate
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 {
///     for j in (i+1)..4 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// assert!(is_k_degenerate(&g, 3).unwrap());
/// assert!(!is_k_degenerate(&g, 2).unwrap());
/// ```
pub fn is_k_degenerate(graph: &Graph, k: u32) -> IgraphResult<bool> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(true);
    }

    let cores = coreness(graph)?;
    let max_core = cores.iter().copied().max().unwrap_or(0);
    Ok(max_core <= k)
}

/// Return the degeneracy of a graph.
///
/// The degeneracy is the smallest k such that the graph is
/// k-degenerate, i.e., the maximum coreness value.
///
/// Directed graphs are treated as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degeneracy};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// assert_eq!(degeneracy(&g).unwrap(), 1);
/// ```
pub fn degeneracy(graph: &Graph) -> IgraphResult<u32> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0);
    }

    let cores = coreness(graph)?;
    Ok(cores.iter().copied().max().unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_k_degenerate(&g, 0).unwrap());
        assert_eq!(degeneracy(&g).unwrap(), 0);
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_k_degenerate(&g, 0).unwrap());
        assert_eq!(degeneracy(&g).unwrap(), 0);
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(!is_k_degenerate(&g, 0).unwrap());
        assert!(is_k_degenerate(&g, 1).unwrap());
        assert_eq!(degeneracy(&g).unwrap(), 1);
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_k_degenerate(&g, 1).unwrap());
        assert!(is_k_degenerate(&g, 2).unwrap());
        assert_eq!(degeneracy(&g).unwrap(), 2);
    }

    #[test]
    fn tree() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_k_degenerate(&g, 1).unwrap());
        assert!(!is_k_degenerate(&g, 0).unwrap());
        assert_eq!(degeneracy(&g).unwrap(), 1);
    }

    #[test]
    fn complete_k4() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(!is_k_degenerate(&g, 2).unwrap());
        assert!(is_k_degenerate(&g, 3).unwrap());
        assert_eq!(degeneracy(&g).unwrap(), 3);
    }

    #[test]
    fn complete_k5() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert_eq!(degeneracy(&g).unwrap(), 4);
    }

    #[test]
    fn edgeless() {
        let g = Graph::with_vertices(5);
        assert!(is_k_degenerate(&g, 0).unwrap());
        assert_eq!(degeneracy(&g).unwrap(), 0);
    }

    #[test]
    fn star() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_k_degenerate(&g, 1).unwrap());
        assert_eq!(degeneracy(&g).unwrap(), 1);
    }

    #[test]
    fn c4() {
        // C_4: coreness 2
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_k_degenerate(&g, 2).unwrap());
        assert!(!is_k_degenerate(&g, 1).unwrap());
        assert_eq!(degeneracy(&g).unwrap(), 2);
    }

    #[test]
    fn directed_treated_as_undirected() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_k_degenerate(&g, 2).unwrap());
        assert_eq!(degeneracy(&g).unwrap(), 2);
    }

    #[test]
    fn large_k_always_true() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert!(is_k_degenerate(&g, 100).unwrap());
    }

    #[test]
    fn path() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_k_degenerate(&g, 1).unwrap());
        assert_eq!(degeneracy(&g).unwrap(), 1);
    }
}
