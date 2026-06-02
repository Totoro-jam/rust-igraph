//! Dijkstra single-source shortest distances + paths/parents/cutoff +
//! mode-aware variants + all-shortest-paths (ALGO-SP-001 / SP-001b /
//! SP-001c).
//!
//! Counterparts of:
//! - `igraph_distances_dijkstra()`              — `references/igraph/src/paths/dijkstra.c:322`
//! - `igraph_distances_dijkstra_cutoff()`       — `references/igraph/src/paths/dijkstra.c:107`
//! - `igraph_get_shortest_paths_dijkstra()`     — `references/igraph/src/paths/dijkstra.c:412`
//! - `igraph_get_shortest_path_dijkstra()`      — `references/igraph/src/paths/dijkstra.c:689`
//! - `igraph_get_all_shortest_paths_dijkstra()` — `references/igraph/src/paths/dijkstra.c:797`
//!
//! Mode (SP-001c): on directed graphs the [`DijkstraMode`] parameter
//! selects which adjacency the BFS follows (`Out` / `In` / `All`). The
//! legacy entry points without `_with_mode` keep `Out` semantics
//! (matches upstream defaults). For undirected graphs every mode is
//! identical (every edge is bidirectional). The all-shortest-paths
//! variant takes a `mode` parameter from the start because it didn't
//! exist before SP-001c.
//!
//! All edge weights must be non-negative and not NaN; otherwise we
//! return [`IgraphError::InvalidArgument`]. Edges of weight
//! [`f64::INFINITY`] are rejected at validation time (matches SP-001's
//! contract — upstream silently skips them in the relax loop, but we've
//! always required finite weights at the boundary). The relax loop
//! still defensively checks each edge so changes to validation don't
//! corrupt the heap with `INF + INF = INF`.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Min-heap entry. `Ord` is reversed so that `BinaryHeap` (a max-heap)
/// pops the smallest distance first. NaN distances are forbidden by the
/// public API contract — `total_cmp` is therefore safe.
#[derive(Copy, Clone)]
struct Frontier(f64, VertexId);

impl PartialEq for Frontier {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
impl Eq for Frontier {}
impl Ord for Frontier {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering: smaller distance is "greater" for the heap
        // so it gets popped first.
        other.0.total_cmp(&self.0).then(other.1.cmp(&self.1))
    }
}
impl PartialOrd for Frontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Direction selector for the mode-aware Dijkstra entry points
/// (SP-001c). Counterpart of `igraph_neimode_t` restricted to the
/// three values `igraph_distances_dijkstra` accepts. For undirected
/// graphs every variant collapses to [`DijkstraMode::All`] (every
/// edge is bidirectional).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DijkstraMode {
    /// Follow outgoing edges (`IGRAPH_OUT`). Default for the legacy
    /// entry points without `_with_mode`.
    Out,
    /// Follow incoming edges (`IGRAPH_IN`).
    In,
    /// Ignore direction — treat every edge as bidirectional
    /// (`IGRAPH_ALL`).
    All,
}

/// Per-vertex mode-aware incident edges. For undirected graphs every
/// mode collapses to [`Graph::incident`]'s merged adjacency.
pub(super) fn incident_for_mode(
    graph: &Graph,
    v: VertexId,
    mode: DijkstraMode,
) -> IgraphResult<Vec<EdgeId>> {
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

/// Validate `weights`: length matches `graph.ecount()`, no negative,
/// no NaN, no non-finite values. Shared by every public entry point.
pub(super) fn validate_weights(graph: &Graph, weights: &[f64]) -> IgraphResult<()> {
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
        if w < 0.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is negative ({w}); Dijkstra requires non-negative weights"
            )));
        }
        if !w.is_finite() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is not finite ({w})"
            )));
        }
    }
    Ok(())
}

/// Inner Dijkstra loop seeded with one or more source vertices at
/// distance 0. Records distances and (optionally) the inbound edge for
/// each settled vertex. `cutoff` is `None` to disable, `Some(c)` to
/// stop relaxing at distances strictly greater than `c` (matches the
/// `mindist > cutoff` early-skip in `igraph_i_distances_dijkstra_cutoff`).
fn dijkstra_inner(
    graph: &Graph,
    sources: &[VertexId],
    weights: &[f64],
    cutoff: Option<f64>,
    record_inbound: bool,
    mode: DijkstraMode,
) -> IgraphResult<(Vec<f64>, Vec<Option<EdgeId>>)> {
    let n = graph.vcount();
    let n_us = n as usize;
    let mut dist = vec![f64::INFINITY; n_us];
    let mut inbound: Vec<Option<EdgeId>> = if record_inbound {
        vec![None; n_us]
    } else {
        Vec::new()
    };
    let mut visited = vec![false; n_us];

    let mut heap: BinaryHeap<Frontier> = BinaryHeap::new();
    for &s in sources {
        if s >= n {
            return Err(IgraphError::VertexOutOfRange { id: s, n });
        }
        if dist[s as usize] > 0.0 {
            dist[s as usize] = 0.0;
            heap.push(Frontier(0.0, s));
        }
    }

    while let Some(Frontier(d, v)) = heap.pop() {
        let v_us = v as usize;
        if visited[v_us] {
            continue;
        }
        if let Some(c) = cutoff {
            if d > c {
                continue;
            }
        }
        visited[v_us] = true;

        for eid in incident_for_mode(graph, v, mode)? {
            let w = weights[eid as usize];
            if !w.is_finite() {
                continue;
            }
            let other = graph.edge_other(eid as EdgeId, v)?;
            let nd = d + w;
            if nd < dist[other as usize] {
                dist[other as usize] = nd;
                if record_inbound {
                    inbound[other as usize] = Some(eid as EdgeId);
                }
                heap.push(Frontier(nd, other));
            }
        }
    }

    Ok((dist, inbound))
}

fn dist_vec_to_options(dist: Vec<f64>) -> Vec<Option<f64>> {
    dist.into_iter()
        .map(|d| if d.is_infinite() { None } else { Some(d) })
        .collect()
}

/// Single-source Dijkstra distances.
///
/// Returns `Vec<Option<f64>>` of length `vcount`: `result[v] = Some(d)`
/// if there is a path from `source` to `v` with weighted distance `d`,
/// `None` if `v` is unreachable. The source's own distance is
/// `Some(0.0)` whenever `source` is a valid vertex.
///
/// `weights[e]` is the weight of edge `e`; `weights.len()` must equal
/// `graph.ecount()`. All weights must be `>= 0` and finite (no NaN, no
/// infinity).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dijkstra_distances};
///
/// // Triangle 0-1-2 with weights 1, 4, 2 → dist(0→2) = 1+2 via 0-1-2
/// // (3.0) is shorter than the direct 0-2 edge weight 4.0.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();   // edge 0, weight 1
/// g.add_edge(0, 2).unwrap();   // edge 1, weight 4
/// g.add_edge(1, 2).unwrap();   // edge 2, weight 2
/// let d = dijkstra_distances(&g, 0, &[1.0, 4.0, 2.0]).unwrap();
/// assert_eq!(d, vec![Some(0.0), Some(1.0), Some(3.0)]);
/// ```
pub fn dijkstra_distances(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
) -> IgraphResult<Vec<Option<f64>>> {
    validate_weights(graph, weights)?;
    let (dist, _) = dijkstra_inner(graph, &[source], weights, None, false, DijkstraMode::Out)?;
    Ok(dist_vec_to_options(dist))
}

/// Output of [`dijkstra_paths`]. `parents[v]` is the predecessor of
/// `v` along the shortest-path tree from the single source (or `None`
/// for the source itself and for unreachable vertices — the caller can
/// disambiguate via `distances[v]`). `inbound_edges[v]` is the edge id
/// `v` was reached through; `None` for the source and unreachable
/// vertices.
///
/// Counterpart of the `vertices`+`parents`+`inbound_edges` outputs of
/// `igraph_get_shortest_paths_dijkstra`. Upstream uses sentinels
/// `-1`/`-2` instead of `Option`s; this Rust port uses `None` for both
/// "source" and "unreachable", so callers should consult `distances`
/// to distinguish them.
#[derive(Debug, Clone, PartialEq)]
pub struct DijkstraPaths {
    /// `distances[v]` is `Some(d)` if `v` is reachable from the
    /// source, else `None`. `distances[source] == Some(0.0)`.
    pub distances: Vec<Option<f64>>,
    /// `parents[v]` is the vertex `u` such that the shortest-path
    /// tree edge into `v` goes `u → v`. `None` for the source and
    /// for unreachable vertices.
    pub parents: Vec<Option<VertexId>>,
    /// `inbound_edges[v]` is the edge id used to reach `v` in the
    /// shortest-path tree. `None` for the source and for unreachable
    /// vertices.
    pub inbound_edges: Vec<Option<EdgeId>>,
}

/// Single-source Dijkstra returning distances + parents + inbound
/// edges over every vertex.
///
/// Counterpart of `igraph_get_shortest_paths_dijkstra(..., to=igraph_vss_all(),
/// IGRAPH_OUT, parents, inbound_edges)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dijkstra_paths};
///
/// // Triangle 0-1-2 with weights 1, 4, 2 → SPT from 0: parent[1]=0,
/// // parent[2]=1 (via the shortcut), inbound[1]=edge0, inbound[2]=edge2.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let p = dijkstra_paths(&g, 0, &[1.0, 4.0, 2.0]).unwrap();
/// assert_eq!(p.distances, vec![Some(0.0), Some(1.0), Some(3.0)]);
/// assert_eq!(p.parents,   vec![None,      Some(0),   Some(1)]);
/// assert_eq!(p.inbound_edges, vec![None, Some(0), Some(2)]);
/// ```
pub fn dijkstra_paths(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
) -> IgraphResult<DijkstraPaths> {
    validate_weights(graph, weights)?;
    let (dist, inbound) = dijkstra_inner(graph, &[source], weights, None, true, DijkstraMode::Out)?;
    let mut parents = vec![None; graph.vcount() as usize];
    for v in 0..graph.vcount() {
        if let Some(eid) = inbound[v as usize] {
            let other = graph.edge_other(eid, v)?;
            parents[v as usize] = Some(other);
        }
    }
    Ok(DijkstraPaths {
        distances: dist_vec_to_options(dist),
        parents,
        inbound_edges: inbound,
    })
}

/// Reconstruct a single source-to-target path returning vertex and
/// edge sequences. `Ok(None)` if `target` is unreachable; the source
/// vertex appears as the first element when reachable.
///
/// Counterpart of `igraph_get_shortest_path_dijkstra`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dijkstra_path_to};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();   // edge 0, weight 1
/// g.add_edge(1, 2).unwrap();   // edge 1, weight 1
/// g.add_edge(2, 3).unwrap();   // edge 2, weight 1
/// g.add_edge(0, 3).unwrap();   // edge 3, weight 10 — heavier
/// let (vs, es) = dijkstra_path_to(&g, 0, 3, &[1.0, 1.0, 1.0, 10.0])
///     .unwrap()
///     .expect("target reachable");
/// assert_eq!(vs, vec![0, 1, 2, 3]);
/// assert_eq!(es, vec![0, 1, 2]);
/// ```
pub fn dijkstra_path_to(
    graph: &Graph,
    source: VertexId,
    target: VertexId,
    weights: &[f64],
) -> IgraphResult<Option<(Vec<VertexId>, Vec<EdgeId>)>> {
    let n = graph.vcount();
    if target >= n {
        return Err(IgraphError::VertexOutOfRange { id: target, n });
    }
    let p = dijkstra_paths(graph, source, weights)?;
    if p.distances[target as usize].is_none() {
        return Ok(None);
    }
    // Walk parents back from target to source.
    let mut vs = Vec::new();
    let mut es = Vec::new();
    let mut cur = target;
    while let Some(eid) = p.inbound_edges[cur as usize] {
        es.push(eid);
        let parent = p.parents[cur as usize].expect("inbound edge implies parent exists");
        vs.push(cur);
        cur = parent;
    }
    vs.push(cur); // push source last
    vs.reverse();
    es.reverse();
    Ok(Some((vs, es)))
}

/// Single-source Dijkstra distances with an optional `cutoff`. When
/// `cutoff = Some(c)`, vertices whose shortest-path distance exceeds
/// `c` are returned as `None` (unreachable-within-cutoff); the search
/// stops relaxing edges out of any vertex whose `mindist > c`.
///
/// `cutoff = None` is identical to [`dijkstra_distances`].
///
/// Counterpart of `igraph_distances_dijkstra_cutoff(..., IGRAPH_OUT, cutoff)`
/// for a single source. Upstream uses `cutoff < 0` to disable; this
/// Rust port uses `None`.
pub fn dijkstra_distances_cutoff(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
    cutoff: Option<f64>,
) -> IgraphResult<Vec<Option<f64>>> {
    validate_weights(graph, weights)?;
    if let Some(c) = cutoff {
        if c.is_nan() {
            return Err(IgraphError::InvalidArgument("cutoff is NaN".into()));
        }
    }
    let (dist, _) = dijkstra_inner(graph, &[source], weights, cutoff, false, DijkstraMode::Out)?;
    let mut out = dist_vec_to_options(dist);
    if let Some(c) = cutoff {
        // The relax loop already prunes outgoing edges past `c`, but a
        // vertex with `dist == c + ε` may have been settled before the
        // skip kicked in (its outgoing edges were just not relaxed).
        // To match upstream's "distances within cutoff only" semantics
        // we mask anything above `c` to `None` after the fact.
        for d in &mut out {
            if let Some(v) = *d {
                if v > c {
                    *d = None;
                }
            }
        }
    }
    Ok(out)
}

/// Multi-source Dijkstra distances. `result[i][v]` is the shortest
/// distance from `sources[i]` to `v` (or `None` if unreachable / past
/// the cutoff).
///
/// Counterpart of `igraph_distances_dijkstra_cutoff(..., from=sources,
/// to=igraph_vss_all(), weights, IGRAPH_OUT, cutoff)`. Each source is
/// run independently — upstream loops over `fromvit` doing exactly
/// the same.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dijkstra_distances_multi};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let d = dijkstra_distances_multi(&g, &[0, 2], &[1.0, 3.0], None).unwrap();
/// assert_eq!(d[0], vec![Some(0.0), Some(1.0), Some(4.0)]);
/// assert_eq!(d[1], vec![Some(4.0), Some(3.0), Some(0.0)]);
/// ```
pub fn dijkstra_distances_multi(
    graph: &Graph,
    sources: &[VertexId],
    weights: &[f64],
    cutoff: Option<f64>,
) -> IgraphResult<Vec<Vec<Option<f64>>>> {
    validate_weights(graph, weights)?;
    if let Some(c) = cutoff {
        if c.is_nan() {
            return Err(IgraphError::InvalidArgument("cutoff is NaN".into()));
        }
    }
    let mut out = Vec::with_capacity(sources.len());
    for &s in sources {
        out.push(dijkstra_distances_cutoff(graph, s, weights, cutoff)?);
    }
    Ok(out)
}

// ---------------------------------------------------------------
// SP-001c: mode-aware variants of every SP-001/SP-001b entry point.
// On undirected graphs every mode behaves identically (matches
// upstream — every edge is bidirectional).
// ---------------------------------------------------------------

/// Mode-aware Dijkstra distances. Counterpart of
/// `igraph_distances_dijkstra(_, _, source, vss_all(), weights, mode)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dijkstra_distances_with_mode, DijkstraMode};
///
/// // Directed path 0→1→2: OUT reaches forward, IN reaches backward,
/// // ALL collapses to undirected.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let w = [1.0, 2.0];
/// assert_eq!(
///     dijkstra_distances_with_mode(&g, 0, &w, DijkstraMode::Out).unwrap(),
///     vec![Some(0.0), Some(1.0), Some(3.0)]
/// );
/// assert_eq!(
///     dijkstra_distances_with_mode(&g, 0, &w, DijkstraMode::In).unwrap(),
///     vec![Some(0.0), None, None]
/// );
/// ```
pub fn dijkstra_distances_with_mode(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<Vec<Option<f64>>> {
    validate_weights(graph, weights)?;
    let (dist, _) = dijkstra_inner(graph, &[source], weights, None, false, mode)?;
    Ok(dist_vec_to_options(dist))
}

/// Mode-aware [`dijkstra_paths`]. Counterpart of
/// `igraph_get_shortest_paths_dijkstra(_, _, _, source, vss_all(), weights, mode, parents, inbound_edges)`.
pub fn dijkstra_paths_with_mode(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<DijkstraPaths> {
    validate_weights(graph, weights)?;
    let (dist, inbound) = dijkstra_inner(graph, &[source], weights, None, true, mode)?;
    let mut parents = vec![None; graph.vcount() as usize];
    for v in 0..graph.vcount() {
        if let Some(eid) = inbound[v as usize] {
            let other = graph.edge_other(eid, v)?;
            parents[v as usize] = Some(other);
        }
    }
    Ok(DijkstraPaths {
        distances: dist_vec_to_options(dist),
        parents,
        inbound_edges: inbound,
    })
}

/// Mode-aware [`dijkstra_path_to`]. Counterpart of
/// `igraph_get_shortest_path_dijkstra(_, _, _, source, target, weights, mode)`.
pub fn dijkstra_path_to_with_mode(
    graph: &Graph,
    source: VertexId,
    target: VertexId,
    weights: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<Option<(Vec<VertexId>, Vec<EdgeId>)>> {
    let n = graph.vcount();
    if target >= n {
        return Err(IgraphError::VertexOutOfRange { id: target, n });
    }
    let p = dijkstra_paths_with_mode(graph, source, weights, mode)?;
    if p.distances[target as usize].is_none() {
        return Ok(None);
    }
    let mut vs = Vec::new();
    let mut es = Vec::new();
    let mut cur = target;
    while let Some(eid) = p.inbound_edges[cur as usize] {
        es.push(eid);
        let parent = p.parents[cur as usize].expect("inbound edge implies parent exists");
        vs.push(cur);
        cur = parent;
    }
    vs.push(cur);
    vs.reverse();
    es.reverse();
    Ok(Some((vs, es)))
}

/// Mode-aware [`dijkstra_distances_cutoff`]. Counterpart of
/// `igraph_distances_dijkstra_cutoff(_, _, source, vss_all(), weights, mode, cutoff)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dijkstra_distances_cutoff_with_mode, DijkstraMode};
///
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let d = dijkstra_distances_cutoff_with_mode(
///     &g, 0, &[1.0, 2.0], Some(1.5), DijkstraMode::Out
/// ).unwrap();
/// assert_eq!(d, vec![Some(0.0), Some(1.0), None]);
/// ```
pub fn dijkstra_distances_cutoff_with_mode(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
    cutoff: Option<f64>,
    mode: DijkstraMode,
) -> IgraphResult<Vec<Option<f64>>> {
    validate_weights(graph, weights)?;
    if let Some(c) = cutoff {
        if c.is_nan() {
            return Err(IgraphError::InvalidArgument("cutoff is NaN".into()));
        }
    }
    let (dist, _) = dijkstra_inner(graph, &[source], weights, cutoff, false, mode)?;
    let mut out = dist_vec_to_options(dist);
    if let Some(c) = cutoff {
        for d in &mut out {
            if let Some(v) = *d {
                if v > c {
                    *d = None;
                }
            }
        }
    }
    Ok(out)
}

/// Mode-aware [`dijkstra_distances_multi`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dijkstra_distances_multi_with_mode, DijkstraMode};
///
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let d = dijkstra_distances_multi_with_mode(
///     &g, &[0, 2], &[1.0, 2.0], None, DijkstraMode::Out
/// ).unwrap();
/// assert_eq!(d[0], vec![Some(0.0), Some(1.0), Some(3.0)]);
/// assert_eq!(d[1], vec![None, None, Some(0.0)]);
/// ```
pub fn dijkstra_distances_multi_with_mode(
    graph: &Graph,
    sources: &[VertexId],
    weights: &[f64],
    cutoff: Option<f64>,
    mode: DijkstraMode,
) -> IgraphResult<Vec<Vec<Option<f64>>>> {
    validate_weights(graph, weights)?;
    if let Some(c) = cutoff {
        if c.is_nan() {
            return Err(IgraphError::InvalidArgument("cutoff is NaN".into()));
        }
    }
    let mut out = Vec::with_capacity(sources.len());
    for &s in sources {
        out.push(dijkstra_distances_cutoff_with_mode(
            graph, s, weights, cutoff, mode,
        )?);
    }
    Ok(out)
}

// ---------------------------------------------------------------
// SP-001c: all-shortest-paths Dijkstra. Multiple parents per vertex
// (DAG of equal-cost predecessors) + count of geodesics (`nrgeo`).
// ---------------------------------------------------------------

/// Output of [`dijkstra_all_shortest_paths`].
#[derive(Debug, Clone, PartialEq)]
pub struct DijkstraAllPaths {
    /// `vertex_paths[v]` = every shortest source→v path as a vertex
    /// chain (including source and v). Empty for unreachable v;
    /// always `[[source]]` for v == source.
    pub vertex_paths: Vec<Vec<Vec<VertexId>>>,
    /// `edge_paths[v]` mirrors [`Self::vertex_paths`] but with edge ids
    /// (length one less than the matching vertex chain). Empty for
    /// unreachable v; `[[]]` for v == source.
    pub edge_paths: Vec<Vec<Vec<EdgeId>>>,
    /// `nrgeo[v]` = number of distinct shortest paths from source to
    /// v. Zero for unreachable, one for the source itself.
    pub nrgeo: Vec<u64>,
}

/// Tie-tolerant comparison of two distances. Mirrors upstream's
/// `igraph_cmp_epsilon(a, b, eps)` which returns `sign(a − b)` modulo
/// the epsilon band (positive if `a > b`, zero within epsilon,
/// negative if `a < b`).
fn cmp_eps(a: f64, b: f64) -> i32 {
    const EPS: f64 = 1e-10; // `IGRAPH_SHORTEST_PATH_EPSILON`
    if !a.is_finite() || !b.is_finite() {
        // Treat infinities deterministically: only equal if both are
        // identical bit-patterns (both +inf, both -inf, or both NaN —
        // NaN can't appear here because validate_weights rejects it).
        return match a.total_cmp(&b) {
            Ordering::Equal => 0,
            Ordering::Greater => 1,
            Ordering::Less => -1,
        };
    }
    let scale = a.abs().max(b.abs()).max(1.0);
    let diff = a - b;
    if diff.abs() <= EPS * scale {
        0
    } else if diff > 0.0 {
        1
    } else {
        -1
    }
}

/// All shortest weighted paths from `source` to every vertex,
/// preserving every tie. Counterpart of
/// `igraph_get_all_shortest_paths_dijkstra(_, _, _, _, source, vss_all(), weights, mode)`.
///
/// Mirrors upstream's tie handling: equal-cost alternative paths are
/// recorded only when the connecting edge has non-zero weight (avoids
/// infinite loops via 0-weight edges in undirected graphs). On
/// undirected graphs every mode behaves identically.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dijkstra_all_shortest_paths, DijkstraMode};
///
/// // Diamond 0-1-3 and 0-2-3, all weight 1: two distinct shortest
/// // paths to vertex 3.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let r = dijkstra_all_shortest_paths(&g, 0, &[1.0, 1.0, 1.0, 1.0], DijkstraMode::Out)
///     .unwrap();
/// assert_eq!(r.nrgeo, vec![1, 1, 1, 2]);
/// assert_eq!(r.vertex_paths[3].len(), 2); // {0,1,3} and {0,2,3}
/// ```
pub fn dijkstra_all_shortest_paths(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<DijkstraAllPaths> {
    let n = graph.vcount();
    if source >= n {
        return Err(IgraphError::VertexOutOfRange { id: source, n });
    }
    validate_weights(graph, weights)?;

    let n_us = n as usize;
    let mut dist = vec![f64::INFINITY; n_us];
    // For each vertex: parent (predecessor) vertices in the shortest
    // path DAG, plus the matching edge ids. Tracked separately so that
    // a "shorter path found" event can clear both atomically.
    let mut parents: Vec<Vec<VertexId>> = vec![Vec::new(); n_us];
    let mut parent_eids: Vec<Vec<EdgeId>> = vec![Vec::new(); n_us];
    // Order in which vertices were *settled* (popped from the heap with
    // their final distance). Used as a topological order to compute
    // `nrgeo` in linear time after the BFS.
    let mut order: Vec<VertexId> = Vec::new();
    let mut visited = vec![false; n_us];

    let mut heap: BinaryHeap<Frontier> = BinaryHeap::new();
    dist[source as usize] = 0.0;
    heap.push(Frontier(0.0, source));

    while let Some(Frontier(d, v)) = heap.pop() {
        let v_us = v as usize;
        if visited[v_us] {
            continue;
        }
        visited[v_us] = true;
        order.push(v);

        for eid in incident_for_mode(graph, v, mode)? {
            let w = weights[eid as usize];
            if !w.is_finite() {
                continue;
            }
            let other = graph.edge_other(eid as EdgeId, v)?;
            let altdist = d + w;
            let curdist = dist[other as usize];
            let cmp = cmp_eps(curdist, altdist);
            if curdist.is_infinite() {
                // First time we reach `other`.
                dist[other as usize] = altdist;
                parents[other as usize].clear();
                parents[other as usize].push(v);
                parent_eids[other as usize].clear();
                parent_eids[other as usize].push(eid);
                heap.push(Frontier(altdist, other));
            } else if cmp == 0 && w > 0.0 {
                // Equal-cost alternative — record only if the
                // connecting edge has positive weight (mirrors upstream's
                // zero-weight loop guard).
                parents[other as usize].push(v);
                parent_eids[other as usize].push(eid);
            } else if cmp > 0 {
                // Strictly shorter — replace existing parents.
                dist[other as usize] = altdist;
                parents[other as usize].clear();
                parents[other as usize].push(v);
                parent_eids[other as usize].clear();
                parent_eids[other as usize].push(eid);
                heap.push(Frontier(altdist, other));
            }
        }
    }

    // Number of geodesics: source = 1; otherwise sum over parents.
    // We process vertices in heap-settle order to ensure parents are
    // counted before children.
    let mut nrgeo: Vec<u64> = vec![0; n_us];
    if dist[source as usize].is_finite() {
        nrgeo[source as usize] = 1;
    }
    for &v in order.iter().skip(1) {
        let mut acc: u64 = 0;
        for &p in &parents[v as usize] {
            acc = acc.saturating_add(nrgeo[p as usize]);
        }
        nrgeo[v as usize] = acc;
    }

    // Reconstruct vertex/edge paths by depth-first enumeration through
    // the predecessor DAG. For each target v we walk every distinct
    // chain source → ... → v and emit it.
    let mut vertex_paths: Vec<Vec<Vec<VertexId>>> = vec![Vec::new(); n_us];
    let mut edge_paths: Vec<Vec<Vec<EdgeId>>> = vec![Vec::new(); n_us];
    if dist[source as usize].is_finite() {
        vertex_paths[source as usize].push(vec![source]);
        edge_paths[source as usize].push(Vec::new());
    }
    // Walk vertices in `order` (distance-ascending). For each non-source
    // settled vertex, build its paths by extending each parent's paths.
    for &v in order.iter().skip(1) {
        let v_us = v as usize;
        let parent_count = parents[v_us].len();
        for i in 0..parent_count {
            let p = parents[v_us][i];
            let e = parent_eids[v_us][i];
            // Snapshot lengths so we don't iterate over freshly pushed
            // paths if `p == v_us` somehow; that shouldn't happen but
            // it's cheap insurance.
            let par_paths_len = vertex_paths[p as usize].len();
            for j in 0..par_paths_len {
                let mut vp = vertex_paths[p as usize][j].clone();
                vp.push(v);
                vertex_paths[v_us].push(vp);
                let mut ep = edge_paths[p as usize][j].clone();
                ep.push(e);
                edge_paths[v_us].push(ep);
            }
        }
    }

    Ok(DijkstraAllPaths {
        vertex_paths,
        edge_paths,
        nrgeo,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_source_out_of_range() {
        let g = Graph::with_vertices(0);
        assert!(dijkstra_distances(&g, 0, &[]).is_err());
    }

    #[test]
    fn singleton_distance_zero() {
        let g = Graph::with_vertices(1);
        assert_eq!(dijkstra_distances(&g, 0, &[]).unwrap(), vec![Some(0.0)]);
    }

    #[test]
    fn unreachable_yields_none() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let d = dijkstra_distances(&g, 0, &[1.0]).unwrap();
        assert_eq!(d, vec![Some(0.0), Some(1.0), None]);
    }

    #[test]
    fn shortcut_via_smaller_weights() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = dijkstra_distances(&g, 0, &[1.0, 4.0, 2.0]).unwrap();
        assert_eq!(d, vec![Some(0.0), Some(1.0), Some(3.0)]);
    }

    #[test]
    fn directed_respects_edge_direction() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 1).unwrap();
        let d = dijkstra_distances(&g, 0, &[1.0, 1.0]).unwrap();
        // 0 reaches 1 via direct edge; 2 has no incoming path from 0.
        assert_eq!(d, vec![Some(0.0), Some(1.0), None]);
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(dijkstra_distances(&g, 0, &[]).is_err());
        assert!(dijkstra_distances(&g, 0, &[1.0, 2.0]).is_err());
    }

    #[test]
    fn negative_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(dijkstra_distances(&g, 0, &[-1.0]).is_err());
    }

    #[test]
    fn nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(dijkstra_distances(&g, 0, &[f64::NAN]).is_err());
    }

    #[test]
    fn infinite_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(dijkstra_distances(&g, 0, &[f64::INFINITY]).is_err());
    }

    #[test]
    fn zero_weights_match_bfs_distance() {
        // All edges weight 1.0 reduces Dijkstra to BFS distances.
        let mut g = Graph::with_vertices(5);
        for u in 0..4u32 {
            g.add_edge(u, u + 1).unwrap();
        }
        let w = vec![1.0; 4];
        let d = dijkstra_distances(&g, 0, &w).unwrap();
        assert_eq!(
            d,
            vec![Some(0.0), Some(1.0), Some(2.0), Some(3.0), Some(4.0)]
        );
    }

    #[test]
    fn parallel_edges_pick_minimum_weight() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let d = dijkstra_distances(&g, 0, &[5.0, 1.5]).unwrap();
        assert_eq!(d, vec![Some(0.0), Some(1.5)]);
    }

    #[test]
    fn star_graph_distances() {
        // Star: vertex 0 is centre, vertices 1..=4 attached with weight i.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        let w = vec![1.0, 2.5, 0.5, 7.0];
        let d = dijkstra_distances(&g, 0, &w).unwrap();
        assert_eq!(
            d,
            vec![Some(0.0), Some(1.0), Some(2.5), Some(0.5), Some(7.0)]
        );
    }

    // ------------------ SP-001b: paths / parents / cutoff / multi -------------

    fn shortcut_triangle() -> (Graph, Vec<f64>) {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(0, 2).unwrap(); // edge 1
        g.add_edge(1, 2).unwrap(); // edge 2
        (g, vec![1.0, 4.0, 2.0])
    }

    #[test]
    fn paths_distances_match_dijkstra_distances() {
        let (g, w) = shortcut_triangle();
        let p = dijkstra_paths(&g, 0, &w).unwrap();
        assert_eq!(p.distances, dijkstra_distances(&g, 0, &w).unwrap());
    }

    #[test]
    fn paths_parents_and_inbound_edges_record_spt() {
        let (g, w) = shortcut_triangle();
        let p = dijkstra_paths(&g, 0, &w).unwrap();
        // Source has no parent.
        assert_eq!(p.parents[0], None);
        assert_eq!(p.inbound_edges[0], None);
        // 1 reached directly via edge 0.
        assert_eq!(p.parents[1], Some(0));
        assert_eq!(p.inbound_edges[1], Some(0));
        // 2 reached via the 1-2 shortcut, edge id 2.
        assert_eq!(p.parents[2], Some(1));
        assert_eq!(p.inbound_edges[2], Some(2));
    }

    #[test]
    fn paths_unreachable_has_none_parent() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let p = dijkstra_paths(&g, 0, &[1.0]).unwrap();
        assert_eq!(p.distances, vec![Some(0.0), Some(1.0), None]);
        assert_eq!(p.parents, vec![None, Some(0), None]);
        assert_eq!(p.inbound_edges, vec![None, Some(0), None]);
    }

    #[test]
    fn path_to_returns_vertex_and_edge_chain() {
        let (g, w) = shortcut_triangle();
        let (vs, es) = dijkstra_path_to(&g, 0, 2, &w).unwrap().unwrap();
        assert_eq!(vs, vec![0, 1, 2]);
        assert_eq!(es, vec![0, 2]);
    }

    #[test]
    fn path_to_self_is_singleton() {
        let (g, w) = shortcut_triangle();
        let (vs, es) = dijkstra_path_to(&g, 0, 0, &w).unwrap().unwrap();
        assert_eq!(vs, vec![0]);
        assert!(es.is_empty());
    }

    #[test]
    fn path_to_unreachable_is_none() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert_eq!(dijkstra_path_to(&g, 0, 2, &[1.0]).unwrap(), None);
    }

    #[test]
    fn path_to_target_out_of_range_errors() {
        let (g, w) = shortcut_triangle();
        assert!(dijkstra_path_to(&g, 0, 99, &w).is_err());
    }

    #[test]
    fn cutoff_masks_distances_above_cutoff() {
        let mut g = Graph::with_vertices(5);
        for i in 0..4u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let w = vec![1.0; 4];
        let d = dijkstra_distances_cutoff(&g, 0, &w, Some(2.0)).unwrap();
        // Vertices reachable within distance 2: 0, 1, 2; 3 and 4 masked.
        assert_eq!(d, vec![Some(0.0), Some(1.0), Some(2.0), None, None]);
    }

    #[test]
    fn cutoff_none_matches_unbounded_dijkstra() {
        let (g, w) = shortcut_triangle();
        assert_eq!(
            dijkstra_distances_cutoff(&g, 0, &w, None).unwrap(),
            dijkstra_distances(&g, 0, &w).unwrap()
        );
    }

    #[test]
    fn cutoff_zero_keeps_only_source() {
        let (g, w) = shortcut_triangle();
        let d = dijkstra_distances_cutoff(&g, 0, &w, Some(0.0)).unwrap();
        assert_eq!(d, vec![Some(0.0), None, None]);
    }

    #[test]
    fn cutoff_nan_errors() {
        let (g, w) = shortcut_triangle();
        assert!(dijkstra_distances_cutoff(&g, 0, &w, Some(f64::NAN)).is_err());
    }

    #[test]
    fn multi_source_yields_per_source_distances() {
        let (g, w) = shortcut_triangle();
        let r = dijkstra_distances_multi(&g, &[0, 1], &w, None).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0], dijkstra_distances(&g, 0, &w).unwrap());
        assert_eq!(r[1], dijkstra_distances(&g, 1, &w).unwrap());
    }

    #[test]
    fn multi_source_empty_list_yields_empty_result() {
        let (g, w) = shortcut_triangle();
        let r = dijkstra_distances_multi(&g, &[], &w, None).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn multi_source_propagates_cutoff() {
        let mut g = Graph::with_vertices(4);
        for i in 0..3u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let w = vec![1.0; 3];
        let r = dijkstra_distances_multi(&g, &[0, 3], &w, Some(1.0)).unwrap();
        assert_eq!(r[0], vec![Some(0.0), Some(1.0), None, None]);
        assert_eq!(r[1], vec![None, None, Some(1.0), Some(0.0)]);
    }

    #[test]
    fn multi_source_invalid_vertex_errors() {
        let (g, w) = shortcut_triangle();
        assert!(dijkstra_distances_multi(&g, &[0, 99], &w, None).is_err());
    }

    // ---- SP-001c: mode-aware + all-shortest-paths -----------------------

    fn directed_path_3() -> (Graph, Vec<f64>) {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        (g, vec![1.0, 2.0])
    }

    #[test]
    fn legacy_apis_match_with_mode_out() {
        let (g, w) = shortcut_triangle();
        assert_eq!(
            dijkstra_distances(&g, 0, &w).unwrap(),
            dijkstra_distances_with_mode(&g, 0, &w, DijkstraMode::Out).unwrap()
        );
        let p = dijkstra_paths(&g, 0, &w).unwrap();
        let pm = dijkstra_paths_with_mode(&g, 0, &w, DijkstraMode::Out).unwrap();
        assert_eq!(p, pm);
    }

    #[test]
    fn directed_path_in_mode_reverses_reachability() {
        let (g, w) = directed_path_3();
        // OUT: from 0, reaches forward.
        assert_eq!(
            dijkstra_distances_with_mode(&g, 0, &w, DijkstraMode::Out).unwrap(),
            vec![Some(0.0), Some(1.0), Some(3.0)]
        );
        // IN: from 0, no incoming edges so nothing else reachable.
        assert_eq!(
            dijkstra_distances_with_mode(&g, 0, &w, DijkstraMode::In).unwrap(),
            vec![Some(0.0), None, None]
        );
        // ALL: undirected — vertex 2 reaches 0 too.
        assert_eq!(
            dijkstra_distances_with_mode(&g, 2, &w, DijkstraMode::All).unwrap(),
            vec![Some(3.0), Some(2.0), Some(0.0)]
        );
    }

    #[test]
    fn undirected_modes_agree() {
        let (g, w) = shortcut_triangle();
        for m in [DijkstraMode::Out, DijkstraMode::In, DijkstraMode::All] {
            assert_eq!(
                dijkstra_distances_with_mode(&g, 0, &w, m).unwrap(),
                vec![Some(0.0), Some(1.0), Some(3.0)],
                "mode {m:?}"
            );
        }
    }

    #[test]
    fn paths_with_mode_in_records_reverse_parents() {
        let (g, w) = directed_path_3();
        // IN-mode SPT from vertex 2: parents[1]=2, parents[0]=1.
        let p = dijkstra_paths_with_mode(&g, 2, &w, DijkstraMode::In).unwrap();
        assert_eq!(p.distances, vec![Some(3.0), Some(2.0), Some(0.0)]);
        assert_eq!(p.parents[2], None);
        assert_eq!(p.parents[1], Some(2));
        assert_eq!(p.parents[0], Some(1));
    }

    #[test]
    fn path_to_with_mode_directed_in() {
        let (g, w) = directed_path_3();
        let (vs, es) = dijkstra_path_to_with_mode(&g, 2, 0, &w, DijkstraMode::In)
            .unwrap()
            .unwrap();
        assert_eq!(vs, vec![2, 1, 0]);
        assert_eq!(es, vec![1, 0]);
    }

    #[test]
    fn path_to_with_mode_unreachable_via_out() {
        let (g, w) = directed_path_3();
        // OUT from 2: nothing else reachable (sink).
        assert_eq!(
            dijkstra_path_to_with_mode(&g, 2, 0, &w, DijkstraMode::Out).unwrap(),
            None
        );
    }

    #[test]
    fn cutoff_with_mode_masks_above() {
        let (g, w) = directed_path_3();
        let d =
            dijkstra_distances_cutoff_with_mode(&g, 0, &w, Some(1.5), DijkstraMode::Out).unwrap();
        assert_eq!(d, vec![Some(0.0), Some(1.0), None]);
    }

    #[test]
    fn multi_with_mode_sources_independent() {
        let (g, w) = directed_path_3();
        let r =
            dijkstra_distances_multi_with_mode(&g, &[0, 2], &w, None, DijkstraMode::All).unwrap();
        assert_eq!(r[0], vec![Some(0.0), Some(1.0), Some(3.0)]);
        assert_eq!(r[1], vec![Some(3.0), Some(2.0), Some(0.0)]);
    }

    // ---- SP-001c: all-shortest-paths -------------------

    #[test]
    fn all_paths_diamond_yields_two_geodesics() {
        // 0-1-3 and 0-2-3, all weight 1: two distinct shortest paths
        // to vertex 3.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // edge 0
        g.add_edge(0, 2).unwrap(); // edge 1
        g.add_edge(1, 3).unwrap(); // edge 2
        g.add_edge(2, 3).unwrap(); // edge 3
        let r = dijkstra_all_shortest_paths(&g, 0, &[1.0; 4], DijkstraMode::Out).unwrap();
        assert_eq!(r.nrgeo, vec![1, 1, 1, 2]);
        assert_eq!(r.vertex_paths[0], vec![vec![0]]);
        assert_eq!(r.edge_paths[0], vec![Vec::<EdgeId>::new()]);
        assert_eq!(r.vertex_paths[3].len(), 2);
        let mut paths: Vec<Vec<u32>> = r.vertex_paths[3].clone();
        paths.sort();
        assert_eq!(paths, vec![vec![0, 1, 3], vec![0, 2, 3]]);
    }

    #[test]
    fn all_paths_unique_returns_single_chain() {
        let (g, w) = shortcut_triangle();
        let r = dijkstra_all_shortest_paths(&g, 0, &w, DijkstraMode::Out).unwrap();
        assert_eq!(r.nrgeo, vec![1, 1, 1]);
        assert_eq!(r.vertex_paths[2], vec![vec![0, 1, 2]]);
        assert_eq!(r.edge_paths[2], vec![vec![0, 2]]);
    }

    #[test]
    fn all_paths_unreachable_emits_empty() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let r = dijkstra_all_shortest_paths(&g, 0, &[1.0], DijkstraMode::Out).unwrap();
        assert_eq!(r.nrgeo, vec![1, 1, 0]);
        assert!(r.vertex_paths[2].is_empty());
        assert!(r.edge_paths[2].is_empty());
    }

    #[test]
    fn all_paths_directed_in_mode() {
        let (g, w) = directed_path_3();
        let r = dijkstra_all_shortest_paths(&g, 2, &w, DijkstraMode::In).unwrap();
        assert_eq!(r.nrgeo, vec![1, 1, 1]);
        assert_eq!(r.vertex_paths[0], vec![vec![2, 1, 0]]);
        assert_eq!(r.edge_paths[0], vec![vec![1, 0]]);
    }

    #[test]
    fn all_paths_invalid_source_errors() {
        let (g, _) = shortcut_triangle();
        assert!(dijkstra_all_shortest_paths(&g, 99, &[1.0; 3], DijkstraMode::Out).is_err());
    }

    #[test]
    fn all_paths_zero_weight_alt_is_dropped_by_guard() {
        // Triangle with one zero-weight edge (1→2). Upstream's
        // alt-equal-paths branch only fires when the *connecting* edge
        // has positive weight. So the alt path 0→1→2 (cost 1+0=1)
        // ties the direct 0→2 (cost 1) but is *not* recorded — the
        // 1→2 hop has weight 0. We end up with the direct edge only.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // edge 0, weight 1
        g.add_edge(1, 2).unwrap(); // edge 1, weight 0
        g.add_edge(0, 2).unwrap(); // edge 2, weight 1
        let r = dijkstra_all_shortest_paths(&g, 0, &[1.0, 0.0, 1.0], DijkstraMode::Out).unwrap();
        assert_eq!(r.nrgeo[2], 1);
        assert_eq!(r.vertex_paths[2], vec![vec![0, 2]]);
    }
}
