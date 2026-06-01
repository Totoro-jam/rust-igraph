//! Diamond-free graph predicate (ALGO-PR-097).
//!
//! A graph is diamond-free if it contains no induced subgraph
//! isomorphic to the diamond graph (`K_4` minus one edge, also
//! denoted `K_4^-`). The diamond has 4 vertices and 5 edges.
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is diamond-free.
///
/// A diamond-free graph has no induced `K_4` minus one edge. The
/// diamond has vertices {a, b, c, d} with edges {ab, ac, ad, bc, bd}
/// (c and d not adjacent).
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_diamond_free};
///
/// // Triangle is diamond-free
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_diamond_free(&g).unwrap());
///
/// // Diamond: `K_4` minus edge 2-3
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// assert!(!is_diamond_free(&g).unwrap());
/// ```
pub fn is_diamond_free(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n <= 3 {
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

    // A diamond is formed when an edge (u, v) has two or more common
    // neighbors c1, c2 such that c1 and c2 are NOT adjacent.
    // If c1-c2 are adjacent, then {u, v, c1, c2} form K_4.
    // If c1-c2 are non-adjacent, then {u, v, c1, c2} form a diamond.
    //
    // So: for each edge (u, v), find all common neighbors, and check
    // if any pair among them is non-adjacent.
    for u in 0..n {
        let nbrs_u = graph.neighbors(u)?;
        for &v in &nbrs_u {
            if v <= u {
                continue;
            }

            // Find common neighbors of u and v
            let common: Vec<u32> = nbrs_u
                .iter()
                .filter(|&&w| w != v && adj[v as usize][w as usize])
                .copied()
                .collect();

            // Check if any pair of common neighbors is non-adjacent
            for i in 0..common.len() {
                for j in (i + 1)..common.len() {
                    if !adj[common[i] as usize][common[j] as usize] {
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
        assert!(is_diamond_free(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_diamond_free(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_diamond_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_diamond_free(&g).unwrap());
    }

    #[test]
    fn diamond() {
        // K_4 minus edge 2-3: vertices {0,1,2,3}, edges {01,02,03,12,13}
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(!is_diamond_free(&g).unwrap());
    }

    #[test]
    fn k4_is_diamond_free() {
        // K_4 is diamond-free: any 4 vertices induce K_4 (6 edges),
        // not a diamond (5 edges)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_diamond_free(&g).unwrap());
    }

    #[test]
    fn path_diamond_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_diamond_free(&g).unwrap());
    }

    #[test]
    fn cycle_c5_diamond_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_diamond_free(&g).unwrap());
    }

    #[test]
    fn star_diamond_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_diamond_free(&g).unwrap());
    }

    #[test]
    fn two_triangles_sharing_vertex() {
        // 0-1-2-0, 0-3-4-0
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        // Vertex 0 connects to 1,2,3,4. Common neighbors of edge 0-1:
        // just 2. Common neighbors of edge 0-3: just 4. No diamond.
        assert!(is_diamond_free(&g).unwrap());
    }

    #[test]
    fn two_triangles_sharing_edge() {
        // 0-1-2-0, 0-1-3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 3).unwrap();
        // Edge 0-1 has common neighbors 2 and 3.
        // 2-3 not adjacent → diamond!
        assert!(!is_diamond_free(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_diamond_free(&g).unwrap());
    }

    #[test]
    fn isolated_vertices() {
        let g = Graph::with_vertices(10);
        assert!(is_diamond_free(&g).unwrap());
    }

    #[test]
    fn wheel_w5_not_diamond_free() {
        // W_5: hub 0 connected to cycle 1-2-3-4-5-1
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 1).unwrap();
        // Edge 0-1 has common neighbors 2 and 5; 2-5 not adjacent → diamond
        assert!(!is_diamond_free(&g).unwrap());
    }
}
