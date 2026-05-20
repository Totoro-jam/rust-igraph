//! A* shortest path with admissible heuristic (ALGO-SP-005).
//!
//! Counterpart of `igraph_get_shortest_path_astar()` from
//! `references/igraph/src/paths/astar.c:93`.
//!
//! A* finds a single source→target shortest path by exploring vertices
//! in order of their estimated total cost `f(v) = g(v) + h(v, target)`,
//! where `g(v)` is the actual distance from the source and
//! `h(v, target)` is the user-supplied heuristic estimating the
//! remaining distance. The result is guaranteed to be the true
//! shortest path when the heuristic is *admissible* (never
//! overestimates the actual remaining distance). With `h ≡ 0` the
//! algorithm degenerates to Dijkstra.
//!
//! Conventions matching upstream:
//! - `weights = None` is treated as all-unit weights (BFS-equivalent).
//! - `weights[e] = INFINITY` skips the edge during relaxation.
//! - Unreachable target → `Ok(None)`.
//! - `from == to` → vertex chain `[from]` and empty edge chain.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Frontier entry for the A* heap: `(f_score, tiebreaker, vertex)`.
/// Ord is reversed so the smallest `f_score` pops first.
#[derive(Copy, Clone)]
struct Frontier(f64, u64, VertexId);

impl PartialEq for Frontier {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}
impl Eq for Frontier {}
impl Ord for Frontier {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse on f_score; tiebreaker on insertion order then vertex.
        other
            .0
            .total_cmp(&self.0)
            .then(other.1.cmp(&self.1))
            .then(other.2.cmp(&self.2))
    }
}
impl PartialOrd for Frontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn validate_weights(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<()> {
    let Some(w) = weights else {
        return Ok(());
    };
    let m = graph.ecount();
    if w.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "weights vector size ({}) differs from edge count ({})",
            w.len(),
            m
        )));
    }
    for (e, &v) in w.iter().enumerate() {
        if v.is_nan() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is NaN"
            )));
        }
        if v < 0.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is negative ({v}); A* requires non-negative weights"
            )));
        }
    }
    Ok(())
}

/// Per-vertex mode-aware incident edges (mirrors the helper in
/// `paths::dijkstra` but kept private here so `astar` doesn't add a
/// `pub(crate)` cross-module dependency).
fn incident_for_mode(graph: &Graph, v: VertexId, mode: DijkstraMode) -> IgraphResult<Vec<EdgeId>> {
    if !graph.is_directed() {
        return graph.incident(v);
    }
    match mode {
        DijkstraMode::Out => graph.incident(v),
        DijkstraMode::In => graph.incident_in(v),
        DijkstraMode::All => {
            let mut out = graph.incident(v)?;
            out.extend(graph.incident_in(v)?);
            Ok(out)
        }
    }
}

/// Single-source single-target shortest path using A*.
///
/// `weights` is `None` for unweighted (unit-weight BFS-equivalent) or
/// `Some(&[f64])` for weighted; non-negative finite values are required
/// (NaN / negative values return [`IgraphError::InvalidArgument`]).
/// Edges of weight [`f64::INFINITY`] are skipped during relaxation.
///
/// `heuristic(v, target) -> f64` must return a non-negative admissible
/// estimate of the remaining distance from `v` to `target`. With
/// `|_, _| 0.0` (the "null heuristic"), A* reduces to Dijkstra and is
/// guaranteed correct. Inadmissible heuristics may return a path
/// shorter than the true geodesic.
///
/// Returns `Ok(None)` when `target` is unreachable; otherwise
/// `Ok(Some((vs, es)))` with vertex chain (including source and
/// target) and parallel edge id chain.
///
/// Counterpart of `igraph_get_shortest_path_astar(_, _, _, from, to,
/// &weights, mode, heuristic, NULL)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, a_star_path, DijkstraMode};
///
/// // Path 0-1-2-3 unit weights with null heuristic (≡ Dijkstra):
/// // shortest path 0→3 is the chain through every vertex.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let (vs, es) = a_star_path(&g, 0, 3, None, DijkstraMode::Out, |_, _| 0.0)
///     .unwrap()
///     .unwrap();
/// assert_eq!(vs, vec![0, 1, 2, 3]);
/// assert_eq!(es, vec![0, 1, 2]);
/// ```
pub fn a_star_path<H: Fn(VertexId, VertexId) -> f64>(
    graph: &Graph,
    from: VertexId,
    to: VertexId,
    weights: Option<&[f64]>,
    mode: DijkstraMode,
    heuristic: H,
) -> IgraphResult<Option<(Vec<VertexId>, Vec<EdgeId>)>> {
    let n = graph.vcount();
    if from >= n {
        return Err(IgraphError::VertexOutOfRange { id: from, n });
    }
    if to >= n {
        return Err(IgraphError::VertexOutOfRange { id: to, n });
    }
    validate_weights(graph, weights)?;

    if from == to {
        return Ok(Some((vec![from], Vec::new())));
    }

    let n_us = n as usize;
    let mut dist = vec![f64::INFINITY; n_us];
    let mut parent_eid: Vec<Option<EdgeId>> = vec![None; n_us];
    let mut closed = vec![false; n_us];

    let mut tiebreaker: u64 = 0;
    let mut next_tb = || {
        let t = tiebreaker;
        tiebreaker += 1;
        t
    };

    let mut heap: BinaryHeap<Frontier> = BinaryHeap::new();
    dist[from as usize] = 0.0;
    let h0 = heuristic(from, to);
    if h0.is_nan() || h0 < 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "heuristic returned invalid estimate ({h0}); must be non-negative and not NaN"
        )));
    }
    heap.push(Frontier(h0, next_tb(), from));

    let mut found = false;
    while let Some(Frontier(_, _, u)) = heap.pop() {
        if closed[u as usize] {
            continue;
        }
        closed[u as usize] = true;
        if u == to {
            found = true;
            break;
        }

        for eid in incident_for_mode(graph, u, mode)? {
            let w = match weights {
                None => 1.0,
                Some(ws) => ws[eid as usize],
            };
            if !w.is_finite() {
                continue;
            }
            let v = graph.edge_other(eid as EdgeId, u)?;
            if closed[v as usize] {
                continue;
            }
            let altdist = dist[u as usize] + w;
            let curdist = dist[v as usize];
            if !curdist.is_finite() || altdist < curdist {
                dist[v as usize] = altdist;
                parent_eid[v as usize] = Some(eid as EdgeId);
                let h = heuristic(v, to);
                if h.is_nan() || h < 0.0 {
                    return Err(IgraphError::InvalidArgument(format!(
                        "heuristic returned invalid estimate ({h}); must be non-negative and not NaN"
                    )));
                }
                heap.push(Frontier(altdist + h, next_tb(), v));
            }
        }
    }

    if !found {
        return Ok(None);
    }

    // Reconstruct path from to → ... → from by walking parent_eid.
    let mut vs = Vec::new();
    let mut es = Vec::new();
    let mut cur = to;
    while let Some(eid) = parent_eid[cur as usize] {
        es.push(eid);
        vs.push(cur);
        cur = graph.edge_other(eid, cur)?;
    }
    vs.push(cur);
    vs.reverse();
    es.reverse();
    Ok(Some((vs, es)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn null_h(_: VertexId, _: VertexId) -> f64 {
        0.0
    }

    #[test]
    fn unit_weights_match_bfs_chain() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let (vs, es) = a_star_path(&g, 0, 3, None, DijkstraMode::Out, null_h)
            .unwrap()
            .unwrap();
        assert_eq!(vs, vec![0, 1, 2, 3]);
        assert_eq!(es, vec![0, 1, 2]);
    }

    #[test]
    fn weighted_triangle_with_shortcut() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(0, 2).unwrap(); // edge 1
        g.add_edge(1, 2).unwrap(); // edge 2
        let weights = [1.0, 4.0, 2.0];
        let (vs, es) = a_star_path(&g, 0, 2, Some(&weights), DijkstraMode::Out, null_h)
            .unwrap()
            .unwrap();
        // Best: 0→1→2 cost 3 < direct 0→2 cost 4.
        assert_eq!(vs, vec![0, 1, 2]);
        assert_eq!(es, vec![0, 2]);
    }

    #[test]
    fn unreachable_target_returns_none() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert_eq!(
            a_star_path(&g, 0, 2, None, DijkstraMode::Out, null_h).unwrap(),
            None
        );
    }

    #[test]
    fn from_equals_to_singleton_chain() {
        let g = Graph::with_vertices(3);
        let (vs, es) = a_star_path(&g, 1, 1, None, DijkstraMode::Out, null_h)
            .unwrap()
            .unwrap();
        assert_eq!(vs, vec![1]);
        assert!(es.is_empty());
    }

    #[test]
    fn admissible_heuristic_finds_same_path_as_null() {
        // Grid-like graph: 0→1→2 + 0→3→2, all unit weights. With null
        // heuristic A* explores both branches; with an admissible
        // heuristic biased toward vertex 2 (returns 0 for 2, large
        // values for others), it still must find a length-2 path.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(3, 2).unwrap();
        let h = |v: VertexId, target: VertexId| -> f64 { if v == target { 0.0 } else { 1.0 } };
        let (vs, _) = a_star_path(&g, 0, 2, None, DijkstraMode::Out, h)
            .unwrap()
            .unwrap();
        // Either 0→1→2 or 0→3→2 — both have length 2.
        assert_eq!(vs.len(), 3);
        assert_eq!(vs[0], 0);
        assert_eq!(vs[2], 2);
    }

    #[test]
    fn directed_in_mode_walks_reverse_edges() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // OUT-mode from 2 to 0: unreachable.
        assert_eq!(
            a_star_path(&g, 2, 0, None, DijkstraMode::Out, null_h).unwrap(),
            None
        );
        // IN-mode from 2 to 0: walks 2→1→0 (edge ids 1, 0).
        let (vs, es) = a_star_path(&g, 2, 0, None, DijkstraMode::In, null_h)
            .unwrap()
            .unwrap();
        assert_eq!(vs, vec![2, 1, 0]);
        assert_eq!(es, vec![1, 0]);
    }

    #[test]
    fn negative_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let weights = [-1.0_f64];
        assert!(a_star_path(&g, 0, 1, Some(&weights), DijkstraMode::Out, null_h).is_err());
    }

    #[test]
    fn nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let weights = [f64::NAN];
        assert!(a_star_path(&g, 0, 1, Some(&weights), DijkstraMode::Out, null_h).is_err());
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let weights = [1.0_f64, 2.0];
        assert!(a_star_path(&g, 0, 1, Some(&weights), DijkstraMode::Out, null_h).is_err());
    }

    #[test]
    fn out_of_range_source_errors() {
        let g = Graph::with_vertices(2);
        assert!(a_star_path(&g, 99, 0, None, DijkstraMode::Out, null_h).is_err());
        assert!(a_star_path(&g, 0, 99, None, DijkstraMode::Out, null_h).is_err());
    }

    #[test]
    fn negative_heuristic_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let bad_h = |_v: VertexId, _t: VertexId| -1.0_f64;
        assert!(a_star_path(&g, 0, 1, None, DijkstraMode::Out, bad_h).is_err());
    }

    #[test]
    fn infinity_weight_skipped() {
        // 0-1 with weight INF, 0-2 with weight 1, 2-1 with weight 1.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // edge 0, weight INF
        g.add_edge(0, 2).unwrap(); // edge 1, weight 1
        g.add_edge(2, 1).unwrap(); // edge 2, weight 1
        let weights = [f64::INFINITY, 1.0, 1.0];
        let (vs, _) = a_star_path(&g, 0, 1, Some(&weights), DijkstraMode::Out, null_h)
            .unwrap()
            .unwrap();
        // Must route through vertex 2, not via the infinite-weight direct edge.
        assert_eq!(vs, vec![0, 2, 1]);
    }
}
