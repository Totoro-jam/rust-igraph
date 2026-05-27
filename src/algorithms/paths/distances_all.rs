//! All-pairs unweighted shortest distances (ALGO-SP-058).
//!
//! Counterpart of `igraph_distances()` (multi-source mode) from
//! `references/igraph/src/paths/unweighted.c`.
//!
//! Computes shortest-path distances between all pairs of vertices
//! using BFS from each source. Returns an n×n flat matrix.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// All-pairs unweighted shortest distances.
///
/// Returns a flat `Vec<Option<u32>>` of length `n * n` in row-major
/// order, where `result[i * n + j]` is the shortest-path distance
/// from vertex `i` to vertex `j`. `None` means unreachable.
///
/// For undirected graphs, the matrix is symmetric. For directed
/// graphs, follows outgoing edges by default; use
/// [`distances_all_with_mode`] for direction control.
///
/// # Errors
///
/// Returns an error if internal BFS encounters an issue (should not
/// happen for valid graphs).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distances_all};
///
/// // Triangle: all distances are 0 or 1.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let d = distances_all(&g).unwrap();
/// assert_eq!(d[0 * 3 + 1], Some(1)); // 0→1
/// assert_eq!(d[0 * 3 + 2], Some(1)); // 0→2
/// assert_eq!(d[1 * 3 + 2], Some(1)); // 1→2
/// assert_eq!(d[0 * 3 + 0], Some(0)); // self
/// ```
pub fn distances_all(graph: &Graph) -> IgraphResult<Vec<Option<u32>>> {
    let n = graph.vcount();
    let n_us = n as usize;

    if n == 0 {
        return Ok(Vec::new());
    }

    let mut result = vec![
        None;
        n_us.checked_mul(n_us).ok_or_else(|| {
            IgraphError::InvalidArgument("distances_all: n*n overflows usize".into())
        })?
    ];

    if graph.is_directed() {
        let adj = build_out_adj(graph, n_us)?;
        for src in 0..n {
            bfs_distances_with_adj(&adj, src, n_us, &mut result);
        }
    } else {
        for src in 0..n {
            bfs_distances_undirected(graph, src, n_us, &mut result)?;
        }
    }

    Ok(result)
}

/// Direction mode for [`distances_all_with_mode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistancesMode {
    /// Follow outgoing edges.
    Out,
    /// Follow incoming edges.
    In,
    /// Ignore edge direction.
    All,
}

/// All-pairs shortest distances with direction control.
///
/// For undirected graphs, `mode` is ignored.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distances_all_with_mode, DistancesMode};
///
/// // Directed: 0→1→2
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let d = distances_all_with_mode(&g, DistancesMode::Out).unwrap();
/// assert_eq!(d[0 * 3 + 2], Some(2)); // 0→1→2
/// assert_eq!(d[2 * 3 + 0], None);    // 2 cannot reach 0
/// let d_in = distances_all_with_mode(&g, DistancesMode::In).unwrap();
/// assert_eq!(d_in[2 * 3 + 0], Some(2)); // follow incoming from 2
/// ```
pub fn distances_all_with_mode(
    graph: &Graph,
    mode: DistancesMode,
) -> IgraphResult<Vec<Option<u32>>> {
    let n = graph.vcount();
    let n_us = n as usize;

    if n == 0 {
        return Ok(Vec::new());
    }

    let mut result = vec![
        None;
        n_us.checked_mul(n_us).ok_or_else(|| {
            IgraphError::InvalidArgument("distances_all_with_mode: n*n overflows usize".into())
        })?
    ];

    if !graph.is_directed() {
        for src in 0..n {
            bfs_distances_undirected(graph, src, n_us, &mut result)?;
        }
        return Ok(result);
    }

    let adj = match mode {
        DistancesMode::Out => build_out_adj(graph, n_us)?,
        DistancesMode::In => build_in_adj(graph, n_us)?,
        DistancesMode::All => build_all_adj(graph, n_us)?,
    };

    for src in 0..n {
        bfs_distances_with_adj(&adj, src, n_us, &mut result);
    }

    Ok(result)
}

/// BFS from `source` using `graph.neighbors()` (undirected).
fn bfs_distances_undirected(
    graph: &Graph,
    source: VertexId,
    n_us: usize,
    result: &mut [Option<u32>],
) -> IgraphResult<()> {
    let src_us = source as usize;
    let row_offset = src_us * n_us;

    let mut visited = vec![false; n_us];
    let mut queue = VecDeque::new();

    visited[src_us] = true;
    result[row_offset + src_us] = Some(0);
    queue.push_back((source, 0u32));

    while let Some((cur, dist)) = queue.pop_front() {
        let neighbors = graph.neighbors(cur)?;
        let next_dist = dist + 1;
        for &nb in &neighbors {
            let nb_idx = nb as usize;
            if !visited[nb_idx] {
                visited[nb_idx] = true;
                result[row_offset + nb_idx] = Some(next_dist);
                queue.push_back((nb, next_dist));
            }
        }
    }

    Ok(())
}

/// BFS from `source` using a pre-built adjacency list.
fn bfs_distances_with_adj(
    adj: &[Vec<VertexId>],
    source: VertexId,
    n_us: usize,
    result: &mut [Option<u32>],
) {
    let src_us = source as usize;
    let row_offset = src_us * n_us;

    let mut visited = vec![false; n_us];
    let mut queue = VecDeque::new();

    visited[src_us] = true;
    result[row_offset + src_us] = Some(0);
    queue.push_back((source, 0u32));

    while let Some((cur, dist)) = queue.pop_front() {
        let next_dist = dist + 1;
        for &nb in &adj[cur as usize] {
            let nb_idx = nb as usize;
            if !visited[nb_idx] {
                visited[nb_idx] = true;
                result[row_offset + nb_idx] = Some(next_dist);
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
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let d = distances_all(&g).unwrap();
        assert!(d.is_empty());
    }

    #[test]
    fn singleton() {
        let g = Graph::with_vertices(1);
        let d = distances_all(&g).unwrap();
        assert_eq!(d, vec![Some(0)]);
    }

    #[test]
    fn path_graph() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let d = distances_all(&g).unwrap();
        let n = 4usize;
        assert_eq!(d[0], Some(0)); // row 0, col 0
        assert_eq!(d[1], Some(1)); // row 0, col 1
        assert_eq!(d[2], Some(2)); // row 0, col 2
        assert_eq!(d[3], Some(3)); // row 0, col 3
        assert_eq!(d[3 * n], Some(3)); // row 3, col 0
        assert_eq!(d[n + 3], Some(2)); // row 1, col 3
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let d = distances_all(&g).unwrap();
        let n = 3;
        for i in 0..n {
            assert_eq!(d[i * n + i], Some(0));
            for j in 0..n {
                if i != j {
                    assert_eq!(d[i * n + j], Some(1));
                }
            }
        }
    }

    #[test]
    fn two_components() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let d = distances_all(&g).unwrap();
        let n = 4usize;
        assert_eq!(d[1], Some(1)); // row 0, col 1
        assert_eq!(d[2 * n + 3], Some(1));
        assert_eq!(d[2], None); // row 0, col 2
        assert_eq!(d[n + 3], None); // row 1, col 3
    }

    #[test]
    fn cycle_5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        let d = distances_all(&g).unwrap();
        // Row 0: distances from vertex 0
        assert_eq!(d[0], Some(0));
        assert_eq!(d[1], Some(1));
        assert_eq!(d[2], Some(2));
        assert_eq!(d[3], Some(2));
        assert_eq!(d[4], Some(1));
    }

    #[test]
    fn directed_out() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = distances_all(&g).unwrap();
        let n = 3usize;
        assert_eq!(d[2], Some(2)); // row 0, col 2
        assert_eq!(d[2 * n], None); // row 2, col 0
        assert_eq!(d[n], None); // row 1, col 0
    }

    #[test]
    fn directed_in_mode() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = distances_all_with_mode(&g, DistancesMode::In).unwrap();
        let n = 3usize;
        // Following incoming edges: from 2 we can reach 1 (in 1 hop) and 0 (in 2 hops)
        assert_eq!(d[2 * n + 1], Some(1));
        assert_eq!(d[2 * n], Some(2)); // row 2, col 0
        assert_eq!(d[1], None); // row 0, col 1: 0 has no incoming
    }

    #[test]
    fn directed_all_mode() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = distances_all_with_mode(&g, DistancesMode::All).unwrap();
        let n = 3;
        // All mode: treat as undirected
        assert_eq!(d[2], Some(2));
        assert_eq!(d[2 * n], Some(2));
    }

    #[test]
    fn symmetric_undirected() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let d = distances_all(&g).unwrap();
        let n = 5;
        for i in 0..n {
            for j in 0..n {
                assert_eq!(d[i * n + j], d[j * n + i], "not symmetric at ({i},{j})");
            }
        }
    }

    #[test]
    fn isolated_vertices() {
        let g = Graph::with_vertices(3);
        let d = distances_all(&g).unwrap();
        let n = 3;
        for i in 0..n {
            assert_eq!(d[i * n + i], Some(0));
            for j in 0..n {
                if i != j {
                    assert_eq!(d[i * n + j], None);
                }
            }
        }
    }

    #[test]
    fn oracle_star() {
        // Star: center 0, leaves 1,2,3.
        // Verified against python-igraph.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let d = distances_all(&g).unwrap();
        let n = 4;
        // Center to leaves: 1
        for item in d.iter().take(4).skip(1) {
            assert_eq!(*item, Some(1));
        }
        // Leaf to leaf: 2
        assert_eq!(d[n + 2], Some(2));
        assert_eq!(d[n + 3], Some(2));
        assert_eq!(d[2 * n + 3], Some(2));
    }
}
