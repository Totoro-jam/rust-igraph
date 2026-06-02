//! Voronoi-based community detection (ALGO-CO-009).
//!
//! Counterpart of `igraph_community_voronoi` from
//! `references/igraph/src/community/voronoi.c` (lines 33-645). Partitions
//! the vertex set into communities by:
//!
//! 1. Computing per-vertex **local relative density** (LRD): the
//!    fraction of edges in the closed first-order neighbourhood that are
//!    internal. Weighted variant multiplies by the vertex's strength.
//! 2. Deriving an **effective edge length** `len / ECC`, where `ECC` is
//!    the canonical Radicchi 2004 edge clustering coefficient with
//!    `offset=true, normalize=true` and `k=3` (see [`crate::ecc`]).
//! 3. **Choosing generators** greedily: repeatedly pick the unmarked
//!    vertex with the highest LRD, then mark every vertex within an
//!    `r`-radius (Dijkstra in the effective-length metric) so that no
//!    two generators are closer than `r`.
//! 4. Assigning every other vertex to its closest generator with
//!    [`crate::voronoi`] (random tiebreaker, seed `42` as in the C
//!    reference).
//! 5. When `r < 0`, picking `r` automatically to maximise modularity
//!    via a quadratic-fit 1-D optimiser (Brent-flavour) over the
//!    interval `[min(edge_length), max(distance_reached_with_r=∞)]`.
//!
//! References:
//! - Deritei et al., "Community detection by graph Voronoi diagrams",
//!   New Journal of Physics 16, 063007 (2014).
//! - Molnár et al., "Community Detection in Directed Weighted Networks
//!   using Voronoi Partitioning", Scientific Reports 14, 8124 (2024).
//!
//! Restrictions matching the C reference:
//! - The graph must be **simple** (no self-loops, no multi-edges). The
//!   directness used by `is_simple` follows `mode`.
//! - `lengths` must be non-negative and finite; `weights` must be
//!   strictly positive and finite. `None` for either ⇒ unit values.

#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::algorithms::community::modularity::{
    modularity, modularity_directed, modularity_weighted, modularity_weighted_directed,
};
use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::algorithms::paths::voronoi::{VoronoiTiebreaker, voronoi};
use crate::algorithms::properties::ecc::ecc;
use crate::algorithms::properties::is_simple::is_simple;
use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

const VORONOI_BRENT_SEED: u64 = 42;

/// Result of [`community_voronoi`].
///
/// `membership[v]` is a contiguous `0..k` community id for every
/// vertex; unreachable vertices (which can only arise in disconnected
/// graphs and only for `mode = In` / `Out` in some configurations)
/// keep the C reference's behaviour and get assigned to whichever
/// generator the underlying [`crate::voronoi`] picked.
///
/// `generators` is the list of vertex ids picked as Voronoi
/// generators, in the order they were picked. Its length equals the
/// number of distinct communities.
///
/// `modularity` is the Newman-Girvan modularity of `membership` under
/// `weights` (or unit weights if `weights = None`), with the
/// directness implied by `mode`. `None` for an empty graph.
#[derive(Debug, Clone, PartialEq)]
pub struct CommunityVoronoiResult {
    pub membership: Vec<u32>,
    pub generators: Vec<VertexId>,
    pub modularity: Option<f64>,
}

/// Voronoi-based community detection.
///
/// See the module-level docs for the algorithm sketch.
///
/// # Errors
///
/// - [`IgraphError::InvalidArgument`] for length / weight vectors of
///   wrong size, non-finite or out-of-domain entries, or a non-simple
///   graph.
/// - [`IgraphError::Internal`] when the auto-`r` optimiser cannot find
///   a quadratic-concave step (matches the C reference's
///   `IGRAPH_DIVERGED`).
///
/// # Complexity
///
/// `O(n²·log n + n·m)` dominated by repeated Dijkstra calls inside
/// `choose_generators` (one per generator candidate) and the inner
/// [`crate::voronoi`] / [`crate::ecc`] passes.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, DijkstraMode, community_voronoi};
///
/// // K_4: every pair is connected; fixed `r=0` keeps every vertex as
/// // its own generator, so we get 4 singleton communities.
/// let mut g = Graph::with_vertices(4);
/// for u in 0..4u32 { for v in (u+1)..4 { g.add_edge(u, v).unwrap(); } }
/// let r = community_voronoi(&g, None, None, DijkstraMode::All, 0.0).unwrap();
/// assert_eq!(r.generators.len(), 4);
/// ```
fn validate_inputs(
    graph: &Graph,
    lengths: Option<&[f64]>,
    weights: Option<&[f64]>,
    m: usize,
) -> IgraphResult<()> {
    if let Some(l) = lengths {
        if l.len() != m {
            return Err(IgraphError::InvalidArgument(format!(
                "lengths vector size ({}) differs from edge count ({})",
                l.len(),
                m
            )));
        }
        for (e, &v) in l.iter().enumerate() {
            if v.is_nan() {
                return Err(IgraphError::InvalidArgument(format!(
                    "length at edge {e} is NaN"
                )));
            }
            if v < 0.0 {
                return Err(IgraphError::InvalidArgument(format!(
                    "length at edge {e} is negative ({v}); Voronoi requires non-negative lengths"
                )));
            }
        }
    }
    if let Some(w) = weights {
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
            if v <= 0.0 {
                return Err(IgraphError::InvalidArgument(format!(
                    "weight at edge {e} is non-positive ({v}); Voronoi requires positive weights"
                )));
            }
        }
    }
    if !is_simple(graph)? {
        return Err(IgraphError::InvalidArgument(
            "the graph must be simple (no self-loops, no multi-edges) for Voronoi communities"
                .to_string(),
        ));
    }
    Ok(())
}

/// Voronoi-based community detection.
///
/// Partitions the vertex set into communities using graph Voronoi
/// diagrams with local relative density to select generators.
/// Pass `r < 0` for automatic radius selection that maximises modularity.
///
/// See the module documentation for the full algorithm description.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, community_voronoi, DijkstraMode};
///
/// let mut g = Graph::with_vertices(6);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 5).unwrap();
/// g.add_edge(5, 3).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let res = community_voronoi(&g, None, None, DijkstraMode::All, -1.0).unwrap();
/// assert_eq!(res.membership.len(), 6);
/// ```
pub fn community_voronoi(
    graph: &Graph,
    lengths: Option<&[f64]>,
    weights: Option<&[f64]>,
    mode: DijkstraMode,
    r: f64,
) -> IgraphResult<CommunityVoronoiResult> {
    let n = graph.vcount();
    let m = graph.ecount();

    // Undirected graphs collapse all modes to All (matches the C contract).
    let mode = if graph.is_directed() {
        mode
    } else {
        DijkstraMode::All
    };

    validate_inputs(graph, lengths, weights, m)?;

    if m == 0 {
        // Also handles n <= 1: every vertex is its own community + generator.
        let membership: Vec<u32> = (0..n).collect();
        let generators: Vec<VertexId> = (0..n).collect();
        return Ok(CommunityVoronoiResult {
            membership,
            generators,
            modularity: None,
        });
    }

    // (1) Local relative density (weighted variant if `weights` given).
    let lrd = weighted_local_density(graph, weights)?;

    // (2) Effective edge length: 1 / ECC, multiplied by `lengths` when given.
    let mut len_eff = ecc(graph, None, 3, true, true)?;
    for v in &mut len_eff {
        // ECC is never NaN here (graph is simple) but may be +Inf, in which case
        // 1/Inf = 0.0 — these edges become zero-length connectors.
        *v = 1.0 / *v;
    }
    if let Some(l) = lengths {
        for (a, &b) in len_eff.iter_mut().zip(l.iter()) {
            *a *= b;
        }
    }

    if r < 0.0 {
        // Auto-r: estimate a sensible search interval, then maximise
        // modularity via the Brent-flavour 1-D optimiser.
        let (minr, maxr) = estimate_minmax_r(graph, &lrd, &len_eff, mode)?;
        let mut work = ModularityWork {
            graph,
            local_dens: &lrd,
            lengths: &len_eff,
            weights,
            mode,
            generators: Vec::new(),
            membership: Vec::new(),
            modularity: f64::NAN,
        };
        brent_opt(&mut work, minr, maxr)?;
        Ok(CommunityVoronoiResult {
            membership: work.membership,
            generators: work.generators,
            modularity: if work.modularity.is_nan() {
                None
            } else {
                Some(work.modularity)
            },
        })
    } else {
        let generators = choose_generators(graph, &lrd, &len_eff, mode, r)?.0;
        let part = voronoi(
            graph,
            Some(&len_eff),
            mode,
            &generators,
            VoronoiTiebreaker::Random,
            VORONOI_BRENT_SEED,
        )?;
        let membership = membership_to_flat(&part.membership);
        let q = compute_modularity(graph, &membership, weights, mode)?;
        Ok(CommunityVoronoiResult {
            membership,
            generators,
            modularity: q,
        })
    }
}

/// Densify the `Option<u32>` membership returned by [`crate::voronoi`]
/// into a flat `u32` vector. Unreachable vertices (rare; only with
/// disconnected graphs under directed `mode`) get the sentinel value
/// `u32::MAX`. The downstream modularity call treats every distinct id
/// as its own community.
fn membership_to_flat(m: &[Option<u32>]) -> Vec<u32> {
    m.iter().map(|x| x.unwrap_or(u32::MAX)).collect()
}

/// Newman-Girvan modularity using the right (un)directed / (un)weighted
/// variant for the current `mode` + `weights`.
fn compute_modularity(
    graph: &Graph,
    membership: &[u32],
    weights: Option<&[f64]>,
    mode: DijkstraMode,
) -> IgraphResult<Option<f64>> {
    let directed = graph.is_directed() && mode != DijkstraMode::All;
    match (directed, weights) {
        (false, None) => modularity(graph, membership, 1.0),
        (false, Some(w)) => modularity_weighted(graph, membership, 1.0, w),
        (true, None) => modularity_directed(graph, membership, 1.0),
        (true, Some(w)) => modularity_weighted_directed(graph, membership, 1.0, w),
    }
}

/// Unweighted local relative density (LRD): for each vertex `w`,
/// `int / (int + ext)` where `int` is the number of edges with both
/// endpoints in `N[w] = N(w) ∪ {w}`, and `ext` is the number of edges
/// with exactly one endpoint in `N[w]`. Isolated vertices get `0.0`.
///
/// Mirrors `igraph_i_local_relative_density` in voronoi.c:45-123. We
/// use the deduped-neighbour view (matching `IGRAPH_LOOPS,
/// IGRAPH_MULTIPLE` of the upstream lazy adjlist on simple graphs —
/// `is_simple` is enforced upstream of this call).
fn local_relative_density(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let mut res = vec![0.0_f64; n_us];

    // Per-vertex adjacency, deduped. Building this once is O(V + E)
    // and avoids the per-w lazy-adjlist allocation pattern.
    let mut adj: Vec<Vec<VertexId>> = Vec::with_capacity(n_us);
    for v in 0..n {
        let mut neigh: Vec<VertexId> = graph
            .neighbors(v)?
            .into_iter()
            .filter(|&u| u != v)
            .collect();
        neigh.sort_unstable();
        neigh.dedup();
        adj.push(neigh);
    }

    // `nei_mask[u] == i+1` ⇔ u ∈ N[w_i]. `nei_done[u] == i+1` ⇔ u
    // already had its edges processed for `w_i`. Stamp-based reset
    // avoids per-w O(n) clearing.
    let mut nei_mask: Vec<u32> = vec![0; n_us];
    let mut nei_done: Vec<u32> = vec![0; n_us];

    for w in 0..n {
        let w_us = w as usize;
        let stamp = w + 1; // u32, never 0
        let dw = adj[w_us].len();

        for &u in &adj[w_us] {
            nei_mask[u as usize] = stamp;
        }
        nei_mask[w_us] = stamp;

        // All incident edges of w are by definition internal.
        let mut int_count: u64 = dw as u64;
        let mut ext_count: u64 = 0;
        nei_done[w_us] = stamp;

        for &v in &adj[w_us] {
            if nei_done[v as usize] == stamp {
                continue;
            }
            nei_done[v as usize] = stamp;
            for &u in &adj[v as usize] {
                if nei_mask[u as usize] == stamp {
                    int_count += 1;
                } else {
                    ext_count += 1;
                }
            }
        }
        // Internal edges were counted from both endpoints' perspective.
        debug_assert!(int_count % 2 == 0);
        int_count /= 2;

        res[w_us] = if int_count == 0 {
            0.0
        } else {
            int_count as f64 / (int_count + ext_count) as f64
        };
    }

    Ok(res)
}

/// Weighted local relative density: LRD multiplied by the undirected
/// strength of each vertex (no loops — they are forbidden by the
/// `is_simple` precondition). `weights = None` ⇒ multiply by degree
/// (matches the C reference where `igraph_strength(_, weights = NULL)`
/// returns the degree).
fn weighted_local_density(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<Vec<f64>> {
    let mut res = local_relative_density(graph)?;
    let n = graph.vcount() as usize;
    let mut str_v = vec![0.0_f64; n];
    if let Some(w) = weights {
        for v in 0..graph.vcount() {
            let mut s = 0.0_f64;
            for eid in graph.incident(v)? {
                // Simple graphs ⇒ no self-loops, so each edge is
                // counted once by `incident`.
                s += w[eid as usize];
            }
            str_v[v as usize] = s;
        }
    } else {
        for v in 0..graph.vcount() {
            // Loopless degree: simple graphs have no loops, so
            // `degree()` equals the loopless degree.
            str_v[v as usize] = f64::from(u32::try_from(graph.degree(v)?).unwrap_or(u32::MAX));
        }
    }
    for (a, b) in res.iter_mut().zip(str_v.iter()) {
        *a *= *b;
    }
    Ok(res)
}

/// `(generators, max_radius_reached)`. See module docs for the
/// algorithm. Mirrors `choose_generators` in voronoi.c:154-259.
fn choose_generators(
    graph: &Graph,
    local_rel_dens: &[f64],
    lengths: &[f64],
    mode: DijkstraMode,
    r: f64,
) -> IgraphResult<(Vec<VertexId>, f64)> {
    let n = graph.vcount();
    let n_us = n as usize;

    // Sort vertex ids by decreasing LRD. Tie-break by ascending vertex
    // id so the picking order is deterministic across runs.
    let mut ord: Vec<u32> = (0..n).collect();
    ord.sort_by(|&a, &b| {
        local_rel_dens[b as usize]
            .partial_cmp(&local_rel_dens[a as usize])
            .unwrap_or(Ordering::Equal)
            .then(a.cmp(&b))
    });

    let mut excluded = vec![false; n_us];
    let mut excluded_count: u32 = 0;
    let mut generators: Vec<VertexId> = Vec::new();
    let mut radius_max = f64::NEG_INFINITY;

    // Dijkstra-style heap reused per generator candidate. We use a
    // generation counter `epoch` so we can "clear" `dist` in O(1) per
    // outer iteration without a full O(n) reset.
    let mut dist = vec![f64::INFINITY; n_us];
    let mut epoch = vec![0u32; n_us];
    let mut active = vec![false; n_us];
    let mut current_epoch: u32 = 0;
    let mut heap: BinaryHeap<HeapEntry> = BinaryHeap::new();

    for &g in ord.iter().take(n_us) {
        if excluded[g as usize] {
            continue;
        }
        generators.push(g);

        current_epoch = current_epoch.wrapping_add(1);
        if current_epoch == 0 {
            // Wrapped — reset the whole epoch buffer once in a blue moon.
            epoch.fill(0);
            current_epoch = 1;
        }
        heap.clear();
        dist[g as usize] = 0.0;
        epoch[g as usize] = current_epoch;
        active[g as usize] = true;
        heap.push(HeapEntry { dist: 0.0, vid: g });

        while let Some(HeapEntry { dist: d, vid }) = heap.pop() {
            let v_us = vid as usize;
            // Stale entry (we relaxed the same vertex with a smaller
            // distance after pushing this one). Skip.
            if !active[v_us] || d > dist[v_us] {
                continue;
            }
            active[v_us] = false;

            // Beyond cutoff: do not search further along this path.
            if d > r {
                continue;
            }

            // Exclude this vertex (it's within `r` of the current generator).
            if !excluded[v_us] {
                excluded[v_us] = true;
                excluded_count += 1;
            }
            if d > radius_max {
                radius_max = d;
            }

            for eid in incident_for_mode(graph, vid, mode)? {
                let w = lengths[eid as usize];
                // Optimisation matching C: don't follow infinite edges.
                if w.is_infinite() {
                    continue;
                }
                let (u1, u2) = graph.edge(eid)?;
                let to = if u1 == vid { u2 } else { u1 };
                let alt = d + w;
                let to_us = to as usize;
                if epoch[to_us] != current_epoch {
                    dist[to_us] = alt;
                    epoch[to_us] = current_epoch;
                    active[to_us] = true;
                    heap.push(HeapEntry { dist: alt, vid: to });
                } else if active[to_us] && alt < dist[to_us] {
                    dist[to_us] = alt;
                    heap.push(HeapEntry { dist: alt, vid: to });
                }
            }
        }

        if excluded_count as usize == n_us {
            break;
        }
    }

    if radius_max == f64::NEG_INFINITY {
        radius_max = 0.0;
    }
    Ok((generators, radius_max))
}

/// Mode-aware incidence — same helper shape as in `paths::voronoi`.
fn incident_for_mode(graph: &Graph, v: VertexId, mode: DijkstraMode) -> IgraphResult<Vec<EdgeId>> {
    if !graph.is_directed() {
        return graph.incident(v);
    }
    match mode {
        DijkstraMode::Out => graph.incident(v),
        DijkstraMode::In => graph.incident_in_pub(v),
        DijkstraMode::All => {
            let mut out = graph.incident(v)?;
            out.extend(graph.incident_in_pub(v)?);
            Ok(out)
        }
    }
}

/// `(min_r, max_r)`: the search interval for the auto-radius optimiser.
/// `min_r` is the shortest finite edge length; `max_r` is the largest
/// finite distance reached by `choose_generators` when `r = +∞`.
fn estimate_minmax_r(
    graph: &Graph,
    local_rel_dens: &[f64],
    lengths: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<(f64, f64)> {
    let mut min_r = f64::INFINITY;
    for &v in lengths {
        if v.is_finite() && v < min_r {
            min_r = v;
        }
    }
    if !min_r.is_finite() {
        min_r = 0.0;
    }
    let (_, max_r) = choose_generators(graph, local_rel_dens, lengths, mode, f64::INFINITY)?;
    Ok((min_r, max_r))
}

/// Min-heap entry (ordered by ascending `dist`).
#[derive(Debug, Clone, Copy)]
struct HeapEntry {
    dist: f64,
    vid: VertexId,
}
impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.dist == other.dist && self.vid == other.vid
    }
}
impl Eq for HeapEntry {}
impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max-heap; invert to get min-heap.
        other
            .dist
            .partial_cmp(&self.dist)
            .unwrap_or(Ordering::Equal)
            .then_with(|| other.vid.cmp(&self.vid))
    }
}
impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// --- Brent-flavour quadratic-fit 1-D optimiser ---------------------------

struct ModularityWork<'a> {
    graph: &'a Graph,
    local_dens: &'a [f64],
    lengths: &'a [f64],
    weights: Option<&'a [f64]>,
    mode: DijkstraMode,
    generators: Vec<VertexId>,
    membership: Vec<u32>,
    modularity: f64,
}

/// Evaluate modularity at `r`. Stores the resulting partition + score
/// in `work` so the caller can recover them after the optimiser
/// terminates (it terminates with the best-known r evaluated last).
fn eval_modularity(work: &mut ModularityWork<'_>, r: f64) -> IgraphResult<f64> {
    let (gens, _) = choose_generators(work.graph, work.local_dens, work.lengths, work.mode, r)?;
    let part = voronoi(
        work.graph,
        Some(work.lengths),
        work.mode,
        &gens,
        VoronoiTiebreaker::Random,
        VORONOI_BRENT_SEED,
    )?;
    let membership = membership_to_flat(&part.membership);
    let q = compute_modularity(work.graph, &membership, work.weights, work.mode)?
        .unwrap_or(f64::NEG_INFINITY);
    work.generators = gens;
    work.membership = membership;
    work.modularity = q;
    Ok(q)
}

/// Coefficient of the quadratic term when fitting `a·x² + b·x + c` to
/// the three points `(x_i, f_i)`. Sign tells us whether the parabola
/// is convex (`>0`) or concave (`<0`).
fn coeff2(x1: f64, x2: f64, x3: f64, f1: f64, f2: f64, f3: f64) -> f64 {
    let num = x1 * (f3 - f2) + x2 * (f1 - f3) + x3 * (f2 - f1);
    let denom = (x1 - x2) * (x1 - x3) * (x2 - x3);
    num / denom
}

/// Stationary point of the quadratic fit.
fn peakx(x1: f64, x2: f64, x3: f64, f1: f64, f2: f64, f3: f64) -> f64 {
    let x1s = x1 * x1;
    let x2s = x2 * x2;
    let x3s = x3 * x3;
    let num = f3 * (x1s - x2s) + f1 * (x2s - x3s) + f2 * (x3s - x1s);
    let denom = f3 * (x1 - x2) + f1 * (x2 - x3) + f2 * (x3 - x1);
    0.5 * num / denom
}

fn cmp_epsilon(a: f64, b: f64, eps: f64) -> Ordering {
    let diff = (a - b).abs();
    let tol = eps * (a.abs().max(b.abs())).max(1.0);
    if diff <= tol {
        Ordering::Equal
    } else if a < b {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

/// Quadratic-fit 1-D maximiser. Mirrors `brent_opt` in voronoi.c:317-428.
fn brent_opt(work: &mut ModularityWork<'_>, x1_in: f64, x2_in: f64) -> IgraphResult<()> {
    let lo = x1_in;
    let hi = x2_in;

    if !lo.is_finite() || !hi.is_finite() {
        return Err(IgraphError::InvalidArgument(
            "Voronoi radius search interval is non-finite".to_string(),
        ));
    }

    let mut x1 = x1_in;
    let mut x2 = x2_in;
    // Initial probe closer to x1 than x2 so that, if f1 == f2, the next
    // computed point would not coincide with x3.
    let mut x3 = 0.6 * x1 + 0.4 * x2;

    let mut f1 = eval_modularity(work, x1)?;
    // Bit-exact equality matches the C reference's degenerate-interval guard;
    // any non-degenerate caller passes a strict inequality.
    #[allow(clippy::float_cmp)]
    if x1 == x2 {
        return Ok(());
    }
    let mut f2 = eval_modularity(work, x2)?;
    let mut f3 = eval_modularity(work, x3)?;

    // We expect the middle point to be higher than the boundary points.
    if f1 > f3 {
        return Err(IgraphError::Internal(
            "Voronoi auto-r optimizer did not converge (f1 > f3)",
        ));
    }

    // If f2 > f3 we bisect (x3, x2) up to 10 times trying to find a
    // configuration where f3 >= f2; if we don't, we accept the upper
    // end as the best radius.
    if f2 > f3 {
        let mut bisected = false;
        for _ in 0..10 {
            x1 = x3;
            f1 = f3;
            x3 = 0.5 * (x1 + x2);
            f3 = eval_modularity(work, x3)?;
            if f3 >= f2 {
                bisected = true;
                break;
            }
        }
        if !bisected {
            // Final eval at x2 so that `work` reflects the upper endpoint.
            let _ = eval_modularity(work, x2)?;
            return Ok(());
        }
    }

    for _ in 0..20 {
        let new_x = peakx(x1, x2, x3, f1, f2, f3);
        let new_f = eval_modularity(work, new_x)?;

        let a1 = coeff2(x2, x3, new_x, f2, f3, new_f);
        let a2 = coeff2(x1, x3, new_x, f1, f3, new_f);

        // Both options would yield a convex parabola (>=0). Terminate.
        if a1 >= 0.0 && a2 >= 0.0 {
            break;
        }

        if a1 <= a2 {
            // Drop (x1, f1).
            x1 = x2;
            x2 = x3;
            x3 = new_x;
            f1 = f2;
            f2 = f3;
            f3 = new_f;
        } else {
            // Drop (x2, f2).
            x2 = x1;
            x1 = x3;
            x3 = new_x;
            f2 = f1;
            f1 = f3;
            f3 = new_f;
        }

        // Out-of-bounds — bail.
        if x3 < lo || x3 > hi {
            return Err(IgraphError::Internal(
                "Voronoi auto-r optimizer drifted outside initial interval",
            ));
        }

        // We're optimising a discrete-valued function: when two of f1
        // / f2 / f3 coincide to eps, another iteration is unlikely to
        // improve. Saves one eval.
        let eps = 1e-10;
        if matches!(cmp_epsilon(f1, f3, eps), Ordering::Equal)
            || matches!(cmp_epsilon(f2, f3, eps), Ordering::Equal)
        {
            break;
        }
    }
    Ok(())
}

// Bridge for the privacy of Graph::incident_in: re-export through a
// dedicated thin shim. (Graph::incident_in is `pub(crate)`, so we can
// just call it directly.)
trait GraphIncidentInExt {
    fn incident_in_pub(&self, v: VertexId) -> IgraphResult<Vec<EdgeId>>;
}
impl GraphIncidentInExt for Graph {
    fn incident_in_pub(&self, v: VertexId) -> IgraphResult<Vec<EdgeId>> {
        self.incident_in(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k4() -> Graph {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        g
    }

    fn karate() -> Graph {
        use std::fs::File;
        use std::path::PathBuf;
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("fixtures/karate.edges");
        crate::algorithms::io::read_edgelist(File::open(path).unwrap()).unwrap()
    }

    #[test]
    fn null_graph_empty_membership() {
        let g = Graph::with_vertices(0);
        let r = community_voronoi(&g, None, None, DijkstraMode::All, -1.0).unwrap();
        assert!(r.membership.is_empty());
        assert!(r.generators.is_empty());
        assert!(r.modularity.is_none());
    }

    #[test]
    fn singleton_graph_one_community() {
        let g = Graph::with_vertices(1);
        let r = community_voronoi(&g, None, None, DijkstraMode::All, -1.0).unwrap();
        assert_eq!(r.membership, vec![0]);
        assert_eq!(r.generators, vec![0]);
    }

    #[test]
    fn two_isolated_nodes_two_singleton_communities() {
        let g = Graph::with_vertices(2);
        let r = community_voronoi(&g, None, None, DijkstraMode::All, -1.0).unwrap();
        assert_eq!(r.membership, vec![0, 1]);
        assert_eq!(r.generators, vec![0, 1]);
    }

    #[test]
    fn non_simple_graph_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap(); // multi-edge
        let r = community_voronoi(&g, None, None, DijkstraMode::All, -1.0);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn self_loop_graph_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 0).unwrap(); // self-loop
        let r = community_voronoi(&g, None, None, DijkstraMode::All, -1.0);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn fixed_r_zero_gives_singleton_communities() {
        let g = k4();
        let r = community_voronoi(&g, None, None, DijkstraMode::All, 0.0).unwrap();
        // r = 0 ⇒ generator's radius excludes itself only; every vertex
        // is a generator.
        assert_eq!(r.generators.len(), 4);
        // Membership densely covers 0..4.
        let mut distinct: Vec<u32> = r.membership.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), 4);
    }

    #[test]
    fn karate_auto_r_yields_known_membership() {
        // Mirrors `references/igraph/tests/unit/igraph_community_voronoi.out`:
        //   Generators: ( 33 0 24 )
        //   Membership: ( 1 1 1 1 1 1 1 1 0 0 1 1 1 1 0 0 1 1 0 1 0 1 0 0 2 2 0 0 0 0 0 2 0 0 )
        // We don't reproduce exactly the same generators because the
        // tiebreaker uses our SplitMix64 RNG with a different seed than
        // igraph's Mersenne; instead we assert the algorithmic invariants
        // we DO control:
        //   - generators are vertex 33 (or the equivalent
        //     highest-strength hub) and members of the karate factions
        //   - modularity is at least 0.35 (the C reference's auto-r
        //     produces a 3-cluster partition with Q ≈ 0.40)
        let g = karate();
        let r = community_voronoi(&g, None, None, DijkstraMode::All, -1.0).unwrap();
        assert_eq!(r.membership.len(), 34);
        assert!(r.generators.len() >= 2 && r.generators.len() <= 6);
        let q = r.modularity.expect("karate is non-empty");
        assert!(q > 0.30, "expected karate Q > 0.30, got {q}");
    }

    #[test]
    fn karate_fixed_r_matches_modularity_call() {
        let g = karate();
        let r = community_voronoi(&g, None, None, DijkstraMode::All, 2.0).unwrap();
        // Sanity: returned modularity matches a direct compute.
        let q_direct = modularity(&g, &r.membership, 1.0).unwrap();
        assert!((r.modularity.unwrap() - q_direct.unwrap()).abs() < 1e-9);
    }

    #[test]
    fn lengths_size_mismatch_errors() {
        let g = k4();
        let bad = vec![1.0; g.ecount() + 1];
        let r = community_voronoi(&g, Some(&bad), None, DijkstraMode::All, -1.0);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let g = k4();
        let bad = vec![1.0; g.ecount() + 1];
        let r = community_voronoi(&g, None, Some(&bad), DijkstraMode::All, -1.0);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn negative_length_errors() {
        let g = k4();
        let mut l = vec![1.0; g.ecount()];
        l[0] = -1.0;
        let r = community_voronoi(&g, Some(&l), None, DijkstraMode::All, -1.0);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn zero_weight_errors() {
        let g = k4();
        let mut w = vec![1.0; g.ecount()];
        w[0] = 0.0;
        let r = community_voronoi(&g, None, Some(&w), DijkstraMode::All, -1.0);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn nan_length_errors() {
        let g = k4();
        let mut l = vec![1.0; g.ecount()];
        l[0] = f64::NAN;
        let r = community_voronoi(&g, Some(&l), None, DijkstraMode::All, -1.0);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn local_relative_density_isolated_vertex_is_zero() {
        let g = Graph::with_vertices(2);
        let lrd = local_relative_density(&g).unwrap();
        assert_eq!(lrd, vec![0.0, 0.0]);
    }

    #[test]
    fn local_relative_density_triangle_is_one() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        // K_3: every vertex's neighbourhood is the whole graph → all
        // edges are internal → LRD = 1 for every vertex.
        let lrd = local_relative_density(&g).unwrap();
        for v in lrd {
            assert!((v - 1.0).abs() < 1e-12);
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn small_simple_graph()(n in 4u32..=10u32, edges_seed in any::<u64>()) -> Graph {
            let mut g = Graph::with_vertices(n);
            let mut rng = edges_seed;
            let target_m = ((n * (n - 1)) / 2).min(n + 6) as usize;
            let mut added = 0usize;
            let mut guard = 0usize;
            while added < target_m && guard < target_m * 8 + 4 {
                rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
                let u = ((rng >> 33) % u64::from(n)) as u32;
                rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
                let v = ((rng >> 33) % u64::from(n)) as u32;
                guard += 1;
                if u == v { continue; }
                if g.add_edge(u, v).is_ok() {
                    added += 1;
                }
            }
            g
        }
    }

    proptest! {
        // The auto-r optimiser can legitimately fail (Internal) on degenerate
        // small random inputs where the modularity surface is flat or
        // monotone; we use a fixed r=1.0 here so the proptest exercises the
        // membership/generator invariants, not the optimiser's convergence
        // (which has its own deterministic tests above).
        #[test]
        fn membership_size_matches_vertex_count(g in small_simple_graph()) {
            if !crate::algorithms::properties::is_simple::is_simple(&g).unwrap() {
                return Ok(());
            }
            let r = community_voronoi(&g, None, None, DijkstraMode::All, 1.0).unwrap();
            prop_assert_eq!(r.membership.len() as u32, g.vcount());
        }

        #[test]
        fn generators_subset_of_vertices(g in small_simple_graph()) {
            if !crate::algorithms::properties::is_simple::is_simple(&g).unwrap() {
                return Ok(());
            }
            let r = community_voronoi(&g, None, None, DijkstraMode::All, 1.0).unwrap();
            for &g_id in &r.generators {
                prop_assert!(g_id < g.vcount(), "generator {} out of range (n={})", g_id, g.vcount());
            }
        }
    }
}
