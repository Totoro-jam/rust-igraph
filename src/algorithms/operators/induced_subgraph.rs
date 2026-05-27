//! Induced subgraph extraction (ALGO-OP-007).
//!
//! Given a graph and a set of vertex IDs, returns a new graph containing
//! only those vertices and all edges between them, plus the vertex mapping.

use crate::core::error::IgraphError;
use crate::core::{Graph, IgraphResult, VertexId};

/// Result of an induced subgraph extraction.
#[derive(Debug, Clone)]
pub struct InducedSubgraphResult {
    /// The subgraph.
    pub graph: Graph,
    /// Mapping from old vertex IDs to new vertex IDs. Contains `u32::MAX`
    /// for vertices not in the subgraph.
    pub map: Vec<u32>,
    /// Mapping from new vertex IDs to old vertex IDs (inverse map).
    pub invmap: Vec<VertexId>,
}

/// Creates the subgraph induced by the specified vertices.
///
/// Collects the specified vertices and all edges between them into a new
/// graph. Vertex IDs in the result are contiguous starting from 0 and
/// follow the order of the original graph (sorted by increasing old ID).
///
/// Duplicate vertex IDs in `vids` are silently ignored (only the first
/// occurrence counts). The vertex order in the output always matches the
/// ascending order of original IDs regardless of the order in `vids`.
///
/// # Arguments
///
/// * `graph` — the input graph.
/// * `vids` — slice of vertex IDs to include in the subgraph.
///
/// # Errors
///
/// Returns `InvalidArgument` if any vertex ID in `vids` is >= `graph.vcount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, induced_subgraph};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(0, 4).unwrap();
///
/// let result = induced_subgraph(&g, &[0, 1, 2]).unwrap();
/// assert_eq!(result.graph.vcount(), 3);
/// assert_eq!(result.graph.ecount(), 2); // edges 0-1 and 1-2
/// assert_eq!(result.invmap, vec![0, 1, 2]);
/// ```
pub fn induced_subgraph(graph: &Graph, vids: &[VertexId]) -> IgraphResult<InducedSubgraphResult> {
    let n = graph.vcount();

    // Validate vertex IDs
    for &v in vids {
        if v >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "vertex id {v} is out of range [0, {n})"
            )));
        }
    }

    // Build old-to-new mapping. Use u32::MAX as sentinel for "not in subgraph".
    let mut map = vec![u32::MAX; n as usize];
    let mut invmap: Vec<VertexId> = Vec::new();

    // Collect unique vertices and sort by original ID
    let mut sorted_vids: Vec<VertexId> = vids.to_vec();
    sorted_vids.sort_unstable();
    sorted_vids.dedup();

    let no_of_new_nodes = sorted_vids.len();
    invmap.reserve(no_of_new_nodes);

    for (new_id, &old_id) in sorted_vids.iter().enumerate() {
        // SAFETY: new_id < no_of_new_nodes <= n which is u32
        #[allow(clippy::cast_possible_truncation)]
        let new_id_u32 = new_id as u32;
        map[old_id as usize] = new_id_u32;
        invmap.push(old_id);
    }

    // Build edge list for the new graph
    let directed = graph.is_directed();
    let mut new_edges: Vec<(VertexId, VertexId)> = Vec::new();

    for (new_vid_idx, &old_vid) in invmap.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let new_vid = new_vid_idx as u32;
        let incident = graph.incident(old_vid)?;

        if directed {
            for eid in incident {
                let src = graph.edge_source(eid)?;
                if src != old_vid {
                    continue;
                }
                let tgt = graph.edge_target(eid)?;
                let new_tgt = map[tgt as usize];
                if new_tgt == u32::MAX {
                    continue;
                }
                new_edges.push((new_vid, new_tgt));
            }
        } else {
            // Undirected: process each edge once.
            // For an edge (u, v) stored as edge_source=u, edge_target=v,
            // we process it from the vertex that is edge_source.
            // Self-loops: each appears twice in incident list, emit once.
            let mut skip_loop = false;
            for eid in incident {
                let src = graph.edge_source(eid)?;
                if src != old_vid {
                    continue;
                }
                let tgt = graph.edge_target(eid)?;
                let new_tgt = map[tgt as usize];
                if new_tgt == u32::MAX {
                    continue;
                }
                if new_vid == new_tgt {
                    skip_loop = !skip_loop;
                    if skip_loop {
                        continue;
                    }
                }
                new_edges.push((new_vid, new_tgt));
            }
        }
    }

    // Create the new graph (no_of_new_nodes <= n which is u32)
    #[allow(clippy::cast_possible_truncation)]
    let new_node_count = no_of_new_nodes as u32;
    let mut result_graph = Graph::new(new_node_count, directed)?;
    result_graph.add_edges(new_edges)?;

    Ok(InducedSubgraphResult {
        graph: result_graph,
        map,
        invmap,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_vids() {
        let g = Graph::with_vertices(5);
        let result = induced_subgraph(&g, &[]).unwrap();
        assert_eq!(result.graph.vcount(), 0);
        assert_eq!(result.graph.ecount(), 0);
        assert!(result.invmap.is_empty());
    }

    #[test]
    fn test_all_vertices() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let result = induced_subgraph(&g, &[0, 1, 2, 3]).unwrap();
        assert_eq!(result.graph.vcount(), 4);
        assert_eq!(result.graph.ecount(), 3);
        assert_eq!(result.invmap, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_partial_subgraph() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(0, 4).unwrap();

        let result = induced_subgraph(&g, &[1, 2, 3]).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 2);
        assert_eq!(result.invmap, vec![1, 2, 3]);
        assert_eq!(result.map[0], u32::MAX);
        assert_eq!(result.map[1], 0);
        assert_eq!(result.map[2], 1);
        assert_eq!(result.map[3], 2);
        assert_eq!(result.map[4], u32::MAX);
    }

    #[test]
    fn test_duplicate_vids() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let result = induced_subgraph(&g, &[0, 1, 1, 0]).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn test_unordered_vids() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        // Even though vids are [3, 1, 0], result should be ordered [0, 1, 3]
        let result = induced_subgraph(&g, &[3, 1, 0]).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.invmap, vec![0, 1, 3]);
        // Only edge 0-1 is within {0, 1, 3}
        assert_eq!(result.graph.ecount(), 1);
    }

    #[test]
    fn test_invalid_vertex() {
        let g = Graph::with_vertices(3);
        let result = induced_subgraph(&g, &[0, 5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_directed() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let result = induced_subgraph(&g, &[0, 1, 2]).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 3); // 0→1, 1→0, 1→2
        assert!(result.graph.is_directed());
    }

    #[test]
    fn test_self_loop() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap(); // self-loop
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let result = induced_subgraph(&g, &[0, 1]).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 2); // self-loop + edge 0-1
    }

    #[test]
    fn test_isolated_vertices() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();

        // Extract vertices 1, 3, 4 — no edges between them
        let result = induced_subgraph(&g, &[1, 3, 4]).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 0);
    }

    #[test]
    fn test_single_vertex() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let result = induced_subgraph(&g, &[1]).unwrap();
        assert_eq!(result.graph.vcount(), 1);
        assert_eq!(result.graph.ecount(), 0);
        assert_eq!(result.invmap, vec![1]);
    }

    #[test]
    fn test_complete_subgraph() {
        // K4, extract K3 from vertices {0,1,2}
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();

        let result = induced_subgraph(&g, &[0, 1, 2]).unwrap();
        assert_eq!(result.graph.vcount(), 3);
        assert_eq!(result.graph.ecount(), 3); // K3
    }

    #[test]
    fn test_directed_self_loop() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 0).unwrap(); // self-loop
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let result = induced_subgraph(&g, &[0, 1]).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 2); // self-loop + 0→1
    }

    #[test]
    fn test_multi_edges() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap(); // multi-edge
        g.add_edge(1, 2).unwrap();

        let result = induced_subgraph(&g, &[0, 1]).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 2); // both parallel edges preserved
    }

    #[test]
    fn test_map_consistency() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 4).unwrap();
        g.add_edge(4, 0).unwrap();

        let result = induced_subgraph(&g, &[0, 2, 4]).unwrap();
        // Check map and invmap are inverses
        #[allow(clippy::cast_possible_truncation)]
        for (new_id, &old_id) in result.invmap.iter().enumerate() {
            assert_eq!(result.map[old_id as usize], new_id as u32);
        }
        #[allow(clippy::cast_possible_truncation)]
        for (old_id, &new_id) in result.map.iter().enumerate() {
            if new_id != u32::MAX {
                assert_eq!(result.invmap[new_id as usize], old_id as u32);
            }
        }
    }
}
