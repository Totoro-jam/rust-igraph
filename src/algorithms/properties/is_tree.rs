//! Tree predicate (ALGO-PR-023).
//!
//! Counterpart of `igraph_is_tree()` from
//! `references/igraph/src/properties/trees.c:251`. Returns
//! `Some(root)` iff `graph` is a tree (a connected, acyclic graph),
//! and `None` otherwise. The null graph (`vcount == 0`) is by
//! convention **not** a tree.
//!
//! For directed graphs the [`DijkstraMode`] argument selects the
//! orientation:
//! - [`DijkstraMode::Out`]: out-tree / arborescence — every edge
//!   points away from the root.
//! - [`DijkstraMode::In`]: in-tree / anti-arborescence — every edge
//!   points towards the root.
//! - [`DijkstraMode::All`]: ignore orientation; checks the
//!   underlying undirected structure.
//!
//! For undirected graphs the mode argument is ignored (matches
//! upstream).
//!
//! Time complexity: `O(V + E)` (linear scan + DFS).

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::core::Graph;
use crate::core::error::IgraphResult;
use crate::core::graph::{EdgeId, VertexId};

/// Returns `Some(root)` iff `graph` is a tree under `mode`,
/// otherwise `None`. The null graph (`vcount == 0`) is **not** a
/// tree by convention.
///
/// For undirected graphs the canonical root is `0`.
/// For directed graphs:
/// - [`DijkstraMode::Out`]: root is the unique vertex with
///   in-degree `0`.
/// - [`DijkstraMode::In`]: root is the unique vertex with
///   out-degree `0`.
/// - [`DijkstraMode::All`]: orientation ignored; canonical root
///   is `0`.
///
/// Counterpart of `igraph_is_tree(_, _, _, mode)` from
/// `references/igraph/src/properties/trees.c:251`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_tree, DijkstraMode};
///
/// // Undirected path 0-1-2-3: tree, root = 0.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert_eq!(is_tree(&g, DijkstraMode::All).unwrap(), Some(0));
///
/// // Out-arborescence rooted at 0: 0→1, 0→2, 1→3.
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 3)]).unwrap();
/// assert_eq!(is_tree(&g, DijkstraMode::Out).unwrap(), Some(0));
///
/// // 1→2←3→4 is an undirected forest, but not an out-tree.
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edges(vec![(0u32, 1u32), (2, 1), (2, 3)]).unwrap();
/// assert_eq!(is_tree(&g, DijkstraMode::Out).unwrap(), None);
/// // Treated as undirected, it has 3 edges on 4 vertices and is connected ⇒ tree.
/// assert_eq!(is_tree(&g, DijkstraMode::All).unwrap(), Some(0));
/// ```
pub fn is_tree(graph: &Graph, mode: DijkstraMode) -> IgraphResult<Option<VertexId>> {
    let n = graph.vcount();
    let m = graph.ecount();

    if n == 0 {
        return Ok(None);
    }

    // A tree on N vertices has exactly N-1 edges.
    if m as u64 != u64::from(n).saturating_sub(1) {
        return Ok(None);
    }

    // Trivial: single vertex with no edges is a tree.
    if n == 1 {
        return Ok(Some(0));
    }

    let directed = graph.is_directed();
    let effective_mode = if directed { mode } else { DijkstraMode::All };

    let Some(root) = pick_root(graph, n, effective_mode)? else {
        return Ok(None);
    };

    // DFS from root in `effective_mode` direction; count unique
    // visited vertices. With ecount = vcount-1 and all vertices
    // reached, the graph is necessarily acyclic and connected,
    // hence a tree.
    let visited_count = dfs_count(graph, n, root, effective_mode, directed)?;
    if visited_count == n {
        Ok(Some(root))
    } else {
        Ok(None)
    }
}

/// Locate the canonical root vertex per `mode`. Returns `None` if
/// the degree distribution disqualifies the graph from being a tree
/// in this orientation.
fn pick_root(graph: &Graph, n: VertexId, mode: DijkstraMode) -> IgraphResult<Option<VertexId>> {
    match mode {
        DijkstraMode::All => Ok(Some(0)),
        DijkstraMode::Out => find_zero_degree_root(graph, n, true),
        DijkstraMode::In => find_zero_degree_root(graph, n, false),
    }
}

/// Linear scan for the first zero-degree (in-/out-) vertex; bail
/// to `Ok(None)` on `degree > 1` (indicates a non-tree under this
/// orientation).
fn find_zero_degree_root(
    graph: &Graph,
    n: VertexId,
    use_in_degree: bool,
) -> IgraphResult<Option<VertexId>> {
    for v in 0..n {
        let d = if use_in_degree {
            graph.incident_in(v)?.len()
        } else {
            graph.incident(v)?.len()
        };
        if d == 0 {
            return Ok(Some(v));
        }
        if d > 1 {
            return Ok(None);
        }
    }
    Ok(None)
}

/// DFS from `root` in the orientation given by `mode` and count the
/// number of distinct vertices visited.
fn dfs_count(
    graph: &Graph,
    n: VertexId,
    root: VertexId,
    mode: DijkstraMode,
    directed: bool,
) -> IgraphResult<u32> {
    let n_us = n as usize;
    let mut visited: Vec<bool> = vec![false; n_us];
    let mut stack: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut count: u32 = 0;

    stack.push(root);
    while let Some(u) = stack.pop() {
        if visited[u as usize] {
            continue;
        }
        visited[u as usize] = true;
        count = count.saturating_add(1);

        for eid in incident_edges(graph, u, mode, directed)? {
            let nei = graph.edge_other(eid, u)?;
            if !visited[nei as usize] {
                stack.push(nei);
            }
        }
    }
    Ok(count)
}

/// Edges incident on `u` in the requested orientation.
fn incident_edges(
    graph: &Graph,
    u: VertexId,
    mode: DijkstraMode,
    directed: bool,
) -> IgraphResult<Vec<EdgeId>> {
    match mode {
        DijkstraMode::Out => graph.incident(u),
        DijkstraMode::In => graph.incident_in(u),
        DijkstraMode::All => {
            if directed {
                let mut out = graph.incident(u)?;
                out.extend(graph.incident_in(u)?);
                Ok(out)
            } else {
                graph.incident(u)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------- Null / single-vertex --------

    #[test]
    fn null_graph_is_not_a_tree() {
        let g = Graph::with_vertices(0);
        assert_eq!(is_tree(&g, DijkstraMode::All).unwrap(), None);
    }

    #[test]
    fn single_vertex_undirected_is_a_tree() {
        let g = Graph::with_vertices(1);
        assert_eq!(is_tree(&g, DijkstraMode::All).unwrap(), Some(0));
    }

    #[test]
    fn single_vertex_directed_out_is_a_tree() {
        let g = Graph::new(1, true).unwrap();
        assert_eq!(is_tree(&g, DijkstraMode::Out).unwrap(), Some(0));
    }

    // -------- Undirected --------

    #[test]
    fn undirected_path_is_a_tree() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3)]).unwrap();
        assert_eq!(is_tree(&g, DijkstraMode::All).unwrap(), Some(0));
    }

    #[test]
    fn undirected_star_is_a_tree() {
        let mut g = Graph::with_vertices(5);
        g.add_edges(vec![(0u32, 1u32), (0, 2), (0, 3), (0, 4)])
            .unwrap();
        assert_eq!(is_tree(&g, DijkstraMode::All).unwrap(), Some(0));
    }

    #[test]
    fn undirected_triangle_is_not_a_tree() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        assert_eq!(is_tree(&g, DijkstraMode::All).unwrap(), None);
    }

    #[test]
    fn undirected_disconnected_is_not_a_tree() {
        // Two disjoint edges: 0-1, 2-3. ecount=2, vcount=4 → fails ecount=vcount-1.
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (2, 3)]).unwrap();
        assert_eq!(is_tree(&g, DijkstraMode::All).unwrap(), None);
    }

    #[test]
    fn undirected_disconnected_correct_edge_count_is_not_a_tree() {
        // Two parallel + one isolated vertex: edges (0,1), (0,1), vcount=3.
        // ecount=2 = vcount-1 BUT DFS only reaches {0,1}, not 2.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert_eq!(is_tree(&g, DijkstraMode::All).unwrap(), None);
    }

    #[test]
    fn undirected_self_loop_breaks_edge_count() {
        // ecount=2 (self-loop + edge) = vcount-1=2 with vcount=3?
        // vcount=3, ecount=2. edges (0,0), (1,2). ecount = vcount-1 PASSES.
        // But DFS from 0 reaches only {0}, not {1,2}. → None.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(is_tree(&g, DijkstraMode::All).unwrap(), None);
    }

    // -------- Directed: OUT --------

    #[test]
    fn directed_out_arborescence_is_a_tree() {
        // 0→1, 0→2, 1→3.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 3)]).unwrap();
        assert_eq!(is_tree(&g, DijkstraMode::Out).unwrap(), Some(0));
    }

    #[test]
    fn directed_out_chain_is_a_tree() {
        // 0→1→2→3.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3)]).unwrap();
        assert_eq!(is_tree(&g, DijkstraMode::Out).unwrap(), Some(0));
    }

    #[test]
    fn directed_in_arborescence_is_not_an_out_tree() {
        // 1→0, 2→0, 3→1: every edge points TO root 0 — in-tree, not out-tree.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(1u32, 0u32), (2, 0), (3, 1)]).unwrap();
        assert_eq!(is_tree(&g, DijkstraMode::Out).unwrap(), None);
        assert_eq!(is_tree(&g, DijkstraMode::In).unwrap(), Some(0));
    }

    #[test]
    fn directed_v_pattern_to_center_is_not_an_out_tree() {
        // 0→2, 1→2: vertex 2 has in-degree 2.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 2u32), (1, 2)]).unwrap();
        assert_eq!(is_tree(&g, DijkstraMode::Out).unwrap(), None);
    }

    #[test]
    fn directed_cycle_is_not_a_tree() {
        // 0→1→2→0: 3 edges on 3 vertices, ecount ≠ vcount-1.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        assert_eq!(is_tree(&g, DijkstraMode::Out).unwrap(), None);
        assert_eq!(is_tree(&g, DijkstraMode::All).unwrap(), None);
    }

    #[test]
    fn directed_all_mode_treats_as_undirected() {
        // 1→2←3→4: not an out-tree, but undirected it's a path.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (2, 1), (2, 3)]).unwrap();
        assert_eq!(is_tree(&g, DijkstraMode::Out).unwrap(), None);
        assert_eq!(is_tree(&g, DijkstraMode::All).unwrap(), Some(0));
    }
}
