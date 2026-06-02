//! Breadth-first search.
//!
//! Counterpart of `igraph_bfs()` from `references/igraph/src/properties/bfs.c`.
//! - Phase 0 [`bfs`]: simplest variant, returns the visit order.
//! - Phase 1 [`bfs_tree`] (ALGO-TR-001): full multi-output variant,
//!   returns the visit order + per-vertex distance + BFS-tree parent.
//! - [`bfs_simple`]: mode-aware BFS with layer boundaries. Counterpart
//!   of `igraph_bfs_simple` from `references/igraph/src/graph/visitors.c`.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphResult, VertexId};

/// Result of a multi-output BFS scan from a single root. Returned by
/// [`bfs_tree`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BfsTree {
    /// Vertices reachable from the root, in BFS visit order.
    pub order: Vec<VertexId>,
    /// `distances[v] == Some(d)` if `v` is reachable from the root in
    /// `d` edges (`d == 0` for the root); `None` if unreachable.
    pub distances: Vec<Option<u32>>,
    /// `parents[v] == Some(p)` if `v` was BFS-discovered via `p`; `None`
    /// for the root and for unreachable vertices.
    pub parents: Vec<Option<VertexId>>,
}

/// Multi-output BFS from `root`. Returns visit order, per-vertex
/// distances, and per-vertex BFS-tree parent in a single pass.
///
/// Counterpart of `igraph_bfs(_, root, _, _, &order, _, &father, &dist, _, _)`
/// — the common subset of the upstream callback-driven variant.
///
/// For directed graphs traversal follows out-edges (matching upstream's
/// `IGRAPH_OUT` mode default). Errors if `root` is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bfs_tree};
///
/// // Tree:
/// //     0
/// //    / \
/// //   1   2
/// //   |
/// //   3
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
///
/// let r = bfs_tree(&g, 0).unwrap();
/// assert_eq!(r.order, vec![0, 1, 2, 3]);
/// assert_eq!(r.distances, vec![Some(0), Some(1), Some(1), Some(2)]);
/// assert_eq!(r.parents, vec![None, Some(0), Some(0), Some(1)]);
/// ```
pub fn bfs_tree(graph: &Graph, root: VertexId) -> IgraphResult<BfsTree> {
    graph.neighbors(root)?; // surface VertexOutOfRange

    let n = graph.vcount();
    let n_us = n as usize;
    let mut distances: Vec<Option<u32>> = vec![None; n_us];
    let mut parents: Vec<Option<VertexId>> = vec![None; n_us];
    let mut order: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut queue: VecDeque<VertexId> = VecDeque::new();

    distances[root as usize] = Some(0);
    order.push(root);
    queue.push_back(root);

    while let Some(v) = queue.pop_front() {
        let v_dist = distances[v as usize].ok_or(crate::core::IgraphError::Internal(
            "dequeued vertex has no distance",
        ))?;
        let next_dist = v_dist
            .checked_add(1)
            .ok_or(crate::core::IgraphError::Internal(
                "BFS distance overflow (graph diameter exceeds u32::MAX)",
            ))?;
        for w in graph.neighbors(v)? {
            if distances[w as usize].is_none() {
                distances[w as usize] = Some(next_dist);
                parents[w as usize] = Some(v);
                order.push(w);
                queue.push_back(w);
            }
        }
    }

    Ok(BfsTree {
        order,
        distances,
        parents,
    })
}

/// Visit order of vertices reachable from `root`, in BFS order.
///
/// Vertices unreachable from `root` are not included. Errors if `root` is not
/// a valid vertex.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bfs};
///
/// // 0 - 1 - 3
/// // |
/// // 2
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
///
/// let order = bfs(&g, 0).unwrap();
/// assert_eq!(order, vec![0, 1, 2, 3]);
/// ```
pub fn bfs(graph: &Graph, root: VertexId) -> IgraphResult<Vec<VertexId>> {
    // Validate root via the neighbor lookup; this surfaces VertexOutOfRange.
    graph.neighbors(root)?;

    let n = graph.vcount();
    let mut visited = vec![false; n as usize];
    let mut order: Vec<VertexId> = Vec::with_capacity(n as usize);
    let mut queue: VecDeque<VertexId> = VecDeque::new();

    visited[root as usize] = true;
    order.push(root);
    queue.push_back(root);

    while let Some(v) = queue.pop_front() {
        // `neighbors` returns Vec<VertexId> on the new indexed-edgelist
        // backend (ALGO-CORE-001a); a contiguous slice is not free anymore.
        for w in graph.neighbors(v)? {
            if !visited[w as usize] {
                visited[w as usize] = true;
                order.push(w);
                queue.push_back(w);
            }
        }
    }

    Ok(order)
}

/// Direction mode for [`bfs_simple`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BfsMode {
    /// Follow outgoing edges (default for directed graphs).
    Out,
    /// Follow incoming edges.
    In,
    /// Ignore edge direction (always used for undirected graphs).
    All,
}

/// Result of [`bfs_simple`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BfsSimple {
    /// Vertices visited during the traversal, in BFS order.
    pub order: Vec<VertexId>,
    /// Layer boundary indices into [`order`](Self::order). Vertices at
    /// distance `d` from the root are `order[layers[d]..layers[d+1]]`.
    /// The length is `max_distance + 2`.
    pub layers: Vec<usize>,
    /// `parents[v] == Some(p)` if vertex `v` was discovered via `p`;
    /// `parents[root] == None` (root has no parent).
    /// `parents[v] == None` for unreachable vertices as well.
    /// To distinguish root from unreachable, check `order[0] == v`.
    pub parents: Vec<Option<VertexId>>,
}

/// Mode-aware BFS from a single root, returning visit order, layer
/// boundaries, and BFS-tree parents.
///
/// Counterpart of `igraph_bfs_simple` from
/// `references/igraph/src/graph/visitors.c`.
///
/// For undirected graphs the `mode` parameter is ignored (edges are
/// always treated as bidirectional). For directed graphs, [`BfsMode::Out`]
/// follows outgoing edges, [`BfsMode::In`] follows incoming edges, and
/// [`BfsMode::All`] ignores direction.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, BfsMode, bfs_simple};
///
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
///
/// let r = bfs_simple(&g, 0, BfsMode::Out).unwrap();
/// assert_eq!(r.order, vec![0, 1, 2, 3]);
/// assert_eq!(r.layers, vec![0, 1, 3, 4]);
/// assert_eq!(r.parents[3], Some(1));
/// ```
pub fn bfs_simple(graph: &Graph, root: VertexId, mode: BfsMode) -> IgraphResult<BfsSimple> {
    graph.neighbors(root)?; // validate root

    let n = graph.vcount() as usize;
    let use_mode = if graph.is_directed() {
        mode
    } else {
        BfsMode::All
    };

    let mut visited = vec![false; n];
    let mut parents: Vec<Option<VertexId>> = vec![None; n];
    let mut order: Vec<VertexId> = Vec::with_capacity(n);
    let mut layers: Vec<usize> = Vec::new();

    // Queue stores (vertex, distance) pairs.
    let mut queue: VecDeque<(VertexId, u32)> = VecDeque::new();
    let mut last_layer: Option<u32> = None;

    visited[root as usize] = true;
    order.push(root);
    layers.push(0); // layer 0 starts at index 0
    queue.push_back((root, 0));

    while let Some((v, dist)) = queue.pop_front() {
        let neis = match use_mode {
            BfsMode::Out => graph.out_neighbors_vec(v)?,
            BfsMode::In => graph.in_neighbors_vec(v)?,
            BfsMode::All => {
                if graph.is_directed() {
                    let mut combined = graph.out_neighbors_vec(v)?;
                    combined.extend(graph.in_neighbors_vec(v)?);
                    combined
                } else {
                    graph.neighbors(v)?
                }
            }
        };
        let next_dist = dist
            .checked_add(1)
            .ok_or(crate::core::IgraphError::Internal(
                "BFS distance overflow (graph diameter exceeds u32::MAX)",
            ))?;
        for w in neis {
            if !visited[w as usize] {
                visited[w as usize] = true;
                parents[w as usize] = Some(v);
                queue.push_back((w, next_dist));
                if last_layer != Some(next_dist) {
                    layers.push(order.len());
                    last_layer = Some(next_dist);
                }
                order.push(w);
            }
        }
    }

    // Sentinel: final layer boundary = total visited count.
    layers.push(order.len());

    Ok(BfsSimple {
        order,
        layers,
        parents,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_graph(n: u32) -> Graph {
        let mut g = Graph::with_vertices(n);
        for i in 0..n.saturating_sub(1) {
            g.add_edge(i, i + 1).unwrap();
        }
        g
    }

    #[test]
    fn empty_graph_errors_on_any_root() {
        let g = Graph::with_vertices(0);
        let err = bfs(&g, 0).unwrap_err();
        assert!(matches!(
            err,
            crate::core::IgraphError::VertexOutOfRange { id: 0, n: 0 }
        ));
    }

    #[test]
    fn single_vertex_visits_self() {
        let g = Graph::with_vertices(1);
        assert_eq!(bfs(&g, 0).unwrap(), vec![0]);
    }

    #[test]
    fn path_visits_in_order() {
        let g = path_graph(5);
        assert_eq!(bfs(&g, 0).unwrap(), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn bfs_layers_breadth_first_not_depth_first() {
        // 0 -- 1 -- 3
        // |
        // 2
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        // BFS order from 0 must visit both 1 and 2 before 3.
        let order = bfs(&g, 0).unwrap();
        assert_eq!(order[0], 0);
        let pos_3 = order.iter().position(|&x| x == 3).unwrap();
        let pos_1 = order.iter().position(|&x| x == 1).unwrap();
        let pos_2 = order.iter().position(|&x| x == 2).unwrap();
        assert!(pos_3 > pos_1 && pos_3 > pos_2);
    }

    #[test]
    fn unreachable_vertex_excluded() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        // vertex 2 is isolated.
        let order = bfs(&g, 0).unwrap();
        assert_eq!(order, vec![0, 1]);
    }

    #[test]
    fn invalid_root_errors() {
        let g = Graph::with_vertices(2);
        let err = bfs(&g, 5).unwrap_err();
        match err {
            crate::core::IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!(id, 5);
                assert_eq!(n, 2);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn bfs_tree_returns_full_state_for_a_tree() {
        // Star centred at 0, leaves 1, 2, 3.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let r = bfs_tree(&g, 0).unwrap();
        assert_eq!(r.order, vec![0, 1, 2, 3]);
        assert_eq!(r.distances, vec![Some(0), Some(1), Some(1), Some(1)]);
        assert_eq!(r.parents, vec![None, Some(0), Some(0), Some(0)]);
    }

    #[test]
    fn bfs_tree_path_5_is_chain() {
        let g = path_graph(5);
        let r = bfs_tree(&g, 0).unwrap();
        assert_eq!(r.order, vec![0, 1, 2, 3, 4]);
        assert_eq!(
            r.distances,
            vec![Some(0), Some(1), Some(2), Some(3), Some(4)]
        );
        assert_eq!(r.parents, vec![None, Some(0), Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn bfs_tree_unreachable_vertices_have_none() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        // 2 and 3 isolated.
        let r = bfs_tree(&g, 0).unwrap();
        assert_eq!(r.order, vec![0, 1]);
        assert_eq!(r.distances, vec![Some(0), Some(1), None, None]);
        assert_eq!(r.parents, vec![None, Some(0), None, None]);
    }

    #[test]
    fn bfs_tree_invalid_root_errors() {
        let g = Graph::with_vertices(3);
        assert!(bfs_tree(&g, 7).is_err());
    }

    #[test]
    fn bfs_tree_directed_uses_out_edges() {
        // 0 -> 1 -> 2, 1 -> 3: from 0, all 4 reachable.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        let r = bfs_tree(&g, 0).unwrap();
        assert_eq!(r.distances, vec![Some(0), Some(1), Some(2), Some(2)]);
        assert_eq!(r.parents[0], None);
        assert_eq!(r.parents[1], Some(0));
        assert_eq!(r.parents[2], Some(1));
        assert_eq!(r.parents[3], Some(1));
        // From vertex 2, only 2 is reachable (no out-edges).
        let r2 = bfs_tree(&g, 2).unwrap();
        assert_eq!(r2.order, vec![2]);
    }

    // ── bfs_simple tests ────────────────────────────────────────────

    #[test]
    fn bfs_simple_undirected_tree() {
        //     0
        //    / \
        //   1   2
        //   |
        //   3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();

        let r = bfs_simple(&g, 0, BfsMode::Out).unwrap();
        assert_eq!(r.order, vec![0, 1, 2, 3]);
        assert_eq!(r.layers, vec![0, 1, 3, 4]);
        assert_eq!(r.parents[0], None);
        assert_eq!(r.parents[1], Some(0));
        assert_eq!(r.parents[2], Some(0));
        assert_eq!(r.parents[3], Some(1));
    }

    #[test]
    fn bfs_simple_single_vertex() {
        let g = Graph::with_vertices(1);
        let r = bfs_simple(&g, 0, BfsMode::All).unwrap();
        assert_eq!(r.order, vec![0]);
        assert_eq!(r.layers, vec![0, 1]);
        assert_eq!(r.parents, vec![None]);
    }

    #[test]
    fn bfs_simple_invalid_root() {
        let g = Graph::with_vertices(2);
        assert!(bfs_simple(&g, 5, BfsMode::Out).is_err());
    }

    #[test]
    fn bfs_simple_directed_out() {
        // 0 -> 1 -> 2, 0 -> 3
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 3).unwrap();

        let r = bfs_simple(&g, 0, BfsMode::Out).unwrap();
        assert_eq!(r.order, vec![0, 1, 3, 2]);
        assert_eq!(r.layers, vec![0, 1, 3, 4]);
        assert_eq!(r.parents[1], Some(0));
        assert_eq!(r.parents[2], Some(1));
        assert_eq!(r.parents[3], Some(0));
    }

    #[test]
    fn bfs_simple_directed_in() {
        // 1 -> 0, 2 -> 1, 3 -> 0
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(2, 1).unwrap();
        g.add_edge(3, 0).unwrap();

        let r = bfs_simple(&g, 0, BfsMode::In).unwrap();
        assert_eq!(r.order, vec![0, 1, 3, 2]);
        assert_eq!(r.layers, vec![0, 1, 3, 4]);
        assert_eq!(r.parents[1], Some(0));
        assert_eq!(r.parents[3], Some(0));
        assert_eq!(r.parents[2], Some(1));
    }

    #[test]
    fn bfs_simple_directed_all() {
        // 0 -> 1, 2 -> 0
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 0).unwrap();

        let r = bfs_simple(&g, 0, BfsMode::All).unwrap();
        assert_eq!(r.order.len(), 3);
        assert_eq!(r.order[0], 0);
        // Both 1 and 2 are at distance 1
        assert_eq!(r.layers, vec![0, 1, 3]);
    }

    #[test]
    fn bfs_simple_unreachable_vertices() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        // 2, 3 isolated
        let r = bfs_simple(&g, 0, BfsMode::All).unwrap();
        assert_eq!(r.order, vec![0, 1]);
        assert_eq!(r.layers, vec![0, 1, 2]);
        assert_eq!(r.parents[2], None);
        assert_eq!(r.parents[3], None);
    }

    #[test]
    fn bfs_simple_mode_ignored_for_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // BfsMode::In should be ignored for undirected
        let r = bfs_simple(&g, 0, BfsMode::In).unwrap();
        assert_eq!(r.order, vec![0, 1, 2]);
    }

    #[test]
    fn bfs_simple_layers_match_distances() {
        let g = path_graph(6);
        let r = bfs_simple(&g, 0, BfsMode::Out).unwrap();
        // Path: 0-1-2-3-4-5 => each layer has exactly 1 vertex (except root)
        assert_eq!(r.layers, vec![0, 1, 2, 3, 4, 5, 6]);
        for d in 0..6 {
            let layer_verts = &r.order[r.layers[d]..r.layers[d + 1]];
            assert_eq!(layer_verts.len(), 1);
        }
    }
}
