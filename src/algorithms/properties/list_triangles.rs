//! Triangle listing (ALGO-PR-040).
//!
//! Counterpart of `igraph_list_triangles()` from
//! `references/igraph/src/properties/triangles.c`.
//!
//! Enumerates all triangles in a graph, reporting each exactly once as
//! a triplet of vertex IDs.

use crate::core::{Graph, IgraphResult};

/// List all triangles in a graph.
///
/// Returns a vector of `(u, v, w)` triples where each triple represents
/// a triangle. Each triangle is listed exactly once. Edge directions
/// and multi-edges are ignored.
///
/// Uses degree-ordering to ensure each triangle is counted once: for
/// each edge `(u, v)` where `rank(u) > rank(v)`, checks neighbors of
/// `v` that are also neighbors of `u`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, list_triangles};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
///
/// let tris = list_triangles(&g).unwrap();
/// assert_eq!(tris.len(), 2); // triangles: (0,1,2) and (0,2,3)
/// ```
pub fn list_triangles(graph: &Graph) -> IgraphResult<Vec<(u32, u32, u32)>> {
    let n = graph.vcount() as usize;

    if n < 3 {
        return Ok(Vec::new());
    }

    // Build simple undirected adjacency list.
    let adj = build_simple_adj(graph)?;

    // Compute degree and rank (higher degree = higher rank).
    let mut degree_order: Vec<usize> = (0..n).collect();
    degree_order.sort_by(|&a, &b| adj[a].len().cmp(&adj[b].len()));

    let mut rank: Vec<usize> = vec![0; n];
    for (i, &node) in degree_order.iter().enumerate() {
        rank[node] = i;
    }

    // Filter adjacency list: keep only neighbors with higher rank.
    let filtered: Vec<Vec<usize>> = (0..n)
        .map(|v| {
            adj[v]
                .iter()
                .copied()
                .filter(|&w| rank[w] > rank[v])
                .collect()
        })
        .collect();

    // Mark array for neighbor lookup.
    let mut mark: Vec<usize> = vec![usize::MAX; n];
    let mut triangles: Vec<(u32, u32, u32)> = Vec::new();

    for node_idx in (0..n).rev() {
        let node = degree_order[node_idx];

        // Mark neighbors of node.
        for &nei in &filtered[node] {
            mark[nei] = node;
        }

        // For each neighbor, check its filtered neighbors.
        for &nei in &filtered[node] {
            for &nei2 in &filtered[nei] {
                if mark[nei2] == node {
                    #[allow(clippy::cast_possible_truncation)]
                    triangles.push((node as u32, nei as u32, nei2 as u32));
                }
            }
        }
    }

    Ok(triangles)
}

/// Build simple undirected adjacency list (sorted, no loops, no multi-edges).
fn build_simple_adj(graph: &Graph) -> IgraphResult<Vec<Vec<usize>>> {
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
    use std::collections::HashSet;

    fn normalize_triangle(t: (u32, u32, u32)) -> Vec<u32> {
        let mut v = vec![t.0, t.1, t.2];
        v.sort_unstable();
        v
    }

    fn triangle_set(tris: &[(u32, u32, u32)]) -> HashSet<Vec<u32>> {
        tris.iter().map(|&t| normalize_triangle(t)).collect()
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let tris = list_triangles(&g).unwrap();
        assert!(tris.is_empty());
    }

    #[test]
    fn no_triangles() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        // C4 has no triangles
        let tris = list_triangles(&g).unwrap();
        assert!(tris.is_empty());
    }

    #[test]
    fn single_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();

        let tris = list_triangles(&g).unwrap();
        assert_eq!(tris.len(), 1);
        let t = normalize_triangle(tris[0]);
        assert_eq!(t, vec![0, 1, 2]);
    }

    #[test]
    fn complete_4() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        // K4 has C(4,3) = 4 triangles
        let tris = list_triangles(&g).unwrap();
        assert_eq!(tris.len(), 4);
        let set = triangle_set(&tris);
        assert!(set.contains(&vec![0, 1, 2]));
        assert!(set.contains(&vec![0, 1, 3]));
        assert!(set.contains(&vec![0, 2, 3]));
        assert!(set.contains(&vec![1, 2, 3]));
    }

    #[test]
    fn two_triangles_sharing_edge() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();

        let tris = list_triangles(&g).unwrap();
        assert_eq!(tris.len(), 2);
        let set = triangle_set(&tris);
        assert!(set.contains(&vec![0, 1, 2]));
        assert!(set.contains(&vec![0, 2, 3]));
    }

    #[test]
    fn disconnected_triangles() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();

        let tris = list_triangles(&g).unwrap();
        assert_eq!(tris.len(), 2);
        let set = triangle_set(&tris);
        assert!(set.contains(&vec![0, 1, 2]));
        assert!(set.contains(&vec![3, 4, 5]));
    }

    #[test]
    fn directed_ignores_direction() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();

        let tris = list_triangles(&g).unwrap();
        assert_eq!(tris.len(), 1);
    }

    #[test]
    fn self_loops_ignored() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // No triangle
        let tris = list_triangles(&g).unwrap();
        assert!(tris.is_empty());
    }

    #[test]
    fn multi_edges_ignored() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();

        let tris = list_triangles(&g).unwrap();
        assert_eq!(tris.len(), 1);
    }

    #[test]
    fn uniqueness() {
        // Each triangle appears exactly once
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        // K5 has C(5,3) = 10 triangles
        let tris = list_triangles(&g).unwrap();
        assert_eq!(tris.len(), 10);
        let set = triangle_set(&tris);
        assert_eq!(set.len(), 10);
    }

    #[test]
    fn star_no_triangles() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();

        let tris = list_triangles(&g).unwrap();
        assert!(tris.is_empty());
    }

    #[test]
    fn two_vertices() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let tris = list_triangles(&g).unwrap();
        assert!(tris.is_empty());
    }
}
