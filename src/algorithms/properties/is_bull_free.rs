//! Bull-free graph predicate (ALGO-PR-100).
//!
//! A graph is bull-free if it contains no induced subgraph isomorphic
//! to the bull graph. The bull has 5 vertices: a triangle {a, b, c}
//! plus two pendant edges b-d and c-e (d and e each have degree 1).
//! It has 5 edges total.
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is bull-free.
///
/// The bull graph is a triangle with two pendant edges from distinct
/// triangle vertices. A bull-free graph has no induced bull.
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_bull_free};
///
/// // Triangle is bull-free
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_bull_free(&g).unwrap());
///
/// // Bull: triangle 0-1-2, pendants 1-3 and 2-4
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(2, 4).unwrap();
/// assert!(!is_bull_free(&g).unwrap());
/// ```
pub fn is_bull_free(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n < 5 {
        return Ok(true);
    }

    let n_usize = n as usize;
    let mut adj = vec![vec![false; n_usize]; n_usize];
    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            adj[v as usize][w as usize] = true;
        }
    }

    // A bull is: triangle {a, b, c} with vertices d adjacent to b
    // (but not a, c, e) and e adjacent to c (but not a, b, d).
    // Strategy: for each triangle {a, b, c}, check if two distinct
    // triangle vertices each have a neighbor outside the triangle
    // that is not adjacent to any other triangle vertex or each other.
    for a in 0..n {
        let ai = a as usize;
        for b in (a + 1)..n {
            let bi = b as usize;
            if !adj[ai][bi] {
                continue;
            }
            for c in (b + 1)..n {
                let ci = c as usize;
                if !adj[ai][ci] || !adj[bi][ci] {
                    continue;
                }

                // Triangle {a, b, c}. Check all 3 pairs of triangle
                // vertices for pendant neighbors forming a bull.
                // Try (b pendant d, c pendant e):
                if has_bull_pendants(&adj, n, ai, bi, ci) {
                    return Ok(false);
                }
                // Try (a pendant d, c pendant e):
                if has_bull_pendants(&adj, n, bi, ai, ci) {
                    return Ok(false);
                }
                // Try (a pendant d, b pendant e):
                if has_bull_pendants(&adj, n, ci, ai, bi) {
                    return Ok(false);
                }
            }
        }
    }

    Ok(true)
}

/// Check if triangle vertices (apex, v1, v2) have pendant neighbors
/// d (of v1) and e (of v2) forming a bull.
/// d must be adjacent to v1 only (not apex, v2, or e).
/// e must be adjacent to v2 only (not apex, v1, or d).
fn has_bull_pendants(adj: &[Vec<bool>], _n: u32, apex: usize, v1: usize, v2: usize) -> bool {
    let row_v1 = &adj[v1];
    let row_apex = &adj[apex];
    let row_v2 = &adj[v2];

    let d_candidates: Vec<usize> = row_v1
        .iter()
        .enumerate()
        .filter(|&(d, &is_adj)| {
            is_adj && d != apex && d != v1 && d != v2 && !row_apex[d] && !row_v2[d]
        })
        .map(|(d, _)| d)
        .collect();

    if d_candidates.is_empty() {
        return false;
    }

    for (e, &v2_to_e) in row_v2.iter().enumerate() {
        if !v2_to_e || e == apex || e == v1 || e == v2 {
            continue;
        }
        if !row_apex[e] && !row_v1[e] {
            for &d in &d_candidates {
                if d != e && !adj[d][e] {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_bull_free(&g).unwrap());
    }

    #[test]
    fn small_graphs() {
        let g = Graph::with_vertices(4);
        assert!(is_bull_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_bull_free(&g).unwrap());
    }

    #[test]
    fn bull() {
        // Triangle 0-1-2, pendants 1-3, 2-4
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 4).unwrap();
        assert!(!is_bull_free(&g).unwrap());
    }

    #[test]
    fn k5_bull_free() {
        // Complete graphs are bull-free (every 5-vertex subgraph is K_5)
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_bull_free(&g).unwrap());
    }

    #[test]
    fn c5_bull_free() {
        // C_5 has no triangles → no bull
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_bull_free(&g).unwrap());
    }

    #[test]
    fn path_bull_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_bull_free(&g).unwrap());
    }

    #[test]
    fn paw_bull_free() {
        // Paw: triangle 0-1-2 + pendant 2-3
        // Only one pendant → not a bull (need two)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_bull_free(&g).unwrap());
    }

    #[test]
    fn diamond_bull_free() {
        // Diamond: K_4 minus one edge
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_bull_free(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_bull_free(&g).unwrap());
    }

    #[test]
    fn star_bull_free() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        assert!(is_bull_free(&g).unwrap());
    }

    #[test]
    fn triangle_with_two_pendants_on_same_vertex() {
        // Triangle 0-1-2, two pendants from vertex 1: 1-3, 1-4
        // Not a bull (pendants must be from different triangle vertices)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        // Triangle {0,1,2}: pendants of 1 are 3,4. But we need pendants
        // from two DIFFERENT triangle vertices. Vertex 0 has no pendant,
        // vertex 2 has no pendant. So no bull.
        assert!(is_bull_free(&g).unwrap());
    }

    #[test]
    fn connected_pendants_not_bull() {
        // Triangle 0-1-2, pendants 1-3, 2-4, but 3-4 adjacent
        // The induced subgraph {0,1,2,3,4} has 3-4 edge → not a bull
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 4).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_bull_free(&g).unwrap());
    }
}
