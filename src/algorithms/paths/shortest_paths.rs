//! Unweighted shortest paths (ALGO-SP-057).
//!
//! Counterpart of `igraph_get_shortest_paths()` from
//! `references/igraph/src/paths/unweighted.c`.
//!
//! Returns one shortest path (as a vertex sequence) from a source
//! vertex to every reachable target in the graph, using BFS.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Returns one shortest path from `source` to every vertex in the graph.
///
/// Uses BFS (unweighted edges). The result is a `Vec` of length
/// `vcount()`, where `result[v]` is the vertex sequence of a shortest
/// path from `source` to `v` (inclusive at both ends). If `v` is
/// unreachable from `source`, `result[v]` is empty.
///
/// For directed graphs, follows edges in the outgoing direction by
/// default. Use [`get_shortest_paths_with_mode`] to control direction.
///
/// # Errors
///
/// - `InvalidArgument` if `source >= vcount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_shortest_paths};
///
/// // Path 0-1-2-3
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let paths = get_shortest_paths(&g, 0).unwrap();
/// assert_eq!(paths[0], vec![0]);
/// assert_eq!(paths[1], vec![0, 1]);
/// assert_eq!(paths[2], vec![0, 1, 2]);
/// assert_eq!(paths[3], vec![0, 1, 2, 3]);
/// ```
pub fn get_shortest_paths(graph: &Graph, source: VertexId) -> IgraphResult<Vec<Vec<VertexId>>> {
    if graph.is_directed() {
        return get_shortest_paths_directed(graph, source, true);
    }
    get_shortest_paths_impl(graph, source)
}

/// Direction mode for [`get_shortest_paths_with_mode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortestPathMode {
    /// Follow outgoing edges (directed graphs).
    Out,
    /// Follow incoming edges (directed graphs).
    In,
    /// Ignore edge direction.
    All,
}

/// Returns one shortest path from `source` to every vertex, with
/// direction control for directed graphs.
///
/// For undirected graphs, `mode` is ignored and all edges are
/// traversed bidirectionally.
///
/// # Errors
///
/// - `InvalidArgument` if `source >= vcount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_shortest_paths_with_mode, ShortestPathMode};
///
/// // Directed: 0→1→2
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// // Forward
/// let fwd = get_shortest_paths_with_mode(&g, 0, ShortestPathMode::Out).unwrap();
/// assert_eq!(fwd[2], vec![0, 1, 2]);
/// // Backward from 2
/// let bwd = get_shortest_paths_with_mode(&g, 2, ShortestPathMode::In).unwrap();
/// assert_eq!(bwd[0], vec![2, 1, 0]);
/// ```
pub fn get_shortest_paths_with_mode(
    graph: &Graph,
    source: VertexId,
    mode: ShortestPathMode,
) -> IgraphResult<Vec<Vec<VertexId>>> {
    if !graph.is_directed() {
        return get_shortest_paths_impl(graph, source);
    }
    match mode {
        ShortestPathMode::Out => get_shortest_paths_directed(graph, source, true),
        ShortestPathMode::In => get_shortest_paths_directed(graph, source, false),
        ShortestPathMode::All => get_shortest_paths_directed_all(graph, source),
    }
}

/// BFS implementation for undirected graphs (uses `graph.neighbors`).
fn get_shortest_paths_impl(graph: &Graph, source: VertexId) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();
    if source >= n {
        return Err(IgraphError::InvalidArgument(format!(
            "get_shortest_paths: source {source} out of range (vcount={n})"
        )));
    }

    let n_us = n as usize;
    let mut parent: Vec<Option<VertexId>> = vec![None; n_us];
    let mut visited = vec![false; n_us];

    let mut queue = VecDeque::new();
    visited[source as usize] = true;
    queue.push_back(source);

    while let Some(cur) = queue.pop_front() {
        let neighbors = graph.neighbors(cur)?;
        for &nb in &neighbors {
            if !visited[nb as usize] {
                visited[nb as usize] = true;
                parent[nb as usize] = Some(cur);
                queue.push_back(nb);
            }
        }
    }

    Ok(build_paths(source, n_us, &parent, &visited))
}

/// BFS for directed graphs with explicit direction control.
fn get_shortest_paths_directed(
    graph: &Graph,
    source: VertexId,
    follow_out: bool,
) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();
    if source >= n {
        return Err(IgraphError::InvalidArgument(format!(
            "get_shortest_paths: source {source} out of range (vcount={n})"
        )));
    }

    let n_us = n as usize;
    let m =
        u32::try_from(graph.ecount()).map_err(|_| IgraphError::Internal("ecount overflows u32"))?;

    // Build adjacency lists in the desired direction.
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n_us];
    for eid in 0..m {
        let (from, to) = graph.edge(eid)?;
        if follow_out {
            adj[from as usize].push(to);
        } else {
            adj[to as usize].push(from);
        }
    }

    let mut parent: Vec<Option<VertexId>> = vec![None; n_us];
    let mut visited = vec![false; n_us];

    let mut queue = VecDeque::new();
    visited[source as usize] = true;
    queue.push_back(source);

    while let Some(cur) = queue.pop_front() {
        for &nb in &adj[cur as usize] {
            if !visited[nb as usize] {
                visited[nb as usize] = true;
                parent[nb as usize] = Some(cur);
                queue.push_back(nb);
            }
        }
    }

    Ok(build_paths(source, n_us, &parent, &visited))
}

/// BFS for directed graphs with `All` mode (ignore edge direction).
fn get_shortest_paths_directed_all(
    graph: &Graph,
    source: VertexId,
) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();
    if source >= n {
        return Err(IgraphError::InvalidArgument(format!(
            "get_shortest_paths: source {source} out of range (vcount={n})"
        )));
    }

    let n_us = n as usize;
    let m =
        u32::try_from(graph.ecount()).map_err(|_| IgraphError::Internal("ecount overflows u32"))?;

    // Build symmetric adjacency lists (both directions).
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n_us];
    for eid in 0..m {
        let (from, to) = graph.edge(eid)?;
        adj[from as usize].push(to);
        if from != to {
            adj[to as usize].push(from);
        }
    }

    let mut parent: Vec<Option<VertexId>> = vec![None; n_us];
    let mut visited = vec![false; n_us];

    let mut queue = VecDeque::new();
    visited[source as usize] = true;
    queue.push_back(source);

    while let Some(cur) = queue.pop_front() {
        for &nb in &adj[cur as usize] {
            if !visited[nb as usize] {
                visited[nb as usize] = true;
                parent[nb as usize] = Some(cur);
                queue.push_back(nb);
            }
        }
    }

    Ok(build_paths(source, n_us, &parent, &visited))
}

/// Reconstruct paths from the parent array.
fn build_paths(
    source: VertexId,
    n_us: usize,
    parent: &[Option<VertexId>],
    visited: &[bool],
) -> Vec<Vec<VertexId>> {
    let mut result: Vec<Vec<VertexId>> = Vec::with_capacity(n_us);
    for (v, &is_visited) in visited.iter().enumerate().take(n_us) {
        if !is_visited {
            result.push(Vec::new());
            continue;
        }
        #[allow(clippy::cast_possible_truncation)] // v < n which is u32
        let v_id = v as u32;
        if v_id == source {
            result.push(vec![source]);
            continue;
        }
        let mut path = Vec::new();
        let mut cur = v_id;
        loop {
            path.push(cur);
            if cur == source {
                break;
            }
            cur = parent[cur as usize].unwrap_or(source);
        }
        path.reverse();
        result.push(path);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        // source out of range for empty graph — but 0 >= 0, err
        // Actually vcount=0, any source is invalid
        assert!(get_shortest_paths(&g, 0).is_err());
    }

    #[test]
    fn singleton() {
        let g = Graph::with_vertices(1);
        let paths = get_shortest_paths(&g, 0).unwrap();
        assert_eq!(paths, vec![vec![0]]);
    }

    #[test]
    fn path_graph() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let paths = get_shortest_paths(&g, 0).unwrap();
        assert_eq!(paths[0], vec![0]);
        assert_eq!(paths[1], vec![0, 1]);
        assert_eq!(paths[2], vec![0, 1, 2]);
        assert_eq!(paths[3], vec![0, 1, 2, 3]);
    }

    #[test]
    fn two_components_unreachable() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let paths = get_shortest_paths(&g, 0).unwrap();
        assert_eq!(paths[0], vec![0]);
        assert_eq!(paths[1], vec![0, 1]);
        assert!(paths[2].is_empty());
        assert!(paths[3].is_empty());
    }

    #[test]
    fn shortest_path_prefers_shorter() {
        // Graph: 0-1-2-3 and 0-3 (direct edge).
        // Shortest path from 0 to 3 should be [0, 3].
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(0, 3).unwrap();
        let paths = get_shortest_paths(&g, 0).unwrap();
        assert_eq!(paths[3].len(), 2); // direct path length 2 (vertices)
        assert_eq!(paths[3][0], 0);
        assert_eq!(paths[3][1], 3);
    }

    #[test]
    fn cycle_graph() {
        // 5-cycle: 0-1-2-3-4-0. From 0, path to 3 can be 0-1-2-3 or 0-4-3.
        // Both length 3. BFS finds one of them.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        let paths = get_shortest_paths(&g, 0).unwrap();
        // Path to 3 should have length 3 (3 vertices).
        assert_eq!(paths[3].len(), 3);
        assert_eq!(paths[3][0], 0);
        assert_eq!(*paths[3].last().unwrap(), 3);
    }

    #[test]
    fn directed_out() {
        // 0→1→2, no path from 0 to anywhere except forward.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let paths = get_shortest_paths_with_mode(&g, 0, ShortestPathMode::Out).unwrap();
        assert_eq!(paths[0], vec![0]);
        assert_eq!(paths[1], vec![0, 1]);
        assert_eq!(paths[2], vec![0, 1, 2]);
    }

    #[test]
    fn directed_in() {
        // 0→1→2. From vertex 2, following incoming edges.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let paths = get_shortest_paths_with_mode(&g, 2, ShortestPathMode::In).unwrap();
        assert_eq!(paths[2], vec![2]);
        assert_eq!(paths[1], vec![2, 1]);
        assert_eq!(paths[0], vec![2, 1, 0]);
    }

    #[test]
    fn directed_unreachable() {
        // 0→1→2. From 2, out mode: nothing reachable.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let paths = get_shortest_paths_with_mode(&g, 2, ShortestPathMode::Out).unwrap();
        assert_eq!(paths[2], vec![2]);
        assert!(paths[0].is_empty());
        assert!(paths[1].is_empty());
    }

    #[test]
    fn directed_all_mode() {
        // 0→1→2. All mode: treat as undirected.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let paths = get_shortest_paths_with_mode(&g, 2, ShortestPathMode::All).unwrap();
        assert_eq!(paths[2], vec![2]);
        assert_eq!(paths[1], vec![2, 1]);
        assert_eq!(paths[0], vec![2, 1, 0]);
    }

    #[test]
    fn star_graph() {
        // Star: center 0, leaves 1,2,3,4.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        let paths = get_shortest_paths(&g, 0).unwrap();
        for leaf in 1..5 {
            assert_eq!(paths[leaf as usize], vec![0, leaf]);
        }
        // From leaf 1 to leaf 2: [1, 0, 2].
        let paths_1 = get_shortest_paths(&g, 1).unwrap();
        assert_eq!(paths_1[2], vec![1, 0, 2]);
    }

    #[test]
    fn source_out_of_range() {
        let g = Graph::with_vertices(3);
        assert!(get_shortest_paths(&g, 5).is_err());
    }

    #[test]
    fn self_loop_ignored() {
        // Vertex 1 has a self-loop. Should not affect path finding.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let paths = get_shortest_paths(&g, 0).unwrap();
        assert_eq!(paths[2], vec![0, 1, 2]);
    }

    #[test]
    fn oracle_6_vertices() {
        // Verified against python-igraph.
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        let paths = get_shortest_paths(&g, 0).unwrap();
        assert_eq!(paths[0], vec![0]);
        assert_eq!(paths[1], vec![0, 1]);
        assert_eq!(paths[4], vec![0, 4]);
        assert_eq!(paths[5], vec![0, 4, 5]);
        // Path to 3: either via 1-2-3 or 4-5-3 (both length 3 vertices).
        assert_eq!(paths[3].len(), 4);
        assert_eq!(paths[3][0], 0);
        assert_eq!(*paths[3].last().unwrap(), 3);
    }
}
