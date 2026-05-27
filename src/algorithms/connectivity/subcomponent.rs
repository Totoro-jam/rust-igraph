//! Subcomponent extraction (ALGO-CC-051).
//!
//! Finds all vertices reachable from a given source vertex.
//! Counterpart of `igraph_subcomponent`.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Direction mode for subcomponent search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubcomponentMode {
    /// Follow edges in the outgoing direction (for directed graphs).
    Out,
    /// Follow edges in the incoming direction (for directed graphs).
    In,
    /// Follow edges in both directions (ignore direction).
    All,
}

/// Returns all vertices reachable from `source` following edges in the
/// specified `mode`.
///
/// For undirected graphs, `mode` is ignored (all modes are equivalent).
///
/// The result always includes `source` itself. Vertices are returned
/// in BFS discovery order.
///
/// # Errors
///
/// Returns `VertexOutOfRange` if `source >= vcount`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, subcomponent, SubcomponentMode};
///
/// let mut g = Graph::new(5, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(3, 4).unwrap();
///
/// // From vertex 0, following OUT edges: reach 0, 1, 2
/// let reach = subcomponent(&g, 0, SubcomponentMode::Out).unwrap();
/// assert_eq!(reach, vec![0, 1, 2]);
///
/// // From vertex 2, following OUT edges: only 2 (no outgoing)
/// let reach = subcomponent(&g, 2, SubcomponentMode::Out).unwrap();
/// assert_eq!(reach, vec![2]);
///
/// // From vertex 2, following IN edges: reach 2, 1, 0
/// let reach = subcomponent(&g, 2, SubcomponentMode::In).unwrap();
/// assert_eq!(reach, vec![2, 1, 0]);
///
/// // Following ALL: reach 0, 1, 2 (component of 0-1-2)
/// let reach = subcomponent(&g, 1, SubcomponentMode::All).unwrap();
/// assert_eq!(reach.len(), 3);
/// ```
pub fn subcomponent(
    graph: &Graph,
    source: VertexId,
    mode: SubcomponentMode,
) -> IgraphResult<Vec<VertexId>> {
    let n = graph.vcount();
    if source >= n {
        return Err(IgraphError::VertexOutOfRange { id: source, n });
    }

    if n == 0 {
        return Ok(Vec::new());
    }

    // Build adjacency based on mode
    let adj = build_adj(graph, mode)?;

    // BFS from source
    let mut visited = vec![false; n as usize];
    let mut result = Vec::new();
    let mut queue = std::collections::VecDeque::new();

    visited[source as usize] = true;
    queue.push_back(source);

    while let Some(v) = queue.pop_front() {
        result.push(v);
        for &nb in &adj[v as usize] {
            if !visited[nb as usize] {
                visited[nb as usize] = true;
                queue.push_back(nb);
            }
        }
    }

    Ok(result)
}

fn build_adj(graph: &Graph, mode: SubcomponentMode) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        if src == tgt {
            continue;
        }

        if !graph.is_directed() || mode == SubcomponentMode::All {
            adj[src as usize].push(tgt);
            adj[tgt as usize].push(src);
        } else if mode == SubcomponentMode::Out {
            adj[src as usize].push(tgt);
        } else {
            // In mode
            adj[tgt as usize].push(src);
        }
    }

    Ok(adj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_out_of_range() {
        let g = Graph::with_vertices(3);
        assert!(subcomponent(&g, 5, SubcomponentMode::All).is_err());
    }

    #[test]
    fn test_single_vertex() {
        let g = Graph::with_vertices(1);
        let r = subcomponent(&g, 0, SubcomponentMode::All).unwrap();
        assert_eq!(r, vec![0]);
    }

    #[test]
    fn test_disconnected_undirected() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = subcomponent(&g, 0, SubcomponentMode::All).unwrap();
        assert_eq!(r.len(), 2);
        assert!(r.contains(&0));
        assert!(r.contains(&1));
    }

    #[test]
    fn test_connected_undirected() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = subcomponent(&g, 0, SubcomponentMode::All).unwrap();
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn test_directed_out() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 0).unwrap();
        let r = subcomponent(&g, 0, SubcomponentMode::Out).unwrap();
        assert_eq!(r, vec![0, 1, 2]);
    }

    #[test]
    fn test_directed_in() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 0).unwrap();
        let r = subcomponent(&g, 2, SubcomponentMode::In).unwrap();
        assert_eq!(r, vec![2, 1, 0, 3]);
    }

    #[test]
    fn test_directed_all() {
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 1).unwrap();
        g.add_edge(3, 4).unwrap();
        let r = subcomponent(&g, 0, SubcomponentMode::All).unwrap();
        assert_eq!(r.len(), 3);
        assert!(r.contains(&0));
        assert!(r.contains(&1));
        assert!(r.contains(&2));
    }

    #[test]
    fn test_isolated_vertex() {
        let g = Graph::with_vertices(5);
        let r = subcomponent(&g, 3, SubcomponentMode::All).unwrap();
        assert_eq!(r, vec![3]);
    }

    #[test]
    fn test_self_loop_ignored() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let r = subcomponent(&g, 0, SubcomponentMode::Out).unwrap();
        assert_eq!(r, vec![0, 1]);
    }

    #[test]
    fn test_cycle_directed_out() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let r = subcomponent(&g, 0, SubcomponentMode::Out).unwrap();
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn test_bfs_order() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        let r = subcomponent(&g, 0, SubcomponentMode::Out).unwrap();
        assert_eq!(r[0], 0);
        // 1 and 2 are BFS-level 1 (order depends on edge iteration)
        assert!(r[1] == 1 || r[1] == 2);
    }

    #[test]
    fn test_undirected_mode_ignored() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // All three modes should produce the same result for undirected
        let out = subcomponent(&g, 0, SubcomponentMode::Out).unwrap();
        let in_ = subcomponent(&g, 0, SubcomponentMode::In).unwrap();
        let all = subcomponent(&g, 0, SubcomponentMode::All).unwrap();
        assert_eq!(out.len(), 3);
        assert_eq!(in_.len(), 3);
        assert_eq!(all.len(), 3);
    }
}
