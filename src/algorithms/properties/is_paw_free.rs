//! Paw-free graph predicate (ALGO-PR-099).
//!
//! A graph is paw-free if it contains no induced subgraph isomorphic
//! to the paw graph. The paw is a triangle with one pendant edge
//! (4 vertices, 4 edges): vertices {a, b, c, d} with edges
//! {ab, ac, bc, cd} (d is pendant, adjacent only to c).
//!
//! Paw-free graphs are exactly the graphs where every connected
//! component is either triangle-free or complete.
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is paw-free.
///
/// The paw is a triangle plus a pendant edge. A paw-free graph has
/// no induced paw. Equivalently, every connected component is
/// either triangle-free or complete.
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_paw_free};
///
/// // Triangle is paw-free (no pendant vertex)
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_paw_free(&g).unwrap());
///
/// // Triangle + pendant: paw!
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(!is_paw_free(&g).unwrap());
/// ```
pub fn is_paw_free(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n < 4 {
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

    // Check for induced paw: find a triangle {a, b, c} where some
    // vertex d is adjacent to exactly one of {a, b, c}.
    // Equivalently: for each edge (a, b), find common neighbor c
    // (forming a triangle), then check if any neighbor of a, b, or c
    // is adjacent to exactly one member of the triangle.
    for a in 0..n {
        let ai = a as usize;
        for b in (a + 1)..n {
            let bi = b as usize;
            if !adj[ai][bi] {
                continue;
            }

            // Find common neighbors of a and b (forming triangles)
            for c in (b + 1)..n {
                let ci = c as usize;
                if !adj[ai][ci] || !adj[bi][ci] {
                    continue;
                }

                // Triangle {a, b, c} found. Check if any vertex d
                // is adjacent to exactly one of {a, b, c}.
                for d in 0..n {
                    if d == a || d == b || d == c {
                        continue;
                    }
                    let di = d as usize;
                    let count =
                        u32::from(adj[ai][di]) + u32::from(adj[bi][di]) + u32::from(adj[ci][di]);
                    if count == 1 {
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
        assert!(is_paw_free(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_paw_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_paw_free(&g).unwrap());
    }

    #[test]
    fn paw() {
        // Triangle 0-1-2 + pendant 2-3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_paw_free(&g).unwrap());
    }

    #[test]
    fn k4_paw_free() {
        // K_4 is complete → paw-free (no vertex adjacent to exactly 1 of a triangle)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_paw_free(&g).unwrap());
    }

    #[test]
    fn path_paw_free() {
        // Path: no triangles → no paw
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_paw_free(&g).unwrap());
    }

    #[test]
    fn c5_paw_free() {
        // C_5 is triangle-free → paw-free
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_paw_free(&g).unwrap());
    }

    #[test]
    fn diamond_not_paw_free() {
        // Diamond: K_4 minus edge 2-3. Triangle {0,1,2} with vertex 3
        // adjacent to 0 and 1 but not 2 → vertex 3 adj to 2 of triangle
        // Actually: diamond = {01, 02, 03, 12, 13}. Triangle {0,1,2}.
        // Vertex 3 is adjacent to 0 and 1 (count=2) → not exactly 1.
        // Triangle {0,1,3}: vertex 2 adjacent to 0 and 1 (count=2) → not 1.
        // So diamond might actually be paw-free? Let me check:
        // Triangle {0,1,2}: vertex 3 adj to 0(yes) 1(yes) 2(no) → count=2
        // Triangle {0,1,3}: vertex 2 adj to 0(yes) 1(yes) 3(no) → count=2
        // No triangle has a vertex adjacent to exactly 1 member → paw-free!
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_paw_free(&g).unwrap());
    }

    #[test]
    fn bull_not_paw_free() {
        // Bull: triangle 0-1-2, pendants 1-3 and 2-4
        // Triangle {0,1,2}, vertex 3 adjacent to 1 only → count=1 → paw!
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 4).unwrap();
        assert!(!is_paw_free(&g).unwrap());
    }

    #[test]
    fn disconnected_triangle_plus_edge() {
        // Triangle + isolated edge: no paw (the edge vertex is in
        // a different component from the triangle)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        // Triangle {0,1,2}: vertex 3 adj to 0(no) 1(no) 2(no) → count=0
        // vertex 4 adj to 0(no) 1(no) 2(no) → count=0
        // No paw!
        assert!(is_paw_free(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_paw_free(&g).unwrap());
    }

    #[test]
    fn isolated_vertices() {
        let g = Graph::with_vertices(10);
        assert!(is_paw_free(&g).unwrap());
    }

    #[test]
    fn windmill_not_paw_free() {
        // Wd(3,2): two triangles sharing vertex 0
        // 0-1-2-0, 0-3-4-0
        // Triangle {0,1,2}: vertex 3 adj to 0 only → count=1 → paw!
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(!is_paw_free(&g).unwrap());
    }
}
