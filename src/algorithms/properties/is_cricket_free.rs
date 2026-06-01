//! Cricket-free graph predicate (ALGO-PR-104).
//!
//! A graph is cricket-free if it contains no induced cricket. The
//! cricket has 5 vertices: a triangle {a, b, c} plus two pendant edges
//! from the *same* triangle vertex, say b-d and b-e (d and e each have
//! degree 1 in the cricket). It has 5 edges total.
//!
//! The cricket differs from the bull: in a bull the two pendants hang
//! from *different* triangle vertices; in a cricket they hang from the
//! *same* vertex.
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is cricket-free.
///
/// The cricket is a triangle with two pendant edges from the same
/// triangle vertex. A cricket-free graph has no induced cricket.
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_cricket_free};
///
/// // Triangle is cricket-free
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_cricket_free(&g).unwrap());
///
/// // Cricket: triangle {0,1,2}, pendants 1-3 and 1-4
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(1, 4).unwrap();
/// assert!(!is_cricket_free(&g).unwrap());
/// ```
pub fn is_cricket_free(graph: &Graph) -> IgraphResult<bool> {
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

    // Induced cricket: triangle {a, b, c} with two pendant vertices d, e
    // adjacent to b (the "hub") but not to a, c, or each other.
    // For each triangle, check each of the 3 vertices as potential hub.
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

                // Triangle {a, b, c}. Try each vertex as the pendant hub.
                if has_cricket_pendants(&adj, n_usize, ai, bi, ci) {
                    return Ok(false);
                }
                if has_cricket_pendants(&adj, n_usize, bi, ai, ci) {
                    return Ok(false);
                }
                if has_cricket_pendants(&adj, n_usize, ci, ai, bi) {
                    return Ok(false);
                }
            }
        }
    }

    Ok(true)
}

/// Check if vertex `hub` (part of triangle {hub, t1, t2}) has two
/// pendant neighbors d, e that form an induced cricket.
/// d and e must be adjacent to hub only, not to t1, t2, or each other.
fn has_cricket_pendants(adj: &[Vec<bool>], n: usize, hub: usize, t1: usize, t2: usize) -> bool {
    let row_hub = &adj[hub];
    let row_t1 = &adj[t1];
    let row_t2 = &adj[t2];

    let pendants: Vec<usize> = row_hub
        .iter()
        .enumerate()
        .take(n)
        .filter(|&(d, &is_adj)| {
            is_adj && d != hub && d != t1 && d != t2 && !row_t1[d] && !row_t2[d]
        })
        .map(|(d, _)| d)
        .collect();

    if pendants.len() < 2 {
        return false;
    }

    for (i, &d) in pendants.iter().enumerate() {
        for &e in &pendants[(i + 1)..] {
            if !adj[d][e] {
                return true;
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
        assert!(is_cricket_free(&g).unwrap());
    }

    #[test]
    fn small_graphs() {
        let g = Graph::with_vertices(4);
        assert!(is_cricket_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_cricket_free(&g).unwrap());
    }

    #[test]
    fn cricket() {
        // Triangle {0,1,2}, pendants 1-3 and 1-4
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(!is_cricket_free(&g).unwrap());
    }

    #[test]
    fn bull_is_cricket_free() {
        // Bull: triangle {0,1,2}, pendants from DIFFERENT vertices: 1-3, 2-4
        // Not a cricket (pendants from same vertex)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 4).unwrap();
        assert!(is_cricket_free(&g).unwrap());
    }

    #[test]
    fn k5_cricket_free() {
        // `K_5`: all edges present → no induced cricket (pendants would need non-edges)
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_cricket_free(&g).unwrap());
    }

    #[test]
    fn paw_cricket_free() {
        // Paw: triangle {0,1,2} + pendant 2-3. Only one pendant → not a cricket
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_cricket_free(&g).unwrap());
    }

    #[test]
    fn pendants_adjacent_not_cricket() {
        // Triangle {0,1,2}, "pendants" 1-3, 1-4 but 3-4 adjacent
        // Induced subgraph has edge 3-4 → not a cricket (cricket has no d-e edge)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_cricket_free(&g).unwrap());
    }

    #[test]
    fn pendant_connected_to_triangle_vertex() {
        // Triangle {0,1,2}, 1-3, 1-4, but 3-0 also present
        // d=3 is adjacent to a=0, so not a pendant → not a cricket on this subgraph
        // But check: triangle {0,1,2} with hub=1: pendant candidates must NOT
        // be adjacent to 0 or 2. d=3 is adjacent to 0 → d=3 not a pendant.
        // So only e=4 is a pendant of hub=1, and we need ≥ 2.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(is_cricket_free(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_cricket_free(&g).unwrap());
    }

    #[test]
    fn star_cricket_free() {
        // Star: no triangles → no cricket
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        assert!(is_cricket_free(&g).unwrap());
    }

    #[test]
    fn path_cricket_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_cricket_free(&g).unwrap());
    }

    #[test]
    fn three_pendants_from_hub() {
        // Triangle {0,1,2}, three pendants from 1: 1-3, 1-4, 1-5
        // Any pair of non-adjacent pendants forms a cricket
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(1, 5).unwrap();
        assert!(!is_cricket_free(&g).unwrap());
    }
}
