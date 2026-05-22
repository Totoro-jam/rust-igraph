//! Biconnected components (ALGO-CC-011).
//!
//! Counterpart of `igraph_biconnected_components()` from
//! `references/igraph/src/connectivity/components.c:1032-1227`. Same
//! iterative DFS with low-link tracking that powers
//! [`super::articulation::articulation_points`], extended to also
//! collect the vertex set and tree edges of every biconnected
//! component, plus the count.
//!
//! Phase-1 minimal slice ships these outputs in one bundle:
//! - `count` — total number of biconnected components.
//! - `components[i]` — vertex ids of the i-th component.
//! - `tree_edges[i]` — edge ids that form a spanning tree of the i-th
//!   component (i.e. the DFS-tree edges of that component, popped off
//!   the edge stack).
//! - `component_edges[i]` — *all* edge ids inside the i-th component
//!   (not just the spanning tree). Counterpart of upstream's
//!   `component_edges` argument; see ALGO-CC-012.
//! - `articulation_points` — vertices whose removal would disconnect
//!   the graph (deduplicated, in DFS-discovery order).
//!
//! igraph's convention: isolated vertices are *not* part of any
//! biconnected component (see upstream doc at
//! `components.c:990-998`). The two-vertex graph with one edge is
//! biconnected.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Output of [`biconnected_components`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BiconnectedComponents {
    /// Total number of biconnected components.
    pub count: u32,
    /// `components[i]` lists the vertex ids of the i-th component.
    pub components: Vec<Vec<VertexId>>,
    /// `tree_edges[i]` lists the edge ids that form a spanning tree of
    /// the i-th component. Same length as `components`.
    pub tree_edges: Vec<Vec<EdgeId>>,
    /// `component_edges[i]` lists *every* edge id whose endpoints both
    /// lie in the i-th component. Same length as `components`.
    /// Counterpart of upstream's `component_edges` output (CC-012).
    pub component_edges: Vec<Vec<EdgeId>>,
    /// Articulation points of the graph (in DFS-discovery order).
    pub articulation_points: Vec<VertexId>,
}

/// Compute the biconnected components of `graph`.
///
/// Counterpart of `igraph_biconnected_components()`. The graph is
/// treated as undirected. Isolated vertices are not part of any
/// component. See [`BiconnectedComponents`] for the output shape.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, biconnected_components};
///
/// // Cycle 0-1-2-0 plus pendant 2-3-4: components are {0,1,2}, {2,3}, {3,4}.
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// let bc = biconnected_components(&g).unwrap();
/// assert_eq!(bc.count, 3);
/// let mut aps = bc.articulation_points.clone();
/// aps.sort_unstable();
/// assert_eq!(aps, vec![2, 3]);
/// // component_edges partitions every edge across all components.
/// let total: usize = bc.component_edges.iter().map(Vec::len).sum();
/// assert_eq!(total, g.ecount());
/// ```
#[allow(clippy::too_many_lines)] // Direct port of upstream's tightly-coupled DFS state.
pub fn biconnected_components(graph: &Graph) -> IgraphResult<BiconnectedComponents> {
    let n = graph.vcount();
    let mut out = BiconnectedComponents {
        count: 0,
        components: Vec::new(),
        tree_edges: Vec::new(),
        component_edges: Vec::new(),
        articulation_points: Vec::new(),
    };
    if n == 0 {
        return Ok(out);
    }
    let n_us = n as usize;

    // Per-vertex DFS state, identical to ALGO-CC-010 articulation_points.
    let mut num: Vec<u32> = vec![0; n_us];
    let mut low: Vec<u32> = vec![0; n_us];
    let mut nextptr: Vec<usize> = vec![0; n_us];
    let mut found: Vec<bool> = vec![false; n_us];
    let mut path: Vec<VertexId> = Vec::new();
    // edgestack tracks edges in DFS-tree order; on AP detection we pop
    // back to the parent's edge to extract the just-finished component.
    let mut edgestack: Vec<EdgeId> = Vec::new();
    let mut counter: u32 = 1;

    // Pre-cache incident edges (matches upstream `IGRAPH_LOOPS_TWICE`).
    let mut inc: Vec<Vec<EdgeId>> = Vec::with_capacity(n_us);
    for v in 0..n {
        inc.push(graph.incident(v)?);
    }

    // `comps_so_far` is upstream's `comps` counter — used to mark
    // `vertex_added` so the same vertex isn't double-counted within a
    // component's vertex list.
    let mut comps_so_far: u32 = 0;
    let mut vertex_added: Vec<u32> = vec![0; n_us]; // 0 = not added

    for i in 0..n {
        if low[i as usize] != 0 {
            continue;
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
                let edge = nbrs[actnext];
                let nei = graph.edge_other(edge, act)?;
                if low[nei as usize] == 0 {
                    if act == i {
                        rootdfs = rootdfs.saturating_add(1);
                    }
                    edgestack.push(edge);
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
                // Step UP.
                path.pop();
                if let Some(&prev) = path.last() {
                    let prev_us = prev as usize;
                    if low[act_us] < low[prev_us] {
                        low[prev_us] = low[act_us];
                    }
                    if low[act_us] >= num[prev_us] {
                        // Articulation point detection (skip the root).
                        if !found[prev_us] && prev != i {
                            out.articulation_points.push(prev);
                            found[prev_us] = true;
                        }
                        out.count = out
                            .count
                            .checked_add(1)
                            .ok_or(IgraphError::Internal("component count overflow"))?;
                        comps_so_far = comps_so_far
                            .checked_add(1)
                            .ok_or(IgraphError::Internal("component count overflow"))?;

                        // Pop edges off `edgestack` until we hit one
                        // touching `prev` — those edges form the just-finished
                        // biconnected component.
                        let mut comp_tree_edges: Vec<EdgeId> = Vec::new();
                        let mut comp_vertices: Vec<VertexId> = Vec::new();
                        while let Some(e) = edgestack.pop() {
                            let (from, to) = graph.edge(e)?;
                            comp_tree_edges.push(e);
                            if vertex_added[from as usize] != comps_so_far {
                                vertex_added[from as usize] = comps_so_far;
                                comp_vertices.push(from);
                            }
                            if vertex_added[to as usize] != comps_so_far {
                                vertex_added[to as usize] = comps_so_far;
                                comp_vertices.push(to);
                            }
                            if from == prev || to == prev {
                                break;
                            }
                        }
                        // After comp_vertices is built, re-scan each
                        // vertex's incidence to gather every edge whose
                        // other endpoint is also in this component. The
                        // `nei < vert` guard prevents picking each edge
                        // twice — direct port of upstream's
                        // `component_edges` block at components.c:1216.
                        let mut comp_edges: Vec<EdgeId> = Vec::new();
                        for &vert in &comp_vertices {
                            for &e in &inc[vert as usize] {
                                let nei = graph.edge_other(e, vert)?;
                                if vertex_added[nei as usize] == comps_so_far && nei < vert {
                                    comp_edges.push(e);
                                }
                            }
                        }
                        out.components.push(comp_vertices);
                        out.tree_edges.push(comp_tree_edges);
                        out.component_edges.push(comp_edges);
                    }
                }
            }
        }

        if rootdfs >= 2 {
            out.articulation_points.push(i);
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted<T: Ord + Clone>(slice: &[T]) -> Vec<T> {
        let mut v = slice.to_vec();
        v.sort_unstable();
        v
    }

    #[test]
    fn empty_graph_yields_empty() {
        let g = Graph::with_vertices(0);
        let bc = biconnected_components(&g).unwrap();
        assert_eq!(bc.count, 0);
        assert!(bc.components.is_empty());
        assert!(bc.tree_edges.is_empty());
        assert!(bc.component_edges.is_empty());
        assert!(bc.articulation_points.is_empty());
    }

    #[test]
    fn isolated_vertices_yield_zero_components() {
        let g = Graph::with_vertices(5);
        let bc = biconnected_components(&g).unwrap();
        assert_eq!(bc.count, 0);
        assert!(bc.components.is_empty());
        assert!(bc.component_edges.is_empty());
    }

    #[test]
    fn two_vertices_one_edge_is_one_component() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let bc = biconnected_components(&g).unwrap();
        assert_eq!(bc.count, 1);
        assert_eq!(sorted(&bc.components[0]), vec![0, 1]);
        assert_eq!(bc.tree_edges[0], vec![0]);
        assert_eq!(bc.component_edges[0], vec![0]);
        assert!(bc.articulation_points.is_empty());
    }

    #[test]
    fn triangle_is_single_biconnected_component() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let bc = biconnected_components(&g).unwrap();
        assert_eq!(bc.count, 1);
        assert_eq!(sorted(&bc.components[0]), vec![0, 1, 2]);
        // All 3 cycle edges must appear in component_edges (not just tree).
        assert_eq!(sorted(&bc.component_edges[0]), vec![0, 1, 2]);
        // Spanning tree of a 3-cycle has only 2 edges.
        assert_eq!(bc.tree_edges[0].len(), 2);
        assert!(bc.articulation_points.is_empty());
    }

    #[test]
    fn k4_complete_component_edges_has_all_six_edges() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let bc = biconnected_components(&g).unwrap();
        assert_eq!(bc.count, 1);
        // K4: all 6 edges, but spanning tree only has 3.
        assert_eq!(bc.component_edges[0].len(), 6);
        assert_eq!(bc.tree_edges[0].len(), 3);
        let mut all = bc.component_edges[0].clone();
        all.sort_unstable();
        assert_eq!(all, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn pendant_component_edges_match_tree() {
        // Bridge edge alone: tree_edges == component_edges (single edge).
        // 0-1-2-0 cycle + 2-3 bridge. The 2-3 component has 1 edge total.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(1, 2).unwrap(); // edge 1
        g.add_edge(2, 0).unwrap(); // edge 2
        g.add_edge(2, 3).unwrap(); // edge 3
        let bc = biconnected_components(&g).unwrap();
        assert_eq!(bc.count, 2);
        // Find the bridge component (size 2 vertices, 1 edge).
        let bridge_idx = bc
            .components
            .iter()
            .position(|c| c.len() == 2)
            .expect("bridge component");
        assert_eq!(bc.tree_edges[bridge_idx], bc.component_edges[bridge_idx]);
        // Cycle component has 3 edges in component_edges, 2 in tree_edges.
        let cycle_idx = 1 - bridge_idx;
        assert_eq!(bc.component_edges[cycle_idx].len(), 3);
        assert_eq!(bc.tree_edges[cycle_idx].len(), 2);
    }

    #[test]
    fn component_edges_partition_non_bridge_edges() {
        // Every non-bridge edge belongs to exactly one biconnected
        // component; bridge edges are themselves single-edge components.
        // Combined, component_edges across all components partition E.
        let mut g = Graph::with_vertices(7);
        for &(u, v) in &[
            (0u32, 1),
            (1, 2),
            (2, 0),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 3),
            (0, 6),
        ] {
            g.add_edge(u, v).unwrap();
        }
        let bc = biconnected_components(&g).unwrap();
        let total: usize = bc.component_edges.iter().map(Vec::len).sum();
        // 8 edges, all should be partitioned (no edge in two components).
        assert_eq!(total, g.ecount());
        // No duplicate edge ids across components.
        let mut all: Vec<_> = bc.component_edges.iter().flatten().copied().collect();
        all.sort_unstable();
        let mut deduped = all.clone();
        deduped.dedup();
        assert_eq!(all, deduped);
    }

    #[test]
    fn cycle_with_pendant_three_components() {
        // 0-1-2-0 cycle + 2-3-4 pendant: components are {0,1,2}, {2,3}, {3,4}.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let bc = biconnected_components(&g).unwrap();
        assert_eq!(bc.count, 3);
        let mut aps = bc.articulation_points.clone();
        aps.sort_unstable();
        assert_eq!(aps, vec![2, 3]);
        // Each component has the right size; sum equals 7
        // (cycle contributes 3 vertices, each pendant edge contributes 2,
        // shared vertex 2 counted twice, shared vertex 3 counted twice).
        let total_vertices: usize = bc.components.iter().map(Vec::len).sum();
        assert_eq!(total_vertices, 7);
    }

    #[test]
    fn star_4_three_components() {
        // 0-1, 0-2, 0-3: each edge is its own biconnected component.
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        let bc = biconnected_components(&g).unwrap();
        assert_eq!(bc.count, 3);
        assert_eq!(bc.articulation_points, vec![0]);
        for comp in &bc.components {
            assert_eq!(comp.len(), 2);
        }
    }

    #[test]
    fn upstream_test_fixture_produces_4_components() {
        // From references/igraph/tests/unit/igraph_biconnected_components.c
        // 10v graph (vertex 9 isolated), 9 edges:
        //   0-1, 1-2, 2-3, 3-0, 2-4, 4-5, 5-2, 5-6, 7-8.
        // Expected per upstream .out: 4 components, articulation points {2, 5}.
        let mut g = Graph::with_vertices(10);
        for &(u, v) in &[
            (0u32, 1),
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
        let bc = biconnected_components(&g).unwrap();
        assert_eq!(bc.count, 4);
        let mut aps = bc.articulation_points.clone();
        aps.sort_unstable();
        assert_eq!(aps, vec![2, 5]);
    }

    #[test]
    fn k4_complete_one_component() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let bc = biconnected_components(&g).unwrap();
        assert_eq!(bc.count, 1);
        assert_eq!(sorted(&bc.components[0]), vec![0, 1, 2, 3]);
        assert!(bc.articulation_points.is_empty());
    }

    #[test]
    fn aps_match_articulation_points_function() {
        // Cross-check: the AP set from this function matches CC-010.
        let mut g = Graph::with_vertices(7);
        for &(u, v) in &[
            (0u32, 1),
            (1, 2),
            (2, 0),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 3),
            (0, 6),
        ] {
            g.add_edge(u, v).unwrap();
        }
        let bc = biconnected_components(&g).unwrap();
        let mut bc_aps = bc.articulation_points.clone();
        bc_aps.sort_unstable();
        let mut ap = super::super::articulation::articulation_points(&g).unwrap();
        ap.sort_unstable();
        assert_eq!(bc_aps, ap);
    }
}
