//! Connectivity predicate (ALGO-CC-050).
//!
//! Checks whether a graph is connected without computing full component structure.
//! Counterpart of `igraph_is_connected`.

use crate::core::{Graph, IgraphResult};

/// Connectivity mode for directed graphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectednessMode {
    /// Weak connectivity (ignore edge directions).
    Weak,
    /// Strong connectivity (respect edge directions).
    Strong,
}

/// Tests whether the graph is connected.
///
/// For undirected graphs, `mode` is ignored and the function checks
/// whether all vertices are reachable from each other.
///
/// For directed graphs:
/// - `Weak`: checks if the underlying undirected graph is connected.
/// - `Strong`: checks if every vertex is reachable from every other vertex
///   following edge directions.
///
/// An empty graph (0 vertices) is considered connected. A graph with
/// a single vertex is always connected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_connected, ConnectednessMode};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_connected(&g, ConnectednessMode::Weak).unwrap());
///
/// // Disconnected graph
/// let g = Graph::with_vertices(4);
/// assert!(!is_connected(&g, ConnectednessMode::Weak).unwrap());
///
/// // Directed: weakly but not strongly connected
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert!(is_connected(&g, ConnectednessMode::Weak).unwrap());
/// assert!(!is_connected(&g, ConnectednessMode::Strong).unwrap());
/// ```
pub fn is_connected(graph: &Graph, mode: ConnectednessMode) -> IgraphResult<bool> {
    let n = graph.vcount();

    if n <= 1 {
        return Ok(true);
    }

    if mode == ConnectednessMode::Strong && graph.is_directed() {
        return is_strongly_connected(graph);
    }

    is_weakly_connected(graph)
}

fn is_weakly_connected(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    let nn = n as usize;

    // Build symmetric adjacency list from all edges
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); nn];
    let ecount = graph.ecount();
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        if src != tgt {
            adj[src as usize].push(tgt);
            adj[tgt as usize].push(src);
        }
    }

    // BFS from vertex 0
    let mut visited = vec![false; nn];
    let mut queue = std::collections::VecDeque::new();

    visited[0] = true;
    queue.push_back(0u32);
    let mut count = 1u32;

    while let Some(v) = queue.pop_front() {
        for &nb in &adj[v as usize] {
            if !visited[nb as usize] {
                visited[nb as usize] = true;
                count += 1;
                queue.push_back(nb);
            }
        }
    }

    Ok(count == n)
}

fn is_strongly_connected(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    let nn = n as usize;
    let ecount = graph.ecount();

    // Build forward and reverse adjacency lists
    let mut fwd: Vec<Vec<u32>> = vec![Vec::new(); nn];
    let mut rev: Vec<Vec<u32>> = vec![Vec::new(); nn];
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        if src != tgt {
            fwd[src as usize].push(tgt);
            rev[tgt as usize].push(src);
        }
    }

    // Forward BFS from vertex 0
    let forward_count = bfs_count_adj(&fwd, n, 0);
    if forward_count != n {
        return Ok(false);
    }

    // Backward BFS from vertex 0 (using reverse adj)
    let backward_count = bfs_count_adj(&rev, n, 0);
    Ok(backward_count == n)
}

fn bfs_count_adj(adj: &[Vec<u32>], n: u32, start: u32) -> u32 {
    let mut visited = vec![false; n as usize];
    let mut queue = std::collections::VecDeque::new();

    visited[start as usize] = true;
    queue.push_back(start);
    let mut count = 1u32;

    while let Some(v) = queue.pop_front() {
        for &nb in &adj[v as usize] {
            if !visited[nb as usize] {
                visited[nb as usize] = true;
                count += 1;
                queue.push_back(nb);
            }
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_connected(&g, ConnectednessMode::Weak).unwrap());
    }

    #[test]
    fn test_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_connected(&g, ConnectednessMode::Weak).unwrap());
        assert!(is_connected(&g, ConnectednessMode::Strong).unwrap());
    }

    #[test]
    fn test_two_connected() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_connected(&g, ConnectednessMode::Weak).unwrap());
    }

    #[test]
    fn test_two_disconnected() {
        let g = Graph::with_vertices(2);
        assert!(!is_connected(&g, ConnectednessMode::Weak).unwrap());
    }

    #[test]
    fn test_path_connected() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_connected(&g, ConnectednessMode::Weak).unwrap());
    }

    #[test]
    fn test_two_components() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_connected(&g, ConnectednessMode::Weak).unwrap());
    }

    #[test]
    fn test_directed_weakly_connected() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_connected(&g, ConnectednessMode::Weak).unwrap());
    }

    #[test]
    fn test_directed_not_strongly_connected() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_connected(&g, ConnectednessMode::Strong).unwrap());
    }

    #[test]
    fn test_directed_strongly_connected() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_connected(&g, ConnectednessMode::Strong).unwrap());
    }

    #[test]
    fn test_complete_graph() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_connected(&g, ConnectednessMode::Weak).unwrap());
    }

    #[test]
    fn test_star_connected() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_connected(&g, ConnectednessMode::Weak).unwrap());
    }

    #[test]
    fn test_directed_star_weak_not_strong() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(is_connected(&g, ConnectednessMode::Weak).unwrap());
        assert!(!is_connected(&g, ConnectednessMode::Strong).unwrap());
    }
}
