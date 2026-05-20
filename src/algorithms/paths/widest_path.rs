//! Widest-path widths (ALGO-SP-010).
//!
//! Counterpart of `igraph_widest_path_widths_dijkstra()` from
//! `references/igraph/src/paths/widest_paths.c:596`. Given an
//! edge-weighted graph, returns for each vertex `v` the maximum
//! **bottleneck width** of any path from `source` to `v` — that is,
//! the largest minimum-edge-weight along any such path.
//!
//! Algorithm: a max-priority-queue variant of Dijkstra. Instead of
//! relaxing `dist[u] = min(dist[u], dist[v] + w)` we relax
//! `width[u] = max(width[u], min(width[v], w))`. The pop order
//! processes vertices in decreasing width, so once a vertex is
//! settled the recorded width is optimal.
//!
//! Time complexity: `O((V + E) log V)` — same shape as Dijkstra.
//!
//! Convention:
//! - `widths[source] == Some(f64::INFINITY)` (no edge constraints
//!   yet; matches upstream's `IGRAPH_INFINITY` sentinel)
//! - `widths[v] == None` if `v` is unreachable from `source`
//!   (upstream uses `-IGRAPH_INFINITY`)
//! - Edges with weight `-f64::INFINITY` are ignored (matches
//!   upstream)

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Max-heap entry: pop yields the largest `width` first, with smaller
/// vertex id breaking ties. NaN widths are excluded by validation.
#[derive(Copy, Clone)]
struct WidestFrontier(f64, VertexId);

impl PartialEq for WidestFrontier {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
impl Eq for WidestFrontier {}
impl Ord for WidestFrontier {
    fn cmp(&self, other: &Self) -> Ordering {
        // Natural order: larger width is "greater" so BinaryHeap pops it
        // first. Smaller vertex id tiebreaks (deterministic for ties).
        self.0.total_cmp(&other.0).then(other.1.cmp(&self.1))
    }
}
impl PartialOrd for WidestFrontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn validate_weights(graph: &Graph, weights: &[f64]) -> IgraphResult<()> {
    let m = graph.ecount();
    if weights.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "weights vector size ({}) differs from edge count ({})",
            weights.len(),
            m
        )));
    }
    for (e, &w) in weights.iter().enumerate() {
        if w.is_nan() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is NaN"
            )));
        }
    }
    Ok(())
}

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

/// Single-source widest-path widths on `graph` from `source`,
/// following out-edges on directed graphs.
///
/// Returns `widths[v]`:
/// - `Some(f64::INFINITY)` for `v == source` (no path constraint yet)
/// - `Some(w)` if `v` is reachable, with `w` the maximum bottleneck
///   width of any `source → v` path
/// - `None` if `v` is unreachable
///
/// A path's *bottleneck width* is the minimum edge weight along it;
/// the *widest path* maximises this bottleneck across all
/// source→target paths. Useful in network-capacity problems.
///
/// `weights[e]` is the width of edge `e`; length must equal
/// `graph.ecount()`. NaN widths are rejected. Edges with weight
/// `-f64::INFINITY` are treated as "edge absent" (matches upstream).
///
/// Counterpart of `igraph_widest_path_widths_dijkstra(_, _,
/// vss(source), vss_all(), weights, IGRAPH_OUT)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, widest_path_widths};
///
/// // Triangle 0-1-2 with edge weights 1, 4, 2.
/// // Widest 0→2 path: direct edge (width 4) beats 0-1-2 (min(1,2) = 1).
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();  // edge 0, width 1
/// g.add_edge(0, 2).unwrap();  // edge 1, width 4
/// g.add_edge(1, 2).unwrap();  // edge 2, width 2
/// let w = widest_path_widths(&g, 0, &[1.0, 4.0, 2.0]).unwrap();
/// assert_eq!(w[0], Some(f64::INFINITY));
/// assert_eq!(w[1], Some(2.0));  // via 0-2-1: min(4, 2) = 2
/// assert_eq!(w[2], Some(4.0));  // direct
/// ```
pub fn widest_path_widths(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
) -> IgraphResult<Vec<Option<f64>>> {
    widest_path_widths_with_mode(graph, source, weights, DijkstraMode::Out)
}

/// Widest-path widths with directed-mode selection. Mirrors
/// [`widest_path_widths`] but lets you choose OUT/IN/ALL traversal
/// for directed graphs (ignored on undirected).
///
/// Counterpart of `igraph_widest_path_widths_dijkstra(_, _,
/// vss(source), vss_all(), weights, mode)`.
pub fn widest_path_widths_with_mode(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<Vec<Option<f64>>> {
    let (widths, _) = widest_inner(graph, source, weights, mode)?;
    Ok(widths
        .into_iter()
        .map(|w| {
            if w == f64::NEG_INFINITY {
                None
            } else {
                Some(w)
            }
        })
        .collect())
}

/// Shared SPFA-style loop. Returns the raw widths vector (sentinel
/// `-∞` = unreachable, `+∞` = source itself) and the per-vertex
/// inbound edge from the widest-paths tree (`None` for source and
/// unreachable vertices). The public APIs strip the sentinels and
/// either drop or use the parent-edge chain.
fn widest_inner(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<(Vec<f64>, Vec<Option<EdgeId>>)> {
    let n = graph.vcount();
    if source >= n {
        return Err(IgraphError::VertexOutOfRange { id: source, n });
    }
    validate_weights(graph, weights)?;

    let n_usize = n as usize;
    let mut widths: Vec<f64> = vec![f64::NEG_INFINITY; n_usize];
    widths[source as usize] = f64::INFINITY;
    let mut parent_eid: Vec<Option<EdgeId>> = vec![None; n_usize];

    let mut heap: BinaryHeap<WidestFrontier> = BinaryHeap::new();
    heap.push(WidestFrontier(f64::INFINITY, source));

    while let Some(WidestFrontier(w, v)) = heap.pop() {
        if w < widths[v as usize] {
            continue;
        }

        let incidents = incident_for_mode(graph, v, mode)?;
        for eid in incidents {
            let edge_w = weights[eid as usize];
            if edge_w == f64::NEG_INFINITY {
                continue;
            }
            let other = graph.edge_other(eid, v)?;
            let alt = w.min(edge_w);
            if alt > widths[other as usize] {
                widths[other as usize] = alt;
                parent_eid[other as usize] = Some(eid);
                heap.push(WidestFrontier(alt, other));
            }
        }
    }

    Ok((widths, parent_eid))
}

/// Widest path from `from` to `to`: returns the vertex sequence and
/// the edge sequence along the widest (maximum-bottleneck) path.
///
/// Returns `Some((vertices, edges))` on success, with
/// `vertices[0] == from`, `*vertices.last().unwrap() == to`, and
/// `edges.len() == vertices.len() - 1`. Returns `None` if `to` is
/// unreachable from `from`. Self-target (`from == to`) returns
/// `Some((vec![from], vec![]))` — the trivial zero-edge path.
///
/// Same semantics as [`widest_path_widths`] for weights: negative
/// finite weights act as small bottlenecks, `-f64::INFINITY` weights
/// are ignored, NaN is rejected.
///
/// Counterpart of `igraph_get_widest_path(_, _, _, from, to,
/// weights, IGRAPH_OUT)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, widest_path};
///
/// // Triangle 0-1-2 with weights 1, 4, 2 — widest 0→1 path goes via 2.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();  // edge 0, width 1
/// g.add_edge(0, 2).unwrap();  // edge 1, width 4
/// g.add_edge(1, 2).unwrap();  // edge 2, width 2
/// let path = widest_path(&g, 0, 1, &[1.0, 4.0, 2.0]).unwrap().unwrap();
/// assert_eq!(path.0, vec![0, 2, 1]);
/// assert_eq!(path.1, vec![1, 2]);
/// ```
pub fn widest_path(
    graph: &Graph,
    from: VertexId,
    to: VertexId,
    weights: &[f64],
) -> IgraphResult<Option<(Vec<VertexId>, Vec<EdgeId>)>> {
    widest_path_with_mode(graph, from, to, weights, DijkstraMode::Out)
}

/// Widest path with mode selection. Mirrors [`widest_path`] but lets
/// you pick OUT/IN/ALL traversal on directed graphs.
///
/// Counterpart of `igraph_get_widest_path(_, _, _, from, to, weights, mode)`.
pub fn widest_path_with_mode(
    graph: &Graph,
    from: VertexId,
    to: VertexId,
    weights: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<Option<(Vec<VertexId>, Vec<EdgeId>)>> {
    let n = graph.vcount();
    if to >= n {
        return Err(IgraphError::VertexOutOfRange { id: to, n });
    }
    // `from` is validated inside `widest_inner`.
    let (widths, parent_eid) = widest_inner(graph, from, weights, mode)?;

    // Trivial self-target.
    if from == to {
        return Ok(Some((vec![from], Vec::new())));
    }
    // Unreachable: parent_eid is None at `to` and `to != from`.
    if widths[to as usize] == f64::NEG_INFINITY {
        return Ok(None);
    }

    // Walk back along parent_eid from `to` to `from`.
    let mut edges: Vec<EdgeId> = Vec::new();
    let mut vertices: Vec<VertexId> = vec![to];
    let mut cur = to;
    while cur != from {
        let eid = parent_eid[cur as usize].ok_or(IgraphError::Internal(
            "missing parent edge while walking widest path",
        ))?;
        let prev = graph.edge_other(eid, cur)?;
        edges.push(eid);
        vertices.push(prev);
        cur = prev;
    }
    vertices.reverse();
    edges.reverse();
    Ok(Some((vertices, edges)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_picks_direct_edge_when_wider() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // width 1
        g.add_edge(0, 2).unwrap(); // width 4
        g.add_edge(1, 2).unwrap(); // width 2
        let w = widest_path_widths(&g, 0, &[1.0, 4.0, 2.0]).unwrap();
        // Source: ∞
        assert_eq!(w[0], Some(f64::INFINITY));
        // 0 → 1 direct = 1, vs 0-2-1 = min(4, 2) = 2. Widest = 2.
        assert_eq!(w[1], Some(2.0));
        // 0 → 2 direct = 4, vs 0-1-2 = min(1, 2) = 1. Widest = 4.
        assert_eq!(w[2], Some(4.0));
    }

    #[test]
    fn chain_bottleneck_is_minimum_weight() {
        // 0-1-2-3 with weights 5, 1, 3.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let w = widest_path_widths(&g, 0, &[5.0, 1.0, 3.0]).unwrap();
        assert_eq!(w[0], Some(f64::INFINITY));
        assert_eq!(w[1], Some(5.0));
        // 0→2 bottleneck = min(5, 1) = 1
        assert_eq!(w[2], Some(1.0));
        // 0→3 bottleneck = min(5, 1, 3) = 1
        assert_eq!(w[3], Some(1.0));
    }

    #[test]
    fn unreachable_vertex_is_none() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let w = widest_path_widths(&g, 0, &[2.0, 3.0]).unwrap();
        assert_eq!(w[0], Some(f64::INFINITY));
        assert_eq!(w[1], Some(2.0));
        assert_eq!(w[2], None);
        assert_eq!(w[3], None);
    }

    #[test]
    fn negative_infinity_edge_ignored() {
        // 0-1 has -∞ width → effectively absent. 0-2 via 0-1-2 not
        // possible; only direct 0-2 if it exists.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // -∞ width
        g.add_edge(1, 2).unwrap();
        let w = widest_path_widths(&g, 0, &[f64::NEG_INFINITY, 1.0]).unwrap();
        assert_eq!(w[0], Some(f64::INFINITY));
        assert_eq!(w[1], None); // edge 0-1 ignored
        assert_eq!(w[2], None);
    }

    #[test]
    fn directed_out_mode_default() {
        // 0 → 1 → 2 with weights 5, 3. From 0: w[1]=5, w[2]=3.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let w = widest_path_widths(&g, 0, &[5.0, 3.0]).unwrap();
        assert_eq!(w[0], Some(f64::INFINITY));
        assert_eq!(w[1], Some(5.0));
        assert_eq!(w[2], Some(3.0));
    }

    #[test]
    fn directed_in_mode_reverses() {
        // 0 → 1 → 2 from 2 with IN mode: 2 reaches 1 (width 3), then 0 (min(3, 5) = 3).
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let w = widest_path_widths_with_mode(&g, 2, &[5.0, 3.0], DijkstraMode::In).unwrap();
        assert_eq!(w[0], Some(3.0));
        assert_eq!(w[1], Some(3.0));
        assert_eq!(w[2], Some(f64::INFINITY));
    }

    #[test]
    fn source_out_of_range_errors() {
        let g = Graph::with_vertices(3);
        let err = widest_path_widths(&g, 99, &[]).unwrap_err();
        assert!(matches!(
            err,
            IgraphError::VertexOutOfRange { id: 99, n: 3 }
        ));
    }

    #[test]
    fn nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = widest_path_widths(&g, 0, &[f64::NAN]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = widest_path_widths(&g, 0, &[1.0, 2.0]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn empty_graph_no_edges() {
        let g = Graph::with_vertices(3);
        let w = widest_path_widths(&g, 0, &[]).unwrap();
        assert_eq!(w[0], Some(f64::INFINITY));
        assert_eq!(w[1], None);
        assert_eq!(w[2], None);
    }

    #[test]
    fn negative_weights_allowed_as_bottleneck() {
        // A negative *finite* edge weight is allowed; it just acts as
        // a small bottleneck. -∞ is the ignore sentinel.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let w = widest_path_widths(&g, 0, &[-1.0, 5.0]).unwrap();
        assert_eq!(w[0], Some(f64::INFINITY));
        assert_eq!(w[1], Some(-1.0));
        // 0 → 2 bottleneck = min(-1, 5) = -1
        assert_eq!(w[2], Some(-1.0));
    }

    #[test]
    fn multiple_parallel_edges_pick_widest() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap(); // width 1
        g.add_edge(0, 1).unwrap(); // width 5 — should win
        g.add_edge(0, 1).unwrap(); // width 3
        let w = widest_path_widths(&g, 0, &[1.0, 5.0, 3.0]).unwrap();
        assert_eq!(w[0], Some(f64::INFINITY));
        assert_eq!(w[1], Some(5.0));
    }

    // -------- ALGO-SP-011: widest_path path construction --------

    #[test]
    fn widest_path_triangle_via_shortcut() {
        // Same setup as widest_path_widths triangle: 0→1 via 0-2-1
        // is wider (bottleneck 2) than the direct edge (width 1).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // edge 0, width 1
        g.add_edge(0, 2).unwrap(); // edge 1, width 4
        g.add_edge(1, 2).unwrap(); // edge 2, width 2
        let (vs, es) = widest_path(&g, 0, 1, &[1.0, 4.0, 2.0])
            .unwrap()
            .expect("0→1 is reachable");
        assert_eq!(vs, vec![0, 2, 1]);
        assert_eq!(es, vec![1, 2]);
    }

    #[test]
    fn widest_path_direct_edge_when_widest() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // edge 0, width 1
        g.add_edge(0, 2).unwrap(); // edge 1, width 4 — direct is widest
        g.add_edge(1, 2).unwrap(); // edge 2, width 2
        let (vs, es) = widest_path(&g, 0, 2, &[1.0, 4.0, 2.0])
            .unwrap()
            .expect("0→2 reachable");
        assert_eq!(vs, vec![0, 2]);
        assert_eq!(es, vec![1]);
    }

    #[test]
    fn widest_path_self_target_returns_trivial() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let (vs, es) = widest_path(&g, 0, 0, &[5.0]).unwrap().unwrap();
        assert_eq!(vs, vec![0]);
        assert!(es.is_empty());
    }

    #[test]
    fn widest_path_unreachable_target_returns_none() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let result = widest_path(&g, 0, 2, &[1.0, 1.0]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn widest_path_chain_returns_full_chain() {
        // 0-1-2-3 unit widths; widest path is the chain itself.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let (vs, es) = widest_path(&g, 0, 3, &[1.0, 1.0, 1.0]).unwrap().unwrap();
        assert_eq!(vs, vec![0, 1, 2, 3]);
        assert_eq!(es, vec![0, 1, 2]);
    }

    #[test]
    fn widest_path_directed_respects_direction() {
        // Directed 0 → 1 → 2: 0 → 2 reachable in OUT mode.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let (vs, _) = widest_path(&g, 0, 2, &[5.0, 3.0]).unwrap().unwrap();
        assert_eq!(vs, vec![0, 1, 2]);
        // Reverse direction: not reachable in OUT mode.
        assert!(widest_path(&g, 2, 0, &[5.0, 3.0]).unwrap().is_none());
        // IN mode from 2 to 0 reaches via reverse traversal.
        let (vs, _) = widest_path_with_mode(&g, 2, 0, &[5.0, 3.0], DijkstraMode::In)
            .unwrap()
            .unwrap();
        assert_eq!(vs, vec![2, 1, 0]);
    }

    #[test]
    fn widest_path_from_out_of_range_errors() {
        let g = Graph::with_vertices(3);
        let err = widest_path(&g, 99, 0, &[]).unwrap_err();
        assert!(matches!(
            err,
            IgraphError::VertexOutOfRange { id: 99, n: 3 }
        ));
    }

    #[test]
    fn widest_path_to_out_of_range_errors() {
        let g = Graph::with_vertices(3);
        let err = widest_path(&g, 0, 99, &[]).unwrap_err();
        assert!(matches!(
            err,
            IgraphError::VertexOutOfRange { id: 99, n: 3 }
        ));
    }

    #[test]
    fn widest_path_negative_infinity_edge_breaks_chain() {
        // 0-1 has -∞ width → effectively missing; path can't use it.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = widest_path(&g, 0, 2, &[f64::NEG_INFINITY, 1.0]).unwrap();
        assert!(r.is_none());
    }
}
