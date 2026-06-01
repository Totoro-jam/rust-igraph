//! Claw-free graph predicate (ALGO-PR-096).
//!
//! A graph is claw-free if it contains no induced subgraph
//! isomorphic to `K_{1,3}` (the complete bipartite graph with parts
//! of size 1 and 3, also called the "claw" or "star on 3 edges").
//!
//! Equivalently, for every vertex v, the open neighborhood N(v) does
//! not contain three mutually non-adjacent vertices (i.e., no
//! independent set of size 3 in N(v)).
//!
//! Claw-free graphs include line graphs and complements of
//! triangle-free graphs.
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is claw-free.
///
/// A claw-free graph has no induced `K_{1,3}`. This is checked by
/// verifying that for every vertex, no three of its neighbors are
/// mutually non-adjacent.
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_claw_free};
///
/// // Triangle is claw-free
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_claw_free(&g).unwrap());
///
/// // Star `K_{1,3}` IS a claw
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// assert!(!is_claw_free(&g).unwrap());
/// ```
pub fn is_claw_free(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n <= 3 {
        // Can't have K_{1,3} with fewer than 4 vertices
        return Ok(true);
    }

    // Build adjacency for fast pair lookup
    let n_usize = n as usize;
    let mut adj = vec![vec![false; n_usize]; n_usize];
    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            adj[v as usize][w as usize] = true;
        }
    }

    // For each vertex v, check if N(v) contains an independent set of size 3
    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        let deg = nbrs.len();
        if deg < 3 {
            continue;
        }

        // Check all triples of neighbors
        for i in 0..deg {
            for j in (i + 1)..deg {
                if adj[nbrs[i] as usize][nbrs[j] as usize] {
                    continue;
                }
                // nbrs[i] and nbrs[j] are non-adjacent
                for k in (j + 1)..deg {
                    if !adj[nbrs[i] as usize][nbrs[k] as usize]
                        && !adj[nbrs[j] as usize][nbrs[k] as usize]
                    {
                        // Found independent triple in N(v) → induced K_{1,3}
                        return Ok(false);
                    }
                }
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
        let g = Graph::with_vertices(0);
        assert!(is_claw_free(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_claw_free(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_claw_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_claw_free(&g).unwrap());
    }

    #[test]
    fn k4() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_claw_free(&g).unwrap());
    }

    #[test]
    fn star_k13_is_claw() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(!is_claw_free(&g).unwrap());
    }

    #[test]
    fn star_k14_not_claw_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(!is_claw_free(&g).unwrap());
    }

    #[test]
    fn path_claw_free() {
        // Path 0-1-2-3: max degree 2, can't have K_{1,3}
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_claw_free(&g).unwrap());
    }

    #[test]
    fn cycle_c5_claw_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_claw_free(&g).unwrap());
    }

    #[test]
    fn line_graph_of_k4() {
        // Line graph of K4 has 6 vertices (one per edge of K4)
        // Line graphs are always claw-free (Whitney's theorem)
        // Edges of K4: 01, 02, 03, 12, 13, 23
        // Two edges adjacent iff they share a vertex
        let mut g = Graph::with_vertices(6);
        // 01-02, 01-03, 01-12, 01-13
        g.add_edge(0, 1).unwrap(); // 01-02
        g.add_edge(0, 2).unwrap(); // 01-03
        g.add_edge(0, 3).unwrap(); // 01-12
        g.add_edge(0, 4).unwrap(); // 01-13
        // 02-03, 02-12, 02-23
        g.add_edge(1, 2).unwrap(); // 02-03
        g.add_edge(1, 3).unwrap(); // 02-12
        g.add_edge(1, 5).unwrap(); // 02-23
        // 03-13, 03-23
        g.add_edge(2, 4).unwrap(); // 03-13
        g.add_edge(2, 5).unwrap(); // 03-23
        // 12-13, 12-23
        g.add_edge(3, 4).unwrap(); // 12-13
        g.add_edge(3, 5).unwrap(); // 12-23
        // 13-23
        g.add_edge(4, 5).unwrap(); // 13-23
        assert!(is_claw_free(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_claw_free(&g).unwrap());
    }

    #[test]
    fn diamond_claw_free() {
        // Diamond: K4 minus one edge
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        // Vertex 0 has neighbors {1, 2, 3}; 1-2 adjacent, 2-3 adjacent,
        // but 1-3 not adjacent. That's only 2 non-adjacent, need 3.
        // No vertex has 3 mutually non-adjacent neighbors.
        assert!(is_claw_free(&g).unwrap());
    }

    #[test]
    fn petersen_not_claw_free() {
        // Petersen graph is NOT claw-free (3-regular, 10 vertices)
        let mut g = Graph::with_vertices(10);
        // Outer cycle
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        // Inner pentagram
        g.add_edge(5, 7).unwrap();
        g.add_edge(7, 9).unwrap();
        g.add_edge(9, 6).unwrap();
        g.add_edge(6, 8).unwrap();
        g.add_edge(8, 5).unwrap();
        // Spokes
        g.add_edge(0, 5).unwrap();
        g.add_edge(1, 6).unwrap();
        g.add_edge(2, 7).unwrap();
        g.add_edge(3, 8).unwrap();
        g.add_edge(4, 9).unwrap();
        assert!(!is_claw_free(&g).unwrap());
    }

    #[test]
    fn two_triangles_sharing_edge_claw_free() {
        // 0-1-2-0, 1-2-3-1
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_claw_free(&g).unwrap());
    }
}
