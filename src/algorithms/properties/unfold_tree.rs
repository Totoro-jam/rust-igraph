//! Unfold a graph into a tree (ALGO-PR-036).
//!
//! Performs BFS from given roots and replicates vertices visited more
//! than once. The result is a tree (or forest) with the same local
//! neighbourhood structure as the original graph.
//!
//! Counterpart of `igraph_unfold_tree`.

use std::collections::VecDeque;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Result of unfolding a graph into a tree.
#[derive(Debug, Clone)]
pub struct UnfoldTreeResult {
    /// The unfolded tree (or forest).
    pub tree: Graph,
    /// Mapping from tree vertices to original graph vertices.
    /// `vertex_index[v_tree]` gives the original vertex ID.
    pub vertex_index: Vec<VertexId>,
}

/// Unfold a graph into a tree by BFS, replicating multiply-visited
/// vertices.
///
/// Starting from each root, BFS explores the graph. The first time a
/// vertex is reached, it keeps its original ID. Subsequent visits via
/// different edges create replica vertices with new IDs. The result
/// is always a tree (or forest if multiple roots cover disjoint
/// components).
///
/// For directed graphs, `mode` controls traversal: `Out` follows
/// outgoing edges, `In` follows incoming edges, `All` ignores
/// direction.
///
/// # Errors
///
/// Returns an error if any root is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, unfold_tree, DegreeMode};
///
/// // Triangle: unfolding from vertex 0 creates a tree with 4 vertices.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let result = unfold_tree(&g, &[0], DegreeMode::All).unwrap();
/// // BFS from 0: sees 1,2 directly. Edge 1-2 creates a replica.
/// assert_eq!(result.tree.vcount(), 4);
/// assert_eq!(result.tree.ecount(), 3);
/// ```
pub fn unfold_tree(
    graph: &Graph,
    roots: &[VertexId],
    mode: crate::algorithms::properties::degree::DegreeMode,
) -> IgraphResult<UnfoldTreeResult> {
    let n = graph.vcount();
    let ecount = graph.ecount();

    for &r in roots {
        if r >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "unfold_tree: root {r} out of range (vcount = {n})"
            )));
        }
    }

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);
    let mut vertex_index: Vec<VertexId> = (0..n).collect();
    let mut seen_vertices = vec![false; n as usize];
    let mut seen_edges = vec![false; ecount];
    let mut queue: VecDeque<VertexId> = VecDeque::new();
    let mut tree_vcount = n;

    for &root in roots {
        if seen_vertices[root as usize] {
            continue;
        }
        seen_vertices[root as usize] = true;
        queue.push_back(root);

        while let Some(actnode) = queue.pop_front() {
            let incident_edges = get_incident(graph, actnode, mode)?;

            for eid in incident_edges {
                if seen_edges[eid as usize] {
                    continue;
                }
                seen_edges[eid as usize] = true;

                let (from, to) = graph.edge(eid)?;
                let nei = if from == actnode { to } else { from };

                if seen_vertices[nei as usize] {
                    let new_vid = tree_vcount;
                    tree_vcount += 1;
                    vertex_index.push(nei);

                    if from == nei {
                        edges.push((new_vid, to));
                    } else {
                        edges.push((from, new_vid));
                    }
                } else {
                    edges.push((from, to));
                    seen_vertices[nei as usize] = true;
                    queue.push_back(nei);
                }
            }
        }
    }

    let mut tree = Graph::new(tree_vcount, graph.is_directed())?;
    tree.add_edges(edges)?;

    Ok(UnfoldTreeResult { tree, vertex_index })
}

fn get_incident(
    graph: &Graph,
    v: VertexId,
    mode: crate::algorithms::properties::degree::DegreeMode,
) -> IgraphResult<Vec<EdgeId>> {
    use crate::algorithms::properties::degree::DegreeMode;

    match mode {
        DegreeMode::Out => graph.incident(v),
        DegreeMode::In => graph.incident_in(v),
        DegreeMode::All => {
            let mut out = graph.incident(v)?;
            if graph.is_directed() {
                let in_edges = graph.incident_in(v)?;
                for eid in in_edges {
                    if !out.contains(&eid) {
                        out.push(eid);
                    }
                }
            }
            Ok(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::properties::degree::DegreeMode;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let result = unfold_tree(&g, &[], DegreeMode::All).unwrap();
        assert_eq!(result.tree.vcount(), 0);
        assert_eq!(result.tree.ecount(), 0);
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let result = unfold_tree(&g, &[0], DegreeMode::All).unwrap();
        assert_eq!(result.tree.vcount(), 1);
        assert_eq!(result.tree.ecount(), 0);
    }

    #[test]
    fn tree_stays_tree() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let result = unfold_tree(&g, &[0], DegreeMode::All).unwrap();
        assert_eq!(result.tree.vcount(), 4);
        assert_eq!(result.tree.ecount(), 3);
        assert_eq!(result.vertex_index, vec![0, 1, 2, 3]);
    }

    #[test]
    fn triangle_unfolds() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let result = unfold_tree(&g, &[0], DegreeMode::All).unwrap();
        // Triangle has one extra edge beyond a tree; one vertex gets replicated
        assert_eq!(result.tree.vcount(), 4);
        assert_eq!(result.tree.ecount(), 3);
        // The first 3 entries in vertex_index are identity
        assert_eq!(result.vertex_index[0], 0);
        assert_eq!(result.vertex_index[1], 1);
        assert_eq!(result.vertex_index[2], 2);
    }

    #[test]
    fn k4_unfolds() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let result = unfold_tree(&g, &[0], DegreeMode::All).unwrap();
        // K4: 6 edges, spanning tree has 3 → 3 extra edges → 3 replicas
        assert_eq!(result.tree.ecount(), 6);
        assert_eq!(result.tree.vcount(), 7);
    }

    #[test]
    fn two_components() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let result = unfold_tree(&g, &[0, 2], DegreeMode::All).unwrap();
        // Already a forest, no replication
        assert_eq!(result.tree.vcount(), 4);
        assert_eq!(result.tree.ecount(), 2);
    }

    #[test]
    fn directed_out_mode() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let result = unfold_tree(&g, &[0], DegreeMode::Out).unwrap();
        // From 0: out-edges to 1, 2. From 1: out-edge to 2 (replica).
        assert_eq!(result.tree.vcount(), 4);
        assert_eq!(result.tree.ecount(), 3);
    }

    #[test]
    fn root_out_of_range() {
        let g = Graph::with_vertices(3);
        assert!(unfold_tree(&g, &[5], DegreeMode::All).is_err());
    }

    #[test]
    fn vertex_index_correctness() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let result = unfold_tree(&g, &[0], DegreeMode::All).unwrap();
        // All vertex_index entries should be valid original vertices
        for &orig in &result.vertex_index {
            assert!(orig < 3);
        }
    }

    #[test]
    fn cycle_5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        let result = unfold_tree(&g, &[0], DegreeMode::All).unwrap();
        // C5: 5 edges, spanning tree 4 → 1 replica
        assert_eq!(result.tree.vcount(), 6);
        assert_eq!(result.tree.ecount(), 5);
    }
}
