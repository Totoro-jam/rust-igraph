//! Leiden community detection (ALGO-CO-003).
//!
//! Counterpart of `igraph_community_leiden()` /
//! `igraph_community_leiden_simple()` from
//! `references/igraph/src/community/leiden.c`.
//!
//! Traag, Waltman, van Eck (2019): *From Louvain to Leiden: guaranteeing
//! well-connected communities*. Scientific Reports 9(1) 5233.
//! <https://doi.org/10.1038/s41598-019-41695-z>
//!
//! Three-phase per-iteration loop, repeated until the partition is
//! stable (or for a fixed `n_iterations`):
//!
//! - **Local moving (fast)**: a queue of unstable vertices is processed
//!   in seeded random order; each vertex is greedily moved to the
//!   neighbour cluster that strictly improves the quality. Whenever a
//!   vertex moves, its stable neighbours not in the new cluster are
//!   pushed back onto the queue.
//! - **Refinement**: each cluster is split into well-connected
//!   subclusters by re-running a local-moving pass starting from a
//!   singleton partition within the cluster. Candidate merges are
//!   sampled with probability `∝ exp(diff/β)` over the *non-negative*
//!   diffs — this is the key fix vs Louvain that guarantees connected,
//!   well-separated communities.
//! - **Aggregation**: the super-graph's vertices are the **refined**
//!   subclusters (not the original clusters); the initial partition of
//!   the super-graph maps each refined subcluster back to its parent
//!   cluster. This is what lets later iterations recover the
//!   refinement-introduced splits.
//!
//! The quality function is the generic Reichardt-Bornholdt form
//!
//! ```text
//!   Q = (1 / 2m) Σ_c (e_c − γ · N_c²)        (undirected)
//! ```
//!
//! parameterised by per-vertex weights `n_i` (so `N_c = Σ_{v∈c} n_v`)
//! and the resolution `γ`. Setting `n_i = k_i` (degree) and dividing
//! `γ` by `2m` recovers Newman-Girvan modularity; setting `n_i = 1`
//! recovers the Constant Potts Model; setting `n_i = 1` and scaling
//! `γ` by the weighted density recovers the Erdős-Rényi objective.
//! These three objectives are exposed via [`LeidenObjective`].
//!
//! Self-rolled, dependency-free. Determinism comes from an inline
//! `SplitMix64` PRNG seeded by the caller. The convenience entrypoint
//! [`leiden`] pins `seed = 0` so repeated calls on the same graph
//! produce identical partitions.

// All `usize` -> `u32` casts in this module are bounded by `n`
// (the graph vertex count or its aggregated super-graph), which
// originates from `Graph::vcount(): u32` and so can never truncate.
// The single-char names (`u`, `v`, `w`, `c`, `m`) are domain-standard
// for graph endpoints / weights / communities / edge count.
//
// `needless_range_loop` is allowed because we frequently iterate by
// index across several parallel arrays (membership / vertex_weight /
// loop_weight); zipping them all up loses both clarity and (in some
// hot loops) speed. `cast_precision_loss` is allowed because the
// graph quantities exceed `2^53` only on inputs vastly larger than
// the algorithm can handle anyway. `unnecessary_wraps` /
// `ptr_arg` / `too_many_lines` are allowed for kernel functions that
// directly mirror the structure of the upstream C source.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::ptr_arg,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::unnecessary_wraps
)]

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Hard cap on aggregation rounds inside a single Leiden iteration.
/// Real-world graphs converge in handfuls of levels; the cap is a
/// pathological-input guard.
const MAX_LEVELS_PER_ITERATION: usize = 64;

/// Default randomness used in the refinement step — same default as
/// upstream `igraph_community_leiden_simple` (β = 0.01).
pub const LEIDEN_DEFAULT_BETA: f64 = 0.01;

/// Default number of outer iterations. The reference paper and
/// upstream both note that two iterations are usually enough.
pub const LEIDEN_DEFAULT_ITERATIONS: i32 = 2;

/// Quality function (objective) that Leiden optimises. Mirrors
/// `igraph_leiden_objective_t`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LeidenObjective {
    /// Generalised modularity
    /// `Q = 1/(2m) Σ_ij (A_ij − γ k_i k_j / (2m)) δ(c_i,c_j)`.
    /// Edge weights must not be negative.
    Modularity,
    /// Constant Potts Model
    /// `Q = 1/(2m) Σ_ij (A_ij − γ) δ(c_i,c_j)`.
    /// Edge weights are allowed to be negative.
    Cpm,
    /// Erdős-Rényi `Q = 1/(2m) Σ_ij (A_ij − γ·p) δ(c_i,c_j)`, with
    /// `p` the weighted density. Edge weights must not be negative.
    Er,
}

/// Full set of options for [`leiden_with_options`]. Construct via
/// `LeidenOptions::default()` and then mutate the fields you care
/// about, mirroring the way upstream C exposes per-parameter defaults.
#[derive(Debug, Clone)]
pub struct LeidenOptions {
    /// Quality function. Default [`LeidenObjective::Modularity`].
    pub objective: LeidenObjective,
    /// Resolution parameter γ. Default `1.0`. For CPM, sensible values
    /// are very small (e.g. `0.05`); for modularity, `1.0` is the
    /// classical choice.
    pub resolution: f64,
    /// Refinement randomness. Default [`LEIDEN_DEFAULT_BETA`].
    pub beta: f64,
    /// Number of outer iterations to run. Negative ⇒ iterate until a
    /// pass produces no change. Default [`LEIDEN_DEFAULT_ITERATIONS`].
    pub n_iterations: i32,
    /// PRNG seed (drives both the local-moving shuffle and the
    /// refinement sampling). Default `0`.
    pub seed: u64,
    /// If `Some`, start from this membership vector instead of the
    /// singleton partition. Length must equal `vcount`.
    pub start: Option<Vec<u32>>,
}

impl Default for LeidenOptions {
    fn default() -> Self {
        Self {
            objective: LeidenObjective::Modularity,
            resolution: 1.0,
            beta: LEIDEN_DEFAULT_BETA,
            n_iterations: LEIDEN_DEFAULT_ITERATIONS,
            seed: 0,
            start: None,
        }
    }
}

/// Result of a Leiden run.
#[derive(Debug, Clone)]
pub struct LeidenResult {
    /// Length-`vcount` vector of compacted community labels in `0..k`.
    pub membership: Vec<u32>,
    /// Final value of the chosen objective function on `membership`.
    pub quality: f64,
    /// Number of distinct communities (`k`).
    pub nb_clusters: u32,
    /// Number of outer iterations actually executed.
    pub n_iterations_run: u32,
    /// Quality value at the end of each outer iteration.
    pub qualities: Vec<f64>,
}

// ============================================================================
//                              Public API
// ============================================================================

/// Run Leiden with the default modularity objective, `γ = 1`,
/// `β = 0.01`, two iterations, seed `0`.
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph` is directed.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, leiden};
///
/// // Two triangles joined by a single bridge edge.
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let r = leiden(&g).unwrap();
/// assert_eq!(r.membership[0], r.membership[1]);
/// assert_eq!(r.membership[3], r.membership[4]);
/// assert_ne!(r.membership[0], r.membership[3]);
/// assert!(r.quality > 0.30);
/// ```
pub fn leiden(graph: &Graph) -> IgraphResult<LeidenResult> {
    leiden_with_options(graph, None, &LeidenOptions::default())
}

/// Run Leiden with per-edge weights (modularity objective, `γ = 1`,
/// `β = 0.01`, two iterations, seed `0`).
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph` is directed.
/// - [`IgraphError::InvalidArgument`] if `weights.len() != ecount`,
///   any weight is non-finite, or any weight is negative.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, leiden_weighted};
///
/// // Two K3s + bridge with heavy intra-clique and very thin bridge:
/// // Leiden splits the two triangles.
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let weights = vec![10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 0.01];
/// let r = leiden_weighted(&g, &weights).unwrap();
/// assert_eq!(r.membership[0], r.membership[1]);
/// assert_ne!(r.membership[0], r.membership[3]);
/// ```
pub fn leiden_weighted(graph: &Graph, weights: &[f64]) -> IgraphResult<LeidenResult> {
    leiden_with_options(graph, Some(weights), &LeidenOptions::default())
}

/// Run Leiden with the full set of options.
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph` is directed.
/// - [`IgraphError::InvalidArgument`] for malformed weights,
///   `opts.resolution < 0`, `!opts.beta.is_finite()`, `opts.beta < 0`,
///   `!opts.resolution.is_finite()`, negative weights with
///   [`LeidenObjective::Modularity`] or [`LeidenObjective::Er`], or a
///   non-`None` `opts.start` whose length differs from `vcount`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, LeidenObjective, LeidenOptions, leiden_with_options};
///
/// // Same partition every time for a fixed seed.
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let opts = LeidenOptions {
///     objective: LeidenObjective::Cpm,
///     resolution: 0.1,
///     seed: 42,
///     ..LeidenOptions::default()
/// };
/// let a = leiden_with_options(&g, None, &opts).unwrap();
/// let b = leiden_with_options(&g, None, &opts).unwrap();
/// assert_eq!(a.membership, b.membership);
/// ```
pub fn leiden_with_options(
    graph: &Graph,
    weights: Option<&[f64]>,
    opts: &LeidenOptions,
) -> IgraphResult<LeidenResult> {
    if graph.is_directed() {
        return Err(IgraphError::Unsupported(
            "leiden currently supports undirected graphs only",
        ));
    }
    if !opts.resolution.is_finite() || opts.resolution < 0.0 {
        return Err(IgraphError::InvalidArgument(
            "resolution parameter must be non-negative and finite".to_string(),
        ));
    }
    if !opts.beta.is_finite() || opts.beta < 0.0 {
        return Err(IgraphError::InvalidArgument(
            "beta parameter must be non-negative and finite".to_string(),
        ));
    }
    validate_weights(graph, weights, opts.objective)?;

    let n = graph.vcount() as usize;

    if n == 0 {
        return Ok(LeidenResult {
            membership: Vec::new(),
            quality: 0.0,
            nb_clusters: 0,
            n_iterations_run: 0,
            qualities: Vec::new(),
        });
    }

    // Initial membership: start vector or singleton partition.
    let mut membership: Vec<u32> = if let Some(start) = opts.start.as_deref() {
        if start.len() != n {
            return Err(IgraphError::InvalidArgument(format!(
                "start membership length ({}) differs from vcount ({n})",
                start.len()
            )));
        }
        let mut m = start.to_vec();
        // Reindex to guarantee dense [0, k) labels.
        reindex_membership(&mut m);
        m
    } else {
        (0..n as u32).collect()
    };

    // Compute (effective_resolution, vertex_weights) from the chosen
    // objective. These are passed to the kernel verbatim.
    let edge_weights: Vec<f64> = match weights {
        Some(w) => w.to_vec(),
        None => vec![1.0; graph.ecount()],
    };
    let (eff_resolution, vertex_weights) =
        derive_objective_params(graph, &edge_weights, opts.objective, opts.resolution)?;

    let mut rng = SplitMix64::new(opts.seed);

    let mut qualities: Vec<f64> = Vec::new();
    let mut n_iterations_run: u32 = 0;
    let mut changed = true;

    let max_iter = if opts.n_iterations < 0 {
        u32::MAX
    } else {
        u32::try_from(opts.n_iterations).unwrap_or(u32::MAX)
    };

    for _itr in 0..max_iter {
        if opts.n_iterations < 0 && !changed {
            break;
        }
        changed = false;
        community_leiden_pass(
            graph,
            &edge_weights,
            &vertex_weights,
            eff_resolution,
            opts.beta,
            &mut membership,
            &mut rng,
            &mut changed,
        )?;
        n_iterations_run += 1;
        let q = compute_quality(
            graph,
            &edge_weights,
            &vertex_weights,
            &membership,
            eff_resolution,
        );
        qualities.push(q);
    }

    let nb_clusters = reindex_membership(&mut membership);
    let quality = compute_quality(
        graph,
        &edge_weights,
        &vertex_weights,
        &membership,
        eff_resolution,
    );

    Ok(LeidenResult {
        membership,
        quality,
        nb_clusters,
        n_iterations_run,
        qualities,
    })
}

// ============================================================================
//                       Objective parameter derivation
// ============================================================================

/// Derive `(effective_resolution, vertex_weights)` from the user-facing
/// objective. Mirrors the dispatch table in `igraph_community_leiden_simple`.
fn derive_objective_params(
    graph: &Graph,
    edge_weights: &[f64],
    objective: LeidenObjective,
    user_resolution: f64,
) -> IgraphResult<(f64, Vec<f64>)> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();

    match objective {
        LeidenObjective::Modularity => {
            // Vertex weights = weighted degree (strength); IGRAPH_LOOPS so
            // self-loops count once each — matches igraph_strength().
            let mut strength = vec![0.0_f64; n];
            for e in 0..m {
                let e_id = u32::try_from(e).map_err(|_| {
                    IgraphError::InvalidArgument("edge count exceeds u32::MAX".into())
                })?;
                let (u, v) = graph.edge(e_id as EdgeId)?;
                let w = edge_weights[e];
                strength[u as usize] += w;
                if u != v {
                    strength[v as usize] += w;
                }
            }
            let total: f64 = strength.iter().sum();
            if total <= 0.0 {
                // No edges, or all-zero weights ⇒ no rescaling to do
                // and γ has no effect on any non-trivial cluster.
                return Ok((user_resolution, strength));
            }
            // Undirected: total = 2m. Rescale γ by 1/(2m).
            Ok((user_resolution / total, strength))
        }
        LeidenObjective::Cpm => Ok((user_resolution, vec![1.0; n])),
        LeidenObjective::Er => {
            // Density with loops allowed: p = total_weight / (n*(n+1)/2).
            let total: f64 = edge_weights.iter().sum();
            if n == 0 {
                return Ok((user_resolution, Vec::new()));
            }
            let denom = (n as f64) * (n as f64 + 1.0) * 0.5;
            let p = if denom > 0.0 { total / denom } else { 0.0 };
            Ok((user_resolution * p, vec![1.0; n]))
        }
    }
}

// ============================================================================
//                            Working state (per level)
// ============================================================================

/// Working graph + per-cluster bookkeeping for one aggregation level.
///
/// We carry around the explicit edge weights vector parallel to the
/// adjacency list because the per-vertex weight `n_v` lives separately
/// (it is *not* the degree in the generic quality formula — it is the
/// caller-chosen objective weight).
///
/// Non-loop edges appear twice in `adj` (once per endpoint). Self-loops
/// live in `loop_weight` (raw weight, not doubled).
struct LeidenLevel {
    n: usize,
    /// `adj[v]` = `(neighbour, weight)` for every non-loop incident edge.
    adj: Vec<Vec<(u32, f64)>>,
    /// Raw self-loop weight at `v` (not doubled).
    loop_weight: Vec<f64>,
    /// Caller-chosen per-vertex weight `n_v` (degree for modularity,
    /// `1.0` for CPM/ER, etc.). Distinct from the actual graph degree.
    vertex_weight: Vec<f64>,
}

fn build_level_from_graph(
    graph: &Graph,
    edge_weights: &[f64],
    vertex_weights: &[f64],
) -> LeidenLevel {
    let n = graph.vcount() as usize;
    let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    let mut loop_weight = vec![0.0_f64; n];
    let m = graph.ecount();
    for e in 0..m {
        // We trust the graph's edge() to succeed because we already
        // validated weights.len() == ecount upstream.
        let Ok((u, v)) = graph.edge(e as EdgeId) else {
            continue;
        };
        let w = edge_weights[e];
        if u == v {
            loop_weight[u as usize] += w;
        } else {
            adj[u as usize].push((v, w));
            adj[v as usize].push((u, w));
        }
    }
    LeidenLevel {
        n,
        adj,
        loop_weight,
        vertex_weight: vertex_weights.to_vec(),
    }
}

fn build_level_from_edges(
    n: usize,
    edges: &[(u32, u32, f64)],
    vertex_weights: Vec<f64>,
) -> LeidenLevel {
    let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    let mut loop_weight = vec![0.0_f64; n];
    for &(u, v, w) in edges {
        if u == v {
            loop_weight[u as usize] += w;
        } else {
            adj[u as usize].push((v, w));
            adj[v as usize].push((u, w));
        }
    }
    LeidenLevel {
        n,
        adj,
        loop_weight,
        vertex_weight: vertex_weights,
    }
}

// ============================================================================
//                            One full pass (= one C "community_leiden")
// ============================================================================

fn community_leiden_pass(
    graph: &Graph,
    edge_weights: &[f64],
    vertex_weights: &[f64],
    resolution: f64,
    beta: f64,
    membership: &mut Vec<u32>,
    rng: &mut SplitMix64,
    changed: &mut bool,
) -> IgraphResult<()> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(());
    }

    // Build the level-0 working state from the input graph.
    let mut level = build_level_from_graph(graph, edge_weights, vertex_weights);

    // The "aggregate vertex" map records, for each original vertex,
    // its current super-vertex id. Starts as identity.
    let mut aggregate_vertex: Vec<u32> = (0..n as u32).collect();

    // Working membership on the current level. Starts as the caller's
    // initial membership.
    let mut work_membership: Vec<u32> = membership.clone();

    // Reindex once up-front; subsequent values are produced by
    // leiden_fastmove_vertices/reindex calls inside the loop.
    reindex_membership(&mut work_membership);

    for level_idx in 0..MAX_LEVELS_PER_ITERATION {
        // ---- Phase 1: fast local moves --------------------------------
        let (nb_clusters, level_changed) =
            leiden_fastmove_vertices(&level, resolution, &mut work_membership, rng);
        if level_changed {
            *changed = true;
        }

        // Continue only if there is room to coarsen further.
        let continue_clustering = (nb_clusters as usize) < level.n;
        if !continue_clustering {
            break;
        }

        // On every non-first level, propagate the current super-vertex
        // membership down to the original vertices (parallels the
        // `if (level > 0) ... membership[i] = i_membership[aggregate_vertex[i]]`
        // block in the C source).
        if level_idx > 0 {
            for i in 0..n {
                let v_agg = aggregate_vertex[i] as usize;
                membership[i] = work_membership[v_agg];
            }
        } else {
            // Level 0: the membership is the work_membership itself.
            membership.copy_from_slice(&work_membership);
        }

        // ---- Phase 2: refinement --------------------------------------
        let mut refined_membership: Vec<u32> = vec![0; level.n];
        let mut nb_refined_clusters: usize = 0;

        // Bucket vertices by their (current) cluster.
        let mut clusters: Vec<Vec<u32>> = vec![Vec::new(); nb_clusters as usize];
        for v in 0..level.n {
            let c = work_membership[v] as usize;
            clusters[c].push(v as u32);
        }

        for (c, vertex_subset) in clusters.iter().enumerate() {
            leiden_merge_vertices(
                &level,
                vertex_subset,
                &work_membership,
                c as u32,
                resolution,
                beta,
                &mut nb_refined_clusters,
                &mut refined_membership,
                rng,
            );
        }

        // If refinement didn't actually split anything (every refined
        // cluster is a singleton), fall back to aggregating on the
        // current clustering instead.
        if nb_refined_clusters >= level.n {
            refined_membership.copy_from_slice(&work_membership);
            nb_refined_clusters = nb_clusters as usize;
        }

        // ---- Update the original→super map for the next level --------
        for i in 0..n {
            let v_agg = aggregate_vertex[i] as usize;
            aggregate_vertex[i] = refined_membership[v_agg];
        }

        // ---- Phase 3: aggregate --------------------------------------
        let (next_level, next_membership) = leiden_aggregate(
            &level,
            &work_membership,
            &refined_membership,
            nb_refined_clusters,
        );

        level = next_level;
        work_membership = next_membership;
        reindex_membership(&mut work_membership);
    }

    // Re-derive the final original-vertex membership from the most
    // recent work_membership through the aggregate_vertex map. If we
    // only ran one level (no aggregation), work_membership *is* the
    // answer for the original vertices.
    if level.n == n {
        membership.copy_from_slice(&work_membership);
    } else {
        for i in 0..n {
            let v_agg = aggregate_vertex[i] as usize;
            membership[i] = work_membership[v_agg];
        }
    }

    // Compact labels to [0, k).
    reindex_membership(membership);
    Ok(())
}

// ============================================================================
//                          Phase 1: fastmove vertices
// ============================================================================

fn leiden_fastmove_vertices(
    level: &LeidenLevel,
    resolution: f64,
    membership: &mut [u32],
    rng: &mut SplitMix64,
) -> (u32, bool) {
    let n = level.n;
    if n == 0 {
        return (0, false);
    }

    // Per-cluster weight + count administration.
    let mut cluster_weight = vec![0.0_f64; n];
    let mut cluster_size: Vec<u32> = vec![0; n];
    for v in 0..n {
        let c = membership[v] as usize;
        cluster_weight[c] += level.vertex_weight[v];
        cluster_size[c] += 1;
    }

    // Stack of empty cluster ids — picking the top gives an unused
    // slot we can recycle when a vertex prefers to be a singleton.
    let mut empty_clusters: Vec<u32> = Vec::new();
    for c in 0..n {
        if cluster_size[c] == 0 {
            empty_clusters.push(c as u32);
        }
    }

    // Initial queue: every vertex in a shuffled order.
    let mut order: Vec<u32> = (0..n as u32).collect();
    if n > 1 {
        shuffle_in_place(&mut order, rng);
    }
    let mut queue: std::collections::VecDeque<u32> = order.into_iter().collect();
    let mut stable: Vec<bool> = vec![false; n];

    // Scratch: per-cluster summed edge weight from the popped vertex,
    // plus the list of touched neighbour-clusters so we can reset them
    // in O(deg) instead of O(n).
    let mut edge_weight_per_cluster = vec![0.0_f64; n];
    let mut neighbor_clusters: Vec<u32> = Vec::new();
    let mut cluster_touched: Vec<bool> = vec![false; n];

    let mut changed = false;

    while let Some(v) = queue.pop_front() {
        let v_idx = v as usize;
        let weight_v = level.vertex_weight[v_idx];
        let current_cluster = membership[v_idx] as usize;

        // Strip v from its cluster.
        cluster_weight[current_cluster] -= weight_v;
        cluster_size[current_cluster] -= 1;
        if cluster_size[current_cluster] == 0 {
            empty_clusters.push(current_cluster as u32);
        }

        // Default neighbour set: include the top of the empty stack so
        // that "become a new singleton" is always an option.
        neighbor_clusters.clear();
        let empty_top = empty_clusters.last().copied();
        if let Some(top) = empty_top {
            let top_us = top as usize;
            if !cluster_touched[top_us] {
                cluster_touched[top_us] = true;
                neighbor_clusters.push(top);
            }
        }

        // Sum edge weights into each neighbour cluster.
        for &(u, w) in &level.adj[v_idx] {
            let u_us = u as usize;
            if u_us == v_idx {
                continue;
            }
            let c = membership[u_us];
            let c_us = c as usize;
            if !cluster_touched[c_us] {
                cluster_touched[c_us] = true;
                neighbor_clusters.push(c);
            }
            edge_weight_per_cluster[c_us] += w;
        }

        // Score the identity move (back into current_cluster).
        let mut best_cluster = current_cluster as u32;
        let mut max_diff = edge_weight_per_cluster[current_cluster]
            - weight_v * cluster_weight[current_cluster] * resolution;

        for &c in &neighbor_clusters {
            let c_us = c as usize;
            let diff = edge_weight_per_cluster[c_us] - weight_v * cluster_weight[c_us] * resolution;
            if diff > max_diff {
                best_cluster = c;
                max_diff = diff;
            }
        }

        // Reset scratch slots.
        for &c in &neighbor_clusters {
            let c_us = c as usize;
            edge_weight_per_cluster[c_us] = 0.0;
            cluster_touched[c_us] = false;
        }

        let best_us = best_cluster as usize;

        // Re-attach v to best_cluster.
        cluster_weight[best_us] += weight_v;
        cluster_size[best_us] += 1;
        if empty_top == Some(best_cluster) && cluster_size[best_us] == 1 {
            empty_clusters.pop();
        }

        // Mark v stable (regardless of whether it actually moved).
        stable[v_idx] = true;

        if best_us != current_cluster {
            changed = true;
            membership[v_idx] = best_cluster;
            // Push stable neighbours that aren't in the new cluster back
            // onto the queue — they may want to follow v.
            for &(u, _) in &level.adj[v_idx] {
                let u_us = u as usize;
                if stable[u_us] && membership[u_us] as usize != best_us {
                    queue.push_back(u);
                    stable[u_us] = false;
                }
            }
        }
    }

    let nb_clusters = reindex_membership(membership);
    (nb_clusters, changed)
}

// ============================================================================
//                              Phase 2: refinement
// ============================================================================

fn leiden_merge_vertices(
    level: &LeidenLevel,
    vertex_subset: &[u32],
    parent_membership: &[u32],
    parent_cluster: u32,
    resolution: f64,
    beta: f64,
    nb_refined_clusters: &mut usize,
    refined_membership: &mut [u32],
    rng: &mut SplitMix64,
) {
    let n_sub = vertex_subset.len();
    if n_sub == 0 {
        return;
    }
    if n_sub == 1 {
        // Singleton-cluster shortcut: a single vertex is always its own
        // refined cluster. We still need to assign it a refined id.
        refined_membership[vertex_subset[0] as usize] = (*nb_refined_clusters) as u32;
        *nb_refined_clusters += 1;
        return;
    }

    // Local cluster bookkeeping over the n_sub-vertex sub-problem.
    let mut sub_cluster_weight = vec![0.0_f64; n_sub];
    let mut sub_nb_vertices: Vec<u32> = vec![0; n_sub];
    let mut external_edge_weight = vec![0.0_f64; n_sub];
    let mut total_subset_weight = 0.0_f64;

    // Initial singleton partition inside the subset: each vertex maps
    // to a unique local cluster index 0..n_sub. We reuse the global
    // `refined_membership` slot for storage but treat it as
    // local-index-space inside this function.
    for (i, &v) in vertex_subset.iter().enumerate() {
        let v_us = v as usize;
        refined_membership[v_us] = i as u32;
        sub_cluster_weight[i] += level.vertex_weight[v_us];
        total_subset_weight += level.vertex_weight[v_us];
        sub_nb_vertices[i] += 1;

        for &(u, w) in &level.adj[v_us] {
            let u_us = u as usize;
            if u_us != v_us && parent_membership[u_us] == parent_cluster {
                external_edge_weight[i] += w;
            }
        }
    }

    // Walk the subset in shuffled order.
    let mut order: Vec<u32> = vertex_subset.to_vec();
    if n_sub > 1 {
        shuffle_in_place(&mut order, rng);
    }

    let mut non_singleton: Vec<bool> = vec![false; n_sub];
    let mut edge_weight_per_local = vec![0.0_f64; n_sub];
    let mut neighbor_locals: Vec<u32> = Vec::new();
    let mut local_touched: Vec<bool> = vec![false; n_sub];
    let mut cum_trans_diff = vec![0.0_f64; n_sub];

    for &v in &order {
        let v_us = v as usize;
        let current = refined_membership[v_us] as usize;

        // Skip non-singletons — refinement only ever moves out of a
        // singleton (per the Leiden paper / C reference).
        if non_singleton[current] {
            continue;
        }

        // Skip vertices whose singleton is not well-connected to the
        // rest of the parent cluster.
        let vw_prod =
            sub_cluster_weight[current] * (total_subset_weight - sub_cluster_weight[current]);
        if external_edge_weight[current] < vw_prod * resolution {
            continue;
        }

        // Detach v from its singleton.
        sub_cluster_weight[current] = 0.0;
        sub_nb_vertices[current] = 0;

        // Build the neighbour-cluster list, restricted to neighbours
        // that share the same parent_cluster.
        neighbor_locals.clear();
        if !local_touched[current] {
            local_touched[current] = true;
            neighbor_locals.push(current as u32);
        }
        for &(u, w) in &level.adj[v_us] {
            let u_us = u as usize;
            if u_us != v_us && parent_membership[u_us] == parent_cluster {
                let c = refined_membership[u_us];
                let c_us = c as usize;
                if !local_touched[c_us] {
                    local_touched[c_us] = true;
                    neighbor_locals.push(c);
                }
                edge_weight_per_local[c_us] += w;
            }
        }

        // Score moves.
        let weight_v = level.vertex_weight[v_us];
        let mut best_cluster = current as u32;
        let mut max_diff = 0.0_f64;
        let mut total_cum = 0.0_f64;

        for (j, &c) in neighbor_locals.iter().enumerate() {
            let c_us = c as usize;
            let vw_prod_c =
                sub_cluster_weight[c_us] * (total_subset_weight - sub_cluster_weight[c_us]);
            if external_edge_weight[c_us] >= vw_prod_c * resolution {
                let diff =
                    edge_weight_per_local[c_us] - weight_v * sub_cluster_weight[c_us] * resolution;
                if diff > max_diff {
                    best_cluster = c;
                    max_diff = diff;
                }
                if diff >= 0.0 {
                    if beta > 0.0 {
                        total_cum += (diff / beta).exp();
                    } else if diff > 0.0 {
                        // β → 0 limit collapses sampling to argmax;
                        // a non-zero positive diff dominates.
                        total_cum = f64::INFINITY;
                    }
                }
            }
            cum_trans_diff[j] = total_cum;
        }

        // Pick a target cluster.
        let chosen_cluster = if total_cum.is_finite() && total_cum > 0.0 {
            let r = rng.next_f64() * total_cum;
            // Find first j such that cum_trans_diff[j] >= r.
            let mut idx = neighbor_locals.len() - 1;
            for (j, &cum) in cum_trans_diff
                .iter()
                .enumerate()
                .take(neighbor_locals.len())
            {
                if cum >= r {
                    idx = j;
                    break;
                }
            }
            neighbor_locals[idx]
        } else if total_cum == 0.0 {
            // No eligible (>0) cluster — leave in current.
            current as u32
        } else {
            best_cluster
        };

        let chosen_us = chosen_cluster as usize;

        // Re-attach v to chosen cluster.
        sub_cluster_weight[chosen_us] += weight_v;
        sub_nb_vertices[chosen_us] += 1;

        // Update external_edge_weight[chosen]: edges from v to
        // vertices already in chosen go internal; edges from v to
        // vertices in other (parent-internal) clusters go external.
        for &(u, w) in &level.adj[v_us] {
            let u_us = u as usize;
            if parent_membership[u_us] == parent_cluster {
                if refined_membership[u_us] as usize == chosen_us {
                    external_edge_weight[chosen_us] -= w;
                } else {
                    external_edge_weight[chosen_us] += w;
                }
            }
        }

        if chosen_us != current {
            refined_membership[v_us] = chosen_cluster;
            non_singleton[chosen_us] = true;
        }

        // Reset scratch slots for the next iteration.
        for &c in &neighbor_locals {
            let c_us = c as usize;
            edge_weight_per_local[c_us] = 0.0;
            local_touched[c_us] = false;
        }
    }

    // Translate local cluster ids back into the global numbering, with
    // dense contiguous labels starting at `*nb_refined_clusters`.
    let base = *nb_refined_clusters;
    // local_to_global[i] = u32::MAX when unassigned, else the global id.
    let mut local_to_global: Vec<u32> = vec![u32::MAX; n_sub];
    let mut next_global = base;
    for &v in vertex_subset {
        let v_us = v as usize;
        let local = refined_membership[v_us] as usize;
        if local_to_global[local] == u32::MAX {
            local_to_global[local] = next_global as u32;
            next_global += 1;
        }
        refined_membership[v_us] = local_to_global[local];
    }
    *nb_refined_clusters = next_global;
}

// ============================================================================
//                              Phase 3: aggregate
// ============================================================================

/// Aggregate the current level on the basis of `refined_membership`.
/// Returns the new level and the new working membership (mapping each
/// new super-vertex back to its parent cluster).
fn leiden_aggregate(
    level: &LeidenLevel,
    parent_membership: &[u32],
    refined_membership: &[u32],
    nb_refined: usize,
) -> (LeidenLevel, Vec<u32>) {
    let k = nb_refined;

    // Build new edge list. We aggregate inter-cluster edges via a
    // dedup-by-linear-scan adjacency map (one Vec<(u,w)> per cluster,
    // entries kept with `to > from` so each super-edge appears once),
    // and self-loops via a per-cluster accumulator.
    let mut inter: Vec<Vec<(u32, f64)>> = vec![Vec::new(); k];
    let mut loops: Vec<f64> = vec![0.0_f64; k];
    let mut agg_vertex_weight: Vec<f64> = vec![0.0_f64; k];
    // Membership of each refined cluster = its parent cluster id (taken
    // from any of its vertices — they all share the same parent).
    let mut new_membership: Vec<u32> = vec![0; k];

    for v in 0..level.n {
        let c = refined_membership[v] as usize;
        agg_vertex_weight[c] += level.vertex_weight[v];
        new_membership[c] = parent_membership[v];

        // Carry self-loops over.
        let lw = level.loop_weight[v];
        if lw > 0.0 {
            loops[c] += lw;
        }

        for &(u, w) in &level.adj[v] {
            let u_us = u as usize;
            // Each non-loop edge appears twice in `adj`; only process
            // it from the lower endpoint to avoid double-counting.
            if u_us <= v {
                continue;
            }
            let c2 = refined_membership[u_us] as usize;
            if c == c2 {
                loops[c] += w;
            } else {
                let (a, b) = if c < c2 { (c, c2) } else { (c2, c) };
                push_or_merge(&mut inter[a], b as u32, w);
            }
        }
    }

    // Flatten into an edge list with `from <= to`.
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    for c in 0..k {
        if loops[c] > 0.0 {
            edges.push((c as u32, c as u32, loops[c]));
        }
        for &(c2, w) in &inter[c] {
            edges.push((c as u32, c2, w));
        }
    }

    let next_level = build_level_from_edges(k, &edges, agg_vertex_weight);
    (next_level, new_membership)
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

// ============================================================================
//                              Quality computation
// ============================================================================

/// Quality of a membership on the original graph, matching upstream
/// `leiden_quality()`. Undirected only.
fn compute_quality(
    graph: &Graph,
    edge_weights: &[f64],
    vertex_weights: &[f64],
    membership: &[u32],
    resolution: f64,
) -> f64 {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    if n == 0 {
        return 0.0;
    }

    let mut q = 0.0_f64;
    let mut total_edge_weight = 0.0_f64;
    for e in 0..m {
        let Ok((u, v)) = graph.edge(e as EdgeId) else {
            continue;
        };
        let w = edge_weights[e];
        total_edge_weight += w;
        if membership[u as usize] == membership[v as usize] {
            // Undirected: directed_multiplier = 2.0
            q += 2.0 * w;
        }
    }

    // Cluster sums of vertex weights.
    let k = membership.iter().copied().max().map_or(0, |m| m + 1) as usize;
    let mut cluster_weight = vec![0.0_f64; k];
    for v in 0..n {
        cluster_weight[membership[v] as usize] += vertex_weights[v];
    }
    for c in 0..k {
        q -= resolution * cluster_weight[c] * cluster_weight[c];
    }

    if total_edge_weight <= 0.0 {
        return 0.0;
    }
    q / (2.0 * total_edge_weight)
}

// ============================================================================
//                            Membership reindexing
// ============================================================================

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

// ============================================================================
//                              Weight validation
// ============================================================================

fn validate_weights(
    graph: &Graph,
    weights: Option<&[f64]>,
    objective: LeidenObjective,
) -> IgraphResult<()> {
    let Some(w) = weights else {
        return Ok(());
    };
    let m = graph.ecount();
    if w.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "weights vector size ({}) differs from edge count ({m})",
            w.len(),
        )));
    }
    let allow_negative = matches!(objective, LeidenObjective::Cpm);
    for (e, &v) in w.iter().enumerate() {
        if !v.is_finite() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is not finite ({v})"
            )));
        }
        if !allow_negative && v < 0.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is negative ({v}); leiden with this objective requires non-negative weights"
            )));
        }
    }
    Ok(())
}

// ============================================================================
//                                  PRNG
// ============================================================================

struct SplitMix64(u64);

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        // Avoid the all-zero pathological state.
        Self(seed.wrapping_add(0x9E37_79B9_7F4A_7C15))
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
        usize::try_from(r).unwrap_or(0)
    }
    /// Uniform `[0, 1)` via the top 53 bits — standard practice.
    fn next_f64(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) * (1.0 / ((1u64 << 53) as f64))
    }
}

fn shuffle_in_place<T>(slice: &mut [T], rng: &mut SplitMix64) {
    let len = slice.len();
    for i in (1..len).rev() {
        let j = rng.gen_index(i + 1);
        slice.swap(i, j);
    }
}

// ============================================================================
//                                  Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    fn add_edges(g: &mut Graph, edges: &[(u32, u32)]) {
        for &(u, v) in edges {
            g.add_edge(u, v).unwrap();
        }
    }

    #[test]
    fn empty_graph_returns_empty_membership() {
        let g = Graph::with_vertices(0);
        let r = leiden(&g).unwrap();
        assert_eq!(r.membership.len(), 0);
        assert_eq!(r.quality, 0.0);
        assert_eq!(r.nb_clusters, 0);
    }

    #[test]
    fn isolated_vertices_keep_singletons_after_modularity_reduction() {
        // With modularity objective and no edges, vertex strengths are
        // all 0 → effective_resolution stays at the user value but γ
        // multiplies a 0 cluster_weight, so any partition has the same
        // quality. We accept whatever the algorithm produces but
        // require well-formed output.
        let g = Graph::with_vertices(4);
        let r = leiden(&g).unwrap();
        assert_eq!(r.membership.len(), 4);
        let k = r.nb_clusters;
        assert!((1..=4).contains(&k));
    }

    #[test]
    fn two_triangles_bridge_split() {
        let mut g = Graph::with_vertices(6);
        add_edges(
            &mut g,
            &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
        );
        let r = leiden(&g).unwrap();
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[0], r.membership[2]);
        assert_eq!(r.membership[3], r.membership[4]);
        assert_eq!(r.membership[3], r.membership[5]);
        assert_ne!(r.membership[0], r.membership[3]);
    }

    #[test]
    fn directed_input_rejected() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(leiden(&g).is_err());
        assert!(leiden_weighted(&g, &[1.0]).is_err());
        assert!(leiden_with_options(&g, None, &LeidenOptions::default()).is_err());
    }

    #[test]
    fn weight_validation() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(leiden_weighted(&g, &[1.0]).is_err());
        assert!(leiden_weighted(&g, &[1.0, -0.1]).is_err());
        assert!(leiden_weighted(&g, &[1.0, f64::NAN]).is_err());
        assert!(leiden_weighted(&g, &[1.0, f64::INFINITY]).is_err());
        // CPM allows negative weights.
        let opts = LeidenOptions {
            objective: LeidenObjective::Cpm,
            ..LeidenOptions::default()
        };
        assert!(leiden_with_options(&g, Some(&[1.0, -0.1]), &opts).is_ok());
    }

    #[test]
    fn deterministic_under_seed() {
        let mut g = Graph::with_vertices(6);
        add_edges(
            &mut g,
            &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
        );
        let opts = LeidenOptions {
            seed: 12345,
            ..LeidenOptions::default()
        };
        let a = leiden_with_options(&g, None, &opts).unwrap();
        let b = leiden_with_options(&g, None, &opts).unwrap();
        assert_eq!(a.membership, b.membership);
        assert!((a.quality - b.quality).abs() < 1e-12);
    }

    #[test]
    fn cpm_objective_works() {
        let mut g = Graph::with_vertices(6);
        add_edges(
            &mut g,
            &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
        );
        // For CPM on the two-triangles-plus-bridge graph (N=6, m=7):
        //   q_union = 14 − γ·36     (one cluster of 6)
        //   q_split = 12 − γ·18     (two cliques of 3)
        // Split is optimal iff γ > (14 − 12) / (36 − 18) = 1/9 ≈ 0.111.
        // Pick γ = 0.5 to leave headroom for the local-move heuristic.
        let opts = LeidenOptions {
            objective: LeidenObjective::Cpm,
            resolution: 0.5,
            ..LeidenOptions::default()
        };
        let r = leiden_with_options(&g, None, &opts).unwrap();
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[0], r.membership[2]);
        assert_eq!(r.membership[3], r.membership[4]);
        assert_eq!(r.membership[3], r.membership[5]);
        assert_ne!(r.membership[0], r.membership[3]);
    }

    #[test]
    fn negative_resolution_rejected() {
        let g = Graph::with_vertices(3);
        let opts = LeidenOptions {
            resolution: -0.1,
            ..LeidenOptions::default()
        };
        assert!(leiden_with_options(&g, None, &opts).is_err());
    }

    #[test]
    fn negative_beta_rejected() {
        let g = Graph::with_vertices(3);
        let opts = LeidenOptions {
            beta: -0.1,
            ..LeidenOptions::default()
        };
        assert!(leiden_with_options(&g, None, &opts).is_err());
    }
}
