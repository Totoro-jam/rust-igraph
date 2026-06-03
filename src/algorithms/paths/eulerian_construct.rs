//! Eulerian path / cycle construction (ALGO-CC-041).
//!
//! Counterpart of `igraph_eulerian_path()` and `igraph_eulerian_cycle()`
//! from `references/igraph/src/paths/eulerian.c:345-450` (the undirected
//! Hierholzer driver). Returns the sequence of edge ids that traverse
//! every edge exactly once.
//!
//! Phase-1 minimal slice: undirected only. Directed Hierholzer (different
//! adjacency tracking — see `igraph_i_eulerian_path_directed` at
//! `eulerian.c:453+`) ships in CC-042.

use crate::algorithms::paths::eulerian::is_eulerian;
use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Build an Eulerian cycle if one exists, or return an error.
///
/// Unlike [`eulerian_path`], this function requires that every vertex
/// with nonzero degree has even degree (undirected) or equal in-/out-degree
/// (directed). Returns `Err` if no Eulerian cycle exists.
///
/// Counterpart of `igraph_eulerian_cycle()` from
/// `references/igraph/src/paths/eulerian.c:594`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, eulerian_cycle};
///
/// // Triangle 0-1-2-0: every vertex has even degree → cycle exists.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let cycle = eulerian_cycle(&g).unwrap();
/// assert_eq!(cycle.len(), 3);
///
/// // Path 0-1-2: has Euler path but no Euler cycle → error.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert!(eulerian_cycle(&g).is_err());
/// ```
pub fn eulerian_cycle(graph: &Graph) -> IgraphResult<Vec<EdgeId>> {
    let cls = is_eulerian(graph)?;
    if !cls.has_cycle {
        return Err(IgraphError::InvalidArgument(
            "the graph does not have an Eulerian cycle".to_string(),
        ));
    }

    let m = graph.ecount();
    if m == 0 {
        return Ok(Vec::new());
    }

    // Delegate to the Hierholzer implementation in eulerian_path.
    // Since has_cycle is true, eulerian_path will also succeed and
    // produce a cycle (the walk starts and ends at the same vertex).
    match eulerian_path(graph)? {
        Some(edges) => Ok(edges),
        None => Err(IgraphError::Internal(
            "has_cycle is true but eulerian_path returned None",
        )),
    }
}

/// Build an Eulerian path or cycle for `graph` if one exists.
/// Returns `Some(edge_ids)` (the walk visits each edge exactly once)
/// or `None` if no Eulerian walk exists.
///
/// Counterpart of `igraph_eulerian_path()` (returns the path if any)
/// from `references/igraph/src/paths/eulerian.c:345`. Undirected only
/// in this slice; directed graphs return an error.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, eulerian_path};
///
/// // Triangle 0-1-2-0: every vertex has even degree → Euler cycle exists.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();   // edge 0
/// g.add_edge(1, 2).unwrap();   // edge 1
/// g.add_edge(2, 0).unwrap();   // edge 2
/// let walk = eulerian_path(&g).unwrap().unwrap();
/// assert_eq!(walk.len(), 3);
///
/// // Path 0-1-2: 2 odd-degree vertices → Euler path (no cycle).
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let walk = eulerian_path(&g).unwrap().unwrap();
/// assert_eq!(walk.len(), 2);
///
/// // K4: 4 odd-degree vertices → no Euler path.
/// let mut g = Graph::with_vertices(4);
/// for u in 0..4u32 {
///     for v in (u + 1)..4 {
///         g.add_edge(u, v).unwrap();
///     }
/// }
/// assert_eq!(eulerian_path(&g).unwrap(), None);
/// ```
pub fn eulerian_path(graph: &Graph) -> IgraphResult<Option<Vec<EdgeId>>> {
    let cls = is_eulerian(graph)?;
    if !cls.has_path {
        return Ok(None);
    }

    let n = graph.vcount();
    let m = graph.ecount();
    if m == 0 {
        // Empty walk for graphs with no edges (still trivially Eulerian).
        return Ok(Some(Vec::new()));
    }

    let n_us = n as usize;
    let m_us = m;
    let directed = graph.is_directed();

    // Per-vertex incident-edge lists. For undirected graphs `incident()`
    // returns LOOPS_TWICE (self-loops appear twice); upstream's Hierholzer
    // uses LOOPS_ONCE so we dedupe. For directed graphs we use the `oi`-side
    // out-edges directly via `incident()`'s directed branch (same semantics
    // as upstream's `IGRAPH_OUT` mode).
    let mut inc: Vec<Vec<EdgeId>> = Vec::with_capacity(n_us);
    for v in 0..n {
        let raw = graph.incident(v)?;
        if directed {
            // Already out-only and one entry per edge.
            inc.push(raw);
        } else {
            let mut seen: std::collections::HashSet<EdgeId> = std::collections::HashSet::new();
            let mut out: Vec<EdgeId> = Vec::with_capacity(raw.len());
            for e in raw {
                if seen.insert(e) {
                    out.push(e);
                }
            }
            inc.push(out);
        }
    }

    // Track simple "remaining degree" per vertex via the count of unvisited
    // incident edges. Upstream uses `igraph_degree(_, IGRAPH_LOOPS)` which
    // counts self-loops twice; we use the pre-built inc list and the visited
    // bitset.
    let mut visited: Vec<bool> = vec![false; m_us];
    let mut next_idx: Vec<usize> = vec![0; n_us];

    // Pick the start vertex: per upstream's logic in is_eulerian helpers,
    // it's an odd-degree vertex if `has_path && !has_cycle`, otherwise
    // any vertex with a non-zero unvisited incident edge.
    let start_of_path = pick_start_vertex(graph, cls)?;

    // Hierholzer's algorithm (iterative). Two stacks: `tracker` is the
    // current walk; `path` is the output (built in reverse).
    let mut tracker: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut edge_tracker: Vec<EdgeId> = Vec::with_capacity(m_us);
    let mut path: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut edge_path: Vec<EdgeId> = Vec::with_capacity(m_us);

    tracker.push(start_of_path);
    let mut curr = start_of_path;

    loop {
        // Advance through `curr`'s next unvisited incident edge, if any.
        let curr_us = curr as usize;
        // Skip already-visited edges in the per-vertex iterator.
        while next_idx[curr_us] < inc[curr_us].len()
            && visited[inc[curr_us][next_idx[curr_us]] as usize]
        {
            next_idx[curr_us] += 1;
        }
        if next_idx[curr_us] < inc[curr_us].len() {
            let edge = inc[curr_us][next_idx[curr_us]];
            visited[edge as usize] = true;
            next_idx[curr_us] += 1;
            tracker.push(curr);
            edge_tracker.push(edge);
            curr = if directed {
                // Directed: edge always points from curr → to[edge].
                graph.edge_target(edge)?
            } else {
                graph.edge_other(edge, curr)?
            };
        } else {
            // Dead end at `curr`: pop the walk.
            path.push(curr);
            if let Some(prev) = tracker.pop() {
                if let Some(curr_e) = edge_tracker.pop() {
                    edge_path.push(curr_e);
                }
                curr = prev;
            } else {
                break;
            }
        }
    }

    // edge_path was filled with the walk in reverse; reverse it now to get
    // the forward edge order. (Upstream pops to the result vector, which
    // also reverses the order.)
    edge_path.reverse();
    let _ = path; // vertex sequence — not part of the Phase-1 return

    Ok(Some(edge_path))
}

fn pick_start_vertex(
    graph: &Graph,
    cls: crate::algorithms::paths::eulerian::EulerianClassification,
) -> IgraphResult<VertexId> {
    let n = graph.vcount();
    let directed = graph.is_directed();
    if cls.has_cycle {
        // Any vertex with non-zero (out-)degree.
        for v in 0..n {
            let has_edges = if directed {
                !graph.incident(v)?.is_empty()
            } else {
                graph.neighbors_iter(v)?.len() > 0
            };
            if has_edges {
                return Ok(v);
            }
        }
        // Should be unreachable since `m == 0` returns early above.
        Err(IgraphError::Internal("no edges but cls.has_cycle"))
    } else if directed {
        // Directed path: pick the vertex with `out_degree - in_degree == 1`
        // (the unique source per is_eulerian's outgoing_excess accounting).
        for v in 0..n {
            let out_deg = graph.incident(v)?.len();
            // in-degree via the in-side
            let in_deg = compute_in_degree(graph, v)?;
            if out_deg > in_deg && (out_deg - in_deg) == 1 {
                return Ok(v);
            }
        }
        Err(IgraphError::Internal(
            "directed path: no vertex with outgoing_excess == 1",
        ))
    } else {
        // Undirected path: smallest-id odd-degree vertex.
        for v in 0..n {
            let deg = graph.degree(v)?;
            if deg % 2 != 0 {
                return Ok(v);
            }
        }
        Err(IgraphError::Internal(
            "has_path && !has_cycle but no odd-degree vertex found",
        ))
    }
}

fn compute_in_degree(graph: &Graph, v: VertexId) -> IgraphResult<usize> {
    // Total degree via undirected `degree` would mix in/out; use a fresh
    // count via in-side inferred from edges (n times m worst case is fine
    // here — only called once per vertex during start selection).
    let mut count = 0usize;
    let m = u32::try_from(graph.ecount())
        .map_err(|_| IgraphError::InvalidArgument("edge count exceeds u32 range".into()))?;
    for e in 0..m {
        if graph.edge_target(e)? == v {
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walk_validates(graph: &Graph, walk: &[EdgeId]) -> bool {
        // Each edge appears exactly once.
        let mut seen: Vec<bool> = vec![false; graph.ecount()];
        for &edge in walk {
            let idx = edge as usize;
            if idx >= graph.ecount() || seen[idx] {
                return false;
            }
            seen[idx] = true;
        }
        // Walk is consecutively connected.
        if walk.len() < 2 {
            return true;
        }
        for i in 0..walk.len() - 1 {
            let (a, b) = graph.edge(walk[i]).unwrap();
            let (c, d) = graph.edge(walk[i + 1]).unwrap();
            if !(a == c || a == d || b == c || b == d) {
                return false;
            }
        }
        true
    }

    #[test]
    fn empty_graph_returns_empty_walk() {
        let g = Graph::with_vertices(0);
        assert_eq!(eulerian_path(&g).unwrap(), Some(Vec::new()));
    }

    #[test]
    fn isolated_vertices_return_empty_walk() {
        let g = Graph::with_vertices(5);
        assert_eq!(eulerian_path(&g).unwrap(), Some(Vec::new()));
    }

    #[test]
    fn triangle_yields_three_edge_walk() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let walk = eulerian_path(&g).unwrap().unwrap();
        assert_eq!(walk.len(), 3);
        assert!(walk_validates(&g, &walk));
    }

    #[test]
    fn path_3_yields_two_edge_walk() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let walk = eulerian_path(&g).unwrap().unwrap();
        assert_eq!(walk.len(), 2);
        assert!(walk_validates(&g, &walk));
    }

    #[test]
    fn k4_has_no_eulerian_walk() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert_eq!(eulerian_path(&g).unwrap(), None);
    }

    #[test]
    fn disconnected_components_no_walk() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(eulerian_path(&g).unwrap(), None);
    }

    #[test]
    fn ring_5_walk_visits_all_edges() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            g.add_edge(i, (i + 1) % 5).unwrap();
        }
        let walk = eulerian_path(&g).unwrap().unwrap();
        assert_eq!(walk.len(), 5);
        assert!(walk_validates(&g, &walk));
    }

    #[test]
    fn directed_3_cycle_yields_3_edge_walk() {
        // 0 -> 1 -> 2 -> 0: directed cycle, has Eulerian cycle.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let walk = eulerian_path(&g).unwrap().unwrap();
        assert_eq!(walk.len(), 3);
    }

    #[test]
    fn directed_path_yields_chain_walk() {
        // 0 -> 1 -> 2: directed Eulerian path (out_excess at 0 = 1).
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let walk = eulerian_path(&g).unwrap().unwrap();
        assert_eq!(walk, vec![0, 1]); // edges traversed in order
    }

    #[test]
    fn directed_no_eulerian_returns_none() {
        // 0 -> 1 -> 2 plus 0 -> 2: two out-edges from 0, two in to 2 → no path.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        assert_eq!(eulerian_path(&g).unwrap(), None);
    }

    // ---- eulerian_cycle tests ----

    #[test]
    fn cycle_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let cycle = eulerian_cycle(&g).unwrap();
        assert_eq!(cycle.len(), 3);
        assert!(walk_validates(&g, &cycle));
    }

    #[test]
    fn cycle_ring5() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            g.add_edge(i, (i + 1) % 5).unwrap();
        }
        let cycle = eulerian_cycle(&g).unwrap();
        assert_eq!(cycle.len(), 5);
        assert!(walk_validates(&g, &cycle));
    }

    #[test]
    fn cycle_empty_graph() {
        let g = Graph::with_vertices(0);
        let cycle = eulerian_cycle(&g).unwrap();
        assert!(cycle.is_empty());
    }

    #[test]
    fn cycle_isolated_vertices() {
        let g = Graph::with_vertices(4);
        let cycle = eulerian_cycle(&g).unwrap();
        assert!(cycle.is_empty());
    }

    #[test]
    fn cycle_path_graph_errors() {
        // Path 0-1-2 has Euler path but no cycle.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(eulerian_cycle(&g).is_err());
    }

    #[test]
    fn cycle_k4_errors() {
        // K4: 4 odd-degree vertices → no Euler path or cycle.
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(eulerian_cycle(&g).is_err());
    }

    #[test]
    fn cycle_directed_3cycle() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let cycle = eulerian_cycle(&g).unwrap();
        assert_eq!(cycle.len(), 3);
    }

    #[test]
    fn cycle_directed_path_errors() {
        // 0->1->2: directed Euler path but no cycle.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(eulerian_cycle(&g).is_err());
    }

    #[test]
    fn complex_eulerian_path_test_eulerian_r() {
        // R test-eulerian.R 6-vertex literal-graph case has Euler path but no cycle.
        // Edges: 0-1, 1-2, 2-3, 3-4, 4-0, 0-5, 5-3, 3-1, 1-5, 5-4 (10 edges).
        let mut g = Graph::with_vertices(6);
        for &(u, v) in &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 0),
            (0, 5),
            (5, 3),
            (3, 1),
            (1, 5),
            (5, 4),
        ] {
            g.add_edge(u, v).unwrap();
        }
        let walk = eulerian_path(&g).unwrap().unwrap();
        assert_eq!(walk.len(), 10, "must visit every edge exactly once");
        assert!(walk_validates(&g, &walk));
    }
}
