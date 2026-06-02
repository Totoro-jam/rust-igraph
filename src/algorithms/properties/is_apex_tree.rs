//! Apex tree predicate (ALGO-PR-128).
//!
//! A graph is an apex tree if there exists a vertex whose removal
//! makes the remaining graph a tree. This is stricter than apex
//! forest (which requires only a forest after removal).
//!
//! Every tree is trivially an apex tree (remove any leaf).
//! Every graph with exactly one cycle where the cycle passes
//! through some vertex is an apex tree.
//!
//! Directed graphs are treated as undirected.

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::algorithms::properties::is_tree::is_tree;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is an apex tree.
///
/// A graph is an apex tree if removing some single vertex yields
/// a tree. Tests each vertex as a candidate apex.
///
/// Directed graphs are treated as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_apex_tree};
///
/// // Triangle: remove any vertex → single edge (tree) ✓
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_apex_tree(&g).unwrap());
///
/// // Two disjoint triangles: no single vertex removal yields tree
/// let mut g = Graph::with_vertices(6);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 5).unwrap();
/// g.add_edge(5, 3).unwrap();
/// assert!(!is_apex_tree(&g).unwrap());
/// ```
pub fn is_apex_tree(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();

    if n <= 1 {
        return Ok(true);
    }

    if is_tree(graph, DijkstraMode::All)?.is_some() {
        return Ok(true);
    }

    let n_usize = n as usize;
    let mut adj = vec![vec![false; n_usize]; n_usize];
    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            adj[v as usize][w as usize] = true;
            adj[w as usize][v as usize] = true;
        }
    }

    for apex in 0..n_usize {
        if is_tree_without(&adj, apex, n_usize) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn is_tree_without(adj: &[Vec<bool>], removed: usize, n: usize) -> bool {
    let remaining = n - 1;
    if remaining == 0 {
        return true;
    }

    let mut visited = vec![false; n];
    visited[removed] = true;

    let Some(start) = (0..n).find(|&v| v != removed) else {
        return false;
    };
    let mut stack = vec![start];
    visited[start] = true;
    let mut visited_count = 1usize;
    let mut edge_count = 0usize;

    while let Some(u) = stack.pop() {
        for (v, &is_adj) in adj[u].iter().enumerate() {
            if v == removed || !is_adj {
                continue;
            }
            edge_count += 1;
            if !visited[v] {
                visited[v] = true;
                visited_count += 1;
                stack.push(v);
            }
        }
    }

    edge_count /= 2;

    // Tree iff connected (visited_count == remaining) and edges == remaining - 1
    visited_count == remaining && edge_count == remaining - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_apex_tree(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_apex_tree(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_apex_tree(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_apex_tree(&g).unwrap());
    }

    #[test]
    fn tree() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_apex_tree(&g).unwrap());
    }

    #[test]
    fn c4() {
        // C_4: remove any vertex → P_3 (tree) ✓
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_apex_tree(&g).unwrap());
    }

    #[test]
    fn k4() {
        // K_4: remove any vertex → K_3 (triangle, NOT a tree).
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(!is_apex_tree(&g).unwrap());
    }

    #[test]
    fn two_triangles_not_apex_tree() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        assert!(!is_apex_tree(&g).unwrap());
    }

    #[test]
    fn two_triangles_shared_vertex() {
        // 0-1-2-0 and 0-3-4-0. Remove 0 → {1,2,3,4} with edges 1-2, 3-4.
        // That's a forest, NOT a tree (two components).
        // So NOT apex tree (but IS apex forest).
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!is_apex_tree(&g).unwrap());
    }

    #[test]
    fn diamond() {
        // Diamond: 0-1,0-2,0-3,1-2,1-3 (K_4 minus 2-3).
        // Remove 0: {1,2,3} with edges 1-2,1-3 → star, which is a tree ✓
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_apex_tree(&g).unwrap());
    }

    #[test]
    fn theta_graph() {
        // Theta: 0-1-3 and 0-2-3 (two paths between 0 and 3).
        // Remove 0 → {1,2,3} with edges 1-3,2-3 → star on 3 → tree ✓
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_apex_tree(&g).unwrap());
    }

    #[test]
    fn edgeless_not_apex_tree() {
        // 4 isolated vertices: removing any one leaves 3 isolated vertices (forest, not tree)
        let g = Graph::with_vertices(4);
        assert!(!is_apex_tree(&g).unwrap());
    }

    #[test]
    fn two_isolated() {
        // 2 isolated: removing one yields single vertex (tree) ✓
        let g = Graph::with_vertices(2);
        assert!(is_apex_tree(&g).unwrap());
    }

    #[test]
    fn directed_treated_as_undirected() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_apex_tree(&g).unwrap());
    }

    #[test]
    fn wheel_w4() {
        // W_4: center 0, rim 1-2-3-4-1. Remove 0 → C_4 (not a tree).
        // Remove rim vertex 1 → 0 connected to 2,3,4; plus 2-3,3-4.
        // Edges: 0-2,0-3,0-4,2-3,3-4. 4 vertices, 5 edges. Not tree (4 verts need 3 edges).
        // NOT apex tree.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 1).unwrap();
        assert!(!is_apex_tree(&g).unwrap());
    }

    #[test]
    fn tree_with_extra_edge() {
        // Tree 0-1-2-3 plus edge 0-2. Only one cycle through 0-1-2.
        // Remove 0: {1,2,3} with edges 1-2,2-3 → path → tree ✓
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(0, 2).unwrap();
        assert!(is_apex_tree(&g).unwrap());
    }
}
