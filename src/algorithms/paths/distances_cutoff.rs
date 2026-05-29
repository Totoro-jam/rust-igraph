//! Unweighted shortest-path distances with cutoff (ALGO-SP-061).
//!
//! Counterpart of `igraph_i_distances_unweighted_cutoff()` from
//! `references/igraph/src/paths/unweighted.c:102-219`. BFS that stops
//! exploring vertices beyond a caller-specified distance limit.
//!
//! Single-source entry points mirror [`super::distances::distances`]
//! with an added `cutoff` parameter; multi-source entry points mirror
//! [`super::distances_from::distances_from`] with cutoff.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Direction mode for cutoff-aware unweighted distance functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistancesCutoffMode {
    /// Follow outgoing edges.
    Out,
    /// Follow incoming edges.
    In,
    /// Ignore edge direction.
    All,
}

/// Unweighted shortest-path lengths from `source` to every vertex,
/// ignoring paths longer than `cutoff`.
///
/// Vertices beyond the cutoff (or in a different component) get `None`.
/// `cutoff == None` behaves identically to [`super::distances::distances`].
///
/// Counterpart of `igraph_distances_cutoff(_, NULL, _, source, all,
/// IGRAPH_OUT, cutoff)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distances_cutoff};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
///
/// let d = distances_cutoff(&g, 0, Some(2)).unwrap();
/// assert_eq!(d, vec![Some(0), Some(1), Some(2), None, None]);
///
/// // No cutoff — same as `distances`.
/// let d_all = distances_cutoff(&g, 0, None).unwrap();
/// assert_eq!(d_all, vec![Some(0), Some(1), Some(2), Some(3), Some(4)]);
/// ```
pub fn distances_cutoff(
    graph: &Graph,
    source: VertexId,
    cutoff: Option<u32>,
) -> IgraphResult<Vec<Option<u32>>> {
    distances_cutoff_with_mode(graph, source, cutoff, DistancesCutoffMode::Out)
}

/// Mode-aware [`distances_cutoff`]. For undirected graphs `mode` is
/// ignored.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distances_cutoff_with_mode, DistancesCutoffMode};
///
/// // Directed: 0→1→2→3
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let d = distances_cutoff_with_mode(&g, 0, Some(1), DistancesCutoffMode::Out).unwrap();
/// assert_eq!(d, vec![Some(0), Some(1), None, None]);
///
/// let d_in = distances_cutoff_with_mode(&g, 3, Some(2), DistancesCutoffMode::In).unwrap();
/// assert_eq!(d_in, vec![None, Some(2), Some(1), Some(0)]);
/// ```
pub fn distances_cutoff_with_mode(
    graph: &Graph,
    source: VertexId,
    cutoff: Option<u32>,
    mode: DistancesCutoffMode,
) -> IgraphResult<Vec<Option<u32>>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }
    if source >= n {
        return Err(IgraphError::InvalidArgument(format!(
            "distances_cutoff: source {source} out of range (vcount={n})"
        )));
    }

    if !graph.is_directed() {
        return bfs_cutoff_undirected(graph, source, n as usize, cutoff);
    }

    let n_us = n as usize;
    let adj = match mode {
        DistancesCutoffMode::Out => build_out_adj(graph, n_us)?,
        DistancesCutoffMode::In => build_in_adj(graph, n_us)?,
        DistancesCutoffMode::All => build_all_adj(graph, n_us)?,
    };
    Ok(bfs_cutoff_with_adj(&adj, source, n_us, cutoff))
}

/// Multi-source unweighted distances with cutoff, direction-aware.
///
/// Returns a flat `Vec<Option<u32>>` of length `sources.len() * n` in
/// row-major order, where `result[i * n + j]` is the shortest-path
/// distance from `sources[i]` to vertex `j` (or `None` if unreachable /
/// past the cutoff).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distances_cutoff_multi, DistancesCutoffMode};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// let d = distances_cutoff_multi(&g, &[0, 2], Some(1), DistancesCutoffMode::Out).unwrap();
/// let n = 5;
/// // Row 0 (from 0, cutoff 1): only 0 and 1 reachable
/// assert_eq!(d[0], Some(0));
/// assert_eq!(d[1], Some(1));
/// assert_eq!(d[2], None);
/// // Row 1 (from 2, cutoff 1): 1, 2, 3
/// assert_eq!(d[n + 1], Some(1));
/// assert_eq!(d[n + 2], Some(0));
/// assert_eq!(d[n + 3], Some(1));
/// assert_eq!(d[n + 4], None);
/// ```
pub fn distances_cutoff_multi(
    graph: &Graph,
    sources: &[VertexId],
    cutoff: Option<u32>,
    mode: DistancesCutoffMode,
) -> IgraphResult<Vec<Option<u32>>> {
    let n = graph.vcount();
    for &s in sources {
        if s >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "distances_cutoff_multi: source {s} out of range (vcount={n})"
            )));
        }
    }

    if sources.is_empty() {
        return Ok(Vec::new());
    }

    let n_us = n as usize;
    let total = sources.len().checked_mul(n_us).ok_or_else(|| {
        IgraphError::InvalidArgument("distances_cutoff_multi: result size overflows".into())
    })?;
    let mut result = vec![None; total];

    if !graph.is_directed() {
        for (row, &src) in sources.iter().enumerate() {
            bfs_cutoff_undirected_into(graph, src, n_us, cutoff, row, &mut result)?;
        }
        return Ok(result);
    }

    let adj = match mode {
        DistancesCutoffMode::Out => build_out_adj(graph, n_us)?,
        DistancesCutoffMode::In => build_in_adj(graph, n_us)?,
        DistancesCutoffMode::All => build_all_adj(graph, n_us)?,
    };

    for (row, &src) in sources.iter().enumerate() {
        bfs_cutoff_with_adj_into(&adj, src, n_us, cutoff, row, &mut result);
    }

    Ok(result)
}

// ---- internal BFS helpers ----

fn bfs_cutoff_undirected(
    graph: &Graph,
    source: VertexId,
    n_us: usize,
    cutoff: Option<u32>,
) -> IgraphResult<Vec<Option<u32>>> {
    let mut dist: Vec<Option<u32>> = vec![None; n_us];
    let mut queue: VecDeque<(VertexId, u32)> = VecDeque::new();

    dist[source as usize] = Some(0);
    queue.push_back((source, 0));

    while let Some((v, d)) = queue.pop_front() {
        if let Some(c) = cutoff {
            if d >= c {
                continue;
            }
        }
        let next = d.checked_add(1).ok_or(IgraphError::Internal(
            "distance overflow (graph diameter exceeds u32::MAX)",
        ))?;
        for w in graph.neighbors(v)? {
            if dist[w as usize].is_none() {
                dist[w as usize] = Some(next);
                queue.push_back((w, next));
            }
        }
    }

    Ok(dist)
}

fn bfs_cutoff_undirected_into(
    graph: &Graph,
    source: VertexId,
    n_us: usize,
    cutoff: Option<u32>,
    row: usize,
    result: &mut [Option<u32>],
) -> IgraphResult<()> {
    let offset = row * n_us;
    let mut visited = vec![false; n_us];
    let mut queue: VecDeque<(VertexId, u32)> = VecDeque::new();

    visited[source as usize] = true;
    result[offset + source as usize] = Some(0);
    queue.push_back((source, 0));

    while let Some((v, d)) = queue.pop_front() {
        if let Some(c) = cutoff {
            if d >= c {
                continue;
            }
        }
        let next = d + 1;
        for w in graph.neighbors(v)? {
            let w_idx = w as usize;
            if !visited[w_idx] {
                visited[w_idx] = true;
                result[offset + w_idx] = Some(next);
                queue.push_back((w, next));
            }
        }
    }

    Ok(())
}

fn bfs_cutoff_with_adj(
    adj: &[Vec<VertexId>],
    source: VertexId,
    n_us: usize,
    cutoff: Option<u32>,
) -> Vec<Option<u32>> {
    let mut dist: Vec<Option<u32>> = vec![None; n_us];
    let mut queue: VecDeque<(VertexId, u32)> = VecDeque::new();

    dist[source as usize] = Some(0);
    queue.push_back((source, 0));

    while let Some((v, d)) = queue.pop_front() {
        if let Some(c) = cutoff {
            if d >= c {
                continue;
            }
        }
        let next = d + 1;
        for &w in &adj[v as usize] {
            if dist[w as usize].is_none() {
                dist[w as usize] = Some(next);
                queue.push_back((w, next));
            }
        }
    }

    dist
}

fn bfs_cutoff_with_adj_into(
    adj: &[Vec<VertexId>],
    source: VertexId,
    n_us: usize,
    cutoff: Option<u32>,
    row: usize,
    result: &mut [Option<u32>],
) {
    let offset = row * n_us;
    let mut visited = vec![false; n_us];

    visited[source as usize] = true;
    result[offset + source as usize] = Some(0);
    let mut queue: VecDeque<(VertexId, u32)> = VecDeque::new();
    queue.push_back((source, 0));

    while let Some((v, d)) = queue.pop_front() {
        if let Some(c) = cutoff {
            if d >= c {
                continue;
            }
        }
        let next = d + 1;
        for &w in &adj[v as usize] {
            let w_idx = w as usize;
            if !visited[w_idx] {
                visited[w_idx] = true;
                result[offset + w_idx] = Some(next);
                queue.push_back((w, next));
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

    // ---- distances_cutoff (single-source) ----

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert_eq!(
            distances_cutoff(&g, 0, Some(5)).unwrap(),
            Vec::<Option<u32>>::new()
        );
    }

    #[test]
    fn single_vertex_no_edges() {
        let g = Graph::with_vertices(1);
        assert_eq!(distances_cutoff(&g, 0, Some(0)).unwrap(), vec![Some(0)]);
    }

    #[test]
    fn cutoff_zero_returns_only_source() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = distances_cutoff(&g, 0, Some(0)).unwrap();
        assert_eq!(d, vec![Some(0), None, None]);
    }

    #[test]
    fn cutoff_one_returns_direct_neighbors() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let d = distances_cutoff(&g, 0, Some(1)).unwrap();
        assert_eq!(d, vec![Some(0), Some(1), None, None]);
    }

    #[test]
    fn cutoff_covers_entire_graph() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let d = distances_cutoff(&g, 0, Some(100)).unwrap();
        assert_eq!(d, vec![Some(0), Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn none_cutoff_same_as_no_limit() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let d = distances_cutoff(&g, 0, None).unwrap();
        assert_eq!(d, vec![Some(0), Some(1), Some(2), Some(3), Some(4)]);
    }

    #[test]
    fn disconnected_components() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let d = distances_cutoff(&g, 0, Some(10)).unwrap();
        assert_eq!(d, vec![Some(0), Some(1), None, None]);
    }

    #[test]
    fn invalid_source() {
        let g = Graph::with_vertices(3);
        assert!(distances_cutoff(&g, 5, Some(1)).is_err());
    }

    #[test]
    fn directed_out_cutoff() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let d = distances_cutoff(&g, 0, Some(2)).unwrap();
        assert_eq!(d, vec![Some(0), Some(1), Some(2), None]);
    }

    #[test]
    fn directed_in_mode_cutoff() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let d = distances_cutoff_with_mode(&g, 3, Some(1), DistancesCutoffMode::In).unwrap();
        assert_eq!(d, vec![None, None, Some(1), Some(0)]);
    }

    #[test]
    fn directed_all_mode_cutoff() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = distances_cutoff_with_mode(&g, 2, Some(1), DistancesCutoffMode::All).unwrap();
        assert_eq!(d, vec![None, Some(1), Some(0)]);
    }

    #[test]
    fn cycle_with_cutoff() {
        let mut g = Graph::with_vertices(6);
        for i in 0..5u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        g.add_edge(5, 0).unwrap();
        let d = distances_cutoff(&g, 0, Some(2)).unwrap();
        assert_eq!(d, vec![Some(0), Some(1), Some(2), None, Some(2), Some(1)]);
    }

    #[test]
    fn self_loop_with_cutoff() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let d = distances_cutoff(&g, 0, Some(1)).unwrap();
        assert_eq!(d, vec![Some(0), Some(1)]);
    }

    // ---- distances_cutoff_multi ----

    #[test]
    fn multi_empty_sources() {
        let g = Graph::with_vertices(3);
        assert!(
            distances_cutoff_multi(&g, &[], Some(1), DistancesCutoffMode::Out)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn multi_basic() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let d = distances_cutoff_multi(&g, &[0, 4], Some(2), DistancesCutoffMode::Out).unwrap();
        let n = 5;
        // Row 0 (from 0): 0,1,2,None,None
        assert_eq!(d[0], Some(0));
        assert_eq!(d[1], Some(1));
        assert_eq!(d[2], Some(2));
        assert_eq!(d[3], None);
        assert_eq!(d[4], None);
        // Row 1 (from 4): None,None,2,1,0
        assert_eq!(d[n], None);
        assert_eq!(d[n + 1], None);
        assert_eq!(d[n + 2], Some(2));
        assert_eq!(d[n + 3], Some(1));
        assert_eq!(d[n + 4], Some(0));
    }

    #[test]
    fn multi_directed_in() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = distances_cutoff_multi(&g, &[2], Some(1), DistancesCutoffMode::In).unwrap();
        assert_eq!(d, vec![None, Some(1), Some(0)]);
    }

    #[test]
    fn multi_invalid_source() {
        let g = Graph::with_vertices(3);
        assert!(distances_cutoff_multi(&g, &[0, 10], Some(1), DistancesCutoffMode::Out).is_err());
    }

    #[test]
    fn consistency_with_uncutoff_distances() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 0).unwrap();

        let full = crate::algorithms::paths::distances::distances(&g, 0).unwrap();
        let cutoff_large = distances_cutoff(&g, 0, Some(100)).unwrap();
        assert_eq!(full, cutoff_large);
    }
}
