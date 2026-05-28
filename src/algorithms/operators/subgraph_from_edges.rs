//! Subgraph from edges (ALGO-OP-024).
//!
//! Counterpart of `igraph_subgraph_from_edges()` from
//! `references/igraph/src/operators/subgraph.c`.
//!
//! Creates a subgraph containing only the specified edges (and optionally
//! only the vertices incident to those edges).

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Result of a subgraph-from-edges extraction.
#[derive(Debug, Clone)]
pub struct SubgraphFromEdgesResult {
    /// The resulting subgraph.
    pub graph: Graph,
    /// Mapping from old vertex IDs to new vertex IDs. Contains `u32::MAX`
    /// for vertices not in the subgraph (only meaningful when
    /// `delete_vertices` was `true`).
    pub vertex_map: Vec<u32>,
    /// Mapping from new vertex IDs to old vertex IDs.
    pub vertex_invmap: Vec<VertexId>,
    /// Mapping from new edge indices (positions in the result) to old edge IDs.
    pub edge_map: Vec<u32>,
}

/// Create a subgraph containing only the specified edges.
///
/// Given a set of edge IDs, builds a new graph that keeps exactly those
/// edges. When `delete_vertices` is `true`, vertices that have no
/// incident edges in the selection are removed and IDs are renumbered.
/// When `false`, all original vertices are kept.
///
/// Duplicate edge IDs are silently ignored. Edge ordering in the result
/// follows the ascending order of original edge IDs.
///
/// # Arguments
///
/// * `graph` — the input graph.
/// * `eids` — slice of edge IDs to retain.
/// * `delete_vertices` — if `true`, remove isolated vertices.
///
/// # Errors
///
/// Returns `InvalidArgument` if any edge ID in `eids` is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, subgraph_from_edges};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap(); // eid 0
/// g.add_edge(1, 2).unwrap(); // eid 1
/// g.add_edge(2, 3).unwrap(); // eid 2
/// g.add_edge(3, 4).unwrap(); // eid 3
///
/// // Keep only edges 0 and 2: (0,1) and (2,3)
/// let result = subgraph_from_edges(&g, &[0, 2], true).unwrap();
/// assert_eq!(result.graph.ecount(), 2);
/// assert_eq!(result.graph.vcount(), 4); // vertices 0,1,2,3 (4 removed)
/// ```
pub fn subgraph_from_edges(
    graph: &Graph,
    eids: &[u32],
    delete_vertices: bool,
) -> IgraphResult<SubgraphFromEdgesResult> {
    let n = graph.vcount();
    let m = graph.ecount();

    // Validate edge IDs and collect them into a sorted, deduplicated set.
    let mut edge_set: Vec<bool> = vec![false; m];
    for &eid in eids {
        if eid as usize >= m {
            return Err(IgraphError::InvalidArgument(format!(
                "subgraph_from_edges: edge ID {eid} out of range (graph has {m} edges)"
            )));
        }
        edge_set[eid as usize] = true;
    }

    // Determine which vertices remain.
    let mut vertex_remain: Vec<bool> = if delete_vertices {
        vec![false; n as usize]
    } else {
        vec![true; n as usize]
    };

    // Collect retained edges (sorted by original ID) and mark incident vertices.
    let mut retained_edges: Vec<(u32, u32, u32)> = Vec::new(); // (from, to, old_eid)
    for (eid_usize, &keep) in edge_set.iter().enumerate() {
        if keep {
            #[allow(clippy::cast_possible_truncation)]
            let eid = eid_usize as u32;
            let (from, to) = graph.edge(eid)?;
            retained_edges.push((from, to, eid));
            if delete_vertices {
                vertex_remain[from as usize] = true;
                vertex_remain[to as usize] = true;
            }
        }
    }

    // Build vertex mapping.
    let mut vertex_map: Vec<u32> = vec![u32::MAX; n as usize];
    let mut vertex_invmap: Vec<VertexId> = Vec::new();
    let mut new_vid: u32 = 0;
    for vid in 0..n {
        if vertex_remain[vid as usize] {
            vertex_map[vid as usize] = new_vid;
            vertex_invmap.push(vid);
            new_vid = new_vid.checked_add(1).ok_or(IgraphError::Internal(
                "subgraph_from_edges: vertex count overflow",
            ))?;
        }
    }

    let new_vcount = new_vid;

    // Build the new graph.
    let mut result_graph = if graph.is_directed() {
        Graph::new(new_vcount, true)?
    } else {
        Graph::with_vertices(new_vcount)
    };

    let mut edge_map: Vec<u32> = Vec::with_capacity(retained_edges.len());
    for (from, to, old_eid) in &retained_edges {
        let new_from = vertex_map[*from as usize];
        let new_to = vertex_map[*to as usize];
        result_graph.add_edge(new_from, new_to)?;
        edge_map.push(*old_eid);
    }

    Ok(SubgraphFromEdgesResult {
        graph: result_graph,
        vertex_map,
        vertex_invmap,
        edge_map,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_undirected() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 3).unwrap(); // 2
        g.add_edge(3, 4).unwrap(); // 3
        g.add_edge(0, 4).unwrap(); // 4

        let r = subgraph_from_edges(&g, &[0, 2, 4], true).unwrap();
        assert_eq!(r.graph.ecount(), 3);
        // Vertices 0,1,2,3,4 are all incident to at least one kept edge
        assert_eq!(r.graph.vcount(), 5);
        assert_eq!(r.edge_map, vec![0, 2, 4]);
    }

    #[test]
    fn delete_isolated_vertices() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 3).unwrap(); // 2
        g.add_edge(3, 4).unwrap(); // 3

        // Keep only edge 0: (0,1) — vertices 2,3,4 become isolated
        let r = subgraph_from_edges(&g, &[0], true).unwrap();
        assert_eq!(r.graph.ecount(), 1);
        assert_eq!(r.graph.vcount(), 2);
        assert_eq!(r.vertex_invmap, vec![0, 1]);
    }

    #[test]
    fn keep_all_vertices() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 3).unwrap(); // 2
        g.add_edge(3, 4).unwrap(); // 3

        // Keep only edge 0, but don't delete vertices
        let r = subgraph_from_edges(&g, &[0], false).unwrap();
        assert_eq!(r.graph.ecount(), 1);
        assert_eq!(r.graph.vcount(), 5);
        // All vertices mapped to themselves
        for i in 0..5u32 {
            assert_eq!(r.vertex_map[i as usize], i);
        }
    }

    #[test]
    fn empty_edge_set() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let r = subgraph_from_edges(&g, &[], true).unwrap();
        assert_eq!(r.graph.ecount(), 0);
        assert_eq!(r.graph.vcount(), 0);
        assert!(r.vertex_invmap.is_empty());
    }

    #[test]
    fn empty_edge_set_keep_vertices() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();

        let r = subgraph_from_edges(&g, &[], false).unwrap();
        assert_eq!(r.graph.ecount(), 0);
        assert_eq!(r.graph.vcount(), 3);
    }

    #[test]
    fn duplicate_edge_ids() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1

        let r = subgraph_from_edges(&g, &[0, 0, 1, 1, 0], true).unwrap();
        assert_eq!(r.graph.ecount(), 2);
        assert_eq!(r.graph.vcount(), 3);
    }

    #[test]
    fn edge_id_out_of_range() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();

        let err = subgraph_from_edges(&g, &[5], true).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn directed_graph() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 3).unwrap(); // 2
        g.add_edge(3, 0).unwrap(); // 3

        let r = subgraph_from_edges(&g, &[0, 2], true).unwrap();
        assert!(r.graph.is_directed());
        assert_eq!(r.graph.ecount(), 2);
        // Vertices: 0,1 from edge 0; 2,3 from edge 2
        assert_eq!(r.graph.vcount(), 4);
    }

    #[test]
    fn directed_delete_vertices() {
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(2, 3).unwrap(); // 1
        g.add_edge(3, 4).unwrap(); // 2

        // Keep only edge 1: (2,3). Vertices 0,1,4 are isolated.
        let r = subgraph_from_edges(&g, &[1], true).unwrap();
        assert_eq!(r.graph.vcount(), 2);
        assert_eq!(r.graph.ecount(), 1);
        assert_eq!(r.vertex_invmap, vec![2, 3]);
        // Check mapping
        assert_eq!(r.vertex_map[2], 0);
        assert_eq!(r.vertex_map[3], 1);
    }

    #[test]
    fn vertex_map_consistency() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(1, 3).unwrap(); // 0
        g.add_edge(3, 5).unwrap(); // 1

        let r = subgraph_from_edges(&g, &[0, 1], true).unwrap();
        assert_eq!(r.graph.vcount(), 3);
        assert_eq!(r.vertex_invmap, vec![1, 3, 5]);
        assert_eq!(r.vertex_map[1], 0);
        assert_eq!(r.vertex_map[3], 1);
        assert_eq!(r.vertex_map[5], 2);
        // Unmapped vertices
        assert_eq!(r.vertex_map[0], u32::MAX);
        assert_eq!(r.vertex_map[2], u32::MAX);
        assert_eq!(r.vertex_map[4], u32::MAX);
    }

    #[test]
    fn all_edges_selected() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let r = subgraph_from_edges(&g, &[0, 1, 2], true).unwrap();
        assert_eq!(r.graph.vcount(), 4);
        assert_eq!(r.graph.ecount(), 3);
    }

    #[test]
    fn self_loop_edge() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap(); // 0: self-loop
        g.add_edge(0, 1).unwrap(); // 1
        g.add_edge(1, 2).unwrap(); // 2

        let r = subgraph_from_edges(&g, &[0], true).unwrap();
        assert_eq!(r.graph.vcount(), 1);
        assert_eq!(r.graph.ecount(), 1);
        assert_eq!(r.vertex_invmap, vec![0]);
    }

    #[test]
    fn edge_map_ordering() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 3).unwrap(); // 2

        // Request in reverse order — result should still be sorted
        let r = subgraph_from_edges(&g, &[2, 0], true).unwrap();
        assert_eq!(r.edge_map, vec![0, 2]);
    }
}
