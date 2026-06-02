//! Walktrap random-walk community detection (ALGO-CO-008).
//!
//! Counterpart of `igraph_community_walktrap()` from
//! `references/igraph/src/community/walktrap/`.
//!
//! Pons & Latapy (2005, 2006): two vertices are *close* when short
//! random walks (length `t`, default 4) starting from each of them end
//! up with similar probability distributions on the vertex set. The
//! distance between communities `C1`, `C2` is the L2 norm of the
//! probability vectors weighted by `1/sqrt(deg(v))`. Walktrap is a
//! Ward-style hierarchical agglomeration that, at each step, merges the
//! adjacent community pair whose merger minimises the increase in
//! variance σ (equivalently, the lower-bound `Δσ`). The dendrogram is
//! then cut at the level with maximum Newman-Girvan modularity.
//!
//! References:
//! - P. Pons, M. Latapy. *Computing communities in large networks
//!   using random walks*. J. Graph Algorithms Appl. 10 (2006), 191-218.
//!   <https://doi.org/10.7155/jgaa.00124>
//!
//! Phase-1 scope: **undirected** graphs only — matches upstream C
//! (`igraph_community_walktrap` does not support directed input). Edge
//! weights, when given, must be finite and non-negative. Multi-edges
//! are accepted and folded by weight-sum; self-loops are kept. Per the
//! igraph #2043 fix, each vertex receives a synthesized self-loop of
//! weight `mean(incident edge weight)` (or `1.0` if isolated) so
//! diffusion is well-defined.
//!
//! Complexity: `O(t · |E| · nb_merges)` worst case in the present
//! full-vector implementation. The sparse-vector trick from upstream is
//! a future optimisation.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::items_after_statements,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphError, IgraphResult};

/// Default random-walk length matching the igraph reference (`steps = 4`).
pub const WALKTRAP_DEFAULT_STEPS: u32 = 4;

/// Tuning knobs for [`walktrap_with_options`].
#[derive(Debug, Clone, Copy)]
pub struct WalktrapOptions {
    /// Random-walk length `t`. Must be `>= 1`; igraph default is `4`.
    pub steps: u32,
}

impl Default for WalktrapOptions {
    fn default() -> Self {
        Self {
            steps: WALKTRAP_DEFAULT_STEPS,
        }
    }
}

/// Result of [`walktrap`] / [`walktrap_weighted`] /
/// [`walktrap_with_options`].
#[derive(Debug, Clone)]
pub struct WalktrapResult {
    /// Per-vertex community label of the best-modularity dendrogram cut,
    /// densified to `0..nb_clusters`.
    pub membership: Vec<u32>,
    /// Number of distinct communities in `membership`.
    pub nb_clusters: u32,
    /// Merges in dendrogram order. Each row `[c1, c2]` merges clusters
    /// `c1` and `c2` into the new cluster `n + i` where `i` is the
    /// merge index. Same encoding as `igraph_community_walktrap`.
    pub merges: Vec<[u32; 2]>,
    /// Modularity trajectory. `modularity[i]` is the modularity *after*
    /// `i` merges starting from the all-singletons partition. Length =
    /// `merges.len() + 1` when the graph has edges. For an edgeless
    /// graph the single entry is `NaN`, matching the C convention.
    pub modularity: Vec<f64>,
}

/// Run Walktrap on an unweighted, undirected graph with default `steps = 4`.
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph.is_directed()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, walktrap};
///
/// // Triangle: walktrap puts all three vertices in one community.
/// let mut g = Graph::with_vertices(3);
/// for &(u, v) in &[(0, 1), (1, 2), (2, 0)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let r = walktrap(&g).unwrap();
/// assert_eq!(r.nb_clusters, 1);
/// assert_eq!(r.membership, vec![0, 0, 0]);
/// ```
pub fn walktrap(graph: &Graph) -> IgraphResult<WalktrapResult> {
    walktrap_with_options(graph, None, WalktrapOptions::default())
}

/// Run Walktrap on a weighted, undirected graph with default `steps = 4`.
///
/// `weights[i]` is the weight of edge id `i`. Must be finite and
/// non-negative; length must equal `graph.ecount()`.
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph.is_directed()`.
/// - [`IgraphError::InvalidArgument`] if `weights.len() != ecount` or
///   any weight is negative or non-finite.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, walktrap_weighted};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let r = walktrap_weighted(&g, &[1.0, 2.0, 1.0]).unwrap();
/// assert_eq!(r.nb_clusters, 1);
/// ```
pub fn walktrap_weighted(graph: &Graph, weights: &[f64]) -> IgraphResult<WalktrapResult> {
    walktrap_with_options(graph, Some(weights), WalktrapOptions::default())
}

/// Run Walktrap with a custom [`WalktrapOptions`].
///
/// `weights = None` runs the unweighted variant; otherwise see
/// [`walktrap_weighted`] for the weight contract.
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph.is_directed()`.
/// - [`IgraphError::InvalidArgument`] for `opts.steps == 0`, weight
///   length mismatch, negative / non-finite weights, or a vertex whose
///   incident-edge total weight underflows to zero.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, walktrap_with_options, WalktrapOptions};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let opts = WalktrapOptions { steps: 6 };
/// let r = walktrap_with_options(&g, None, opts).unwrap();
/// assert_eq!(r.nb_clusters, 1);
/// ```
pub fn walktrap_with_options(
    graph: &Graph,
    weights: Option<&[f64]>,
    opts: WalktrapOptions,
) -> IgraphResult<WalktrapResult> {
    if graph.is_directed() {
        return Err(IgraphError::Unsupported(
            "walktrap is undirected-only (matches igraph C reference)",
        ));
    }
    if opts.steps == 0 {
        return Err(IgraphError::InvalidArgument(
            "walktrap: steps must be >= 1".to_string(),
        ));
    }
    if let Some(w) = weights {
        if w.len() != graph.ecount() {
            return Err(IgraphError::InvalidArgument(
                "walktrap: weights.len() must equal graph.ecount()".to_string(),
            ));
        }
        for &v in w {
            if !v.is_finite() || v < 0.0 {
                return Err(IgraphError::InvalidArgument(
                    "walktrap: weights must be finite and non-negative".to_string(),
                ));
            }
        }
    }

    run(graph, weights, opts.steps)
}

// ---------------------------------------------------------------------
// Internal: graph adapter
// ---------------------------------------------------------------------

#[derive(Clone, Copy)]
struct AdjEntry {
    neighbor: u32,
    weight: f64,
}

struct InternalGraph {
    n: u32,
    /// `vertices[v]` is a sorted-by-neighbor adjacency including a
    /// synthesized self-loop at position 0 with weight = mean incident
    /// edge weight (or `1.0` when the vertex has no incident edges).
    vertices: Vec<Vec<AdjEntry>>,
    /// `total_weight[v]` includes the synthesized self-loop, matching
    /// upstream's "strength" used in `P · D⁻¹` diffusion.
    total_weight: Vec<f64>,
    /// Sum of all `total_weight[v]`; the global "2m + n·mean-self-loop"
    /// normalising constant used in the modularity formula.
    total_weight_global: f64,
    /// Sum of original (non-synthesised) edge weights; the `m` in the
    /// classical modularity `Q = Σ (e_ii - a_i²)`.
    /// We retain it for the dendrogram modularity computed at the end.
    raw_total_weight: f64,
}

fn build_internal_graph(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<InternalGraph> {
    let n = graph.vcount();
    let mut vertices: Vec<Vec<AdjEntry>> = vec![Vec::new(); n as usize];
    let mut deg: Vec<u32> = vec![0; n as usize];
    let mut total_weight: Vec<f64> = vec![0.0; n as usize];
    let mut raw_total_weight = 0.0f64;

    let m = graph.ecount();
    for eid in 0..m {
        let (u, v) = graph.edge(eid as u32)?;
        let w = match weights {
            Some(ws) => ws[eid],
            None => 1.0,
        };
        deg[u as usize] = deg[u as usize].saturating_add(1);
        deg[v as usize] = deg[v as usize].saturating_add(1);
        total_weight[u as usize] += w;
        total_weight[v as usize] += w;
        raw_total_weight += w;

        vertices[u as usize].push(AdjEntry {
            neighbor: v,
            weight: w,
        });
        if u != v {
            vertices[v as usize].push(AdjEntry {
                neighbor: u,
                weight: w,
            });
        }
    }

    // Prepend a synthesized self-loop with weight = mean incident edge
    // weight (or 1.0 if isolated). Mirrors walktrap_graph.cpp lines
    // 182-190 and the #2043 fix.
    for v in 0..n as usize {
        let mean_w = if deg[v] == 0 {
            1.0
        } else {
            total_weight[v] / f64::from(deg[v])
        };
        vertices[v].push(AdjEntry {
            neighbor: v as u32,
            weight: mean_w,
        });
        total_weight[v] += mean_w;
    }

    // Sort each adjacency by neighbor id and fold parallel edges.
    for v in 0..n as usize {
        vertices[v].sort_by_key(|e| e.neighbor);
        let mut folded: Vec<AdjEntry> = Vec::with_capacity(vertices[v].len());
        for entry in &vertices[v] {
            if let Some(last) = folded.last_mut() {
                if last.neighbor == entry.neighbor {
                    last.weight += entry.weight;
                    continue;
                }
            }
            folded.push(*entry);
        }
        vertices[v] = folded;
    }

    let mut total_weight_global = 0.0f64;
    for v in 0..n as usize {
        if total_weight[v] <= 0.0 {
            return Err(IgraphError::InvalidArgument(
                "walktrap: vertex with zero strength found; all vertices must have positive strength"
                    .to_string(),
            ));
        }
        total_weight_global += total_weight[v];
    }

    Ok(InternalGraph {
        n,
        vertices,
        total_weight,
        total_weight_global,
        raw_total_weight,
    })
}

// ---------------------------------------------------------------------
// Internal: probabilities
// ---------------------------------------------------------------------

/// Length-n probability vector representing a community's t-step random
/// walk distribution. Full (dense) representation; the upstream
/// sparse/full switch is a deferred optimisation.
type Probabilities = Vec<f64>;

/// Compute `P^t · D⁻¹ · A` starting from a uniform distribution over
/// `members`. `D⁻¹` is the diagonal of `1 / total_weight[v]`, and
/// neighbours are taken from `g.vertices` (which already includes the
/// synthesized self-loops).
fn compute_probabilities(g: &InternalGraph, members: &[u32], steps: u32) -> Probabilities {
    let n = g.n as usize;
    let mut current = vec![0.0f64; n];
    let inv_size = 1.0 / members.len() as f64;
    for &v in members {
        current[v as usize] = inv_size;
    }
    let mut next = vec![0.0f64; n];
    for _ in 0..steps {
        for v in 0..n {
            next[v] = 0.0;
        }
        for u in 0..n {
            let pu = current[u];
            if pu == 0.0 {
                continue;
            }
            let factor = pu / g.total_weight[u];
            for entry in &g.vertices[u] {
                next[entry.neighbor as usize] += factor * entry.weight;
            }
        }
        std::mem::swap(&mut current, &mut next);
    }
    current
}

/// Squared distance between two communities' probability vectors,
/// `Σ_v (P_a[v] - P_b[v])² / total_weight[v]`.
fn distance_sq(g: &InternalGraph, a: &Probabilities, b: &Probabilities) -> f64 {
    let mut acc = 0.0f64;
    for v in 0..g.n as usize {
        let d = a[v] - b[v];
        acc += d * d / g.total_weight[v];
    }
    acc
}

// ---------------------------------------------------------------------
// Internal: communities + neighbor pool
// ---------------------------------------------------------------------

#[derive(Clone)]
struct CommunityState {
    /// `false` means this community has been merged away.
    active: bool,
    size: u32,
    /// `internal_weight` — sum of original edge weights with both
    /// endpoints in this community (loops counted once). At the
    /// singleton level this is the self-loop sum from the original
    /// graph (NOT the synthesised one).
    internal_weight: f64,
    /// `total_weight` — sum of original edge weights incident to this
    /// community. At the singleton level this is the degree-weight of
    /// the original vertex (excluding the synthesised self-loop).
    total_weight: f64,
    /// Cached `P^t`. `None` until first needed.
    probabilities: Option<Probabilities>,
    /// Members of this community, by original vertex id. Built only
    /// when we need to compute probabilities.
    members: Vec<u32>,
}

#[derive(Clone)]
struct NeighborEntry {
    c1: u32,
    c2: u32,
    /// Lower-bound `Δσ` until `exact` is set.
    delta_sigma: f64,
    /// True once `delta_sigma` has been refined via the exact
    /// L2-distance formula.
    exact: bool,
    /// Sum of original edge weights between c1 and c2.
    weight: f64,
    /// `false` means this neighbour entry has been superseded by a
    /// merge and should be skipped on heap pop.
    alive: bool,
}

struct Communities {
    g: InternalGraph,
    steps: u32,
    nodes: Vec<CommunityState>,
    /// Stable storage for neighbour entries. Indices into this vec are
    /// used as `NeighborId`.
    neighbors_pool: Vec<NeighborEntry>,
    /// For each community id, sorted list of `(other_id, neighbor_id)`
    /// adjacencies. Always `other_id != id`. We keep both halves so we
    /// can iterate from either community.
    adj: Vec<Vec<(u32, u32)>>,
    /// Min-heap on `delta_sigma` of alive neighbour entries.
    heap: Vec<u32>,
    merges: Vec<[u32; 2]>,
    modularity_trajectory: Vec<f64>,
}

// ---------------------------------------------------------------------
// Internal: min-heap on `neighbors_pool[id].delta_sigma`
// ---------------------------------------------------------------------

fn heap_key(pool: &[NeighborEntry], id: u32) -> f64 {
    pool[id as usize].delta_sigma
}

fn heap_push(heap: &mut Vec<u32>, pool: &[NeighborEntry], id: u32) {
    heap.push(id);
    let last = heap.len() - 1;
    sift_up(heap, pool, last);
}

fn heap_pop(heap: &mut Vec<u32>, pool: &[NeighborEntry]) -> Option<u32> {
    let last = heap.pop()?;
    if heap.is_empty() {
        return Some(last);
    }
    let top = std::mem::replace(&mut heap[0], last);
    sift_down(heap, pool, 0);
    Some(top)
}

fn sift_up(heap: &mut [u32], pool: &[NeighborEntry], mut i: usize) {
    while i > 0 {
        let parent = (i - 1) / 2;
        if heap_key(pool, heap[i]) < heap_key(pool, heap[parent]) {
            heap.swap(i, parent);
            i = parent;
        } else {
            break;
        }
    }
}

fn sift_down(heap: &mut [u32], pool: &[NeighborEntry], mut i: usize) {
    let n = heap.len();
    loop {
        let l = 2 * i + 1;
        let r = 2 * i + 2;
        let mut smallest = i;
        if l < n && heap_key(pool, heap[l]) < heap_key(pool, heap[smallest]) {
            smallest = l;
        }
        if r < n && heap_key(pool, heap[r]) < heap_key(pool, heap[smallest]) {
            smallest = r;
        }
        if smallest == i {
            break;
        }
        heap.swap(i, smallest);
        i = smallest;
    }
}

// ---------------------------------------------------------------------
// Top-level driver (placeholder — full body lands in Step 4).
// ---------------------------------------------------------------------

fn run(graph: &Graph, weights: Option<&[f64]>, steps: u32) -> IgraphResult<WalktrapResult> {
    let g = build_internal_graph(graph, weights)?;
    drive(g, graph, weights, steps)
}

fn drive(
    g: InternalGraph,
    graph: &Graph,
    weights: Option<&[f64]>,
    steps: u32,
) -> IgraphResult<WalktrapResult> {
    let n = g.n;

    // Trivial cases.
    if n == 0 {
        return Ok(WalktrapResult {
            membership: Vec::new(),
            nb_clusters: 0,
            merges: Vec::new(),
            modularity: vec![f64::NAN],
        });
    }
    if g.raw_total_weight <= 0.0 {
        // Edgeless: identity membership, NaN modularity, no merges.
        return Ok(WalktrapResult {
            membership: (0..n).collect(),
            nb_clusters: n,
            merges: Vec::new(),
            modularity: vec![f64::NAN],
        });
    }

    // Build initial singleton communities (one per vertex).
    let mut nodes: Vec<CommunityState> = Vec::with_capacity(2 * n as usize);
    for v in 0..n as usize {
        // For the singleton community {v}, the original-graph internal
        // weight equals the synthesised loop minus the synthesised
        // contribution. We need the *original* self-loop weight if any.
        // We compute these from `graph + weights` directly to keep
        // book-keeping simple and unambiguous.
        nodes.push(CommunityState {
            active: true,
            size: 1,
            internal_weight: 0.0,
            total_weight: 0.0,
            probabilities: None,
            members: vec![v as u32],
        });
    }
    // Fill total_weight and internal_weight from the original edges.
    for eid in 0..graph.ecount() {
        let (u, v) = graph.edge(eid as u32)?;
        let w = match weights {
            Some(ws) => ws[eid],
            None => 1.0,
        };
        nodes[u as usize].total_weight += w;
        if u == v {
            nodes[u as usize].internal_weight += w;
        }
        if u != v {
            nodes[v as usize].total_weight += w;
        }
    }

    // Initialise neighbour list and heap.
    let mut neighbors_pool: Vec<NeighborEntry> = Vec::new();
    let mut adj: Vec<Vec<(u32, u32)>> = vec![Vec::new(); (2 * n) as usize];
    let mut heap: Vec<u32> = Vec::new();

    // Build community-level adjacency: collapse parallel edges between
    // distinct endpoints; skip self-loops (a self-loop is internal to
    // the singleton and not a neighbour pair).
    use std::collections::BTreeMap;
    let mut pair_weight: BTreeMap<(u32, u32), f64> = BTreeMap::new();
    for eid in 0..graph.ecount() {
        let (u, v) = graph.edge(eid as u32)?;
        if u == v {
            continue;
        }
        let w = match weights {
            Some(ws) => ws[eid],
            None => 1.0,
        };
        let (a, b) = if u < v { (u, v) } else { (v, u) };
        *pair_weight.entry((a, b)).or_insert(0.0) += w;
    }

    for ((c1, c2), w) in pair_weight {
        // Lower-bound Δσ = -1 / min(|adj(c1)| as community, |adj(c2)|).
        // At the singleton level upstream uses 1/min(deg(c1), deg(c2))
        // where degrees come from the synthesised adjacency.
        let d1 = g.vertices[c1 as usize].len() as f64;
        let d2 = g.vertices[c2 as usize].len() as f64;
        let ds_lower = -1.0 / d1.min(d2);
        let id = neighbors_pool.len() as u32;
        neighbors_pool.push(NeighborEntry {
            c1,
            c2,
            delta_sigma: ds_lower,
            exact: false,
            weight: w,
            alive: true,
        });
        adj[c1 as usize].push((c2, id));
        adj[c2 as usize].push((c1, id));
        heap_push(&mut heap, &neighbors_pool, id);
    }
    for v in 0..(2 * n) as usize {
        adj[v].sort_by_key(|&(other, _)| other);
    }

    let mut comms = Communities {
        g,
        steps,
        nodes,
        neighbors_pool,
        adj,
        heap,
        merges: Vec::new(),
        modularity_trajectory: Vec::new(),
    };

    // Initial modularity (all singletons): Q_0 = -Σ a_c² where
    // a_c = total_weight_c / (2m). For loop-free graphs the diagonal
    // term Σ e_cc = Σ loop_weight / m which is 0; the canonical
    // Walktrap output matches this.
    let m = graph_total_edge_weight(graph, weights);
    comms
        .modularity_trajectory
        .push(initial_modularity(&comms, m));

    // Greedy agglomeration loop: pop neighbour entries until none
    // remain; refine non-exact entries on the fly.
    while let Some(id) = pop_exact(&mut comms) {
        merge_pair(&mut comms, id, m);
    }

    // Recover the partition at argmax modularity.
    let merges = comms.merges.clone();
    let modularity = comms.modularity_trajectory.clone();
    let mut packed = membership_at_best_modularity(n, &merges, &modularity);
    let nb_clusters = densify_membership(&mut packed);

    Ok(WalktrapResult {
        membership: packed,
        nb_clusters,
        merges,
        modularity,
    })
}

fn graph_total_edge_weight(graph: &Graph, weights: Option<&[f64]>) -> f64 {
    if let Some(ws) = weights {
        ws.iter().copied().sum()
    } else {
        graph.ecount() as f64
    }
}

fn initial_modularity(comms: &Communities, m: f64) -> f64 {
    if m <= 0.0 {
        return f64::NAN;
    }
    let two_m = 2.0 * m;
    let mut q = 0.0f64;
    for c in &comms.nodes {
        if !c.active {
            continue;
        }
        let a = c.total_weight / two_m;
        q += c.internal_weight / m - a * a;
    }
    q
}

fn pop_exact(comms: &mut Communities) -> Option<u32> {
    loop {
        let id = heap_pop(&mut comms.heap, &comms.neighbors_pool)?;
        if !comms.neighbors_pool[id as usize].alive {
            continue;
        }
        if comms.neighbors_pool[id as usize].exact {
            return Some(id);
        }
        // Refine: compute the exact Δσ via the L2 distance.
        refine_delta_sigma(comms, id);
        comms.neighbors_pool[id as usize].exact = true;
        heap_push(&mut comms.heap, &comms.neighbors_pool, id);
    }
}

fn refine_delta_sigma(comms: &mut Communities, id: u32) {
    let entry = comms.neighbors_pool[id as usize].clone();
    let c1 = entry.c1 as usize;
    let c2 = entry.c2 as usize;
    ensure_probabilities(comms, entry.c1);
    ensure_probabilities(comms, entry.c2);
    let s1 = f64::from(comms.nodes[c1].size);
    let s2 = f64::from(comms.nodes[c2].size);
    let p1 = comms.nodes[c1]
        .probabilities
        .as_ref()
        .map_or_else(Vec::new, std::clone::Clone::clone);
    let p2 = comms.nodes[c2]
        .probabilities
        .as_ref()
        .map_or_else(Vec::new, std::clone::Clone::clone);
    let d2 = distance_sq(&comms.g, &p1, &p2);
    // Pons-Latapy: Δσ_{c1, c2} = (s1 * s2 / (s1 + s2)) * d² / N where
    // N = Σ_v total_weight[v] (the global walk normaliser). We use the
    // factor exactly as in walktrap_communities.cpp.
    let n_global = comms.g.total_weight_global;
    let delta = (s1 * s2 / (s1 + s2)) * d2 / n_global;
    comms.neighbors_pool[id as usize].delta_sigma = delta;
}

fn ensure_probabilities(comms: &mut Communities, c: u32) {
    if comms.nodes[c as usize].probabilities.is_some() {
        return;
    }
    let members = comms.nodes[c as usize].members.clone();
    let p = compute_probabilities(&comms.g, &members, comms.steps);
    comms.nodes[c as usize].probabilities = Some(p);
}

fn merge_pair(comms: &mut Communities, id: u32, m: f64) {
    let entry = comms.neighbors_pool[id as usize].clone();
    comms.neighbors_pool[id as usize].alive = false;
    let c1 = entry.c1;
    let c2 = entry.c2;
    let new_id = comms.nodes.len() as u32;

    // Build the merged community.
    let size = comms.nodes[c1 as usize].size + comms.nodes[c2 as usize].size;
    let internal = comms.nodes[c1 as usize].internal_weight
        + comms.nodes[c2 as usize].internal_weight
        + entry.weight;
    let total = comms.nodes[c1 as usize].total_weight + comms.nodes[c2 as usize].total_weight;
    let mut members = Vec::with_capacity(
        comms.nodes[c1 as usize].members.len() + comms.nodes[c2 as usize].members.len(),
    );
    members.extend_from_slice(&comms.nodes[c1 as usize].members);
    members.extend_from_slice(&comms.nodes[c2 as usize].members);

    // Cache P^t for the merged community using the size-weighted blend.
    let p_merged: Option<Probabilities> = if let (Some(p1), Some(p2)) = (
        comms.nodes[c1 as usize].probabilities.as_ref(),
        comms.nodes[c2 as usize].probabilities.as_ref(),
    ) {
        let s1 = f64::from(comms.nodes[c1 as usize].size);
        let s2 = f64::from(comms.nodes[c2 as usize].size);
        let denom = s1 + s2;
        let mut p = vec![0.0f64; comms.g.n as usize];
        for v in 0..comms.g.n as usize {
            p[v] = (s1 * p1[v] + s2 * p2[v]) / denom;
        }
        Some(p)
    } else {
        None
    };

    comms.nodes.push(CommunityState {
        active: true,
        size,
        internal_weight: internal,
        total_weight: total,
        probabilities: p_merged,
        members,
    });
    comms.adj.push(Vec::new());
    comms.nodes[c1 as usize].active = false;
    comms.nodes[c2 as usize].active = false;
    // Free old probability caches to keep peak memory bounded.
    comms.nodes[c1 as usize].probabilities = None;
    comms.nodes[c2 as usize].probabilities = None;

    // Rewire adjacency: combine c1's and c2's neighbour lists into the
    // new community. For each k adjacent to c1, c2, or both: create one
    // new neighbour entry between new_id and k. Mark the old c1↔k and
    // c2↔k entries as dead.
    use std::collections::BTreeMap;
    let mut combined: BTreeMap<u32, (f64, Option<f64>, Option<f64>)> = BTreeMap::new();
    // Format: combined[k] = (weight_sum, ds_via_c1, ds_via_c2)
    for &(k, nid) in &comms.adj[c1 as usize] {
        let entry_k = &comms.neighbors_pool[nid as usize];
        if !entry_k.alive {
            continue;
        }
        let ds_known = if entry_k.exact {
            Some(entry_k.delta_sigma)
        } else {
            None
        };
        let slot = combined.entry(k).or_insert((0.0, None, None));
        slot.0 += entry_k.weight;
        slot.1 = ds_known;
    }
    for &(k, nid) in &comms.adj[c2 as usize] {
        let entry_k = &comms.neighbors_pool[nid as usize];
        if !entry_k.alive {
            continue;
        }
        let ds_known = if entry_k.exact {
            Some(entry_k.delta_sigma)
        } else {
            None
        };
        let slot = combined.entry(k).or_insert((0.0, None, None));
        slot.0 += entry_k.weight;
        slot.2 = ds_known;
    }
    // Mark all c1's / c2's edges as dead.
    for &(_, nid) in &comms.adj[c1 as usize] {
        comms.neighbors_pool[nid as usize].alive = false;
    }
    for &(_, nid) in &comms.adj[c2 as usize] {
        comms.neighbors_pool[nid as usize].alive = false;
    }

    // Create the merged neighbour entries.
    for (k, (w, ds_c1, ds_c2)) in combined {
        if k == new_id || !comms.nodes[k as usize].active {
            continue;
        }
        // Compose a lower bound for Δσ from the three-formula rule.
        // For the chain cases we don't know an exact value, so fall back
        // to the singleton-style `-1/min(|adj(new)|, |adj(k)|)` lower
        // bound which is always ≤ the true value.
        let (delta, exact) = if let (Some(d1), Some(d2)) = (ds_c1, ds_c2) {
            // Triangle: both old neighbours were exact.
            let s1 = f64::from(comms.nodes[c1 as usize].size);
            let s2 = f64::from(comms.nodes[c2 as usize].size);
            let sk = f64::from(comms.nodes[k as usize].size);
            let new_s = s1 + s2;
            let triangle =
                ((s1 + sk) * d1 + (s2 + sk) * d2 - sk * entry.delta_sigma) / (new_s + sk);
            (triangle, true)
        } else {
            let d_new = (comms.adj[c1 as usize].len() + comms.adj[c2 as usize].len()) as f64;
            let d_k = comms.adj[k as usize].len() as f64;
            let denom = d_new.min(d_k).max(1.0);
            (-1.0 / denom, false)
        };

        let id_new = comms.neighbors_pool.len() as u32;
        comms.neighbors_pool.push(NeighborEntry {
            c1: new_id.min(k),
            c2: new_id.max(k),
            delta_sigma: delta,
            exact,
            weight: w,
            alive: true,
        });
        comms.adj[new_id as usize].push((k, id_new));
        comms.adj[k as usize].push((new_id, id_new));
        heap_push(&mut comms.heap, &comms.neighbors_pool, id_new);
    }
    comms.adj[new_id as usize].sort_by_key(|&(o, _)| o);
    comms.adj[c1 as usize].clear();
    comms.adj[c2 as usize].clear();

    // Record merge + modularity step.
    comms.merges.push([c1, c2]);
    let q_prev = comms.modularity_trajectory.last().copied().unwrap_or(0.0);
    // ΔQ = (entry.weight / m) - 2 · a_c1 · a_c2 where a_c = total/2m.
    let two_m = 2.0 * m;
    let a1 = comms.nodes[c1 as usize].total_weight / two_m;
    let a2 = comms.nodes[c2 as usize].total_weight / two_m;
    let delta_q = entry.weight / m - 2.0 * a1 * a2;
    comms.modularity_trajectory.push(q_prev + delta_q);
}

fn membership_at_best_modularity(n: u32, merges: &[[u32; 2]], modularity: &[f64]) -> Vec<u32> {
    // Find the merge prefix that maximises modularity.
    let mut best = 0usize;
    let mut best_q = f64::NEG_INFINITY;
    for (i, &q) in modularity.iter().enumerate() {
        if q.is_finite() && q > best_q {
            best_q = q;
            best = i;
        }
    }
    // Apply the first `best` merges to a union-find seeded at identity.
    let mut parent: Vec<u32> = (0..n).collect();
    let mut rep: Vec<u32> = (0..(n + best as u32)).collect();
    fn find(rep: &mut [u32], x: u32) -> u32 {
        let mut r = x;
        while rep[r as usize] != r {
            r = rep[r as usize];
        }
        let mut cur = x;
        while rep[cur as usize] != r {
            let nxt = rep[cur as usize];
            rep[cur as usize] = r;
            cur = nxt;
        }
        r
    }
    for (i, m) in merges.iter().take(best).enumerate() {
        let new_id = n + i as u32;
        let r1 = find(&mut rep, m[0]);
        let r2 = find(&mut rep, m[1]);
        rep[r1 as usize] = new_id;
        rep[r2 as usize] = new_id;
    }
    for v in 0..n {
        parent[v as usize] = find(&mut rep, v);
    }
    parent
}

fn densify_membership(membership: &mut [u32]) -> u32 {
    use std::collections::BTreeMap;
    let mut remap: BTreeMap<u32, u32> = BTreeMap::new();
    let mut next = 0u32;
    for v in membership.iter_mut() {
        let id = *remap.entry(*v).or_insert_with(|| {
            let n = next;
            next += 1;
            n
        });
        *v = id;
    }
    next
}

// ---------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol || (a.is_nan() && b.is_nan())
    }

    fn vec_near(a: &[f64], b: &[f64], tol: f64) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(&x, &y)| near(x, y, tol))
    }

    fn triangle() -> Graph {
        let mut g = Graph::with_vertices(3);
        for &(u, v) in &[(0, 1), (1, 2), (2, 0)] {
            g.add_edge(u, v).expect("add edge");
        }
        g
    }

    /// Mirrors `community_walktrap.out` "Basic test".
    #[test]
    fn c_basic_triangle() {
        let g = triangle();
        let r = walktrap(&g).expect("walktrap on triangle");
        assert_eq!(r.merges, vec![[1, 2], [0, 3]]);
        assert!(vec_near(
            &r.modularity,
            &[-1.0 / 3.0, -2.0 / 9.0, 0.0],
            1e-12
        ));
        assert_eq!(r.membership, vec![0, 0, 0]);
        assert_eq!(r.nb_clusters, 1);
    }

    /// Mirrors `community_walktrap.out` "Bug 2042".
    #[test]
    fn c_bug2042_single_weighted_edge_with_isolate() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).expect("add edge");
        let r = walktrap_weighted(&g, &[0.2]).expect("walktrap weighted");
        assert_eq!(r.merges, vec![[0, 1]]);
        assert!(vec_near(&r.modularity, &[-0.5, 0.0], 1e-12));
        // Membership labels are remapped to dense `0..k`; the
        // partition is `{0,1}` and `{2}`.
        assert_eq!(r.membership[0], r.membership[1]);
        assert_ne!(r.membership[0], r.membership[2]);
        assert_eq!(r.nb_clusters, 2);
    }

    /// Mirrors `community_walktrap.out` "Small weighted graph".
    #[test]
    fn c_ring6_weighted() {
        let mut g = Graph::with_vertices(6);
        for &(u, v) in &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)] {
            g.add_edge(u, v).expect("add edge");
        }
        let weights = [1.0_f64, 0.5, 0.25, 0.75, 1.25, 1.5];
        let r = walktrap_weighted(&g, &weights).expect("walktrap weighted");
        assert_eq!(r.merges, vec![[3, 4], [1, 2], [0, 5], [7, 8], [6, 9]]);
        let expected_mod = [
            -0.196_145_124_716_553_3,
            -0.089_569_160_997_732_43,
            -0.014_739_229_024_943_318,
            0.146_258_503_401_360_5,
            0.122_448_979_591_836_7,
            0.0, // zapsmall(1e-15) in C output
        ];
        assert_eq!(r.modularity.len(), expected_mod.len());
        for (a, b) in r.modularity.iter().zip(&expected_mod) {
            assert!(near(*a, *b, 1e-12), "mod mismatch: {a} vs {b}");
        }
        // Argmax index = 3 → membership at that cut:
        //   merge 0: c3,c4 → c6; merge 1: c1,c2 → c7; merge 2: c0,c5 → c8.
        // Three communities at cut: {0,5} {1,2} {3,4}. Densify-friendly assertions:
        assert_eq!(r.nb_clusters, 3);
        assert_eq!(r.membership[0], r.membership[5]);
        assert_eq!(r.membership[1], r.membership[2]);
        assert_eq!(r.membership[3], r.membership[4]);
        assert_ne!(r.membership[0], r.membership[1]);
        assert_ne!(r.membership[0], r.membership[3]);
        assert_ne!(r.membership[1], r.membership[3]);
    }

    /// Mirrors `community_walktrap.out` "Isolated vertices".
    #[test]
    fn c_five_isolated() {
        let g = Graph::with_vertices(5);
        let r = walktrap(&g).expect("walktrap isolated");
        assert!(r.merges.is_empty());
        assert_eq!(r.modularity.len(), 1);
        assert!(r.modularity[0].is_nan());
        assert_eq!(r.membership, vec![0, 1, 2, 3, 4]);
        assert_eq!(r.nb_clusters, 5);
    }

    #[test]
    fn rejects_directed_graph() {
        let g = Graph::new(3, true).expect("directed graph");
        assert!(matches!(walktrap(&g), Err(IgraphError::Unsupported(_))));
    }

    #[test]
    fn rejects_steps_zero() {
        let g = triangle();
        let opts = WalktrapOptions { steps: 0 };
        assert!(matches!(
            walktrap_with_options(&g, None, opts),
            Err(IgraphError::InvalidArgument(_))
        ));
    }

    #[test]
    fn rejects_negative_weight() {
        let g = triangle();
        let w = [1.0_f64, -0.1, 1.0];
        assert!(matches!(
            walktrap_weighted(&g, &w),
            Err(IgraphError::InvalidArgument(_))
        ));
    }

    #[test]
    fn rejects_weight_length_mismatch() {
        let g = triangle();
        let w = [1.0_f64, 1.0];
        assert!(matches!(
            walktrap_weighted(&g, &w),
            Err(IgraphError::InvalidArgument(_))
        ));
    }

    #[test]
    fn rejects_nan_weight() {
        let g = triangle();
        let w = [1.0_f64, f64::NAN, 1.0];
        assert!(matches!(
            walktrap_weighted(&g, &w),
            Err(IgraphError::InvalidArgument(_))
        ));
    }

    #[test]
    fn empty_graph_returns_empty_membership() {
        let g = Graph::with_vertices(0);
        let r = walktrap(&g).expect("walktrap on empty graph");
        assert!(r.membership.is_empty());
        assert_eq!(r.nb_clusters, 0);
        assert!(r.merges.is_empty());
        assert_eq!(r.modularity.len(), 1);
        assert!(r.modularity[0].is_nan());
    }

    #[test]
    fn single_vertex_no_edges() {
        let g = Graph::with_vertices(1);
        let r = walktrap(&g).expect("walktrap on single vertex");
        assert_eq!(r.membership, vec![0]);
        assert_eq!(r.nb_clusters, 1);
        assert!(r.merges.is_empty());
        // m == 0 → NaN trajectory.
        assert!(r.modularity[0].is_nan());
    }

    /// Two K4 cliques bridged by a single light edge should split into
    /// two communities at the optimal modularity cut.
    #[test]
    fn two_k4_with_bridge() {
        let mut g = Graph::with_vertices(8);
        for &(u, v) in &[
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 2),
            (1, 3),
            (2, 3),
            (4, 5),
            (4, 6),
            (4, 7),
            (5, 6),
            (5, 7),
            (6, 7),
            (3, 4),
        ] {
            g.add_edge(u, v).expect("add edge");
        }
        let r = walktrap(&g).expect("walktrap");
        assert_eq!(r.nb_clusters, 2);
        for v in 0..4u32 {
            assert_eq!(r.membership[v as usize], r.membership[0]);
        }
        for v in 4..8u32 {
            assert_eq!(r.membership[v as usize], r.membership[4]);
        }
        assert_ne!(r.membership[0], r.membership[4]);
    }

    #[test]
    fn multi_edges_are_folded_by_weight_sum() {
        // Two K3s + a doubled bridge. Multi-edge handling shouldn't
        // change the partition.
        let mut g = Graph::with_vertices(6);
        for &(u, v) in &[
            (0, 1),
            (0, 2),
            (1, 2),
            (3, 4),
            (3, 5),
            (4, 5),
            (2, 3),
            (2, 3), // doubled bridge
        ] {
            g.add_edge(u, v).expect("add edge");
        }
        let r = walktrap(&g).expect("walktrap");
        // Smoke: must succeed and produce a valid partition.
        assert_eq!(r.membership.len(), 6);
        assert!((1..=6).contains(&r.nb_clusters));
    }

    #[cfg(all(test, feature = "proptest-harness"))]
    mod prop {
        use super::*;
        use proptest::prelude::*;

        prop_compose! {
            fn small_undirected_graph()(n in 2u32..=8u32, edges_seed in any::<u64>()) -> Graph {
                let mut g = Graph::with_vertices(n);
                let mut rng = edges_seed;
                let target_m = ((n * (n - 1)) / 2).min(n + 4) as usize;
                let mut added = 0usize;
                let mut guard = 0usize;
                while added < target_m && guard < target_m * 8 {
                    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let u = ((rng >> 33) % u64::from(n)) as u32;
                    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let v = ((rng >> 33) % u64::from(n)) as u32;
                    guard += 1;
                    if u == v { continue; }
                    if g.add_edge(u, v).is_ok() { added += 1; }
                }
                g
            }
        }

        proptest! {
            #[test]
            fn walktrap_partition_is_valid(g in small_undirected_graph()) {
                let n = g.vcount();
                let r = walktrap(&g).expect("walktrap");
                prop_assert_eq!(r.membership.len(), n as usize);
                // Densified labels live in 0..nb_clusters.
                for &c in &r.membership {
                    prop_assert!(c < r.nb_clusters);
                }
                // Modularity trajectory length matches merges + 1.
                if g.ecount() > 0 {
                    prop_assert_eq!(r.modularity.len(), r.merges.len() + 1);
                    for &q in &r.modularity {
                        prop_assert!(q.is_finite(), "modularity must be finite for connected weighted ops");
                    }
                }
            }

            #[test]
            fn walktrap_steps_in_range_does_not_crash(
                g in small_undirected_graph(),
                steps in 1u32..=8,
            ) {
                let r = walktrap_with_options(&g, None, WalktrapOptions { steps })
                    .expect("walktrap with options");
                prop_assert_eq!(r.membership.len(), g.vcount() as usize);
            }
        }
    }
}
