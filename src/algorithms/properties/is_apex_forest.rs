//! Apex forest predicate (ALGO-PR-120).
//!
//! A graph is an apex forest if there exists a vertex whose removal
//! makes the remaining graph a forest (acyclic). Equivalently, all
//! cycles in the graph pass through a common vertex.
//!
//! Every forest is trivially an apex forest (remove any vertex).
//! Every graph with a single cycle passing through one vertex is
//! an apex forest.
//!
//! Directed graphs are treated as undirected.

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::algorithms::properties::is_forest::is_forest;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is an apex forest.
///
/// A graph is an apex forest if removing some single vertex yields a
/// forest. Tests each vertex as a candidate apex.
///
/// Directed graphs are treated as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_apex_forest};
///
/// // Triangle: remove any vertex → single edge (forest) ✓
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_apex_forest(&g).unwrap());
///
/// // Two disjoint triangles: no single vertex removal yields forest
/// let mut g = Graph::with_vertices(6);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 5).unwrap();
/// g.add_edge(5, 3).unwrap();
/// assert!(!is_apex_forest(&g).unwrap());
/// ```
pub fn is_apex_forest(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();

    if n == 0 {
        return Ok(true);
    }

    if is_forest(graph, DijkstraMode::All)?.is_some() {
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
        if is_forest_without(&adj, apex, n_usize) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Check if the graph minus vertex `removed` is a forest.
fn is_forest_without(adj: &[Vec<bool>], removed: usize, n: usize) -> bool {
    let mut visited = vec![false; n];
    visited[removed] = true;

    let remaining = n - 1;
    if remaining == 0 {
        return true;
    }

    let mut edge_count = 0usize;
    let mut component_count = 0usize;
    let mut visited_count = 0usize;

    for start in 0..n {
        if visited[start] {
            continue;
        }
        component_count += 1;
        let mut stack = vec![start];
        visited[start] = true;

        while let Some(u) = stack.pop() {
            visited_count += 1;
            for (v, &is_adj) in adj[u].iter().enumerate() {
                if v == removed || !is_adj {
                    continue;
                }
                edge_count += 1;
                if !visited[v] {
                    visited[v] = true;
                    stack.push(v);
                }
            }
        }
    }

    // Each edge counted twice (u→v and v→u)
    edge_count /= 2;

    // Forest iff edges == vertices - components
    edge_count == visited_count - component_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_apex_forest(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_apex_forest(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_apex_forest(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_apex_forest(&g).unwrap());
    }

    #[test]
    fn tree() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_apex_forest(&g).unwrap());
    }

    #[test]
    fn c4() {
        // C_4: remove any vertex → P_3 (forest) ✓
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_apex_forest(&g).unwrap());
    }

    #[test]
    fn k4() {
        // K_4: remove any vertex → K_3 (triangle, NOT a forest).
        // So K_4 is NOT an apex forest.
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(!is_apex_forest(&g).unwrap());
    }

    #[test]
    fn two_triangles_not_apex() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        assert!(!is_apex_forest(&g).unwrap());
    }

    #[test]
    fn wheel_w4() {
        // W_4: center 0, rim 1-2-3-4-1.
        // Remove center → C_4 (has cycle) → not forest.
        // Remove rim vertex 1 → 0 connected to 2,3,4; plus 2-3,3-4.
        // Edges: 0-2,0-3,0-4,2-3,3-4. Has cycle 0-2-3-0 → not forest.
        // K_4 subgraph minus one edge... still has cycle.
        // Actually let's check: remove 1 from W_4.
        // Remaining: 0,2,3,4 with edges 0-2,0-3,0-4,2-3,3-4.
        // 4 vertices, 5 edges. Forest needs 3 edges. NOT forest.
        // W_4 is NOT apex forest.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 1).unwrap();
        assert!(!is_apex_forest(&g).unwrap());
    }

    #[test]
    fn diamond_apex() {
        // Diamond: 0-1,0-2,0-3,1-2,1-3 (K_4 minus 2-3).
        // Remove 0: remaining 1,2,3 with edges 1-2,1-3 → star → forest ✓.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_apex_forest(&g).unwrap());
    }

    #[test]
    fn theta_graph_apex() {
        // Theta: two paths between 0 and 3: 0-1-3 and 0-2-3.
        // Remove 0 → 1,2,3 with edges 1-3,2-3 → forest ✓.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_apex_forest(&g).unwrap());
    }

    #[test]
    fn edgeless() {
        let g = Graph::with_vertices(4);
        assert!(is_apex_forest(&g).unwrap());
    }

    #[test]
    fn directed_treated_as_undirected() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_apex_forest(&g).unwrap());
    }

    #[test]
    fn two_triangles_shared_vertex() {
        // 0-1-2-0 and 0-3-4-0. All cycles through 0.
        // Remove 0 → 1-2, 3-4 (two edges, forest) ✓.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_apex_forest(&g).unwrap());
    }
}
