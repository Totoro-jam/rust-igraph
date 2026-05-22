//! Louvain multilevel community detection (ALGO-CO-002).
//!
//! Counterpart of `igraph_community_multilevel()` from
//! `references/igraph/src/community/louvain.c`.
//!
//! Blondel, Guillaume, Lambiotte, Lefebvre (2008):
//! *Fast unfolding of communities in large networks.* JSTAT 10008(10):
//! 6. <https://doi.org/10.1088/1742-5468/2008/10/P10008>
//!
//! Two-phase loop:
//!
//! - **Local moving**: in a deterministic-but-seeded shuffled order,
//!   each vertex is greedily reassigned to the neighbour community
//!   that maximises modularity gain `ΔQ`.
//! - **Aggregation**: every community becomes a super-vertex; intra-
//!   community edges turn into self-loops and parallel inter-community
//!   edges are summed.
//!
//! Repeats until a pass produces no improvement (modularity does not
//! strictly increase). Modularity is the Newman-Girvan formulation,
//! optionally with a resolution parameter `γ` (default 1.0).
//!
//! Self-rolled, dependency-free. Determinism comes from an inline
//! `SplitMix64` PRNG seeded by the caller; the default convenience
//! wrappers use seed `0` so that repeated calls on the same graph
//! produce identical partitions.

// All `usize` -> `u32` casts in this module are bounded by `n`
// (the graph vertex count), which originates from `Graph::vcount(): u32`
// and so can never truncate. The single-char names (`u`, `v`, `w`, `c`)
// are domain-standard for graph endpoints / weights / communities.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult};

const MAX_LEVELS: usize = 64;
const MAX_PASSES_PER_LEVEL: usize = 256;

/// Result of a Louvain run.
///
/// `membership` is a length-`vcount` vector of compacted community
/// labels (each in `0..k` where `k` is the number of communities at
/// the final level). `modularity` is the final modularity score.
/// `levels` and `modularities` record the per-level partitions and
/// modularity values respectively (length = number of aggregation
/// rounds + 1).
#[derive(Debug, Clone)]
pub struct LouvainResult {
    pub membership: Vec<u32>,
    pub modularity: f64,
    pub levels: Vec<Vec<u32>>,
    pub modularities: Vec<f64>,
}

/// Run Louvain with the default options (`γ = 1`, unweighted,
/// deterministic seed `0`).
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph` is directed.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, louvain};
///
/// // Two triangles joined by a bridge edge form two obvious communities.
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let result = louvain(&g).unwrap();
/// assert_eq!(result.membership[0], result.membership[1]);
/// assert_eq!(result.membership[3], result.membership[4]);
/// assert_ne!(result.membership[0], result.membership[3]);
/// assert!(result.modularity > 0.35);
/// ```
pub fn louvain(graph: &Graph) -> IgraphResult<LouvainResult> {
    louvain_with_options(graph, None, 1.0, 0)
}

/// Run Louvain with per-edge weights (`γ = 1`, deterministic seed `0`).
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph` is directed.
/// - [`IgraphError::InvalidArgument`] if `weights.len() != ecount` or
///   any weight is negative or NaN.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, louvain_weighted};
///
/// // Two K3s + bridge. Heavy intra-clique weights vs a thin bridge:
/// // Louvain splits the two triangles.
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let weights = vec![10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 0.01];
/// let r = louvain_weighted(&g, &weights).unwrap();
/// assert_eq!(r.membership[0], r.membership[1]);
/// assert_ne!(r.membership[0], r.membership[3]);
/// ```
pub fn louvain_weighted(graph: &Graph, weights: &[f64]) -> IgraphResult<LouvainResult> {
    louvain_with_options(graph, Some(weights), 1.0, 0)
}

/// Run Louvain with the full set of options.
///
/// - `weights`: per-edge weights or `None` for unit weights. Must be
///   non-negative and not NaN.
/// - `resolution`: γ ≥ 0. Lower values produce fewer, larger communities;
///   higher values produce more, smaller communities. `γ = 1` recovers
///   the classical Newman-Girvan modularity.
/// - `seed`: PRNG seed for the node-order shuffle. Same seed produces
///   the same partition on the same graph.
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph` is directed.
/// - [`IgraphError::InvalidArgument`] for malformed weights or
///   `resolution < 0` / `!resolution.is_finite()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, louvain_with_options};
///
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// // Determinism: same seed reproduces the same partition bit-for-bit.
/// let a = louvain_with_options(&g, None, 1.0, 42).unwrap();
/// let b = louvain_with_options(&g, None, 1.0, 42).unwrap();
/// assert_eq!(a.membership, b.membership);
/// ```
pub fn louvain_with_options(
    graph: &Graph,
    weights: Option<&[f64]>,
    resolution: f64,
    seed: u64,
) -> IgraphResult<LouvainResult> {
    if graph.is_directed() {
        return Err(IgraphError::Unsupported(
            "louvain requires an undirected graph",
        ));
    }
    if !resolution.is_finite() || resolution < 0.0 {
        return Err(IgraphError::InvalidArgument(
            "resolution parameter must be non-negative and finite".to_string(),
        ));
    }
    validate_weights(graph, weights)?;

    let vcount = graph.vcount() as usize;
    if vcount == 0 {
        return Ok(LouvainResult {
            membership: Vec::new(),
            modularity: 0.0,
            levels: Vec::new(),
            modularities: Vec::new(),
        });
    }

    // Build the initial working representation from the input graph.
    let mut state = build_initial_state(graph, weights)?;

    // Track the cumulative mapping from original vertex IDs to the
    // current super-vertex IDs. Starts as the identity permutation.
    let mut original_to_current: Vec<u32> = (0..vcount).map(|i| i as u32).collect();

    let mut levels: Vec<Vec<u32>> = Vec::new();
    let mut modularities: Vec<f64> = Vec::new();
    let mut rng = SplitMix64::new(seed);

    for _level in 0..MAX_LEVELS {
        let initial_n = state.n;
        let prev_q = current_modularity(&state, resolution);

        // Run one full local-moving pass loop until no further
        // improvement happens within this level.
        let (q, changed_any) = local_moving_loop(&mut state, resolution, &mut rng);

        // Re-index the resulting membership to contiguous [0, k) labels.
        let k = reindex_membership(&mut state.membership);

        // Update the cumulative original→current mapping.
        for slot in &mut original_to_current {
            *slot = state.membership[*slot as usize];
        }

        // Record the level snapshot (membership on original vertices).
        levels.push(original_to_current.clone());
        modularities.push(q);

        // Stop if no merge happened or modularity didn't improve.
        if !changed_any || k as usize == initial_n || q <= prev_q + Q_EPS {
            break;
        }

        // Aggregate into super-vertex graph for next level.
        state = aggregate(&state, k as usize);
    }

    // The cumulative mapping IS the final membership.
    let membership = original_to_current;
    let modularity = modularities.last().copied().unwrap_or(0.0);

    Ok(LouvainResult {
        membership,
        modularity,
        levels,
        modularities,
    })
}

// ---------- modularity-gain epsilon ----------

const Q_EPS: f64 = 1e-10;

// ---------- internal state ----------

/// Working graph + per-community state for one level. Non-self-loop
/// edges contribute two adjacency entries (one from each endpoint);
/// self-loops never appear in `adj` — they live in `loop_weight`.
///
/// Self-loops follow the upstream `IGRAPH_LOOPS_TWICE` convention: a
/// raw self-loop of weight `w` contributes `2w` to both `loop_weight`
/// and `vertex_weight` (`k_v`), matching `A_vv = 2w` in the symmetric
/// adjacency matrix.
struct WorkingState {
    n: usize,
    /// `adj[v]` = list of `(neighbour, weight)` for every non-loop
    /// incident edge of `v`. Each non-loop edge appears twice (once
    /// per endpoint).
    adj: Vec<Vec<(u32, f64)>>,
    /// `loop_weight[v]` = `2 ×` raw self-loop weight at vertex `v`.
    loop_weight: Vec<f64>,
    /// `vertex_weight[v]` = `k_v` = sum of `A_vu` for all `u`. For a
    /// non-loop edge of weight `w`, contributes `w` to both endpoints;
    /// for a self-loop, contributes `2w` to the single endpoint.
    vertex_weight: Vec<f64>,
    /// `membership[v]` = current community label.
    membership: Vec<u32>,
    /// Per-community total incident weight, summed with the same loop
    /// convention as `vertex_weight`.
    weight_all: Vec<f64>,
    /// Per-community internal weight `Σ_{i,j ∈ c} A_ij` =
    /// `2 ×` non-loop intra weight `+ 2 ×` raw loop weight.
    weight_inside: Vec<f64>,
    /// Per-community vertex count.
    size: Vec<u32>,
    /// `2 × Σ_e w_e` — Newman-Girvan normaliser used in the gain.
    weight_sum: f64,
}

fn build_initial_state(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<WorkingState> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    let m_u32 = u32::try_from(m)
        .map_err(|_| IgraphError::InvalidArgument("edge count exceeds u32::MAX".into()))?;
    let mut edges: Vec<(u32, u32, f64)> = Vec::with_capacity(m);
    for e in 0..m_u32 {
        let (u, v) = graph.edge(e as EdgeId)?;
        let w = weights.map_or(1.0, |ws| ws[e as usize]);
        let (a, b) = if u <= v { (u, v) } else { (v, u) };
        edges.push((a, b, w));
    }
    Ok(init_from_edges(n, &edges))
}

/// Shared initialisation: build a `WorkingState` from a list of
/// `(u, v, w)` edges with `u <= v`. Self-loops follow the upstream
/// `IGRAPH_LOOPS_TWICE` convention — a loop of weight `w` contributes
/// `2w` to `weight_all` and `2w` to `weight_inside` (since `A_vv = 2w`
/// in the symmetric adjacency).
fn init_from_edges(n: usize, edges: &[(u32, u32, f64)]) -> WorkingState {
    let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    let mut loop_weight = vec![0.0_f64; n];
    let mut vertex_weight = vec![0.0_f64; n];
    let mut total: f64 = 0.0;
    for &(u, v, w) in edges {
        total += w;
        let uu = u as usize;
        let vv = v as usize;
        if uu == vv {
            loop_weight[uu] += 2.0 * w;
            vertex_weight[uu] += 2.0 * w;
        } else {
            adj[uu].push((v, w));
            adj[vv].push((u, w));
            vertex_weight[uu] += w;
            vertex_weight[vv] += w;
        }
    }

    let membership: Vec<u32> = (0..n as u32).collect();
    let weight_all = vertex_weight.clone();
    let weight_inside = loop_weight.clone();
    let size = vec![1_u32; n];
    let weight_sum = 2.0 * total;

    WorkingState {
        n,
        adj,
        loop_weight,
        vertex_weight,
        membership,
        weight_all,
        weight_inside,
        size,
        weight_sum,
    }
}

/// Modularity of the current partition, computed directly from the
/// per-community sums. Mirrors the upstream `i_multilevel_community_modularity`.
fn current_modularity(state: &WorkingState, resolution: f64) -> f64 {
    if state.weight_sum <= 0.0 {
        return 0.0;
    }
    let m = state.weight_sum;
    let mut q = 0.0_f64;
    for c in 0..state.n {
        if state.size[c] > 0 {
            let kall = state.weight_all[c];
            q += (state.weight_inside[c] - resolution * kall * kall / m) / m;
        }
    }
    q
}

/// Repeatedly applies local-moving passes until a pass produces no
/// improvement. Returns `(final_modularity, any_pass_changed_at_least_one)`.
fn local_moving_loop(
    state: &mut WorkingState,
    resolution: f64,
    rng: &mut SplitMix64,
) -> (f64, bool) {
    let mut any_changed = false;
    let mut q = current_modularity(state, resolution);
    let mut node_order: Vec<usize> = (0..state.n).collect();
    if state.n > 1 {
        shuffle_in_place(&mut node_order, rng);
    }

    for _ in 0..MAX_PASSES_PER_LEVEL {
        let pass_q = q;
        let mut changed = false;

        // Upstream applies a single random inversion to node_order
        // every pass (rather than a full reshuffle) — see issue #2650.
        if state.n > 1 {
            let i1 = rng.gen_index(state.n);
            let i2 = rng.gen_index(state.n);
            node_order.swap(i1, i2);
        }

        for &ni in &node_order {
            local_move_vertex(state, ni, resolution, &mut changed);
        }

        q = current_modularity(state, resolution);
        if changed && q > pass_q + Q_EPS {
            any_changed = true;
            continue;
        }
        break;
    }

    (q, any_changed)
}

/// Greedy move of one vertex to its best neighbour community.
///
/// Mirrors the body of `igraph_i_community_multilevel_step` lines
/// 407-472.
fn local_move_vertex(state: &mut WorkingState, ni: usize, resolution: f64, changed: &mut bool) {
    let weight_all_v = state.vertex_weight[ni];
    let weight_loop_v = state.loop_weight[ni];
    let old_id = state.membership[ni] as usize;

    // Build the neighbour-community → summed-weight table. We allocate
    // per call because n changes between levels; the cost is amortised
    // by `adj[ni].len()` being small for sparse graphs.
    let mut neigh_communities: Vec<u32> = Vec::with_capacity(state.adj[ni].len());
    let mut neigh_weights: Vec<f64> = Vec::with_capacity(state.adj[ni].len());
    let mut weight_inside_old = 0.0_f64; // edge weight from ni back to its old community

    for &(nei, w) in &state.adj[ni] {
        let c = state.membership[nei as usize];
        if c as usize == old_id {
            weight_inside_old += w;
        }
        // Linear scan is fine: degrees are small for real graphs and
        // the slot count is bounded by deg(ni).
        if let Some(idx) = neigh_communities.iter().position(|&x| x == c) {
            neigh_weights[idx] += w;
        } else {
            neigh_communities.push(c);
            neigh_weights.push(w);
        }
    }

    // Temporarily strip vertex from its old community.
    state.size[old_id] -= 1;
    state.weight_all[old_id] -= weight_all_v;
    state.weight_inside[old_id] -= 2.0 * weight_inside_old + weight_loop_v;

    // Hunt for the best move.
    let mut best_id = old_id;
    let mut best_gain = 0.0_f64;
    let mut best_weight = weight_inside_old;
    for (idx, &c) in neigh_communities.iter().enumerate() {
        let w_to_c = neigh_weights[idx];
        // Skip moves to the dissolved old slot when re-checking the
        // identity move — the empty-community check below handles it.
        let kall_c = state.weight_all[c as usize];
        // gain = w_to_c - γ * kall_c * weight_all_v / weight_sum
        let gain = w_to_c - resolution * kall_c * weight_all_v / state.weight_sum;
        if gain > best_gain {
            best_gain = gain;
            best_id = c as usize;
            best_weight = w_to_c;
        }
    }

    // Apply the chosen move (which may be the old community).
    state.membership[ni] = best_id as u32;
    state.size[best_id] += 1;
    state.weight_all[best_id] += weight_all_v;
    state.weight_inside[best_id] += 2.0 * best_weight + weight_loop_v;

    if best_id != old_id {
        *changed = true;
    }
}

/// Reindex `membership` so labels become a contiguous `0..k`
/// permutation, then return `k`.
fn reindex_membership(membership: &mut [u32]) -> u32 {
    if membership.is_empty() {
        return 0;
    }
    let max = *membership.iter().max().unwrap_or(&0);
    let mut remap: Vec<i64> = vec![-1; max as usize + 1];
    let mut next: u32 = 0;
    for slot in membership.iter_mut() {
        let old = *slot as usize;
        if remap[old] < 0 {
            remap[old] = i64::from(next);
            next += 1;
        }
        *slot = remap[old] as u32;
    }
    next
}

/// Build the level-`l+1` super-graph: every community becomes a
/// single super-vertex, parallel inter-community edges are summed,
/// intra-community edges become self-loops. Goes through
/// [`init_from_edges`] so the loop accounting stays consistent with
/// the level-0 initialisation.
fn aggregate(state: &WorkingState, k: usize) -> WorkingState {
    // Per-super-vertex (raw) self-loop weight accumulator. Intra-edges
    // and the previous level's loops both feed into here.
    let mut loop_raw: Vec<f64> = vec![0.0_f64; k];
    // For inter-super-vertex edges: keep a dedup adjacency restricted
    // to `mu < mv` so each super-edge appears exactly once.
    let mut inter: Vec<Vec<(u32, f64)>> = vec![Vec::new(); k];

    // Pre-existing self-loops at the working level. `state.loop_weight`
    // is stored doubled (IGRAPH_LOOPS_TWICE convention from init), so
    // the raw weight is half.
    for u in 0..state.n {
        let raw = state.loop_weight[u] * 0.5;
        if raw > 0.0 {
            let mu = state.membership[u] as usize;
            loop_raw[mu] += raw;
        }
    }

    // Non-loop edges in the current `adj`. Each edge appears twice
    // (once per endpoint); dedupe via `u < v`.
    for u in 0..state.n {
        for &(v, w) in &state.adj[u] {
            let v_us = v as usize;
            if u >= v_us {
                continue;
            }
            let mu = state.membership[u] as usize;
            let mv = state.membership[v_us] as usize;
            if mu == mv {
                loop_raw[mu] += w;
            } else {
                let (a, b) = if mu < mv { (mu, mv) } else { (mv, mu) };
                push_or_merge(&mut inter[a], b as u32, w);
            }
        }
    }

    // Flatten into an edge list with `u <= v` and feed through the
    // shared initialiser.
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    for u in 0..k {
        if loop_raw[u] > 0.0 {
            edges.push((u as u32, u as u32, loop_raw[u]));
        }
        for &(v, w) in &inter[u] {
            edges.push((u as u32, v, w));
        }
    }

    init_from_edges(k, &edges)
}

fn push_or_merge(list: &mut Vec<(u32, f64)>, neighbour: u32, weight: f64) {
    for slot in list.iter_mut() {
        if slot.0 == neighbour {
            slot.1 += weight;
            return;
        }
    }
    list.push((neighbour, weight));
}

// ---------- weight validation ----------

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
                "weight at edge {e} is negative ({v}); louvain weights must be non-negative"
            )));
        }
    }
    Ok(())
}

// ---------- PRNG ----------

struct SplitMix64(u64);

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn gen_index(&mut self, bound: usize) -> usize {
        debug_assert!(bound > 0);
        let r = self.next_u64() % (bound as u64);
        usize::try_from(r).expect("bound fits in usize by construction")
    }
}

fn shuffle_in_place<T>(slice: &mut [T], rng: &mut SplitMix64) {
    // Fisher-Yates.
    let len = slice.len();
    for i in (1..len).rev() {
        let j = rng.gen_index(i + 1);
        slice.swap(i, j);
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    fn add_undirected_edges(g: &mut Graph, edges: &[(u32, u32)]) {
        for &(u, v) in edges {
            g.add_edge(u, v).unwrap();
        }
    }

    #[test]
    fn empty_graph_returns_empty_membership() {
        let g = Graph::with_vertices(0);
        let r = louvain(&g).unwrap();
        assert_eq!(r.membership.len(), 0);
        assert_eq!(r.modularity, 0.0);
        assert!(r.levels.is_empty());
    }

    #[test]
    fn isolated_vertices_each_their_own_community() {
        let g = Graph::with_vertices(4);
        let r = louvain(&g).unwrap();
        // With no edges and weight_sum = 0, the local-moving pass should not
        // produce any move; we return the trivial identity partition.
        assert_eq!(r.membership.len(), 4);
        // Ensure labels are within [0, 4).
        for &c in &r.membership {
            assert!(c < 4);
        }
    }

    #[test]
    fn single_edge_two_communities_or_merged() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let r = louvain(&g).unwrap();
        // Either both in same community (Q = 0 for a single edge,
        // both singletons gives Q = -1/2 so merging is preferred).
        assert_eq!(r.membership[0], r.membership[1]);
    }

    #[test]
    fn two_triangles_bridge_finds_two_communities() {
        let mut g = Graph::with_vertices(6);
        add_undirected_edges(
            &mut g,
            &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
        );
        let r = louvain(&g).unwrap();
        // {0,1,2} and {3,4,5} should be split. Robust to label
        // permutation.
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[1], r.membership[2]);
        assert_eq!(r.membership[3], r.membership[4]);
        assert_eq!(r.membership[4], r.membership[5]);
        assert_ne!(r.membership[0], r.membership[3]);
        assert!(r.modularity > 0.35);
    }

    #[test]
    fn complete_k5_gives_one_or_two_communities() {
        let mut g = Graph::with_vertices(5);
        for u in 0..5u32 {
            for v in (u + 1)..5 {
                g.add_edge(u, v).unwrap();
            }
        }
        let r = louvain(&g).unwrap();
        // K_n's modularity is bounded by 0 — any partition gives Q ≤ 0
        // with equality at the trivial single-community partition.
        // Louvain should converge to the single-community case.
        let k: std::collections::BTreeSet<_> = r.membership.iter().copied().collect();
        assert!(k.len() <= 2, "K5 should yield ≤ 2 communities");
        assert!(r.modularity >= -1e-9);
    }

    #[test]
    fn disconnected_components_separate_communities() {
        let mut g = Graph::with_vertices(8);
        // Two K_4 components.
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        for u in 4..8u32 {
            for v in (u + 1)..8 {
                g.add_edge(u, v).unwrap();
            }
        }
        let r = louvain(&g).unwrap();
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[4], r.membership[5]);
        assert_ne!(r.membership[0], r.membership[4]);
        assert!(r.modularity > 0.4);
    }

    #[test]
    fn directed_graph_returns_unsupported() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(louvain(&g).is_err());
    }

    #[test]
    fn weights_length_mismatch_errors() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = louvain_weighted(&g, &[1.0]);
        assert!(r.is_err());
    }

    #[test]
    fn weights_negative_errors() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = louvain_weighted(&g, &[1.0, -0.5]);
        assert!(r.is_err());
    }

    #[test]
    fn weights_nan_errors() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = louvain_weighted(&g, &[1.0, f64::NAN]);
        assert!(r.is_err());
    }

    #[test]
    fn negative_resolution_errors() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert!(louvain_with_options(&g, None, -0.1, 0).is_err());
    }

    #[test]
    fn weighted_unit_matches_unweighted() {
        let mut g = Graph::with_vertices(6);
        add_undirected_edges(
            &mut g,
            &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
        );
        let ones = vec![1.0; g.ecount()];
        let a = louvain(&g).unwrap();
        let b = louvain_weighted(&g, &ones).unwrap();
        // Modularity should match closely.
        assert!((a.modularity - b.modularity).abs() < 1e-9);
        // Membership equivalence (up to label permutation): two
        // vertices are in the same community in one iff they're in
        // the same in the other.
        for i in 0..6 {
            for j in 0..6 {
                let ai = a.membership[i] == a.membership[j];
                let bi = b.membership[i] == b.membership[j];
                assert_eq!(ai, bi, "partition mismatch at ({i},{j})");
            }
        }
    }

    #[test]
    fn modularity_matches_external_recompute() {
        // Cross-check our reported modularity against the standalone
        // `modularity()` function on the same partition.
        let mut g = Graph::with_vertices(6);
        add_undirected_edges(
            &mut g,
            &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
        );
        let r = louvain(&g).unwrap();
        let q_recomputed =
            crate::algorithms::community::modularity::modularity(&g, &r.membership, 1.0)
                .unwrap()
                .unwrap();
        assert!(
            (r.modularity - q_recomputed).abs() < 1e-9,
            "louvain reports {} but modularity() reports {}",
            r.modularity,
            q_recomputed
        );
    }

    #[test]
    fn resolution_zero_gives_single_community() {
        // γ = 0 → all the modularity penalty disappears, the
        // optimiser is allowed to lump everything together.
        let mut g = Graph::with_vertices(6);
        add_undirected_edges(
            &mut g,
            &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
        );
        let r = louvain_with_options(&g, None, 0.0, 0).unwrap();
        let k: std::collections::BTreeSet<_> = r.membership.iter().copied().collect();
        assert_eq!(k.len(), 1);
    }

    #[test]
    fn resolution_high_gives_many_communities() {
        let mut g = Graph::with_vertices(6);
        add_undirected_edges(
            &mut g,
            &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
        );
        let r = louvain_with_options(&g, None, 10.0, 0).unwrap();
        let k: std::collections::BTreeSet<_> = r.membership.iter().copied().collect();
        assert!(
            k.len() >= 3,
            "with γ=10 we expect more granular communities"
        );
    }

    #[test]
    fn deterministic_with_seed() {
        let mut g = Graph::with_vertices(10);
        for u in 0..10u32 {
            for v in (u + 1)..10 {
                if (u + v) % 3 != 0 {
                    g.add_edge(u, v).unwrap();
                }
            }
        }
        let a = louvain_with_options(&g, None, 1.0, 42).unwrap();
        let b = louvain_with_options(&g, None, 1.0, 42).unwrap();
        assert_eq!(a.membership, b.membership);
        assert!((a.modularity - b.modularity).abs() < 1e-12);
    }

    #[test]
    fn levels_recorded() {
        let mut g = Graph::with_vertices(6);
        add_undirected_edges(
            &mut g,
            &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
        );
        let r = louvain(&g).unwrap();
        assert!(!r.levels.is_empty());
        assert_eq!(r.levels.len(), r.modularities.len());
        // Last level's membership should match the final membership.
        assert_eq!(*r.levels.last().unwrap(), r.membership);
    }

    #[test]
    fn karate_club_finds_known_partition() {
        // Karate club: well-studied 4-community partition with
        // Q ≈ 0.42 reported in the original Blondel et al. paper.
        // Using just edges in this test to avoid an external
        // fixture dependency.
        let edges: &[(u32, u32)] = &[
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 5),
            (0, 6),
            (0, 7),
            (0, 8),
            (0, 10),
            (0, 11),
            (0, 12),
            (0, 13),
            (0, 17),
            (0, 19),
            (0, 21),
            (0, 31),
            (1, 2),
            (1, 3),
            (1, 7),
            (1, 13),
            (1, 17),
            (1, 19),
            (1, 21),
            (1, 30),
            (2, 3),
            (2, 7),
            (2, 8),
            (2, 9),
            (2, 13),
            (2, 27),
            (2, 28),
            (2, 32),
            (3, 7),
            (3, 12),
            (3, 13),
            (4, 6),
            (4, 10),
            (5, 6),
            (5, 10),
            (5, 16),
            (6, 16),
            (8, 30),
            (8, 32),
            (8, 33),
            (9, 33),
            (13, 33),
            (14, 32),
            (14, 33),
            (15, 32),
            (15, 33),
            (18, 32),
            (18, 33),
            (19, 33),
            (20, 32),
            (20, 33),
            (22, 32),
            (22, 33),
            (23, 25),
            (23, 27),
            (23, 29),
            (23, 32),
            (23, 33),
            (24, 25),
            (24, 27),
            (24, 31),
            (25, 31),
            (26, 29),
            (26, 33),
            (27, 33),
            (28, 31),
            (28, 33),
            (29, 32),
            (29, 33),
            (30, 32),
            (30, 33),
            (31, 32),
            (31, 33),
            (32, 33),
        ];
        let mut g = Graph::with_vertices(34);
        add_undirected_edges(&mut g, edges);
        let r = louvain(&g).unwrap();
        assert!(
            r.modularity > 0.39,
            "karate club Louvain Q should exceed 0.39, got {}",
            r.modularity
        );
        // Should split into at least 2 and at most 6 communities.
        let k: std::collections::BTreeSet<_> = r.membership.iter().copied().collect();
        assert!(k.len() >= 2 && k.len() <= 6);
    }
}
