//! All shortest paths from a single source (ALGO-SP-059).
//!
//! Counterpart of `igraph_get_all_shortest_paths()` from
//! `references/igraph/src/paths/all_shortest_paths.c`.
//!
//! Returns every shortest path from a source vertex to all reachable
//! targets, using BFS with multi-parent tracking.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Direction mode for [`get_all_shortest_paths`] and
/// [`get_all_shortest_paths_with_mode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllShortestPathsMode {
    /// Follow outgoing edges.
    Out,
    /// Follow incoming edges.
    In,
    /// Ignore edge direction.
    All,
}

/// Result of [`get_all_shortest_paths`] /
/// [`get_all_shortest_paths_with_mode`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllShortestPaths {
    /// All shortest paths grouped by target vertex. `paths[v]` is the
    /// list of all shortest paths from the source to vertex `v`. If `v`
    /// is unreachable, `paths[v]` is empty.
    pub paths: Vec<Vec<Vec<VertexId>>>,
    /// Number of shortest paths to each vertex. `nrgeo[v]` is the count
    /// of shortest paths from source to `v` (0 if unreachable, 1 for
    /// the source itself).
    pub nrgeo: Vec<u64>,
}

/// Find all shortest paths from `source` to every reachable vertex.
///
/// Uses BFS (unweighted edges). For directed graphs, follows outgoing
/// edges by default. Use [`get_all_shortest_paths_with_mode`] for
/// direction control.
///
/// # Errors
///
/// - `InvalidArgument` if `source >= vcount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_all_shortest_paths};
///
/// // Diamond: 0-1, 0-2, 1-3, 2-3. Two shortest paths from 0 to 3.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let r = get_all_shortest_paths(&g, 0).unwrap();
/// assert_eq!(r.paths[3].len(), 2); // two paths to vertex 3
/// assert_eq!(r.nrgeo[3], 2);
/// assert_eq!(r.paths[0], vec![vec![0]]); // self-path
/// ```
pub fn get_all_shortest_paths(graph: &Graph, source: VertexId) -> IgraphResult<AllShortestPaths> {
    if graph.is_directed() {
        return get_all_shortest_paths_directed(graph, source, DirectedMode::Out);
    }
    get_all_shortest_paths_undirected(graph, source)
}

/// Find all shortest paths from `source` with direction control.
///
/// For undirected graphs, `mode` is ignored.
///
/// # Errors
///
/// - `InvalidArgument` if `source >= vcount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_all_shortest_paths_with_mode, AllShortestPathsMode};
///
/// // Directed diamond: 0→1, 0→2, 1→3, 2→3.
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let r = get_all_shortest_paths_with_mode(&g, 0, AllShortestPathsMode::Out).unwrap();
/// assert_eq!(r.paths[3].len(), 2);
/// // In mode from vertex 3
/// let r_in = get_all_shortest_paths_with_mode(&g, 3, AllShortestPathsMode::In).unwrap();
/// assert_eq!(r_in.paths[0].len(), 2); // two paths back to 0
/// ```
pub fn get_all_shortest_paths_with_mode(
    graph: &Graph,
    source: VertexId,
    mode: AllShortestPathsMode,
) -> IgraphResult<AllShortestPaths> {
    if !graph.is_directed() {
        return get_all_shortest_paths_undirected(graph, source);
    }
    let dir_mode = match mode {
        AllShortestPathsMode::Out => DirectedMode::Out,
        AllShortestPathsMode::In => DirectedMode::In,
        AllShortestPathsMode::All => DirectedMode::All,
    };
    get_all_shortest_paths_directed(graph, source, dir_mode)
}

#[derive(Clone, Copy)]
enum DirectedMode {
    Out,
    In,
    All,
}

fn get_all_shortest_paths_undirected(
    graph: &Graph,
    source: VertexId,
) -> IgraphResult<AllShortestPaths> {
    let n = graph.vcount();
    if source >= n {
        return Err(IgraphError::InvalidArgument(format!(
            "get_all_shortest_paths: source {source} out of range (vcount={n})"
        )));
    }

    let n_us = n as usize;
    let mut dist: Vec<Option<u32>> = vec![None; n_us];
    let mut parents: Vec<Vec<VertexId>> = vec![Vec::new(); n_us];
    let mut nrgeo: Vec<u64> = vec![0; n_us];

    dist[source as usize] = Some(0);
    nrgeo[source as usize] = 1;

    let mut queue = VecDeque::new();
    queue.push_back(source);

    while let Some(cur) = queue.pop_front() {
        let cur_dist = dist[cur as usize].unwrap_or(0);
        let next_dist = cur_dist + 1;
        let neighbors = graph.neighbors(cur)?;
        for &nb in &neighbors {
            let nb_idx = nb as usize;
            let nb_dist = dist[nb_idx];
            match nb_dist {
                None => {
                    dist[nb_idx] = Some(next_dist);
                    parents[nb_idx].push(cur);
                    nrgeo[nb_idx] = nrgeo[cur as usize];
                    queue.push_back(nb);
                }
                Some(d) if d == next_dist => {
                    parents[nb_idx].push(cur);
                    nrgeo[nb_idx] += nrgeo[cur as usize];
                }
                _ => {}
            }
        }
    }

    let paths = enumerate_all_paths(source, n_us, &parents, &dist);
    Ok(AllShortestPaths { paths, nrgeo })
}

fn get_all_shortest_paths_directed(
    graph: &Graph,
    source: VertexId,
    mode: DirectedMode,
) -> IgraphResult<AllShortestPaths> {
    let n = graph.vcount();
    if source >= n {
        return Err(IgraphError::InvalidArgument(format!(
            "get_all_shortest_paths: source {source} out of range (vcount={n})"
        )));
    }

    let n_us = n as usize;
    let adj = build_adj(graph, n_us, mode)?;

    let mut dist: Vec<Option<u32>> = vec![None; n_us];
    let mut parents: Vec<Vec<VertexId>> = vec![Vec::new(); n_us];
    let mut nrgeo: Vec<u64> = vec![0; n_us];

    dist[source as usize] = Some(0);
    nrgeo[source as usize] = 1;

    let mut queue = VecDeque::new();
    queue.push_back(source);

    while let Some(cur) = queue.pop_front() {
        let cur_dist = dist[cur as usize].unwrap_or(0);
        let next_dist = cur_dist + 1;
        for &nb in &adj[cur as usize] {
            let nb_idx = nb as usize;
            let nb_dist = dist[nb_idx];
            match nb_dist {
                None => {
                    dist[nb_idx] = Some(next_dist);
                    parents[nb_idx].push(cur);
                    nrgeo[nb_idx] = nrgeo[cur as usize];
                    queue.push_back(nb);
                }
                Some(d) if d == next_dist => {
                    parents[nb_idx].push(cur);
                    nrgeo[nb_idx] += nrgeo[cur as usize];
                }
                _ => {}
            }
        }
    }

    let paths = enumerate_all_paths(source, n_us, &parents, &dist);
    Ok(AllShortestPaths { paths, nrgeo })
}

fn build_adj(graph: &Graph, n_us: usize, mode: DirectedMode) -> IgraphResult<Vec<Vec<VertexId>>> {
    let m =
        u32::try_from(graph.ecount()).map_err(|_| IgraphError::Internal("ecount overflows u32"))?;
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n_us];
    for eid in 0..m {
        let (from, to) = graph.edge(eid)?;
        match mode {
            DirectedMode::Out => adj[from as usize].push(to),
            DirectedMode::In => adj[to as usize].push(from),
            DirectedMode::All => {
                adj[from as usize].push(to);
                if from != to {
                    adj[to as usize].push(from);
                }
            }
        }
    }
    Ok(adj)
}

fn enumerate_all_paths(
    source: VertexId,
    n_us: usize,
    parents: &[Vec<VertexId>],
    dist: &[Option<u32>],
) -> Vec<Vec<Vec<VertexId>>> {
    let mut result: Vec<Vec<Vec<VertexId>>> = vec![Vec::new(); n_us];

    result[source as usize] = vec![vec![source]];

    if n_us <= 1 {
        return result;
    }

    // Process vertices in BFS order (by distance).
    let mut order: Vec<(u32, usize)> = Vec::with_capacity(n_us);
    for (v, d) in dist.iter().enumerate() {
        if let Some(dv) = d {
            order.push((*dv, v));
        }
    }
    order.sort_unstable();

    for &(_, v) in &order {
        #[allow(clippy::cast_possible_truncation)]
        let v_id = v as u32;
        if v_id == source {
            continue;
        }
        let mut paths_to_v: Vec<Vec<VertexId>> = Vec::new();
        for &p in &parents[v] {
            for parent_path in &result[p as usize] {
                let mut path = parent_path.clone();
                path.push(v_id);
                paths_to_v.push(path);
            }
        }
        result[v] = paths_to_v;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_err() {
        let g = Graph::with_vertices(0);
        assert!(get_all_shortest_paths(&g, 0).is_err());
    }

    #[test]
    fn singleton() {
        let g = Graph::with_vertices(1);
        let r = get_all_shortest_paths(&g, 0).unwrap();
        assert_eq!(r.paths[0], vec![vec![0]]);
        assert_eq!(r.nrgeo[0], 1);
    }

    #[test]
    fn path_graph_unique() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = get_all_shortest_paths(&g, 0).unwrap();
        assert_eq!(r.paths[0], vec![vec![0]]);
        assert_eq!(r.paths[1], vec![vec![0, 1]]);
        assert_eq!(r.paths[2], vec![vec![0, 1, 2]]);
        assert_eq!(r.paths[3], vec![vec![0, 1, 2, 3]]);
        for v in 0..4 {
            assert_eq!(r.nrgeo[v], 1);
        }
    }

    #[test]
    fn diamond_two_paths() {
        // 0-1, 0-2, 1-3, 2-3: two shortest paths from 0 to 3.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = get_all_shortest_paths(&g, 0).unwrap();
        assert_eq!(r.nrgeo[3], 2);
        assert_eq!(r.paths[3].len(), 2);
        let mut sorted_paths = r.paths[3].clone();
        sorted_paths.sort();
        assert_eq!(sorted_paths, vec![vec![0, 1, 3], vec![0, 2, 3]]);
    }

    #[test]
    fn two_components_unreachable() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = get_all_shortest_paths(&g, 0).unwrap();
        assert_eq!(r.paths[1], vec![vec![0, 1]]);
        assert!(r.paths[2].is_empty());
        assert!(r.paths[3].is_empty());
        assert_eq!(r.nrgeo[2], 0);
        assert_eq!(r.nrgeo[3], 0);
    }

    #[test]
    fn cycle_5_multiple_shortest() {
        // C5: 0-1-2-3-4-0. From 0 to 3: two paths of length 2
        // (0-4-3 and 0-1-2-3 are NOT same length; 0-4-3 is length 2, 0-1-2-3 is length 3)
        // Actually from 0: paths to 3 are 0-1-2-3 (len 3) and 0-4-3 (len 2).
        // Wait: 0-4-3. Edge 4-0 means 0's neighbors include 4. Edge 3-4.
        // Dist from 0: 1→{1,4}, 2→{2,3}, so shortest to 3 is 2 via 0-4-3.
        // But also 0-1-2 is dist 2 to vertex 2; and 2-3 is dist 1.
        // So dist(0,3) = 2 via 0-4-3. But 0-1-2-3 = dist 3. So only ONE shortest path.
        // Actually edges are 0-1, 1-2, 2-3, 3-4, 4-0.
        // From 0: neighbors are 1 and 4 (dist 1).
        // From 1: neighbor 2 (dist 2). From 4: neighbor 3 (dist 2).
        // So vertex 3 reached at dist 2 via 4. Vertex 2 reached at dist 2 via 1.
        // From 2: neighbor 3 at dist 3 > 2 (already found). So path to 3: only 0-4-3.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        let r = get_all_shortest_paths(&g, 0).unwrap();
        assert_eq!(r.nrgeo[1], 1);
        assert_eq!(r.nrgeo[4], 1);
        assert_eq!(r.nrgeo[2], 1); // only via 0-1-2
        assert_eq!(r.nrgeo[3], 1); // only via 0-4-3
    }

    #[test]
    fn grid_multiple_paths() {
        // 2x3 grid:
        // 0-1-2
        // |   |
        // 3-4-5
        // From 0 to 5: shortest is length 3.
        // Paths: 0-1-2-5 and 0-3-4-5 and 0-1-4-5 and 0-3-4-... wait.
        // Edges: 0-1, 1-2, 0-3, 2-5, 3-4, 4-5
        // Plus 1-4 for full grid? No. Let's add 1-4 to make it a full grid.
        // Actually let me construct it carefully:
        // 0-1, 1-2 (top row), 3-4, 4-5 (bottom row), 0-3, 1-4, 2-5 (columns)
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        let r = get_all_shortest_paths(&g, 0).unwrap();
        // Dist to 5: 0→1→2→5 (3), 0→1→4→5 (3), 0→3→4→5 (3)
        assert_eq!(r.nrgeo[5], 3);
        assert_eq!(r.paths[5].len(), 3);
        let mut sorted = r.paths[5].clone();
        sorted.sort();
        assert_eq!(
            sorted,
            vec![vec![0, 1, 2, 5], vec![0, 1, 4, 5], vec![0, 3, 4, 5]]
        );
    }

    #[test]
    fn directed_out() {
        // Directed diamond: 0→1, 0→2, 1→3, 2→3.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = get_all_shortest_paths(&g, 0).unwrap();
        assert_eq!(r.nrgeo[3], 2);
        assert_eq!(r.paths[3].len(), 2);
    }

    #[test]
    fn directed_in() {
        // Directed diamond: 0→1, 0→2, 1→3, 2→3. From 3, In mode.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = get_all_shortest_paths_with_mode(&g, 3, AllShortestPathsMode::In).unwrap();
        assert_eq!(r.nrgeo[0], 2);
        assert_eq!(r.paths[0].len(), 2);
        let mut sorted = r.paths[0].clone();
        sorted.sort();
        assert_eq!(sorted, vec![vec![3, 1, 0], vec![3, 2, 0]]);
    }

    #[test]
    fn directed_all() {
        // 0→1→2. All mode from 2.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = get_all_shortest_paths_with_mode(&g, 2, AllShortestPathsMode::All).unwrap();
        assert_eq!(r.paths[0], vec![vec![2, 1, 0]]);
        assert_eq!(r.paths[1], vec![vec![2, 1]]);
    }

    #[test]
    fn directed_unreachable() {
        // 0→1→2. Out mode from 2.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = get_all_shortest_paths(&g, 2).unwrap();
        assert_eq!(r.paths[2], vec![vec![2]]);
        assert!(r.paths[0].is_empty());
        assert!(r.paths[1].is_empty());
    }

    #[test]
    fn source_out_of_range() {
        let g = Graph::with_vertices(3);
        assert!(get_all_shortest_paths(&g, 5).is_err());
    }

    #[test]
    fn self_loop_no_effect() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = get_all_shortest_paths(&g, 0).unwrap();
        assert_eq!(r.paths[2], vec![vec![0, 1, 2]]);
        assert_eq!(r.nrgeo[2], 1);
    }

    #[test]
    fn nrgeo_star() {
        // Star: center 0, leaves 1,2,3.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let r = get_all_shortest_paths(&g, 0).unwrap();
        assert_eq!(r.nrgeo[0], 1);
        for leaf in 1u32..4 {
            assert_eq!(r.nrgeo[leaf as usize], 1);
            assert_eq!(r.paths[leaf as usize], vec![vec![0, leaf]]);
        }
        // From leaf 1: paths to 2 and 3 go through center.
        let r1 = get_all_shortest_paths(&g, 1).unwrap();
        assert_eq!(r1.nrgeo[2], 1);
        assert_eq!(r1.paths[2], vec![vec![1, 0, 2]]);
    }
}
