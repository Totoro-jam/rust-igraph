//! Articulation points (ALGO-CC-010).
//!
//! Counterpart of `igraph_articulation_points()` from
//! `references/igraph/src/connectivity/components.c:969-972`, which is
//! itself the
//! `igraph_biconnected_components(_, NULL, NULL, NULL, NULL, &result)`
//! reduction. We port the same iterative DFS with low-link tracking
//! (lines 1085-1209) but emit only the articulation-point vector.
//!
//! Phase-1 minimal slice — full `biconnected_components` (component
//! vertex sets, component edges, spanning-tree edges) lands in CC-011.
//!
//! An *articulation point* is a vertex whose removal increases the
//! number of (weakly) connected components. The graph is treated as
//! undirected.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Articulation points of `graph` (returns vertices in upstream's
/// DFS-discovery order).
///
/// On directed graphs the input is treated as undirected (matching
/// upstream igraph's `IGRAPH_ALL` mode default at
/// `components.c:1060`).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, articulation_points};
///
/// // Cycle 0-1-2-0 plus pendant 2-3-4: vertices 2 and 3 are articulation points.
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// let mut ap = articulation_points(&g).unwrap();
/// ap.sort_unstable();
/// assert_eq!(ap, vec![2, 3]);
/// ```
pub fn articulation_points(graph: &Graph) -> IgraphResult<Vec<VertexId>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }
    let n_us = n as usize;

    // `num[v]` = DFS pre-order index, 1-based; 0 means "unvisited".
    // `low[v]` = lowest `num` reachable from v's DFS subtree using at
    // most one back-edge. Initialised to 0 = unvisited.
    let mut num: Vec<u32> = vec![0; n_us];
    let mut low: Vec<u32> = vec![0; n_us];
    // `nextptr[v]` = next index into v's incident-edge list to visit.
    let mut nextptr: Vec<usize> = vec![0; n_us];
    // dedup ART points (a non-root vertex may be an articulation parent
    // of multiple DFS-tree children; we want one entry).
    let mut found: Vec<bool> = vec![false; n_us];
    // Active DFS path (stack of vertices).
    let mut path: Vec<VertexId> = Vec::new();
    // Output, in upstream's DFS-discovery order.
    let mut articulation: Vec<VertexId> = Vec::new();
    let mut counter: u32 = 1;

    // Pre-cache incident-edge lists once. Memory parity with upstream's
    // `igraph_inclist_init(graph, &inclist, IGRAPH_ALL, IGRAPH_LOOPS_TWICE)`.
    let mut inc: Vec<Vec<EdgeId>> = Vec::with_capacity(n_us);
    for v in 0..n {
        inc.push(graph.incident(v)?);
    }

    for i in 0..n {
        if low[i as usize] != 0 {
            continue; // already part of an earlier DFS tree
        }
        path.push(i);
        let mut rootdfs: u32 = 0;
        num[i as usize] = counter;
        low[i as usize] = counter;
        counter = counter
            .checked_add(1)
            .ok_or(IgraphError::Internal("DFS counter overflow"))?;

        while let Some(&act) = path.last() {
            let act_us = act as usize;
            let nbrs = &inc[act_us];
            let actnext = nextptr[act_us];

            if actnext < nbrs.len() {
                // Step DOWN if neighbour is unvisited; else update low via back-edge.
                let edge = nbrs[actnext];
                let nei = graph.edge_other(edge, act)?;
                if low[nei as usize] == 0 {
                    if act == i {
                        rootdfs = rootdfs.saturating_add(1);
                    }
                    path.push(nei);
                    num[nei as usize] = counter;
                    low[nei as usize] = counter;
                    counter = counter
                        .checked_add(1)
                        .ok_or(IgraphError::Internal("DFS counter overflow"))?;
                } else if num[nei as usize] < low[act_us] {
                    low[act_us] = num[nei as usize];
                }
                nextptr[act_us] += 1;
            } else {
                // Step UP — record articulation if applicable.
                path.pop();
                if let Some(&prev) = path.last() {
                    let prev_us = prev as usize;
                    if low[act_us] < low[prev_us] {
                        low[prev_us] = low[act_us];
                    }
                    if low[act_us] >= num[prev_us] && !found[prev_us] && prev != i {
                        articulation.push(prev);
                        found[prev_us] = true;
                    }
                }
            }
        }

        if rootdfs >= 2 {
            articulation.push(i);
            // No `found` dedup needed — each root is processed once.
        }
    }

    Ok(articulation)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ap_sorted(g: &Graph) -> Vec<VertexId> {
        let mut ap = articulation_points(g).unwrap();
        ap.sort_unstable();
        ap
    }

    #[test]
    fn empty_graph_has_no_articulation_points() {
        let g = Graph::with_vertices(0);
        assert!(articulation_points(&g).unwrap().is_empty());
    }

    #[test]
    fn isolated_vertices_have_no_articulation_points() {
        let g = Graph::with_vertices(5);
        assert!(articulation_points(&g).unwrap().is_empty());
    }

    #[test]
    fn cycle_has_no_articulation_points() {
        // 4-cycle: every vertex is on a cycle, none can disconnect the rest.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(articulation_points(&g).unwrap().is_empty());
    }

    #[test]
    fn path_endpoints_are_not_articulation_points() {
        // Path 0-1-2-3-4: only the internal vertices 1, 2, 3 are articulation.
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        assert_eq!(ap_sorted(&g), vec![1, 2, 3]);
    }

    #[test]
    fn star_centre_is_only_articulation_point() {
        // Star with centre 0 and 4 leaves: removing 0 disconnects all leaves.
        let mut g = Graph::with_vertices(5);
        for v in 1..5 {
            g.add_edge(0, v).unwrap();
        }
        assert_eq!(ap_sorted(&g), vec![0]);
    }

    #[test]
    fn cycle_with_pendant_has_attachment_and_internal_pendant_as_aps() {
        // Cycle 0-1-2-0 plus pendant 2-3-4 → 2 and 3 are articulation.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert_eq!(ap_sorted(&g), vec![2, 3]);
    }

    #[test]
    fn multiple_components_each_contributes_independently() {
        // Two paths 0-1-2 and 3-4-5: 1 and 4 are articulation points.
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        assert_eq!(ap_sorted(&g), vec![1, 4]);
    }

    #[test]
    fn upstream_biconnected_test_fixture_yields_5_and_2() {
        // From references/igraph/tests/unit/igraph_biconnected_components.c
        // Edges: 0-1, 1-2, 2-3, 3-0, 2-4, 4-5, 5-2, 5-6, 7-8 (vertex 9 isolated).
        // Expected articulation points (per .out): {5, 2}.
        let mut g = Graph::with_vertices(10);
        for &(u, v) in &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            (2, 4),
            (4, 5),
            (5, 2),
            (5, 6),
            (7, 8),
        ] {
            g.add_edge(u, v).unwrap();
        }
        assert_eq!(ap_sorted(&g), vec![2, 5]);
    }

    #[test]
    fn self_loop_does_not_create_articulation() {
        // Triangle plus a self-loop on 0: still no articulation points.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(articulation_points(&g).unwrap().is_empty());
    }

    #[test]
    fn parallel_edges_collapse_to_single_link_for_articulation() {
        // 0-1 with three parallel edges plus 1-2: vertex 1 is still
        // articulation (removing it disconnects 2 from 0).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(ap_sorted(&g), vec![1]);
    }

    #[test]
    fn two_triangles_sharing_a_vertex_make_that_vertex_articulation() {
        // 0-1-2-0 and 2-3-4-2: vertex 2 is the only articulation point.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 2).unwrap();
        assert_eq!(ap_sorted(&g), vec![2]);
    }
}
