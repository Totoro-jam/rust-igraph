//! Complete bipartite graph predicate (ALGO-PR-122).
//!
//! A graph is complete bipartite (`K_{m,n}`) if its vertices can be
//! partitioned into two non-empty independent sets such that every
//! vertex in one set is adjacent to every vertex in the other.
//!
//! Every star `K_{1,n}` is complete bipartite. Every single edge is
//! `K_{1,1}`. The empty graph and edgeless graphs with more than one
//! component (or a single part) are not complete bipartite.
//!
//! Directed graphs are treated as undirected.

use crate::algorithms::properties::is_bipartite::{BipartiteResult, is_bipartite};
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is complete bipartite.
///
/// Returns `true` if the graph is isomorphic to `K_{m,n}` for some
/// `m, n ≥ 1`. Uses the bipartite check to find the two parts, then
/// verifies that the edge count equals `m * n`.
///
/// Directed graphs are treated as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_complete_bipartite};
///
/// // `K_{2,3}`: 2 vertices connected to 3 vertices
/// let mut g = Graph::with_vertices(5);
/// for i in 0..2u32 {
///     for j in 2..5u32 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// assert!(is_complete_bipartite(&g).unwrap());
///
/// // `C_6`: bipartite but not complete bipartite
/// let mut g = Graph::with_vertices(6);
/// for i in 0..6u32 {
///     g.add_edge(i, (i + 1) % 6).unwrap();
/// }
/// assert!(!is_complete_bipartite(&g).unwrap());
/// ```
pub fn is_complete_bipartite(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();

    if n < 2 {
        return Ok(false);
    }

    let bip: BipartiteResult = is_bipartite(graph)?;
    if !bip.is_bipartite {
        return Ok(false);
    }

    let part_a = bip.types.iter().filter(|&&t| t).count();
    let part_b = bip.types.iter().filter(|&&t| !t).count();

    if part_a == 0 || part_b == 0 {
        return Ok(false);
    }

    let expected_edges = part_a.saturating_mul(part_b);
    Ok(graph.ecount() == expected_edges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(!is_complete_bipartite(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(!is_complete_bipartite(&g).unwrap());
    }

    #[test]
    fn single_edge_k11() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_complete_bipartite(&g).unwrap());
    }

    #[test]
    fn k12_star() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        assert!(is_complete_bipartite(&g).unwrap());
    }

    #[test]
    fn k22() {
        // K_{2,2} = C_4 with all cross edges
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_complete_bipartite(&g).unwrap());
    }

    #[test]
    fn k23() {
        let mut g = Graph::with_vertices(5);
        for i in 0..2u32 {
            for j in 2..5u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_complete_bipartite(&g).unwrap());
    }

    #[test]
    fn k33() {
        let mut g = Graph::with_vertices(6);
        for i in 0..3u32 {
            for j in 3..6u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_complete_bipartite(&g).unwrap());
    }

    #[test]
    fn c4_is_k22() {
        // C_4 = K_{2,2}: parts {0,2} and {1,3}, all 4 cross edges present
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_complete_bipartite(&g).unwrap());
    }

    #[test]
    fn c6_not_complete_bipartite() {
        let mut g = Graph::with_vertices(6);
        for i in 0..6u32 {
            g.add_edge(i, (i + 1) % 6).unwrap();
        }
        assert!(!is_complete_bipartite(&g).unwrap());
    }

    #[test]
    fn triangle_not_bipartite() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_complete_bipartite(&g).unwrap());
    }

    #[test]
    fn k4_not_bipartite() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(!is_complete_bipartite(&g).unwrap());
    }

    #[test]
    fn edgeless() {
        let g = Graph::with_vertices(4);
        assert!(!is_complete_bipartite(&g).unwrap());
    }

    #[test]
    fn two_components_not_complete_bipartite() {
        // Two K_{1,1} components
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_complete_bipartite(&g).unwrap());
    }

    #[test]
    fn path_p3_is_k12() {
        // P_3 = K_{1,2}: part {1} connects to both {0,2}
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_complete_bipartite(&g).unwrap());
    }

    #[test]
    fn directed_treated_as_undirected() {
        // K_{1,2} as directed
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        assert!(is_complete_bipartite(&g).unwrap());
    }
}
