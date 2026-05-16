//! Bridges (ALGO-CC-014).
//!
//! Counterpart of `igraph_bridges()` from
//! `references/igraph/src/connectivity/components.c:1400-1504`.
//!
//! A *bridge* is an edge whose removal increases the number of (weakly)
//! connected components in the graph. The algorithm is Tarjan-style:
//! one iterative DFS per weak component, tracking each vertex's
//! discovery time `vis[v]` and low-link `low[v]` (the lowest discovery
//! time reachable from `v`'s DFS subtree). When a child's `low > vis[parent]`
//! the parent-child edge is a bridge.
//!
//! To support multigraphs we track the *incoming edge id* for each
//! vertex (rather than the parent vertex id), so a parallel edge back
//! to the parent correctly counts as a back-edge. This mirrors
//! upstream's note at `components.c:1402-1406`.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Bridges of `graph` — edges whose removal would increase the number
/// of (weakly) connected components.
///
/// Counterpart of `igraph_bridges()`. On directed graphs the input is
/// treated as undirected (matches upstream's `IGRAPH_ALL` mode default
/// at `components.c:1417`). Returns edge ids.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bridges};
///
/// // Two triangles sharing a single bridge edge: 0-1-2-0, 3-4-5-3, 2-3.
/// // Only edge id 6 (the 2-3 link) is a bridge.
/// let mut g = Graph::with_vertices(6);
/// g.add_edge(0, 1).unwrap();   // edge 0
/// g.add_edge(1, 2).unwrap();   // edge 1
/// g.add_edge(2, 0).unwrap();   // edge 2
/// g.add_edge(3, 4).unwrap();   // edge 3
/// g.add_edge(4, 5).unwrap();   // edge 4
/// g.add_edge(5, 3).unwrap();   // edge 5
/// g.add_edge(2, 3).unwrap();   // edge 6 — bridge
///
/// assert_eq!(bridges(&g).unwrap(), vec![6]);
/// ```
pub fn bridges(graph: &Graph) -> IgraphResult<Vec<EdgeId>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }
    let n_us = n as usize;

    let mut visited: Vec<bool> = vec![false; n_us];
    let mut vis: Vec<u32> = vec![0; n_us];
    let mut low: Vec<u32> = vec![0; n_us];
    // `incoming_edge[v] = Some(eid)` if v was reached via that edge in
    // the DFS tree; `None` for roots (and unvisited). Using Option matches
    // upstream's `incoming_edge[v] = -1` sentinel.
    let mut incoming_edge: Vec<Option<EdgeId>> = vec![None; n_us];

    // The DFS state is two parallel stacks: `su` (vertex on top of the
    // "active" stack), `si` (the index into u's incident-edge list to
    // process next). Mirrors upstream `components.c:1428-1490`.
    let mut su: Vec<VertexId> = Vec::new();
    let mut si: Vec<usize> = Vec::new();
    let mut time: u32 = 0;

    // Pre-cache incident edges (parity with upstream's
    // `igraph_inclist_init(_, _, IGRAPH_ALL, IGRAPH_LOOPS_TWICE)`).
    let mut inc: Vec<Vec<EdgeId>> = Vec::with_capacity(n_us);
    for v in 0..n {
        inc.push(graph.incident(v)?);
    }

    let mut bridges_out: Vec<EdgeId> = Vec::new();

    for start in 0..n {
        if visited[start as usize] {
            continue;
        }
        su.push(start);
        si.push(0);

        while let Some(u) = su.pop() {
            let i = si.pop().expect("si parallel to su");
            let u_us = u as usize;

            if i == 0 {
                visited[u_us] = true;
                time = time
                    .checked_add(1)
                    .ok_or(IgraphError::Internal("DFS time counter overflow"))?;
                vis[u_us] = time;
                low[u_us] = time;
            }

            let edges = &inc[u_us];
            if i < edges.len() {
                // Re-push (u, i+1) so we resume after handling this edge,
                // then push (v, 0) to descend into the neighbour.
                su.push(u);
                si.push(i + 1);

                let edge = edges[i];
                let v = graph.edge_other(edge, u)?;

                if !visited[v as usize] {
                    incoming_edge[v as usize] = Some(edge);
                    su.push(v);
                    si.push(0);
                } else if Some(edge) != incoming_edge[u_us] {
                    // Back-edge (not the edge we entered u through).
                    // Update u's low-link to the discovery time of v.
                    if vis[v as usize] < low[u_us] {
                        low[u_us] = vis[v as usize];
                    }
                }
            } else {
                // Done with u — propagate low-link up and check bridge condition.
                if let Some(edge) = incoming_edge[u_us] {
                    let w = graph.edge_other(edge, u)?; // parent of u in DFS tree
                    let w_us = w as usize;
                    if low[u_us] < low[w_us] {
                        low[w_us] = low[u_us];
                    }
                    if low[u_us] > vis[w_us] {
                        bridges_out.push(edge);
                    }
                }
            }
        }
    }

    Ok(bridges_out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brsorted(g: &Graph) -> Vec<EdgeId> {
        let mut b = bridges(g).unwrap();
        b.sort_unstable();
        b
    }

    #[test]
    fn empty_graph_has_no_bridges() {
        let g = Graph::with_vertices(0);
        assert!(bridges(&g).unwrap().is_empty());
    }

    #[test]
    fn isolated_vertices_have_no_bridges() {
        let g = Graph::with_vertices(5);
        assert!(bridges(&g).unwrap().is_empty());
    }

    #[test]
    fn cycle_has_no_bridges() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(bridges(&g).unwrap().is_empty());
    }

    #[test]
    fn path_every_edge_is_a_bridge() {
        // Path 0-1-2-3-4: every one of the 4 edges is a bridge.
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        assert_eq!(brsorted(&g), vec![0, 1, 2, 3]);
    }

    #[test]
    fn cycle_with_pendant_bridges_are_pendant_edges() {
        // 0-1-2-0 (cycle) + 2-3-4 (pendant): edge 3 (2-3) and edge 4 (3-4) are bridges.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 0).unwrap(); // 2
        g.add_edge(2, 3).unwrap(); // 3
        g.add_edge(3, 4).unwrap(); // 4
        assert_eq!(brsorted(&g), vec![3, 4]);
    }

    #[test]
    fn parallel_edges_make_neither_a_bridge() {
        // Three parallel edges 0-1: no single edge is a bridge.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(bridges(&g).unwrap().is_empty());
    }

    #[test]
    fn self_loop_is_not_a_bridge() {
        // 0-self + 0-1: only edge 1 (0-1) is a bridge. Self-loop ignored.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap(); // 0
        g.add_edge(0, 1).unwrap(); // 1
        assert_eq!(brsorted(&g), vec![1]);
    }

    #[test]
    fn two_components_each_contributes_independently() {
        // Path 0-1-2 and Path 3-4-5: every edge is a bridge.
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        assert_eq!(brsorted(&g), vec![0, 1, 2, 3]);
    }

    #[test]
    fn two_triangles_joined_by_bridge() {
        // Triangle {0,1,2} + triangle {3,4,5} + edge 2-3 (the bridge).
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        g.add_edge(2, 3).unwrap(); // edge 6 — bridge
        assert_eq!(brsorted(&g), vec![6]);
    }

    #[test]
    fn star_every_edge_is_a_bridge() {
        // Centre 0, leaves 1..4: 4 bridges.
        let mut g = Graph::with_vertices(5);
        for v in 1..5 {
            g.add_edge(0, v).unwrap();
        }
        assert_eq!(brsorted(&g), vec![0, 1, 2, 3]);
    }
}
