//! Connect-neighborhood / graph-power operators (ALGO-OP-011).
//!
//! Connects each vertex to all vertices reachable within k steps.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphResult, VertexId};

/// Returns a new graph where each vertex is connected to all vertices
/// reachable within `order` steps in the original graph.
///
/// Existing edges are preserved. Only new edges (not already present) are
/// added. Self-loops are never added. For undirected graphs, only one edge
/// per pair is created.
///
/// This is equivalent to computing the k-th power of a graph and simplifying
/// (removing multi-edges and self-loops).
///
/// # Arguments
///
/// * `graph` — the input graph (undirected).
/// * `order` — the maximum distance within which vertices are connected.
///   Order < 2 leaves the graph unchanged.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, connect_neighborhood};
///
/// // Path graph: 0-1-2-3
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let cg = connect_neighborhood(&g, 2).unwrap();
/// assert_eq!(cg.vcount(), 4);
/// // Original 3 edges + new edges: (0,2), (1,3) = 5 total
/// assert_eq!(cg.ecount(), 5);
/// ```
pub fn connect_neighborhood(graph: &Graph, order: u32) -> IgraphResult<Graph> {
    let n = graph.vcount();
    let directed = graph.is_directed();

    if order < 2 || n == 0 {
        // Return a structural copy
        let mut result = Graph::new(n, directed)?;
        let ecount = graph.ecount();
        let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);
        for eid in 0..ecount {
            #[allow(clippy::cast_possible_truncation)]
            let eid_u32 = eid as u32;
            edges.push(graph.edge(eid_u32)?);
        }
        result.add_edges(edges)?;
        return Ok(result);
    }

    // Build adjacency list from the original graph
    let adj = build_adjacency_list(graph, directed)?;

    // Collect new edges via BFS from each vertex
    let mut new_edges: Vec<(VertexId, VertexId)> = Vec::new();
    let mut visited = vec![0u32; n as usize];

    for i in 0..n {
        let marker = i + 1;
        visited[i as usize] = marker;

        // Mark direct neighbors as visited
        for &nei in &adj[i as usize] {
            visited[nei as usize] = marker;
        }

        // BFS to find vertices at distance 2..order
        let mut queue: VecDeque<(VertexId, u32)> = VecDeque::new();
        for &nei in &adj[i as usize] {
            queue.push_back((nei, 1));
        }

        while let Some((node, dist)) = queue.pop_front() {
            if dist >= order {
                continue;
            }
            for &nei in &adj[node as usize] {
                if visited[nei as usize] != marker {
                    visited[nei as usize] = marker;
                    if directed || i < nei {
                        new_edges.push((i, nei));
                    }
                    if dist + 1 < order {
                        queue.push_back((nei, dist + 1));
                    }
                }
            }
        }
    }

    // Build result with original edges + new edges
    let ecount = graph.ecount();
    let mut all_edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount + new_edges.len());
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let eid_u32 = eid as u32;
        all_edges.push(graph.edge(eid_u32)?);
    }
    all_edges.extend(new_edges);

    let mut result = Graph::new(n, directed)?;
    result.add_edges(all_edges)?;
    Ok(result)
}

/// Returns the k-th power of a graph as a simple graph.
///
/// In the k-th power, vertex u is connected to vertex v if v is reachable
/// from u within at most k steps. The result is always simple (no self-loops,
/// no multi-edges).
///
/// By convention, the zeroth power has no edges. The first power is the
/// simplified original graph.
///
/// # Arguments
///
/// * `graph` — the input graph.
/// * `order` — non-negative integer, the power to raise the graph to.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, graph_power};
///
/// // Path: 0-1-2-3
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let p2 = graph_power(&g, 2).unwrap();
/// assert_eq!(p2.vcount(), 4);
/// // Edges: (0,1),(0,2),(1,2),(1,3),(2,3) = 5
/// assert_eq!(p2.ecount(), 5);
/// ```
pub fn graph_power(graph: &Graph, order: u32) -> IgraphResult<Graph> {
    let n = graph.vcount();
    let directed = graph.is_directed();

    if order == 0 {
        return Graph::new(n, directed);
    }

    // Build adjacency (simple, no loops) from original
    let adj = build_simple_adjacency_list(graph, directed)?;

    if order == 1 {
        // First power = simplified original
        return build_graph_from_adj(&adj, n, directed);
    }

    // BFS from each vertex to find all within `order` distance
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    let mut visited = vec![0u32; n as usize];

    for i in 0..n {
        let marker = i + 1;
        visited[i as usize] = marker;

        let mut queue: VecDeque<(VertexId, u32)> = VecDeque::new();

        // Seed with direct neighbors
        for &nei in &adj[i as usize] {
            if visited[nei as usize] != marker {
                visited[nei as usize] = marker;
                if directed || i < nei {
                    edges.push((i, nei));
                }
                if order > 1 {
                    queue.push_back((nei, 1));
                }
            }
        }

        // Expand BFS
        while let Some((node, dist)) = queue.pop_front() {
            if dist >= order {
                continue;
            }
            for &nei in &adj[node as usize] {
                if visited[nei as usize] != marker {
                    visited[nei as usize] = marker;
                    if directed || i < nei {
                        edges.push((i, nei));
                    }
                    if dist + 1 < order {
                        queue.push_back((nei, dist + 1));
                    }
                }
            }
        }
    }

    let mut result = Graph::new(n, directed)?;
    result.add_edges(edges)?;
    Ok(result)
}

fn build_adjacency_list(graph: &Graph, _directed: bool) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount() as usize;
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];
    let ecount = graph.ecount();

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let eid_u32 = eid as u32;
        let (src, tgt) = graph.edge(eid_u32)?;
        adj[src as usize].push(tgt);
        if !graph.is_directed() && src != tgt {
            adj[tgt as usize].push(src);
        }
    }
    Ok(adj)
}

fn build_simple_adjacency_list(graph: &Graph, _directed: bool) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount() as usize;
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];
    let ecount = graph.ecount();

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let eid_u32 = eid as u32;
        let (src, tgt) = graph.edge(eid_u32)?;
        if src == tgt {
            continue; // skip self-loops
        }
        adj[src as usize].push(tgt);
        if !graph.is_directed() {
            adj[tgt as usize].push(src);
        }
    }

    // Deduplicate
    for list in &mut adj {
        list.sort_unstable();
        list.dedup();
    }
    Ok(adj)
}

fn build_graph_from_adj(adj: &[Vec<VertexId>], n: u32, directed: bool) -> IgraphResult<Graph> {
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    for (i, neighbors) in adj.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let i_u32 = i as u32;
        for &nei in neighbors {
            if directed || i_u32 < nei {
                edges.push((i_u32, nei));
            }
        }
    }
    let mut result = Graph::new(n, directed)?;
    result.add_edges(edges)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_neighborhood_path() {
        // Path: 0-1-2-3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let cg = connect_neighborhood(&g, 2).unwrap();
        assert_eq!(cg.vcount(), 4);
        // 3 original + 2 new (0-2, 1-3)
        assert_eq!(cg.ecount(), 5);
    }

    #[test]
    fn test_connect_neighborhood_order_1() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();

        let cg = connect_neighborhood(&g, 1).unwrap();
        assert_eq!(cg.ecount(), 1); // unchanged
    }

    #[test]
    fn test_connect_neighborhood_order_0() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();

        let cg = connect_neighborhood(&g, 0).unwrap();
        assert_eq!(cg.ecount(), 1); // unchanged
    }

    #[test]
    fn test_connect_neighborhood_complete() {
        // K3 — order 2 should not add edges (already fully connected)
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();

        let cg = connect_neighborhood(&g, 2).unwrap();
        assert_eq!(cg.ecount(), 3);
    }

    #[test]
    fn test_connect_neighborhood_large_order() {
        // Path: 0-1-2-3, order=10 → complete graph K4 = 6 edges
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let cg = connect_neighborhood(&g, 10).unwrap();
        assert_eq!(cg.ecount(), 6); // K4
    }

    #[test]
    fn test_connect_neighborhood_empty() {
        let g = Graph::with_vertices(0);
        let cg = connect_neighborhood(&g, 5).unwrap();
        assert_eq!(cg.vcount(), 0);
    }

    #[test]
    fn test_connect_neighborhood_isolated() {
        let g = Graph::with_vertices(5);
        let cg = connect_neighborhood(&g, 3).unwrap();
        assert_eq!(cg.ecount(), 0);
    }

    #[test]
    fn test_graph_power_zero() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();

        let p = graph_power(&g, 0).unwrap();
        assert_eq!(p.vcount(), 3);
        assert_eq!(p.ecount(), 0);
    }

    #[test]
    fn test_graph_power_one() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let p = graph_power(&g, 1).unwrap();
        assert_eq!(p.vcount(), 3);
        assert_eq!(p.ecount(), 2);
    }

    #[test]
    fn test_graph_power_two_path() {
        // Path: 0-1-2-3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let p = graph_power(&g, 2).unwrap();
        assert_eq!(p.vcount(), 4);
        // (0,1),(0,2),(1,2),(1,3),(2,3) = 5
        assert_eq!(p.ecount(), 5);
    }

    #[test]
    fn test_graph_power_removes_self_loops() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap(); // self-loop
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let p = graph_power(&g, 1).unwrap();
        // Self-loop removed
        assert_eq!(p.ecount(), 2);
    }

    #[test]
    fn test_graph_power_removes_multi_edges() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap(); // duplicate
        g.add_edge(1, 2).unwrap();

        let p = graph_power(&g, 1).unwrap();
        assert_eq!(p.ecount(), 2);
    }

    #[test]
    fn test_graph_power_directed() {
        // Directed path: 0→1→2
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let p = graph_power(&g, 2).unwrap();
        assert!(p.is_directed());
        assert_eq!(p.vcount(), 3);
        // (0→1),(1→2),(0→2) = 3
        assert_eq!(p.ecount(), 3);
    }
}
