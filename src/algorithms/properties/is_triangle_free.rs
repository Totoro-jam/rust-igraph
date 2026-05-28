//! Triangle-free test (ALGO-PR-039).
//!
//! Counterpart of `igraph_is_triangle_free()` from
//! `references/igraph/src/properties/triangles.c`.
//!
//! Checks whether the graph contains no triangles (3-cycles).

use crate::core::{Graph, IgraphResult};

/// Test whether a graph is triangle-free.
///
/// A graph is triangle-free if it contains no 3-clique (cycle of length 3).
/// Edge directions, multi-edges, and self-loops are ignored for the purpose
/// of this check (the underlying simple undirected graph is tested).
///
/// Uses sorted-neighbor-intersection with early exit for efficiency.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_triangle_free};
///
/// // Path graph: no triangles
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_triangle_free(&g).unwrap());
///
/// // Add edge 0-2 to create a triangle
/// g.add_edge(0, 2).unwrap();
/// assert!(!is_triangle_free(&g).unwrap());
/// ```
pub fn is_triangle_free(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();

    // Trivial cases: need at least 3 vertices and 3 edges for a triangle.
    if n < 3 || m < 3 {
        return Ok(true);
    }

    // Build simple undirected adjacency list (sorted, no loops, no multi-edges).
    let adj = build_sorted_simple_adjlist(graph)?;

    // For each edge (u, v) with u < v, check if their sorted neighbor
    // lists intersect — if so, there's a triangle.
    for u in 0..n {
        let du = adj[u].len();
        if du < 2 {
            continue;
        }
        for &v in &adj[u] {
            if v <= u {
                continue;
            }
            let dv = adj[v].len();
            if dv < 2 {
                continue;
            }

            // Sorted intersection check.
            if has_common_element(&adj[u], &adj[v]) {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Check if two sorted slices have a common element.
fn has_common_element(a: &[usize], b: &[usize]) -> bool {
    let mut i = 0;
    let mut j = 0;
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Equal => return true,
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
    false
}

/// Build a simple undirected adjacency list (sorted, no loops, no multi-edges).
fn build_sorted_simple_adjlist(graph: &Graph) -> IgraphResult<Vec<Vec<usize>>> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

    for eid_usize in 0..m {
        #[allow(clippy::cast_possible_truncation)]
        let eid = eid_usize as u32;
        let (from, to) = graph.edge(eid)?;
        let f = from as usize;
        let t = to as usize;
        if f != t {
            adj[f].push(t);
            adj[t].push(f);
        }
    }

    for neighbors in &mut adj {
        neighbors.sort_unstable();
        neighbors.dedup();
    }

    Ok(adj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_triangle_free(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_triangle_free(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_triangle_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_triangle_free(&g).unwrap());
    }

    #[test]
    fn path_is_triangle_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_triangle_free(&g).unwrap());
    }

    #[test]
    fn cycle_4_is_triangle_free() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_triangle_free(&g).unwrap());
    }

    #[test]
    fn cycle_5_is_triangle_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_triangle_free(&g).unwrap());
    }

    #[test]
    fn complete_4_not_triangle_free() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(!is_triangle_free(&g).unwrap());
    }

    #[test]
    fn bipartite_is_triangle_free() {
        // Complete bipartite K3,3 is triangle-free
        let mut g = Graph::with_vertices(6);
        for i in 0..3u32 {
            for j in 3..6u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_triangle_free(&g).unwrap());
    }

    #[test]
    fn self_loops_ignored() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(1, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(is_triangle_free(&g).unwrap());
    }

    #[test]
    fn multi_edges_ignored() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // Multi-edges don't form a triangle
        assert!(is_triangle_free(&g).unwrap());
    }

    #[test]
    fn directed_ignores_direction() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // Treated as undirected -> triangle exists
        assert!(!is_triangle_free(&g).unwrap());
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(10);
        assert!(is_triangle_free(&g).unwrap());
    }

    #[test]
    fn star_is_triangle_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_triangle_free(&g).unwrap());
    }

    #[test]
    fn petersen_is_triangle_free() {
        // The Petersen graph is triangle-free
        let mut g = Graph::with_vertices(10);
        // Outer cycle
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        // Inner star
        g.add_edge(5, 7).unwrap();
        g.add_edge(7, 9).unwrap();
        g.add_edge(9, 6).unwrap();
        g.add_edge(6, 8).unwrap();
        g.add_edge(8, 5).unwrap();
        // Connections
        g.add_edge(0, 5).unwrap();
        g.add_edge(1, 6).unwrap();
        g.add_edge(2, 7).unwrap();
        g.add_edge(3, 8).unwrap();
        g.add_edge(4, 9).unwrap();

        assert!(is_triangle_free(&g).unwrap());
    }
}
