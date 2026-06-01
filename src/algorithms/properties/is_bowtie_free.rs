//! Bowtie-free graph predicate (ALGO-PR-103).
//!
//! A graph is bowtie-free if it contains no induced bowtie (also called
//! butterfly or hourglass). The bowtie consists of two triangles sharing
//! exactly one vertex (the "center"), with 5 vertices and 6 edges total.
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is bowtie-free (no induced bowtie / butterfly).
///
/// The bowtie is two triangles sharing a single vertex. It has 5 vertices
/// {a, b, c, d, e} where c is the center: edges {a-b, a-c, b-c, c-d,
/// c-e, d-e}, and no other edges among these 5 vertices.
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_bowtie_free};
///
/// // Triangle is bowtie-free
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_bowtie_free(&g).unwrap());
///
/// // Bowtie: triangles {0,1,2} and {2,3,4} sharing vertex 2
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(2, 4).unwrap();
/// g.add_edge(3, 4).unwrap();
/// assert!(!is_bowtie_free(&g).unwrap());
/// ```
pub fn is_bowtie_free(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n < 5 {
        return Ok(true);
    }

    let n_usize = n as usize;
    let mut adj = vec![vec![false; n_usize]; n_usize];
    let mut nbrs_list: Vec<Vec<u32>> = Vec::with_capacity(n_usize);

    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            adj[v as usize][w as usize] = true;
        }
        nbrs_list.push(nbrs);
    }

    // A bowtie has center c with two triangles: {a, b, c} and {d, e, c}
    // where {a, b} and {d, e} are disjoint and no cross-edges exist
    // (a-d, a-e, b-d, b-e all absent).
    //
    // Strategy: for each vertex c, find all edges among its neighbors
    // (pairs that form a triangle with c). If c participates in ≥ 2
    // triangles, check if any two triangles are vertex-disjoint
    // (excluding c) with no cross-edges.
    for c in 0..n {
        let ci = c as usize;
        let cn = &nbrs_list[ci];

        // Collect triangle-partner pairs among neighbors of c
        let mut triangle_edges: Vec<(usize, usize)> = Vec::new();
        for (idx, &u) in cn.iter().enumerate() {
            let ui = u as usize;
            for &v in &cn[(idx + 1)..] {
                let vi = v as usize;
                if adj[ui][vi] {
                    triangle_edges.push((ui, vi));
                }
            }
        }

        if triangle_edges.len() < 2 {
            continue;
        }

        // Check if any two triangle edges form an induced bowtie
        for (i, &(a, b)) in triangle_edges.iter().enumerate() {
            for &(d, e) in &triangle_edges[(i + 1)..] {
                if a == d || a == e || b == d || b == e {
                    continue;
                }
                if !adj[a][d] && !adj[a][e] && !adj[b][d] && !adj[b][e] {
                    return Ok(false);
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
        assert!(is_bowtie_free(&g).unwrap());
    }

    #[test]
    fn small_graphs() {
        let g = Graph::with_vertices(4);
        assert!(is_bowtie_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_bowtie_free(&g).unwrap());
    }

    #[test]
    fn bowtie() {
        // Two triangles sharing vertex 2
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(2, 4).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(!is_bowtie_free(&g).unwrap());
    }

    #[test]
    fn k4_bowtie_free() {
        // `K_4`: any 5-vertex subset would need 5 vertices, but only 4 exist.
        // Even considering all vertices, `K_4` has no bowtie since it only has 4 vertices.
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_bowtie_free(&g).unwrap());
    }

    #[test]
    fn k5_bowtie_free() {
        // `K_5`: two triangles sharing a vertex exist, but cross-edges
        // are also present → no *induced* bowtie
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_bowtie_free(&g).unwrap());
    }

    #[test]
    fn diamond_bowtie_free() {
        // Diamond (`K_4` minus one edge): 4 vertices, not enough for bowtie
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_bowtie_free(&g).unwrap());
    }

    #[test]
    fn two_disjoint_triangles_bowtie_free() {
        // Two triangles that don't share any vertex → no bowtie
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        assert!(is_bowtie_free(&g).unwrap());
    }

    #[test]
    fn two_triangles_sharing_edge_bowtie_free() {
        // Two triangles sharing an edge (diamond with extra vertex): {0,1,2} and {0,1,3}
        // Triangles share edge 0-1, not just a vertex → not a bowtie
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_bowtie_free(&g).unwrap());
    }

    #[test]
    fn bowtie_with_extra_cross_edge() {
        // Bowtie + one cross-edge: {0,1,2} and {2,3,4} but add 0-3
        // Now the induced subgraph on {0,1,2,3,4} has 7 edges, not a bowtie
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(2, 4).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(is_bowtie_free(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_bowtie_free(&g).unwrap());
    }

    #[test]
    fn path_bowtie_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_bowtie_free(&g).unwrap());
    }

    #[test]
    fn star_bowtie_free() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        assert!(is_bowtie_free(&g).unwrap());
    }

    #[test]
    fn triple_bowtie() {
        // Three triangles sharing vertex 0: {0,1,2}, {0,3,4}, {0,5,6}
        // Multiple bowties present
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        g.add_edge(0, 6).unwrap();
        g.add_edge(5, 6).unwrap();
        assert!(!is_bowtie_free(&g).unwrap());
    }
}
