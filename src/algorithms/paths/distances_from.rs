//! Multi-source unweighted shortest distances (ALGO-SP-060).
//!
//! Counterpart of `igraph_distances()` (multi-source mode) from
//! `references/igraph/src/paths/unweighted.c`.
//!
//! Computes shortest-path distances from a set of source vertices to
//! all other vertices, using BFS from each source.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Direction mode for [`distances_from_with_mode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistancesFromMode {
    /// Follow outgoing edges.
    Out,
    /// Follow incoming edges.
    In,
    /// Ignore edge direction.
    All,
}

/// Compute shortest-path distances from multiple sources to all vertices.
///
/// Returns a flat `Vec<Option<u32>>` of length `sources.len() * n` in
/// row-major order, where `result[i * n + j]` is the shortest-path
/// distance from `sources[i]` to vertex `j`. `None` means unreachable.
///
/// For directed graphs, follows outgoing edges by default. Use
/// [`distances_from_with_mode`] for direction control.
///
/// # Errors
///
/// - `InvalidArgument` if any source vertex is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distances_from};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// let d = distances_from(&g, &[0, 2]).unwrap();
/// let n = 5;
/// // Row 0 (from vertex 0):
/// assert_eq!(d[0], Some(0));
/// assert_eq!(d[4], Some(4));
/// // Row 1 (from vertex 2):
/// assert_eq!(d[n + 2], Some(0));
/// assert_eq!(d[n], Some(2)); // vertex 2 to vertex 0
/// ```
pub fn distances_from(graph: &Graph, sources: &[VertexId]) -> IgraphResult<Vec<Option<u32>>> {
    let n = graph.vcount();
    validate_sources(sources, n)?;

    if sources.is_empty() {
        return Ok(Vec::new());
    }

    let n_us = n as usize;
    let total = sources.len().checked_mul(n_us).ok_or_else(|| {
        IgraphError::InvalidArgument("distances_from: result size overflows".into())
    })?;
    let mut result = vec![None; total];

    if graph.is_directed() {
        let adj = build_out_adj(graph, n_us)?;
        for (row, &src) in sources.iter().enumerate() {
            bfs_with_adj(&adj, src, n_us, row, &mut result);
        }
    } else {
        for (row, &src) in sources.iter().enumerate() {
            bfs_undirected(graph, src, n_us, row, &mut result)?;
        }
    }

    Ok(result)
}

/// Compute shortest-path distances from multiple sources with direction control.
///
/// For undirected graphs, `mode` is ignored.
///
/// # Errors
///
/// - `InvalidArgument` if any source vertex is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distances_from_with_mode, DistancesFromMode};
///
/// // Directed: 0→1→2→3
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let d = distances_from_with_mode(&g, &[0, 3], DistancesFromMode::Out).unwrap();
/// let n = 4;
/// assert_eq!(d[3], Some(3));     // 0→1→2→3
/// assert_eq!(d[n], None);        // 3 cannot reach 0 (Out)
/// let d_in = distances_from_with_mode(&g, &[3], DistancesFromMode::In).unwrap();
/// assert_eq!(d_in[0], Some(3));  // follow incoming from 3 back to 0
/// ```
pub fn distances_from_with_mode(
    graph: &Graph,
    sources: &[VertexId],
    mode: DistancesFromMode,
) -> IgraphResult<Vec<Option<u32>>> {
    let n = graph.vcount();
    validate_sources(sources, n)?;

    if sources.is_empty() {
        return Ok(Vec::new());
    }

    let n_us = n as usize;
    let total = sources.len().checked_mul(n_us).ok_or_else(|| {
        IgraphError::InvalidArgument("distances_from: result size overflows".into())
    })?;
    let mut result = vec![None; total];

    if !graph.is_directed() {
        for (row, &src) in sources.iter().enumerate() {
            bfs_undirected(graph, src, n_us, row, &mut result)?;
        }
        return Ok(result);
    }

    let adj = match mode {
        DistancesFromMode::Out => build_out_adj(graph, n_us)?,
        DistancesFromMode::In => build_in_adj(graph, n_us)?,
        DistancesFromMode::All => build_all_adj(graph, n_us)?,
    };

    for (row, &src) in sources.iter().enumerate() {
        bfs_with_adj(&adj, src, n_us, row, &mut result);
    }

    Ok(result)
}

fn validate_sources(sources: &[VertexId], n: u32) -> IgraphResult<()> {
    for &s in sources {
        if s >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "distances_from: source {s} out of range (vcount={n})"
            )));
        }
    }
    Ok(())
}

fn bfs_undirected(
    graph: &Graph,
    source: VertexId,
    n_us: usize,
    row: usize,
    result: &mut [Option<u32>],
) -> IgraphResult<()> {
    let offset = row * n_us;
    let mut visited = vec![false; n_us];
    let mut queue = VecDeque::new();

    visited[source as usize] = true;
    result[offset + source as usize] = Some(0);
    queue.push_back((source, 0u32));

    while let Some((cur, dist)) = queue.pop_front() {
        let next_dist = dist + 1;
        for nb in graph.neighbors_iter(cur)? {
            let nb_idx = nb as usize;
            if !visited[nb_idx] {
                visited[nb_idx] = true;
                result[offset + nb_idx] = Some(next_dist);
                queue.push_back((nb, next_dist));
            }
        }
    }

    Ok(())
}

fn bfs_with_adj(
    adj: &[Vec<VertexId>],
    source: VertexId,
    n_us: usize,
    row: usize,
    result: &mut [Option<u32>],
) {
    let offset = row * n_us;
    let mut visited = vec![false; n_us];
    let mut queue = VecDeque::new();

    visited[source as usize] = true;
    result[offset + source as usize] = Some(0);
    queue.push_back((source, 0u32));

    while let Some((cur, dist)) = queue.pop_front() {
        let next_dist = dist + 1;
        for &nb in &adj[cur as usize] {
            let nb_idx = nb as usize;
            if !visited[nb_idx] {
                visited[nb_idx] = true;
                result[offset + nb_idx] = Some(next_dist);
                queue.push_back((nb, next_dist));
            }
        }
    }
}

fn build_out_adj(graph: &Graph, n_us: usize) -> IgraphResult<Vec<Vec<VertexId>>> {
    let m =
        u32::try_from(graph.ecount()).map_err(|_| IgraphError::Internal("ecount overflows u32"))?;
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n_us];
    for eid in 0..m {
        let (from, to) = graph.edge(eid)?;
        adj[from as usize].push(to);
    }
    Ok(adj)
}

fn build_in_adj(graph: &Graph, n_us: usize) -> IgraphResult<Vec<Vec<VertexId>>> {
    let m =
        u32::try_from(graph.ecount()).map_err(|_| IgraphError::Internal("ecount overflows u32"))?;
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n_us];
    for eid in 0..m {
        let (from, to) = graph.edge(eid)?;
        adj[to as usize].push(from);
    }
    Ok(adj)
}

fn build_all_adj(graph: &Graph, n_us: usize) -> IgraphResult<Vec<Vec<VertexId>>> {
    let m =
        u32::try_from(graph.ecount()).map_err(|_| IgraphError::Internal("ecount overflows u32"))?;
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n_us];
    for eid in 0..m {
        let (from, to) = graph.edge(eid)?;
        adj[from as usize].push(to);
        if from != to {
            adj[to as usize].push(from);
        }
    }
    Ok(adj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_sources() {
        let g = Graph::with_vertices(3);
        let d = distances_from(&g, &[]).unwrap();
        assert!(d.is_empty());
    }

    #[test]
    fn single_source_matches_distances() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let d = distances_from(&g, &[0]).unwrap();
        assert_eq!(d, vec![Some(0), Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn multiple_sources() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let d = distances_from(&g, &[0, 3]).unwrap();
        let n = 4;
        // Row 0 (from 0): [0, 1, 2, 3]
        assert_eq!(d[0], Some(0));
        assert_eq!(d[1], Some(1));
        assert_eq!(d[2], Some(2));
        assert_eq!(d[3], Some(3));
        // Row 1 (from 3): [3, 2, 1, 0]
        assert_eq!(d[n], Some(3));
        assert_eq!(d[n + 1], Some(2));
        assert_eq!(d[n + 2], Some(1));
        assert_eq!(d[n + 3], Some(0));
    }

    #[test]
    fn disconnected() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let d = distances_from(&g, &[0]).unwrap();
        assert_eq!(d[0], Some(0));
        assert_eq!(d[1], Some(1));
        assert_eq!(d[2], None);
        assert_eq!(d[3], None);
    }

    #[test]
    fn source_out_of_range() {
        let g = Graph::with_vertices(3);
        assert!(distances_from(&g, &[5]).is_err());
    }

    #[test]
    fn directed_out() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = distances_from(&g, &[0, 2]).unwrap();
        let n = 3;
        // Row 0 (from 0, Out): [0, 1, 2]
        assert_eq!(d[0], Some(0));
        assert_eq!(d[1], Some(1));
        assert_eq!(d[2], Some(2));
        // Row 1 (from 2, Out): [None, None, 0]
        assert_eq!(d[n], None);
        assert_eq!(d[n + 1], None);
        assert_eq!(d[n + 2], Some(0));
    }

    #[test]
    fn directed_in_mode() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = distances_from_with_mode(&g, &[2], DistancesFromMode::In).unwrap();
        // Following incoming from 2: 2←1←0
        assert_eq!(d[0], Some(2));
        assert_eq!(d[1], Some(1));
        assert_eq!(d[2], Some(0));
    }

    #[test]
    fn directed_all_mode() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = distances_from_with_mode(&g, &[2], DistancesFromMode::All).unwrap();
        // All mode treats as undirected
        assert_eq!(d[0], Some(2));
        assert_eq!(d[1], Some(1));
        assert_eq!(d[2], Some(0));
    }

    #[test]
    fn duplicate_sources() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = distances_from(&g, &[0, 0]).unwrap();
        let n = 3;
        // Both rows should be identical
        assert_eq!(&d[..n], &d[n..]);
    }

    #[test]
    fn star_graph() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        let d = distances_from(&g, &[1, 2]).unwrap();
        let n = 5;
        // From vertex 1: [1, 0, 2, 2, 2]
        assert_eq!(d[0], Some(1));
        assert_eq!(d[1], Some(0));
        assert_eq!(d[2], Some(2));
        assert_eq!(d[3], Some(2));
        assert_eq!(d[4], Some(2));
        // From vertex 2: [1, 2, 0, 2, 2]
        assert_eq!(d[n], Some(1));
        assert_eq!(d[n + 1], Some(2));
        assert_eq!(d[n + 2], Some(0));
    }
}
