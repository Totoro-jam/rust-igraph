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
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, widest_path_widths_with_mode, DijkstraMode};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let w = widest_path_widths_with_mode(&g, 0, &[3.0, 5.0], DijkstraMode::All).unwrap();
/// assert_eq!(w[1], Some(3.0));
/// ```
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

/// Sidecar outputs from a single-source widest-paths run. Carries
/// the widths vector plus the parent-pointer SPT (vertex-side and
/// edge-side). Counterpart of the `parents` and `inbound_edges`
/// outputs of `igraph_get_widest_paths`. Source itself has
/// `parents[source] == None` and `inbound_edges[source] == None`;
/// unreachable vertices also have both `None`. To disambiguate the
/// source from unreachable targets, consult `widths`: source has
/// `Some(f64::INFINITY)`, unreachable has `None`.
#[derive(Debug, Clone, PartialEq)]
pub struct WidestPaths {
    /// `widths[v]`: `Some(f64::INFINITY)` for `v == source`,
    /// `Some(w)` for reachable, `None` for unreachable.
    pub widths: Vec<Option<f64>>,
    /// `parents[v]` is the predecessor of `v` in the widest-paths
    /// spanning tree. `None` for source and unreachable.
    pub parents: Vec<Option<VertexId>>,
    /// `inbound_edges[v]` is the edge id `v` was reached through.
    /// `None` for source and unreachable.
    pub inbound_edges: Vec<Option<EdgeId>>,
}

/// Single-source widest-paths sidecar: widths plus the parent-pointer
/// SPT. Convenient when you want **all** of widths, parent vertices,
/// and inbound edge ids in one call without re-running the SPT.
///
/// Behaves like [`widest_path_widths`] for weight semantics: NaN
/// rejected, `-f64::INFINITY` edges ignored, negative finite weights
/// act as small bottlenecks.
///
/// Counterpart of `igraph_get_widest_paths(_, NULL, NULL, source,
/// vss_all(), weights, IGRAPH_OUT, parents, inbound_edges)` from
/// `references/igraph/src/paths/widest_paths.c:102`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, widest_paths};
///
/// // Triangle 0-1-2 weights (1, 4, 2). Widest 0→1 routes via vertex 2.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();  // edge 0, width 1
/// g.add_edge(0, 2).unwrap();  // edge 1, width 4
/// g.add_edge(1, 2).unwrap();  // edge 2, width 2
/// let sp = widest_paths(&g, 0, &[1.0, 4.0, 2.0]).unwrap();
/// // Source itself
/// assert_eq!(sp.widths[0], Some(f64::INFINITY));
/// assert_eq!(sp.parents[0], None);
/// assert_eq!(sp.inbound_edges[0], None);
/// // Vertex 2 reached directly via edge 1 (widest direct edge)
/// assert_eq!(sp.parents[2], Some(0));
/// assert_eq!(sp.inbound_edges[2], Some(1));
/// // Vertex 1 reached via 2 (bottleneck min(4, 2) = 2 beats direct edge 0 with width 1)
/// assert_eq!(sp.parents[1], Some(2));
/// assert_eq!(sp.inbound_edges[1], Some(2));
/// ```
pub fn widest_paths(graph: &Graph, from: VertexId, weights: &[f64]) -> IgraphResult<WidestPaths> {
    widest_paths_with_mode(graph, from, weights, DijkstraMode::Out)
}

/// Mode-aware variant of [`widest_paths`].
///
/// Counterpart of `igraph_get_widest_paths(_, NULL, NULL, source,
/// vss_all(), weights, mode, parents, inbound_edges)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, widest_paths_with_mode, DijkstraMode};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let sp = widest_paths_with_mode(&g, 0, &[3.0, 5.0], DijkstraMode::All).unwrap();
/// assert!(sp.widths[2].is_some());
/// ```
pub fn widest_paths_with_mode(
    graph: &Graph,
    from: VertexId,
    weights: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<WidestPaths> {
    let (raw_widths, parent_eid) = widest_inner(graph, from, weights, mode)?;
    let n = raw_widths.len();
    let mut parents: Vec<Option<VertexId>> = vec![None; n];
    let widths: Vec<Option<f64>> = raw_widths
        .iter()
        .map(|&w| {
            if w == f64::NEG_INFINITY {
                None
            } else {
                Some(w)
            }
        })
        .collect();
    // Derive parent vertices from parent edges: `edge_other(eid, v)`
    // gives the predecessor of v in the SPT. Source's parent stays
    // None (its parent_eid entry is None by construction).
    for v in 0..n {
        if let Some(eid) = parent_eid[v] {
            let v_u32 = u32::try_from(v)
                .map_err(|_| IgraphError::Internal("vertex index exceeds u32::MAX"))?;
            let prev = graph.edge_other(eid, v_u32)?;
            parents[v] = Some(prev);
        }
    }
    Ok(WidestPaths {
        widths,
        parents,
        inbound_edges: parent_eid,
    })
}

/// Widest path with mode selection. Mirrors [`widest_path`] but lets
/// you pick OUT/IN/ALL traversal on directed graphs.
///
/// Counterpart of `igraph_get_widest_path(_, _, _, from, to, weights, mode)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, widest_path_with_mode, DijkstraMode};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let p = widest_path_with_mode(&g, 0, 2, &[3.0, 5.0], DijkstraMode::All).unwrap();
/// assert!(p.is_some());
/// ```
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
    reconstruct_one(graph, from, to, &widths, &parent_eid)
}

/// Walk back along `parent_eid` from `to` to `from`, building the
/// vertex+edge chains. Returns `None` if `to` is unreachable; for
/// `from == to` returns the trivial single-vertex zero-edge chain.
fn reconstruct_one(
    graph: &Graph,
    from: VertexId,
    to: VertexId,
    widths: &[f64],
    parent_eid: &[Option<EdgeId>],
) -> IgraphResult<Option<(Vec<VertexId>, Vec<EdgeId>)>> {
    if from == to {
        return Ok(Some((vec![from], Vec::new())));
    }
    if widths[to as usize] == f64::NEG_INFINITY {
        return Ok(None);
    }
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

/// One entry of [`widest_paths_to`]'s output: `Some((vertices,
/// edges))` for a reachable target, `None` for unreachable. Each
/// vertex chain begins with the source and ends with the target;
/// each edge chain has length one less.
pub type WidestPathResult = Option<(Vec<VertexId>, Vec<EdgeId>)>;

/// Widest paths from a single source to multiple targets. Returns
/// one [`WidestPathResult`] per element of `targets`, in the same
/// order; `None` means the target is unreachable from `from`.
///
/// Self-target entries (`from == targets[i]`) return the trivial
/// `Some((vec![from], vec![]))`. Repeating the same target id in
/// `targets` is allowed — both entries get the same path.
///
/// Same weight semantics as [`widest_path_widths`]: NaN rejected,
/// `-f64::INFINITY` edges ignored, negative finite weights act as
/// small bottlenecks.
///
/// Counterpart of `igraph_get_widest_paths(_, vertices, edges,
/// from, to, weights, IGRAPH_OUT, parents=NULL, inbound_edges=NULL)`
/// from `references/igraph/src/paths/widest_paths.c:102`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, widest_paths_to};
///
/// // Triangle 0-1-2 weights (1, 4, 2). From 0, paths to 1 and 2.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();  // edge 0, width 1
/// g.add_edge(0, 2).unwrap();  // edge 1, width 4
/// g.add_edge(1, 2).unwrap();  // edge 2, width 2
/// let paths = widest_paths_to(&g, 0, &[1, 2], &[1.0, 4.0, 2.0]).unwrap();
/// // 0→1 goes via the shortcut at 2 (bottleneck 2 beats direct 1)
/// assert_eq!(paths[0].as_ref().unwrap().0, vec![0, 2, 1]);
/// // 0→2 takes the direct edge (width 4 is widest)
/// assert_eq!(paths[1].as_ref().unwrap().0, vec![0, 2]);
/// ```
pub fn widest_paths_to(
    graph: &Graph,
    from: VertexId,
    targets: &[VertexId],
    weights: &[f64],
) -> IgraphResult<Vec<WidestPathResult>> {
    widest_paths_to_with_mode(graph, from, targets, weights, DijkstraMode::Out)
}

/// Mode-aware variant of [`widest_paths_to`].
///
/// Counterpart of `igraph_get_widest_paths(_, vertices, edges,
/// from, to, weights, mode, parents=NULL, inbound_edges=NULL)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, widest_paths_to_with_mode, DijkstraMode};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let paths = widest_paths_to_with_mode(&g, 0, &[2], &[3.0, 5.0], DijkstraMode::All).unwrap();
/// assert!(paths[0].is_some());
/// ```
pub fn widest_paths_to_with_mode(
    graph: &Graph,
    from: VertexId,
    targets: &[VertexId],
    weights: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<Vec<WidestPathResult>> {
    let n = graph.vcount();
    for &t in targets {
        if t >= n {
            return Err(IgraphError::VertexOutOfRange { id: t, n });
        }
    }
    // `from` is validated inside `widest_inner`.
    let (widths, parent_eid) = widest_inner(graph, from, weights, mode)?;
    let mut out = Vec::with_capacity(targets.len());
    for &t in targets {
        out.push(reconstruct_one(graph, from, t, &widths, &parent_eid)?);
    }
    Ok(out)
}

// ---------------------------------------------------------------
// ALGO-SP-012: Floyd-Warshall-based all-pairs widest widths.
// O(V³) — better than V invocations of the Dijkstra-style variant
// on dense graphs.
// ---------------------------------------------------------------

/// All-pairs widest-path widths via the Floyd-Warshall recurrence.
///
/// Returns a `vcount × vcount` matrix where `result[u][v]` is the
/// maximum bottleneck width of any `u → v` path:
/// - `Some(f64::INFINITY)` on the diagonal (`u == v`)
/// - `Some(w)` for reachable pairs with `w` the bottleneck width
/// - `None` for unreachable pairs
///
/// `weights[e]` is the width of edge `e`; length must equal
/// `graph.ecount()`. NaN is rejected; edges with weight
/// `-f64::INFINITY` are ignored (matches upstream). Parallel edges
/// are merged by the wider-wins rule when seeding the matrix.
///
/// Use this on **dense** graphs (`|E| ~ V²`); for sparse graphs the
/// Dijkstra-based [`widest_path_widths`] called from every source is
/// asymptotically faster.
///
/// Counterpart of `igraph_widest_path_widths_floyd_warshall(_, _,
/// vss_all(), vss_all(), weights, IGRAPH_OUT)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, widest_path_widths_floyd_warshall};
///
/// // Undirected triangle (1, 4, 2) — same all-pairs result the
/// // Dijkstra variant produces when run from every source.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let m = widest_path_widths_floyd_warshall(&g, &[1.0, 4.0, 2.0]).unwrap();
/// assert_eq!(m[0][2], Some(4.0));   // direct
/// assert_eq!(m[0][1], Some(2.0));   // via vertex 2: min(4, 2)
/// assert_eq!(m[0][0], Some(f64::INFINITY));
/// ```
pub fn widest_path_widths_floyd_warshall(
    graph: &Graph,
    weights: &[f64],
) -> IgraphResult<Vec<Vec<Option<f64>>>> {
    widest_path_widths_floyd_warshall_with_mode(graph, weights, DijkstraMode::Out)
}

/// Mode-aware variant of [`widest_path_widths_floyd_warshall`].
/// Mode selects how directed edges contribute to the adjacency
/// matrix:
/// - [`DijkstraMode::Out`] populates `M[s][t]` for edge `s → t`
/// - [`DijkstraMode::In`] populates `M[t][s]` for edge `s → t`
/// - [`DijkstraMode::All`] populates both directions
///
/// On undirected graphs every mode collapses to `All` (matches
/// upstream).
///
/// Counterpart of `igraph_widest_path_widths_floyd_warshall(_, _,
/// vss_all(), vss_all(), weights, mode)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, widest_path_widths_floyd_warshall_with_mode, DijkstraMode};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let m = widest_path_widths_floyd_warshall_with_mode(&g, &[3.0, 5.0], DijkstraMode::All).unwrap();
/// assert_eq!(m[0][2], Some(3.0));
/// ```
pub fn widest_path_widths_floyd_warshall_with_mode(
    graph: &Graph,
    weights: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<Vec<Vec<Option<f64>>>> {
    validate_weights(graph, weights)?;
    let vcount = graph.vcount();
    let n_us = vcount as usize;

    // Normalise mode: undirected graph → ALL (matches upstream).
    let effective_mode = if graph.is_directed() {
        mode
    } else {
        DijkstraMode::All
    };
    let (use_out, use_in) = match effective_mode {
        DijkstraMode::Out => (true, false),
        DijkstraMode::In => (false, true),
        DijkstraMode::All => (true, true),
    };

    // Init: -∞ everywhere; +∞ on the diagonal.
    let mut mat: Vec<Vec<f64>> = vec![vec![f64::NEG_INFINITY; n_us]; n_us];
    for (i, row) in mat.iter_mut().enumerate().take(n_us) {
        row[i] = f64::INFINITY;
    }

    // Seed from edges (wider-wins for parallel edges).
    for (e, &w) in weights.iter().enumerate() {
        if w == f64::NEG_INFINITY {
            continue;
        }
        let eid = u32::try_from(e)
            .map_err(|_| IgraphError::Internal("edge id exceeds u32::MAX in FW widest"))?;
        let (s, t) = graph.edge(eid)?;
        let (si, ti) = (s as usize, t as usize);
        if use_out && mat[si][ti] < w {
            mat[si][ti] = w;
        }
        if use_in && mat[ti][si] < w {
            mat[ti][si] = w;
        }
    }

    // Modified FW: relax via every intermediate k.
    // alt = min(M[i][k], M[k][j]); M[i][j] = max(M[i][j], alt).
    // The triple-nested index access is inherent to the recurrence —
    // iterator-style rewrites obscure it.
    #[allow(clippy::needless_range_loop)]
    for k in 0..n_us {
        for j in 0..n_us {
            let width_kj = mat[k][j];
            if j == k || width_kj == f64::NEG_INFINITY {
                continue;
            }
            for i in 0..n_us {
                if i == j || i == k {
                    continue;
                }
                let alt = mat[i][k].min(width_kj);
                if alt > mat[i][j] {
                    mat[i][j] = alt;
                }
            }
        }
    }

    Ok(mat
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|w| {
                    if w == f64::NEG_INFINITY {
                        None
                    } else {
                        Some(w)
                    }
                })
                .collect()
        })
        .collect())
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

    // -------- ALGO-SP-012: FW all-pairs widest widths --------

    #[test]
    fn fw_widest_triangle_matches_dijkstra_per_source() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let weights = [1.0, 4.0, 2.0];
        let fw = widest_path_widths_floyd_warshall(&g, &weights).unwrap();
        // Compare each row to the Dijkstra-based result.
        for u in 0..3u32 {
            let dij = widest_path_widths(&g, u, &weights).unwrap();
            assert_eq!(fw[u as usize], dij, "row {u} mismatch");
        }
    }

    #[test]
    fn fw_widest_diagonal_is_infinity() {
        let g = Graph::with_vertices(3);
        let m = widest_path_widths_floyd_warshall(&g, &[]).unwrap();
        for (i, row) in m.iter().enumerate() {
            for (j, entry) in row.iter().enumerate() {
                if i == j {
                    assert_eq!(*entry, Some(f64::INFINITY));
                } else {
                    assert_eq!(*entry, None);
                }
            }
        }
    }

    #[test]
    fn fw_widest_unreachable_components() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let m = widest_path_widths_floyd_warshall(&g, &[5.0, 7.0]).unwrap();
        assert_eq!(m[0][1], Some(5.0));
        assert_eq!(m[0][2], None);
        assert_eq!(m[2][3], Some(7.0));
        assert_eq!(m[1][3], None);
    }

    #[test]
    fn fw_widest_directed_respects_mode() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let weights = [5.0, 3.0];
        // OUT mode: 0 reaches 1 (5) and 2 (3); 2 doesn't reach back.
        let out = widest_path_widths_floyd_warshall(&g, &weights).unwrap();
        assert_eq!(out[0][1], Some(5.0));
        assert_eq!(out[0][2], Some(3.0));
        assert_eq!(out[2][0], None);
        // IN mode: reversed.
        let in_m =
            widest_path_widths_floyd_warshall_with_mode(&g, &weights, DijkstraMode::In).unwrap();
        assert_eq!(in_m[0][1], None);
        assert_eq!(in_m[2][0], Some(3.0));
        // ALL mode: bidirectional.
        let all =
            widest_path_widths_floyd_warshall_with_mode(&g, &weights, DijkstraMode::All).unwrap();
        assert_eq!(all[0][2], Some(3.0));
        assert_eq!(all[2][0], Some(3.0));
    }

    #[test]
    fn fw_widest_parallel_edges_keep_widest() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap(); // width 1
        g.add_edge(0, 1).unwrap(); // width 5 — wins
        g.add_edge(0, 1).unwrap(); // width 3
        let m = widest_path_widths_floyd_warshall(&g, &[1.0, 5.0, 3.0]).unwrap();
        assert_eq!(m[0][1], Some(5.0));
        assert_eq!(m[1][0], Some(5.0));
    }

    #[test]
    fn fw_widest_negative_infinity_edge_ignored() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // -∞ → ignored
        g.add_edge(1, 2).unwrap();
        let m = widest_path_widths_floyd_warshall(&g, &[f64::NEG_INFINITY, 1.0]).unwrap();
        // 0 can't reach 1 or 2 — the bridge edge is absent.
        assert_eq!(m[0][1], None);
        assert_eq!(m[0][2], None);
        assert_eq!(m[1][2], Some(1.0));
    }

    #[test]
    fn fw_widest_nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = widest_path_widths_floyd_warshall(&g, &[f64::NAN]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn fw_widest_empty_graph_empty_matrix() {
        let g = Graph::with_vertices(0);
        let m = widest_path_widths_floyd_warshall(&g, &[]).unwrap();
        assert!(m.is_empty());
    }

    // -------- ALGO-SP-013: widest_paths_to multi-target --------

    #[test]
    fn widest_paths_to_triangle_two_targets() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // edge 0, width 1
        g.add_edge(0, 2).unwrap(); // edge 1, width 4
        g.add_edge(1, 2).unwrap(); // edge 2, width 2
        let paths = widest_paths_to(&g, 0, &[1, 2], &[1.0, 4.0, 2.0]).unwrap();
        assert_eq!(paths.len(), 2);
        // 0→1 via shortcut at 2: bottleneck min(4, 2) = 2 beats direct 1
        let (vs1, es1) = paths[0].as_ref().unwrap();
        assert_eq!(vs1, &vec![0, 2, 1]);
        assert_eq!(es1, &vec![1, 2]);
        // 0→2 direct: width 4 wins
        let (vs2, es2) = paths[1].as_ref().unwrap();
        assert_eq!(vs2, &vec![0, 2]);
        assert_eq!(es2, &vec![1]);
    }

    #[test]
    fn widest_paths_to_includes_unreachable_as_none() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        // Targets [1, 2, 3]: only 1 reachable.
        let paths = widest_paths_to(&g, 0, &[1, 2, 3], &[1.0, 1.0]).unwrap();
        assert!(paths[0].is_some());
        assert!(paths[1].is_none());
        assert!(paths[2].is_none());
    }

    #[test]
    fn widest_paths_to_empty_targets_returns_empty() {
        let g = Graph::with_vertices(3);
        let paths = widest_paths_to(&g, 0, &[], &[]).unwrap();
        assert!(paths.is_empty());
    }

    #[test]
    fn widest_paths_to_self_target_is_trivial() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        // from == target[0]: trivial path.
        let paths = widest_paths_to(&g, 1, &[1, 0], &[5.0]).unwrap();
        let (vs0, es0) = paths[0].as_ref().unwrap();
        assert_eq!(vs0, &vec![1]);
        assert!(es0.is_empty());
        // 1 → 0: single edge.
        let (vs1, es1) = paths[1].as_ref().unwrap();
        assert_eq!(vs1, &vec![1, 0]);
        assert_eq!(es1, &vec![0]);
    }

    #[test]
    fn widest_paths_to_duplicate_targets_return_same_path() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let paths = widest_paths_to(&g, 0, &[2, 2, 2], &[5.0, 3.0]).unwrap();
        assert_eq!(paths.len(), 3);
        for p in &paths {
            let (vs, _) = p.as_ref().unwrap();
            assert_eq!(vs, &vec![0, 1, 2]);
        }
    }

    #[test]
    fn widest_paths_to_target_out_of_range_errors() {
        let g = Graph::with_vertices(3);
        let err = widest_paths_to(&g, 0, &[1, 99], &[]).unwrap_err();
        assert!(matches!(
            err,
            IgraphError::VertexOutOfRange { id: 99, n: 3 }
        ));
    }

    #[test]
    fn widest_paths_to_directed_in_mode_reverses() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // IN mode from 2: reaches 1 and 0 by walking reverse edges.
        let paths =
            widest_paths_to_with_mode(&g, 2, &[1, 0], &[5.0, 3.0], DijkstraMode::In).unwrap();
        let (vs1, _) = paths[0].as_ref().unwrap();
        assert_eq!(vs1, &vec![2, 1]);
        let (vs0, _) = paths[1].as_ref().unwrap();
        assert_eq!(vs0, &vec![2, 1, 0]);
    }

    #[test]
    fn widest_paths_to_negative_infinity_edge_blocks_target() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // -∞ → ignored
        g.add_edge(1, 2).unwrap();
        let paths = widest_paths_to(&g, 0, &[1, 2], &[f64::NEG_INFINITY, 1.0]).unwrap();
        assert!(paths[0].is_none());
        assert!(paths[1].is_none());
    }

    // -------- ALGO-SP-014: WidestPaths struct (widths + SPT) --------

    #[test]
    fn widest_paths_triangle_struct_fields_consistent() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // edge 0, width 1
        g.add_edge(0, 2).unwrap(); // edge 1, width 4
        g.add_edge(1, 2).unwrap(); // edge 2, width 2
        let sp = widest_paths(&g, 0, &[1.0, 4.0, 2.0]).unwrap();
        // Source-side sentinels
        assert_eq!(sp.widths[0], Some(f64::INFINITY));
        assert_eq!(sp.parents[0], None);
        assert_eq!(sp.inbound_edges[0], None);
        // Vertex 2 reached directly via edge 1 (width 4)
        assert_eq!(sp.widths[2], Some(4.0));
        assert_eq!(sp.parents[2], Some(0));
        assert_eq!(sp.inbound_edges[2], Some(1));
        // Vertex 1 reached via 2 (bottleneck min(4, 2) = 2 beats direct width 1)
        assert_eq!(sp.widths[1], Some(2.0));
        assert_eq!(sp.parents[1], Some(2));
        assert_eq!(sp.inbound_edges[1], Some(2));
    }

    #[test]
    fn widest_paths_widths_match_widest_path_widths() {
        // The struct's `widths` field must match the standalone function.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let weights = [5.0, 1.0, 3.0];
        let sp = widest_paths(&g, 0, &weights).unwrap();
        let standalone = widest_path_widths(&g, 0, &weights).unwrap();
        assert_eq!(sp.widths, standalone);
    }

    #[test]
    fn widest_paths_unreachable_has_none_in_all_three_fields() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let sp = widest_paths(&g, 0, &[5.0, 7.0]).unwrap();
        // Unreachable vertex 2: widths/parents/inbound_edges all None.
        assert_eq!(sp.widths[2], None);
        assert_eq!(sp.parents[2], None);
        assert_eq!(sp.inbound_edges[2], None);
        // Reachable vertex 1 (via edge 0) — parent is source.
        assert_eq!(sp.parents[1], Some(0));
        assert_eq!(sp.inbound_edges[1], Some(0));
    }

    #[test]
    fn widest_paths_directed_in_mode() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // IN mode from 2: reaches 1 (edge 1) and 0 (via edge 0).
        let sp = widest_paths_with_mode(&g, 2, &[5.0, 3.0], DijkstraMode::In).unwrap();
        assert_eq!(sp.widths[2], Some(f64::INFINITY));
        assert_eq!(sp.widths[1], Some(3.0));
        assert_eq!(sp.widths[0], Some(3.0));
        // Parents under IN mode: 1's predecessor reached via edge 1 → from
        // 2's side that is vertex 2 (edge_other(1, 1) = 2).
        assert_eq!(sp.parents[1], Some(2));
        assert_eq!(sp.parents[0], Some(1));
    }

    #[test]
    fn widest_paths_source_out_of_range_errors() {
        let g = Graph::with_vertices(3);
        let err = widest_paths(&g, 99, &[]).unwrap_err();
        assert!(matches!(
            err,
            IgraphError::VertexOutOfRange { id: 99, n: 3 }
        ));
    }

    #[test]
    fn widest_paths_nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = widest_paths(&g, 0, &[f64::NAN]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn widest_paths_spt_endpoints_match_widest_path_chain() {
        // Walking back from a target via parents/inbound_edges must
        // reconstruct exactly the chain returned by widest_path.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let weights = [5.0, 1.0, 3.0];
        let sp = widest_paths(&g, 0, &weights).unwrap();
        let path = widest_path(&g, 0, 3, &weights).unwrap().unwrap();
        // Reconstruct via SPT from target 3.
        let mut reconstructed: Vec<u32> = vec![3];
        let mut cur = 3usize;
        while let Some(prev) = sp.parents[cur] {
            reconstructed.push(prev);
            cur = prev as usize;
        }
        reconstructed.reverse();
        assert_eq!(reconstructed, path.0);
    }
}
