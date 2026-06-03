//! Depth-first search.
//!
//! Counterpart of `igraph_dfs()` from
//! `references/igraph/src/graph/visitors.c:479-661`.
//! - ALGO-TR-002: simplest variant [`dfs`], returns pre-order visit list.
//! - ALGO-TR-003: multi-output variant [`dfs_tree`], returns parents,
//!   discovery/finish timestamps, and pre/post-order.
//! - [`dfs_simple`]: mode-aware DFS with direction control.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphResult, VertexId};

/// Result of a multi-output DFS scan from a single root. Returned by
/// [`dfs_tree`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DfsTree {
    /// Vertices reachable from the root, in DFS pre-order (discovery order).
    pub order: Vec<VertexId>,
    /// Vertices in DFS post-order (finish order).
    pub order_out: Vec<VertexId>,
    /// `parents[v] == Some(p)` if `v` was DFS-discovered from `p`; `None`
    /// for the root and for unreachable vertices.
    pub parents: Vec<Option<VertexId>>,
    /// `dist[v] == Some(d)` if `v` is reachable from the root at DFS-tree
    /// depth `d` (0 for root); `None` if unreachable.
    pub dist: Vec<Option<u32>>,
}

/// Multi-output DFS from `root`. Returns visit order (pre and post),
/// per-vertex parents, and DFS-tree depth in a single pass.
///
/// Counterpart of `igraph_dfs(_, root, _, _, &order, &order_out, &father, &dist, _, _)`.
///
/// For directed graphs, traversal follows out-edges. Errors if `root`
/// is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dfs_tree};
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
/// let r = dfs_tree(&g, 0).unwrap();
/// assert_eq!(r.order[0], 0);
/// assert_eq!(r.parents[0], None); // root
/// assert_eq!(r.parents[1], Some(0));
/// assert_eq!(r.parents[3], Some(1));
/// assert_eq!(r.dist[3], Some(2));
/// assert_eq!(r.order_out.last(), Some(&0)); // root finishes last
/// ```
pub fn dfs_tree(graph: &Graph, root: VertexId) -> IgraphResult<DfsTree> {
    graph.neighbors_iter(root)?;

    let n = graph.vcount();
    let n_us = n as usize;
    let mut visited = vec![false; n_us];
    let mut order: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut order_out: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut parents: Vec<Option<VertexId>> = vec![None; n_us];
    let mut dist: Vec<Option<u32>> = vec![None; n_us];

    let mut stack: VecDeque<(VertexId, usize, Vec<VertexId>)> = VecDeque::new();

    visited[root as usize] = true;
    order.push(root);
    dist[root as usize] = Some(0);
    let mut root_neis: Vec<VertexId> = graph.neighbors_iter(root)?.collect();
    root_neis.reverse();
    stack.push_back((root, 0, root_neis));

    while let Some(&(cur, cursor, ref neis)) = stack.back() {
        let mut next_cursor = cursor;
        let mut found: Option<VertexId> = None;
        while next_cursor < neis.len() {
            let nei = neis[next_cursor];
            next_cursor += 1;
            if !visited[nei as usize] {
                found = Some(nei);
                break;
            }
        }

        if let Some(nei) = found {
            let last = stack.len() - 1;
            stack[last].1 = next_cursor;
            visited[nei as usize] = true;
            order.push(nei);
            parents[nei as usize] = Some(cur);
            #[allow(clippy::cast_possible_truncation)]
            let depth = (stack.len()) as u32; // stack.len() == depth of new node
            dist[nei as usize] = Some(depth);
            let mut nei_neis: Vec<VertexId> = graph.neighbors_iter(nei)?.collect();
            nei_neis.reverse();
            stack.push_back((nei, 0, nei_neis));
        } else {
            order_out.push(cur);
            stack.pop_back();
        }
    }

    Ok(DfsTree {
        order,
        order_out,
        parents,
        dist,
    })
}

/// Pre-order visit of vertices reachable from `root`, in DFS order.
///
/// Vertices unreachable from `root` are not included. Errors if `root`
/// is not a valid vertex. Neighbour iteration follows
/// [`Graph::neighbors`]'s order, which matches `python-igraph`'s
/// `Graph.dfs` and the upstream `igraph_dfs` order.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dfs};
///
/// // 0 - 1 - 3
/// // |
/// // 2
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
///
/// let order = dfs(&g, 0).unwrap();
/// assert_eq!(order[0], 0);                // always start at root
/// assert_eq!(order.len(), 4);              // visits whole component
/// ```
pub fn dfs(graph: &Graph, root: VertexId) -> IgraphResult<Vec<VertexId>> {
    // Validate root via the neighbour lookup; surfaces VertexOutOfRange.
    graph.neighbors_iter(root)?;

    let n = graph.vcount();
    let mut visited = vec![false; n as usize];
    let mut order: Vec<VertexId> = Vec::with_capacity(n as usize);

    // Stack mirrors `igraph_stack_int_t`. Each entry stores the vertex
    // and the cursor into its neighbour list — direct port of the C
    // `nptr` array (visitors.c:516, :593).
    let mut stack: VecDeque<(VertexId, usize, Vec<VertexId>)> = VecDeque::new();

    visited[root as usize] = true;
    order.push(root);
    // Reverse the neighbour list before iteration: matches the upstream
    // `igraph_dfs` pre-order on undirected graphs (and python-igraph's
    // `Graph.dfs`). Concretely, igraph C's lazy adjacency list yields
    // neighbours in opposite order to `igraph_neighbors` for DFS;
    // pre-reversing here keeps our DFS in lockstep with both.
    let mut root_neis: Vec<VertexId> = graph.neighbors_iter(root)?.collect();
    root_neis.reverse();
    stack.push_back((root, 0, root_neis));

    while let Some(&(actvect, cursor, ref neis)) = stack.back() {
        // Advance the cursor until we find an unvisited neighbour or
        // exhaust the neighbour list.
        let mut next_cursor = cursor;
        let mut found: Option<VertexId> = None;
        while next_cursor < neis.len() {
            let nei = neis[next_cursor];
            next_cursor += 1;
            if !visited[nei as usize] {
                found = Some(nei);
                break;
            }
        }

        if let Some(nei) = found {
            // Update current frame's cursor; push the new vertex.
            let last = stack.len() - 1;
            stack[last].1 = next_cursor;
            visited[nei as usize] = true;
            order.push(nei);
            let mut nei_neis: Vec<VertexId> = graph.neighbors_iter(nei)?.collect();
            nei_neis.reverse();
            stack.push_back((nei, 0, nei_neis));
        } else {
            // Subtree at `actvect` is complete; pop.
            let _ = actvect;
            stack.pop_back();
        }
    }

    Ok(order)
}

/// Direction mode for [`dfs_simple`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DfsMode {
    /// Follow outgoing edges (default for directed graphs).
    Out,
    /// Follow incoming edges.
    In,
    /// Ignore edge direction (always used for undirected graphs).
    All,
}

/// Result of [`dfs_simple`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DfsSimple {
    /// Vertices visited during the traversal, in DFS pre-order.
    pub order: Vec<VertexId>,
    /// Vertices in DFS post-order (finish order).
    pub order_out: Vec<VertexId>,
    /// `parents[v] == Some(p)` if vertex `v` was discovered via `p`;
    /// `None` for root and unreachable vertices.
    pub parents: Vec<Option<VertexId>>,
    /// `dist[v] == Some(d)` if `v` is reachable at DFS-tree depth `d`;
    /// `None` if unreachable.
    pub dist: Vec<Option<u32>>,
}

/// Mode-aware DFS from a single root, returning pre/post-order,
/// parents, and DFS-tree depth.
///
/// Counterpart of `igraph_dfs` from
/// `references/igraph/src/graph/visitors.c`.
///
/// For undirected graphs the `mode` parameter is ignored. For directed
/// graphs, [`DfsMode::Out`] follows outgoing edges, [`DfsMode::In`]
/// follows incoming edges, and [`DfsMode::All`] ignores direction.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, DfsMode, dfs_simple};
///
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
///
/// let r = dfs_simple(&g, 0, DfsMode::Out).unwrap();
/// assert_eq!(r.order[0], 0);
/// assert_eq!(r.parents[1], Some(0));
/// assert_eq!(r.dist[2], Some(2));
/// ```
pub fn dfs_simple(graph: &Graph, root: VertexId, mode: DfsMode) -> IgraphResult<DfsSimple> {
    graph.neighbors_iter(root)?; // validate root

    let n = graph.vcount();
    let n_us = n as usize;
    let use_mode = if graph.is_directed() {
        mode
    } else {
        DfsMode::All
    };

    let mut visited = vec![false; n_us];
    let mut order: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut order_out: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut parents: Vec<Option<VertexId>> = vec![None; n_us];
    let mut dist: Vec<Option<u32>> = vec![None; n_us];

    let neighbors = |v: VertexId| -> IgraphResult<Vec<VertexId>> {
        match use_mode {
            DfsMode::Out => graph.out_neighbors_vec(v),
            DfsMode::In => graph.in_neighbors_vec(v),
            DfsMode::All => {
                if graph.is_directed() {
                    let mut combined = graph.out_neighbors_vec(v)?;
                    combined.extend(graph.in_neighbors_vec(v)?);
                    Ok(combined)
                } else {
                    Ok(graph.neighbors_iter(v)?.collect())
                }
            }
        }
    };

    let mut stack: VecDeque<(VertexId, usize, Vec<VertexId>)> = VecDeque::new();

    visited[root as usize] = true;
    order.push(root);
    dist[root as usize] = Some(0);
    let mut root_neis = neighbors(root)?;
    root_neis.reverse();
    stack.push_back((root, 0, root_neis));

    while let Some(&(cur, cursor, ref neis)) = stack.back() {
        let mut next_cursor = cursor;
        let mut found: Option<VertexId> = None;
        while next_cursor < neis.len() {
            let nei = neis[next_cursor];
            next_cursor += 1;
            if !visited[nei as usize] {
                found = Some(nei);
                break;
            }
        }

        if let Some(nei) = found {
            let last = stack.len() - 1;
            stack[last].1 = next_cursor;
            visited[nei as usize] = true;
            order.push(nei);
            parents[nei as usize] = Some(cur);
            #[allow(clippy::cast_possible_truncation)]
            let depth = stack.len() as u32;
            dist[nei as usize] = Some(depth);
            let mut nei_neis = neighbors(nei)?;
            nei_neis.reverse();
            stack.push_back((nei, 0, nei_neis));
        } else {
            order_out.push(cur);
            stack.pop_back();
        }
    }

    Ok(DfsSimple {
        order,
        order_out,
        parents,
        dist,
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
        let err = dfs(&g, 0).unwrap_err();
        assert!(matches!(
            err,
            crate::core::IgraphError::VertexOutOfRange { id: 0, n: 0 }
        ));
    }

    #[test]
    fn single_vertex_visits_self() {
        let g = Graph::with_vertices(1);
        assert_eq!(dfs(&g, 0).unwrap(), vec![0]);
    }

    #[test]
    fn path_visits_in_order() {
        let g = path_graph(5);
        // Path 0-1-2-3-4: DFS from 0 follows the chain.
        assert_eq!(dfs(&g, 0).unwrap(), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn dfs_goes_deep_before_wide() {
        // 0 -- 1 -- 3
        // |
        // 2
        // DFS from 0 must visit either 1-then-3 before backing up to 2,
        // OR 2-then-back to 1-3. Critical: DFS visits one full subtree
        // before another sibling (unlike BFS).
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        let order = dfs(&g, 0).unwrap();
        assert_eq!(order[0], 0);
        assert_eq!(order.len(), 4);
        let pos = |v: u32| order.iter().position(|&x| x == v).unwrap();
        // 3 is at depth 2 via 1; the DFS depth-first nature means
        // pos(3) - pos(1) == 1 OR pos(3) > pos(1) without 2 in between.
        // Easier invariant: 3 must be adjacent in `order` to either 1
        // (parent) or part of the same subtree.
        let p1 = pos(1);
        let p3 = pos(3);
        assert!(p3 == p1 + 1, "3 should follow 1 directly in DFS pre-order");
    }

    #[test]
    fn unreachable_vertex_excluded() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        // Vertex 2 is isolated.
        let order = dfs(&g, 0).unwrap();
        assert_eq!(order.len(), 2);
        assert!(order.contains(&0));
        assert!(order.contains(&1));
    }

    #[test]
    fn invalid_root_errors() {
        let g = Graph::with_vertices(2);
        let err = dfs(&g, 5).unwrap_err();
        match err {
            crate::core::IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!(id, 5);
                assert_eq!(n, 2);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn complete_k4_visits_all_four() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let order = dfs(&g, 0).unwrap();
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![0, 1, 2, 3]);
        assert_eq!(order[0], 0);
    }

    // --- dfs_tree tests ---

    #[test]
    fn dfs_tree_singleton() {
        let g = Graph::with_vertices(1);
        let r = dfs_tree(&g, 0).unwrap();
        assert_eq!(r.order, vec![0]);
        assert_eq!(r.order_out, vec![0]);
        assert_eq!(r.parents, vec![None]);
        assert_eq!(r.dist, vec![Some(0)]);
    }

    #[test]
    fn dfs_tree_path() {
        let g = path_graph(4);
        let r = dfs_tree(&g, 0).unwrap();
        assert_eq!(r.order, vec![0, 1, 2, 3]);
        assert_eq!(r.order_out, vec![3, 2, 1, 0]);
        assert_eq!(r.parents, vec![None, Some(0), Some(1), Some(2)]);
        assert_eq!(r.dist, vec![Some(0), Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn dfs_tree_branching() {
        // 0-1, 0-2, 1-3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        let r = dfs_tree(&g, 0).unwrap();
        assert_eq!(r.order[0], 0);
        assert_eq!(r.parents[0], None);
        assert_eq!(r.dist[0], Some(0));
        // Vertex 3's parent must be 1
        assert_eq!(r.parents[3], Some(1));
        assert_eq!(r.dist[3], Some(2));
        // Root finishes last in post-order
        assert_eq!(*r.order_out.last().unwrap(), 0);
    }

    #[test]
    fn dfs_tree_unreachable() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let r = dfs_tree(&g, 0).unwrap();
        assert_eq!(r.order.len(), 2);
        assert_eq!(r.order_out.len(), 2);
        assert_eq!(r.parents[2], None);
        assert_eq!(r.dist[2], None);
    }

    #[test]
    fn dfs_tree_order_out_reverse_invariant() {
        // For a tree (no cross-edges within reached component),
        // order_out is the reverse of a valid topological post-ordering.
        // Check that every parent finishes after all its children.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        let r = dfs_tree(&g, 0).unwrap();
        let finish_pos = |v: u32| r.order_out.iter().position(|&x| x == v).unwrap();
        // Parent 0 finishes after children 1, 2
        assert!(finish_pos(0) > finish_pos(1));
        assert!(finish_pos(0) > finish_pos(2));
        // Parent 1 finishes after children 3, 4
        assert!(finish_pos(1) > finish_pos(3));
        assert!(finish_pos(1) > finish_pos(4));
    }

    #[test]
    fn dfs_tree_invalid_root() {
        let g = Graph::with_vertices(2);
        assert!(dfs_tree(&g, 5).is_err());
    }

    // ── dfs_simple tests ────────────────────────────────────────────

    #[test]
    fn dfs_simple_undirected_tree() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();

        let r = dfs_simple(&g, 0, DfsMode::Out).unwrap();
        assert_eq!(r.order[0], 0);
        assert_eq!(r.order.len(), 4);
        assert_eq!(r.parents[0], None);
        assert_eq!(r.dist[0], Some(0));
        assert_eq!(*r.order_out.last().unwrap(), 0);
    }

    #[test]
    fn dfs_simple_single_vertex() {
        let g = Graph::with_vertices(1);
        let r = dfs_simple(&g, 0, DfsMode::All).unwrap();
        assert_eq!(r.order, vec![0]);
        assert_eq!(r.order_out, vec![0]);
        assert_eq!(r.dist, vec![Some(0)]);
    }

    #[test]
    fn dfs_simple_invalid_root() {
        let g = Graph::with_vertices(2);
        assert!(dfs_simple(&g, 5, DfsMode::Out).is_err());
    }

    #[test]
    fn dfs_simple_directed_out() {
        // 0 -> 1 -> 2, 0 -> 3
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 3).unwrap();

        let r = dfs_simple(&g, 0, DfsMode::Out).unwrap();
        assert_eq!(r.order[0], 0);
        assert_eq!(r.order.len(), 4);
        assert_eq!(r.parents[1], Some(0));
        assert_eq!(r.parents[2], Some(1));
        assert_eq!(r.dist[2], Some(2));
    }

    #[test]
    fn dfs_simple_directed_in() {
        // 1 -> 0, 2 -> 1, 3 -> 0
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(2, 1).unwrap();
        g.add_edge(3, 0).unwrap();

        let r = dfs_simple(&g, 0, DfsMode::In).unwrap();
        assert_eq!(r.order[0], 0);
        assert_eq!(r.order.len(), 4);
        assert_eq!(r.parents[1], Some(0));
        assert_eq!(r.parents[2], Some(1));
        assert_eq!(r.parents[3], Some(0));
    }

    #[test]
    fn dfs_simple_directed_all() {
        // 0 -> 1, 2 -> 0
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 0).unwrap();

        let r = dfs_simple(&g, 0, DfsMode::All).unwrap();
        assert_eq!(r.order.len(), 3);
        assert_eq!(r.order[0], 0);
    }

    #[test]
    fn dfs_simple_unreachable() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        let r = dfs_simple(&g, 0, DfsMode::All).unwrap();
        assert_eq!(r.order.len(), 2);
        assert_eq!(r.order_out.len(), 2);
        assert_eq!(r.parents[2], None);
        assert_eq!(r.dist[2], None);
    }

    #[test]
    fn dfs_simple_mode_ignored_for_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = dfs_simple(&g, 0, DfsMode::In).unwrap();
        assert_eq!(r.order.len(), 3);
    }

    #[test]
    fn dfs_simple_directed_out_no_reach() {
        // 0 -> 1, 2 -> 0: from 0 following Out, only 0 and 1 reachable
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 0).unwrap();
        let r = dfs_simple(&g, 0, DfsMode::Out).unwrap();
        assert_eq!(r.order.len(), 2);
        assert!(r.order.contains(&0));
        assert!(r.order.contains(&1));
        assert_eq!(r.dist[2], None);
    }
}
