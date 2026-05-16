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
    if graph.is_directed() {
        return Err(IgraphError::Unsupported(
            "directed eulerian_path is CC-042 (Hierholzer directed); not yet ported",
        ));
    }

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

    // Per-vertex incident-edge lists. Self-loops appear once per upstream
    // `IGRAPH_LOOPS_ONCE` (each loop is a single traversable edge here).
    let mut inc: Vec<Vec<EdgeId>> = Vec::with_capacity(n_us);
    for v in 0..n {
        let raw = graph.incident(v)?;
        // Convert LOOPS_TWICE → LOOPS_ONCE: each self-loop appears twice in
        // `incident()`'s default; keep only one copy.
        // Simple dedupe by counting; for self-loops the edge id repeats.
        let mut seen: std::collections::HashSet<EdgeId> = std::collections::HashSet::new();
        let mut out: Vec<EdgeId> = Vec::with_capacity(raw.len());
        for e in raw {
            // Keep first occurrence of every edge id.
            if seen.insert(e) {
                out.push(e);
            }
        }
        inc.push(out);
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
            curr = graph.edge_other(edge, curr)?;
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
    if cls.has_cycle {
        // Any vertex with non-zero degree (skip the all-isolated case which
        // is caught earlier by `m == 0`).
        for v in 0..n {
            if !graph.neighbors(v)?.is_empty() {
                return Ok(v);
            }
        }
        // Should be unreachable since `m == 0` returns early above.
        Err(IgraphError::Internal("no edges but cls.has_cycle"))
    } else {
        // Path-but-not-cycle: there are exactly two odd-degree vertices in
        // the non-singleton component (per is_eulerian undirected logic).
        // Pick the smallest-id odd-degree vertex.
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
    fn directed_graph_returns_unsupported_error() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(eulerian_path(&g).is_err());
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
