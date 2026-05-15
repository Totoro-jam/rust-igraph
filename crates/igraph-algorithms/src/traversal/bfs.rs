//! Breadth-first search.
//!
//! Counterpart of `igraph_bfs()` from `references/igraph/src/properties/bfs.c`.
//! Phase 0 ships the simplest variant: from one root, returning the visit
//! order. Full callback-driven BFS (vids/layers/parents/dist/order/pred/succ)
//! lands in `ALGO-TR-001`.

use std::collections::VecDeque;

use igraph_core::{Graph, IgraphResult, VertexId};

/// Visit order of vertices reachable from `root`, in BFS order.
///
/// Vertices unreachable from `root` are not included. Errors if `root` is not
/// a valid vertex.
///
/// # Examples
///
/// ```
/// use igraph_core::Graph;
/// use igraph_algorithms::traversal::bfs;
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
        for &w in graph.neighbors(v)? {
            if !visited[w as usize] {
                visited[w as usize] = true;
                order.push(w);
                queue.push_back(w);
            }
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
        let err = bfs(&g, 0).unwrap_err();
        assert!(matches!(
            err,
            igraph_core::IgraphError::VertexOutOfRange { id: 0, n: 0 }
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
            igraph_core::IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!(id, 5);
                assert_eq!(n, 2);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
