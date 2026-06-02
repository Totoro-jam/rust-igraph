//! Series-parallel graph predicate (ALGO-PR-127).
//!
//! A graph is series-parallel if it can be reduced to a single edge
//! (or the empty graph) by repeated application of two operations:
//!
//! 1. **Series reduction**: remove a degree-2 vertex and merge its
//!    two incident edges into one.
//! 2. **Parallel reduction**: collapse a pair of parallel edges into
//!    a single edge.
//!
//! Equivalently, a graph is series-parallel if and only if it has no
//! `K_4` minor (Duffin 1965).
//!
//! The algorithm works by iteratively peeling degree-0 and degree-1
//! vertices (trivially removable), then series-reducing degree-2
//! vertices, and collapsing parallel edges, until no more reductions
//! apply. If the graph reduces to at most one edge, it is SP.
//!
//! Directed graphs are treated as undirected.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is series-parallel.
///
/// A graph is series-parallel if it has no `K_4` minor. The test
/// works by iterative series and parallel reductions.
///
/// Directed graphs are treated as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_series_parallel};
///
/// // Path: trivially series-parallel
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_series_parallel(&g).unwrap());
///
/// // K_4: NOT series-parallel (has K_4 minor)
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 {
///     for j in (i+1)..4 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// assert!(!is_series_parallel(&g).unwrap());
/// ```
pub fn is_series_parallel(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(true);
    }

    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    for v in 0..graph.vcount() {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            let ww = w as usize;
            let vv = v as usize;
            if vv < ww {
                adj[vv].push(ww);
                adj[ww].push(vv);
            }
        }
    }

    Ok(sp_reduce(&mut adj, n))
}

fn sp_reduce(adj: &mut [Vec<usize>], n: usize) -> bool {
    let mut alive = vec![true; n];
    let mut changed = true;

    while changed {
        changed = false;

        // Parallel reduction: for each alive vertex, collapse duplicate edges
        for v in 0..n {
            if !alive[v] {
                continue;
            }
            adj[v].sort_unstable();
            let before = adj[v].len();
            adj[v].dedup();
            if adj[v].len() < before {
                changed = true;
                // Also dedup the reverse direction for neighbors
                let nbrs: Vec<usize> = adj[v].clone();
                for w in nbrs {
                    adj[w].sort_unstable();
                    adj[w].dedup();
                }
            }
        }

        for v in 0..n {
            if !alive[v] {
                continue;
            }

            let deg = adj[v].len();

            if deg == 0 {
                alive[v] = false;
                changed = true;
                continue;
            }

            if deg == 1 {
                let w = adj[v][0];
                adj[v].clear();
                adj[w].retain(|&x| x != v);
                alive[v] = false;
                changed = true;
                continue;
            }

            if deg == 2 && adj[v][0] != adj[v][1] {
                let a = adj[v][0];
                let b = adj[v][1];
                adj[v].clear();
                alive[v] = false;

                adj[a].retain(|&x| x != v);
                adj[b].retain(|&x| x != v);
                adj[a].push(b);
                adj[b].push(a);
                changed = true;
            }
        }
    }

    let remaining_edges: usize = adj.iter().map(Vec::len).sum::<usize>() / 2;
    let remaining_verts = alive.iter().filter(|&&a| a).count();

    remaining_verts <= 2 && remaining_edges <= 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn path() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn cycle_c4() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn cycle_c5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn k4_not_sp() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(!is_series_parallel(&g).unwrap());
    }

    #[test]
    fn k5_not_sp() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(!is_series_parallel(&g).unwrap());
    }

    #[test]
    fn diamond_sp() {
        // Diamond: K_4 minus one edge. 4 vertices, 5 edges.
        // Series-parallel because no K_4 minor.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        // Missing edge: 2-3. Diamond has K_4 minus one edge.
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn wheatstone_bridge_sp() {
        // Wheatstone bridge: K_4 minus one edge with subdivided edges
        // This is a classic SP graph
        // 0-1, 0-2, 1-3, 2-3, 1-2 (5 edges, 4 vertices)
        // Same as diamond — SP
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn tree_sp() {
        // Any tree is SP
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(3, 5).unwrap();
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn edgeless() {
        let g = Graph::with_vertices(5);
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn star() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn petersen_not_sp() {
        // Petersen graph contains K_4 minor → not SP
        let mut g = Graph::with_vertices(10);
        let edges = [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 0),
            (5, 7),
            (7, 9),
            (9, 6),
            (6, 8),
            (8, 5),
            (0, 5),
            (1, 6),
            (2, 7),
            (3, 8),
            (4, 9),
        ];
        for (u, v) in edges {
            g.add_edge(u, v).unwrap();
        }
        assert!(!is_series_parallel(&g).unwrap());
    }

    #[test]
    fn k23_sp() {
        // K_{2,3}: 2 + 3 vertices, 6 edges. No K_4 minor (bipartite with
        // max partition size 3, but K_4 has min degree 3 which a side-2
        // bipartite graph can't support). SP.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn two_triangles_shared_edge() {
        // 0-1-2-0 and 1-2-3-1 share edge 1-2.
        // 4 vertices, 5 edges. Diamond shape. SP.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn directed_treated_as_undirected() {
        let mut g = Graph::new(4, true).unwrap();
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        // K_4 directed → treated as undirected → not SP
        assert!(!is_series_parallel(&g).unwrap());
    }

    #[test]
    fn disconnected_sp() {
        // Two separate triangles: each is SP, and SP is closed under disjoint union
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        assert!(is_series_parallel(&g).unwrap());
    }

    #[test]
    fn wheel_w4_not_sp() {
        // W_4: center 0, rim 1-2-3-4-1. Contains K_4 minor (contract rim edge).
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 1).unwrap();
        assert!(!is_series_parallel(&g).unwrap());
    }
}
