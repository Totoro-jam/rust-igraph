//! Geodetic graph predicate (ALGO-PR-078).
//!
//! A geodetic graph has a unique shortest path between every pair of
//! reachable vertices. Trees are geodetic. Complete graphs are
//! geodetic. Block graphs are geodetic.
//!
//! Recognition: for each vertex, run BFS and track the number of
//! shortest-path predecessors. If any vertex is reached via more than
//! one shortest path, the graph is not geodetic. O(V*(V+E)).
//!
//! For directed graphs, shortest paths follow edge direction.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is geodetic.
///
/// A geodetic graph has exactly one shortest path between every pair
/// of connected vertices.
///
/// For directed graphs, paths follow edge direction; unreachable
/// pairs are ignored.
///
/// An empty graph (0 vertices) is geodetic (vacuously).
/// A single vertex is geodetic.
/// Any tree is geodetic.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_geodetic};
///
/// // Tree (path): geodetic
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_geodetic(&g).unwrap());
///
/// // C4 is NOT geodetic (two shortest paths between opposite vertices)
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// assert!(!is_geodetic(&g).unwrap());
/// ```
pub fn is_geodetic(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    if n <= 2 {
        return Ok(true);
    }

    let n_usize = n as usize;
    let mut dist = vec![u32::MAX; n_usize];
    let mut nrgeo = vec![0u32; n_usize];
    let mut queue: VecDeque<u32> = VecDeque::new();

    for source in 0..n {
        dist.fill(u32::MAX);
        nrgeo.fill(0);

        dist[source as usize] = 0;
        nrgeo[source as usize] = 1;
        queue.clear();
        queue.push_back(source);

        while let Some(v) = queue.pop_front() {
            let next_dist = dist[v as usize].saturating_add(1);
            let neighbors = graph.neighbors(v)?;

            for w in neighbors {
                let w_idx = w as usize;
                if dist[w_idx] == u32::MAX {
                    dist[w_idx] = next_dist;
                    nrgeo[w_idx] = 1;
                    queue.push_back(w);
                } else if dist[w_idx] == next_dist {
                    nrgeo[w_idx] = nrgeo[w_idx].saturating_add(1);
                    if nrgeo[w_idx] > 1 {
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
        assert!(is_geodetic(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_geodetic(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_geodetic(&g).unwrap());
    }

    #[test]
    fn path_is_geodetic() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_geodetic(&g).unwrap());
    }

    #[test]
    fn tree_is_geodetic() {
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        g.add_edge(2, 6).unwrap();
        assert!(is_geodetic(&g).unwrap());
    }

    #[test]
    fn triangle_is_geodetic() {
        // K3: unique shortest path between every pair (direct edge)
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_geodetic(&g).unwrap());
    }

    #[test]
    fn k4_is_geodetic() {
        // Complete graph: every pair connected by direct edge
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(is_geodetic(&g).unwrap());
    }

    #[test]
    fn cycle_4_not_geodetic() {
        // C4: vertices 0 and 2 have two shortest paths (0-1-2 and 0-3-2)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_geodetic(&g).unwrap());
    }

    #[test]
    fn cycle_5_is_geodetic() {
        // C5: all pairs at distance 2 have unique shortest paths
        // (odd cycles are geodetic)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_geodetic(&g).unwrap());
    }

    #[test]
    fn cycle_6_not_geodetic() {
        // C6: opposite vertices (e.g., 0 and 3) have two shortest paths
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 0).unwrap();
        assert!(!is_geodetic(&g).unwrap());
    }

    #[test]
    fn cycle_3_is_geodetic() {
        // C3 = K3 → geodetic
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_geodetic(&g).unwrap());
    }

    #[test]
    fn star_is_geodetic() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_geodetic(&g).unwrap());
    }

    #[test]
    fn two_triangles_sharing_vertex_is_geodetic() {
        // Block graph: two K3 sharing vertex 2
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 2).unwrap();
        assert!(is_geodetic(&g).unwrap());
    }

    #[test]
    fn disconnected_trees_geodetic() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        assert!(is_geodetic(&g).unwrap());
    }

    #[test]
    fn edgeless_is_geodetic() {
        let g = Graph::with_vertices(5);
        assert!(is_geodetic(&g).unwrap());
    }

    #[test]
    fn directed_path_geodetic() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_geodetic(&g).unwrap());
    }

    #[test]
    fn directed_diamond_not_geodetic() {
        // 0→1, 0→2, 1→3, 2→3: two shortest paths 0→3
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_geodetic(&g).unwrap());
    }

    #[test]
    fn k4_minus_edge_not_geodetic() {
        // K4 minus edge (2,3): 0 and 1 each connect to 2 and 3
        // Path 2→3: can go 2-0-3 and 2-1-3 → two shortest paths
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(!is_geodetic(&g).unwrap());
    }
}
