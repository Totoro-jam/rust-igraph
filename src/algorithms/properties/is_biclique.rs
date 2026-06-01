//! Complete bipartite graph predicate (ALGO-PR-083).
//!
//! A graph is a complete bipartite graph (biclique) if it is bipartite
//! and every vertex in one part is adjacent to every vertex in the
//! other part.
//!
//! For directed graphs, the function returns `false`.

use crate::algorithms::properties::is_bipartite::is_bipartite;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a complete bipartite graph (biclique).
///
/// Returns `false` for directed graphs.
/// Returns `true` for the empty graph (vacuously).
/// Returns `true` for a single vertex (trivial `K_{1,0}`).
///
/// Self-loops and multi-edges cause `false` (not simple bipartite).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_biclique};
///
/// // K_{2,3}: complete bipartite
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(0, 4).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(1, 4).unwrap();
/// assert!(is_biclique(&g).unwrap());
///
/// // P4 is NOT a biclique (bipartite but not complete bipartite)
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(!is_biclique(&g).unwrap());
/// ```
pub fn is_biclique(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n <= 1 {
        return Ok(true);
    }

    let bp = is_bipartite(graph)?;
    if !bp.is_bipartite {
        return Ok(false);
    }

    let part_a: usize = bp.types.iter().filter(|&&t| !t).count();
    let part_b: usize = bp.types.iter().filter(|&&t| t).count();

    // Expected edges in a complete bipartite graph
    let expected_edges = part_a.saturating_mul(part_b);
    if graph.ecount() != expected_edges {
        return Ok(false);
    }

    // Edge count match + bipartite ⟹ complete bipartite
    // (any missing cross-edge would reduce the count, and
    // bipartiteness ensures no intra-part edges)
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_biclique(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_biclique(&g).unwrap());
    }

    #[test]
    fn single_edge_k11() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_biclique(&g).unwrap());
    }

    #[test]
    fn k13() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(is_biclique(&g).unwrap());
    }

    #[test]
    fn k22() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_biclique(&g).unwrap());
    }

    #[test]
    fn k23() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(is_biclique(&g).unwrap());
    }

    #[test]
    fn k33() {
        let mut g = Graph::with_vertices(6);
        for u in 0..3u32 {
            for v in 3..6u32 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(is_biclique(&g).unwrap());
    }

    #[test]
    fn path_p3_is_k12() {
        // P3 = 0-1-2 is K_{1,2} (star with center 1)
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_biclique(&g).unwrap());
    }

    #[test]
    fn path_p4_not_biclique() {
        // P4 = 0-1-2-3: bipartite {0,2},{1,3} but only 3 of 4 cross-edges
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_biclique(&g).unwrap());
    }

    #[test]
    fn triangle_not_biclique() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_biclique(&g).unwrap());
    }

    #[test]
    fn c4_is_k22() {
        // C4 = K_{2,2}: bipartite {0,2},{1,3}, all 4 cross-edges present
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_biclique(&g).unwrap());
    }

    #[test]
    fn edgeless_two_vertices() {
        // Edgeless 2-vertex graph: bipartite puts both in same part → K_{2,0}
        let g = Graph::with_vertices(2);
        assert!(is_biclique(&g).unwrap());
    }

    #[test]
    fn star_is_biclique() {
        // Star K_{1,4} is a complete bipartite graph
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_biclique(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(!is_biclique(&g).unwrap());
    }

    #[test]
    fn disconnected_not_biclique() {
        // Two components: each is K_{1,1}, but whole graph is not a biclique
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_biclique(&g).unwrap());
    }

    #[test]
    fn k14() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_biclique(&g).unwrap());
    }
}
