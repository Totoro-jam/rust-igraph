//! Depth-first search.
//!
//! Counterpart of `igraph_dfs()` from
//! `references/igraph/src/graph/visitors.c:479-661`.
//! ALGO-TR-002 ships the simplest variant: from one root, return the
//! pre-order visit list. Full callback-driven DFS (in/out callbacks,
//! `parents` / `dist` / `order_out`, unreachable mode) lands in a
//! follow-up AWU.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphResult, VertexId};

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
    let _ = graph.neighbors(root)?;

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
    let mut root_neis = graph.neighbors(root)?;
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
            let mut nei_neis = graph.neighbors(nei)?;
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
}
