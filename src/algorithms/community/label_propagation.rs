//! Label propagation community detection (ALGO-CO-004).
//!
//! Counterpart of `igraph_community_label_propagation()` from
//! `references/igraph/src/community/label_propagation.c`.
//!
//! Raghavan, Albert, Kumara (2007): *Near linear time algorithm to detect
//! community structures in large-scale networks.* Phys Rev E **76**, 036106.
//! <https://doi.org/10.1103/PhysRevE.76.036106>
//!
//! Traag, Šubelj (2023): *Large network community detection by fast label
//! propagation.* Scientific Reports **13**:1. The queue-based "fast"
//! variant. <https://doi.org/10.1038/s41598-023-29610-z>
//!
//! Each vertex's label is updated to a label that is *dominant* among its
//! neighbours (i.e. has the largest summed weight among neighbour labels).
//! Three variants are implemented, mirroring `igraph_lpa_variant_t`:
//!
//! - [`LpaVariant::Fast`] — queue-based (Traag-Šubelj). Vertices to be
//!   re-considered live in a FIFO queue; whenever a vertex changes label,
//!   its neighbours not in the new community and not already queued are
//!   re-enqueued. Terminates when the queue is empty.
//! - [`LpaVariant::Dominance`] — alternating update + control iterations
//!   over the full vertex set. Control iterations re-check that every
//!   vertex still carries a dominant label.
//! - [`LpaVariant::Retention`] — only relabel a vertex when its current
//!   label is not dominant; stop when a full sweep produces no change.
//!
//! Weights, optional initial labels, and an optional `fixed` mask
//! (vertices whose label may not change) are supported. Unlabelled
//! vertices (those carrying `-1` in `initial`) are filled in by a
//! post-pass BFS that gives each remaining unlabelled component its own
//! fresh label.
//!
//! Self-rolled, dependency-free. Determinism comes from an inline
//! `SplitMix64` PRNG seeded by the caller; the convenience entrypoint
//! [`label_propagation`] pins `seed = 0` so repeated calls on the same
//! graph produce identical partitions.

// All `usize` -> `u32` casts in this module are bounded by `n`
// (graph vertex count) which originates from `Graph::vcount(): u32`
// and so can never truncate. The single-char names (`u`, `v`, `w`, `e`,
// `k`) are domain-standard for graph endpoints / weights / edges /
// labels. `needless_range_loop`, `ptr_arg`, `too_many_arguments`,
// `too_many_lines`, `unnecessary_wraps` are allowed for kernel
// functions that mirror the upstream C source.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::ptr_arg,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::unnecessary_wraps
)]

use std::collections::VecDeque;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Which variant of label propagation to run. Mirrors
/// `igraph_lpa_variant_t`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LpaVariant {
    /// Queue-based fast variant (Traag-Šubelj 2023). Default — matches
    /// the python-igraph default.
    Fast,
    /// Alternating update / control iterations over the full vertex set.
    Dominance,
    /// Sweep all vertices; only relabel when the current label is not
    /// dominant.
    Retention,
}

/// Full option bag for [`label_propagation_with_options`].
#[derive(Debug, Clone)]
pub struct LpaOptions {
    /// Algorithm variant. Default [`LpaVariant::Fast`].
    pub variant: LpaVariant,
    /// Optional initial labels. `None` ⇒ every vertex starts as its own
    /// singleton. When `Some`, length must equal `vcount`; negative
    /// entries mean "unlabelled" (filled in by the final BFS step);
    /// non-negative entries must satisfy `label ≤ vcount - 1`.
    pub initial: Option<Vec<i32>>,
    /// Optional per-vertex fixed mask. `true` means the vertex's label
    /// is frozen for the duration of the algorithm. Only meaningful
    /// together with `initial`; unlabelled fixed vertices are silently
    /// un-fixed (matches the upstream warning behaviour). Length must
    /// equal `vcount`.
    pub fixed: Option<Vec<bool>>,
    /// PRNG seed driving the node-order shuffle and the
    /// uniform-random tiebreak among dominant labels. Default `0`.
    pub seed: u64,
}

impl Default for LpaOptions {
    fn default() -> Self {
        Self {
            variant: LpaVariant::Fast,
            initial: None,
            fixed: None,
            seed: 0,
        }
    }
}

/// Result of a label-propagation run.
#[derive(Debug, Clone)]
pub struct LpaResult {
    /// Length-`vcount` vector of compacted community labels in `0..k`.
    pub membership: Vec<u32>,
    /// Number of distinct communities (`k`).
    pub nb_clusters: u32,
}

// ============================================================================
//                              Public API
// ============================================================================

/// Run label propagation with the default options
/// ([`LpaVariant::Fast`], no initial labels, no fixed vertices, seed `0`).
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph` is directed (use
///   [`label_propagation_with_options`] once mode-aware overloads land).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, label_propagation};
///
/// // Two K4 cliques joined by a single bridge edge — each vertex on the
/// // bridge endpoints has 3 in-cluster neighbours vs 1 cross-cluster, so
/// // LPA reliably keeps the bridge cut.
/// let mut g = Graph::with_vertices(8);
/// for u in 0..4 {
///     for v in (u + 1)..4 {
///         g.add_edge(u, v).unwrap();
///     }
/// }
/// for u in 4..8 {
///     for v in (u + 1)..8 {
///         g.add_edge(u, v).unwrap();
///     }
/// }
/// g.add_edge(3, 4).unwrap();
/// let r = label_propagation(&g).unwrap();
/// assert_eq!(r.membership[0], r.membership[1]);
/// assert_eq!(r.membership[4], r.membership[5]);
/// assert_ne!(r.membership[0], r.membership[4]);
/// ```
pub fn label_propagation(graph: &Graph) -> IgraphResult<LpaResult> {
    label_propagation_with_options(graph, None, &LpaOptions::default())
}

/// Run label propagation with per-edge weights (default variant, no
/// initial labels, no fixed vertices, seed `0`).
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph` is directed.
/// - [`IgraphError::InvalidArgument`] if `weights.len() != ecount`, any
///   weight is non-finite, or any weight is negative.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, label_propagation_weighted};
///
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let weights = vec![10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 0.01];
/// let r = label_propagation_weighted(&g, &weights).unwrap();
/// assert_eq!(r.membership[0], r.membership[1]);
/// assert_ne!(r.membership[0], r.membership[3]);
/// ```
pub fn label_propagation_weighted(graph: &Graph, weights: &[f64]) -> IgraphResult<LpaResult> {
    label_propagation_with_options(graph, Some(weights), &LpaOptions::default())
}

/// Run label propagation with the full option bag.
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph` is directed.
/// - [`IgraphError::InvalidArgument`] for malformed weights/initial/fixed
///   inputs as documented on [`LpaOptions`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, LpaOptions, LpaVariant, label_propagation_with_options};
///
/// // Same partition every time for a fixed seed.
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let opts = LpaOptions {
///     variant: LpaVariant::Dominance,
///     seed: 42,
///     ..LpaOptions::default()
/// };
/// let a = label_propagation_with_options(&g, None, &opts).unwrap();
/// let b = label_propagation_with_options(&g, None, &opts).unwrap();
/// assert_eq!(a.membership, b.membership);
/// ```
pub fn label_propagation_with_options(
    graph: &Graph,
    weights: Option<&[f64]>,
    opts: &LpaOptions,
) -> IgraphResult<LpaResult> {
    if graph.is_directed() {
        return Err(IgraphError::Unsupported(
            "label_propagation currently supports undirected graphs only",
        ));
    }
    validate_weights(graph, weights)?;

    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(LpaResult {
            membership: Vec::new(),
            nb_clusters: 0,
        });
    }

    // Validate + materialise initial labels into a working membership
    // vector (negative entries kept as i64::MIN sentinel → translated
    // back to `-1` semantics through `is_labelled`).
    let mut membership: Vec<i32> = if let Some(init) = opts.initial.as_deref() {
        if init.len() != n {
            return Err(IgraphError::InvalidArgument(format!(
                "initial labels length ({}) differs from vcount ({n})",
                init.len()
            )));
        }
        let mut max_label: i32 = -1;
        for &k in init {
            if k > max_label {
                max_label = k;
            }
        }
        if max_label >= 0 && i64::from(max_label) > (n as i64 - 1) {
            return Err(IgraphError::InvalidArgument(format!(
                "initial labels must be in [0, vcount - 1] = [0, {}]; saw {max_label}",
                n - 1
            )));
        }
        init.iter().map(|&k| if k < 0 { -1 } else { k }).collect()
    } else {
        (0..n).map(|i| i as i32).collect()
    };

    // Materialise the fixed mask. Unlabelled-and-fixed vertices are
    // silently un-fixed (mirrors upstream's behaviour of emitting a
    // warning and clearing the bit in the working copy).
    let fixed: Option<Vec<bool>> = match opts.fixed.as_deref() {
        Some(f) => {
            if f.len() != n {
                return Err(IgraphError::InvalidArgument(format!(
                    "fixed mask length ({}) differs from vcount ({n})",
                    f.len()
                )));
            }
            if opts.initial.is_none() {
                // Upstream emits a warning and ignores the mask
                // entirely. We do the same.
                None
            } else {
                let mut out = f.to_vec();
                for v in 0..n {
                    if out[v] && membership[v] < 0 {
                        out[v] = false;
                    }
                }
                Some(out)
            }
        }
        None => None,
    };

    // Build a per-vertex adjacency list once, paired with each edge's
    // weight. IGRAPH_LOOPS_ONCE convention: self-loops appear once.
    let edge_weights: Vec<f64> = match weights {
        Some(w) => w.to_vec(),
        None => vec![1.0; graph.ecount()],
    };
    let adj = build_adjacency(graph, &edge_weights)?;

    let mut rng = SplitMix64::new(opts.seed);

    match opts.variant {
        LpaVariant::Fast => {
            lpa_fast(&adj, &mut membership, fixed.as_deref(), &mut rng);
        }
        LpaVariant::Dominance => {
            lpa_dominance_or_retention(
                &adj,
                &mut membership,
                fixed.as_deref(),
                /* retention */ false,
                &mut rng,
            );
        }
        LpaVariant::Retention => {
            lpa_dominance_or_retention(
                &adj,
                &mut membership,
                fixed.as_deref(),
                /* retention */ true,
                &mut rng,
            );
        }
    }

    // Compact labels into 0..k and BFS-fill any remaining unlabelled
    // components with fresh labels.
    let nb_clusters = finalize_labels(graph, &mut membership, fixed.as_deref(), &mut rng)?;

    // i32 -> u32 once we know everything is in [0, k).
    let membership: Vec<u32> = membership.iter().map(|&k| k as u32).collect();
    Ok(LpaResult {
        membership,
        nb_clusters,
    })
}

// ============================================================================
//                           Adjacency construction
// ============================================================================

/// Build a `(neighbour, weight)` adjacency list with the upstream
/// `IGRAPH_LOOPS_ONCE` convention — a self-loop contributes a single
/// `(v, w)` entry to `adj[v]`, not two.
fn build_adjacency(graph: &Graph, edge_weights: &[f64]) -> IgraphResult<Vec<Vec<(u32, f64)>>> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    for e in 0..m {
        let e_id = u32::try_from(e)
            .map_err(|_| IgraphError::InvalidArgument("edge count exceeds u32::MAX".into()))?;
        let (u, v) = graph.edge(e_id as EdgeId)?;
        let w = edge_weights[e];
        adj[u as usize].push((v, w));
        if u != v {
            adj[v as usize].push((u, w));
        }
    }
    Ok(adj)
}

// ============================================================================
//                          Fast variant (Traag-Šubelj 2023)
// ============================================================================

fn lpa_fast(
    adj: &[Vec<(u32, f64)>],
    membership: &mut [i32],
    fixed: Option<&[bool]>,
    rng: &mut SplitMix64,
) {
    let n = adj.len();
    if n == 0 {
        return;
    }

    // Per-label accumulator + book-keeping for clearing only touched
    // labels (the same trick the C source uses: `nonzero_labels`).
    let mut label_weight = vec![0.0_f64; n];
    let mut touched: Vec<i32> = Vec::with_capacity(8);
    let mut dominant: Vec<i32> = Vec::with_capacity(2);

    // Initial queue: every not-fixed vertex in a shuffled order.
    let mut order: Vec<u32> = match fixed {
        Some(mask) => (0..n as u32).filter(|&v| !mask[v as usize]).collect(),
        None => (0..n as u32).collect(),
    };
    shuffle_in_place(&mut order, rng);

    let mut queue: VecDeque<u32> = order.iter().copied().collect();
    let mut in_queue = vec![false; n];
    for &v in &order {
        in_queue[v as usize] = true;
    }

    while let Some(v1) = queue.pop_front() {
        in_queue[v1 as usize] = false;

        // Accumulate per-label weight across v1's neighbours.
        let neighbours = &adj[v1 as usize];
        let mut max_count: f64 = 0.0;
        dominant.clear();
        touched.clear();
        for &(v2, w) in neighbours {
            let k = membership[v2 as usize];
            if k < 0 {
                continue;
            }
            let was_zero = label_weight[k as usize] == 0.0;
            label_weight[k as usize] += w;
            if was_zero {
                touched.push(k);
            }
            let lw = label_weight[k as usize];
            if max_count < lw {
                max_count = lw;
                dominant.clear();
                dominant.push(k);
            } else if (max_count - lw).abs() < f64::EPSILON || max_count == lw {
                dominant.push(k);
            }
        }

        if !dominant.is_empty() {
            let current = membership[v1 as usize];
            let pick = rng.gen_index(dominant.len());
            let new_label = dominant[pick];

            if new_label != current {
                // Re-enqueue any neighbour that is not in the new
                // community and not already in the queue (and not
                // fixed).
                for &(v2, _) in neighbours {
                    let v2u = v2 as usize;
                    if in_queue[v2u] {
                        continue;
                    }
                    let neigh_label = membership[v2u];
                    let is_fixed = fixed.is_some_and(|m| m[v2u]);
                    if neigh_label != new_label && !is_fixed {
                        queue.push_back(v2);
                        in_queue[v2u] = true;
                    }
                }
            }
            membership[v1 as usize] = new_label;
        }

        // Reset only the labels we actually touched.
        for &k in &touched {
            label_weight[k as usize] = 0.0;
        }
    }
}

// ============================================================================
//                       Dominance + Retention variants
// ============================================================================

fn lpa_dominance_or_retention(
    adj: &[Vec<(u32, f64)>],
    membership: &mut [i32],
    fixed: Option<&[bool]>,
    retention: bool,
    rng: &mut SplitMix64,
) {
    let n = adj.len();
    if n == 0 {
        return;
    }

    let mut label_weight = vec![0.0_f64; n];
    let mut touched: Vec<i32> = Vec::with_capacity(8);
    let mut dominant: Vec<i32> = Vec::with_capacity(2);

    let mut order: Vec<u32> = match fixed {
        Some(mask) => (0..n as u32).filter(|&v| !mask[v as usize]).collect(),
        None => (0..n as u32).collect(),
    };

    let mut running = true;
    let mut control_iteration = true;
    while running {
        if retention {
            running = false;
            shuffle_in_place(&mut order, rng);
        } else if control_iteration {
            running = false;
        } else {
            shuffle_in_place(&mut order, rng);
        }

        for &v1 in &order {
            // Accumulate per-label weight.
            let neighbours = &adj[v1 as usize];
            let mut max_count: f64 = 0.0;
            dominant.clear();
            touched.clear();
            for &(v2, w) in neighbours {
                let k = membership[v2 as usize];
                if k < 0 {
                    continue;
                }
                let was_zero = label_weight[k as usize] == 0.0;
                label_weight[k as usize] += w;
                if was_zero {
                    touched.push(k);
                }
                let lw = label_weight[k as usize];
                if max_count < lw {
                    max_count = lw;
                    dominant.clear();
                    dominant.push(k);
                } else if max_count == lw {
                    dominant.push(k);
                }
            }

            if !dominant.is_empty() {
                if retention {
                    let current = membership[v1 as usize];
                    let keep_current = current >= 0
                        && (current as usize) < n
                        && label_weight[current as usize] >= max_count;
                    if !keep_current {
                        let pick = rng.gen_index(dominant.len());
                        let new_label = dominant[pick];
                        if new_label != current {
                            running = true;
                        }
                        membership[v1 as usize] = new_label;
                    }
                } else if control_iteration {
                    let current = membership[v1 as usize];
                    let still_dominant = current >= 0
                        && (current as usize) < n
                        && label_weight[current as usize] >= max_count;
                    if !still_dominant {
                        running = true;
                    }
                } else {
                    let pick = rng.gen_index(dominant.len());
                    membership[v1 as usize] = dominant[pick];
                }
            }

            for &k in &touched {
                label_weight[k as usize] = 0.0;
            }
        }

        if !retention {
            control_iteration = !control_iteration;
        }
    }
}

// ============================================================================
//                                Finalize
// ============================================================================

/// Compact labels to `[0, k)`, then BFS-fill any remaining unlabelled
/// vertices (each unlabelled connected component receives its own fresh
/// label). Returns `k`.
fn finalize_labels(
    graph: &Graph,
    membership: &mut [i32],
    fixed: Option<&[bool]>,
    rng: &mut SplitMix64,
) -> IgraphResult<u32> {
    let n = membership.len();

    // First pass: dense relabel via a sparse map from "original label"
    // to "compacted label". `relabel[k] == -1` marks "not seen yet".
    let mut max_seen: i32 = -1;
    for &k in membership.iter() {
        if k > max_seen {
            max_seen = k;
        }
    }
    let span = max_seen.max(-1) + 1;
    let mut relabel: Vec<i32> = vec![-1; span.max(0) as usize];

    let mut next_label: i32 = 0;
    let mut unlabelled_left = false;
    for v in 0..n {
        let k = membership[v];
        if k >= 0 {
            let slot = k as usize;
            if relabel[slot] == -1 {
                relabel[slot] = next_label;
                membership[v] = next_label;
                next_label += 1;
            } else {
                membership[v] = relabel[slot];
            }
        } else {
            unlabelled_left = true;
        }
    }

    if unlabelled_left {
        // BFS-fill: every unlabelled connected component gets its own
        // new label. Skip fixed vertices in the seed-iteration order
        // (mirrors upstream).
        let mut order: Vec<u32> = match fixed {
            Some(mask) => (0..n as u32).filter(|&v| !mask[v as usize]).collect(),
            None => (0..n as u32).collect(),
        };
        shuffle_in_place(&mut order, rng);

        let mut bfs_queue: VecDeque<u32> = VecDeque::new();
        for &seed in &order {
            if membership[seed as usize] >= 0 {
                continue;
            }
            let label = next_label;
            next_label += 1;
            membership[seed as usize] = label;
            bfs_queue.push_back(seed);
            while let Some(v) = bfs_queue.pop_front() {
                let neighbours = graph.neighbors(v)?;
                for n_v in neighbours {
                    if membership[n_v as usize] < 0 {
                        membership[n_v as usize] = label;
                        bfs_queue.push_back(n_v);
                    }
                }
            }
        }
    }

    Ok(next_label as u32)
}

// ============================================================================
//                              Weights validation
// ============================================================================

fn validate_weights(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<()> {
    let Some(w) = weights else {
        return Ok(());
    };
    let m = graph.ecount();
    if w.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "weights length ({}) differs from edge count ({m})",
            w.len()
        )));
    }
    for (e, &v) in w.iter().enumerate() {
        if !v.is_finite() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is not finite ({v})"
            )));
        }
        if v < 0.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is negative ({v}); label_propagation requires non-negative weights"
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
        usize::try_from(r).expect("bound fits in usize by construction")
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

    fn assert_dense_labels(r: &LpaResult, n: usize) {
        assert_eq!(r.membership.len(), n);
        if n == 0 {
            assert_eq!(r.nb_clusters, 0);
            return;
        }
        let max = *r.membership.iter().max().unwrap_or(&0);
        assert!((max as usize) < n);
        assert_eq!(max + 1, r.nb_clusters);
        let mut seen = vec![false; r.nb_clusters as usize];
        for &m in &r.membership {
            seen[m as usize] = true;
        }
        assert!(seen.into_iter().all(|b| b));
    }

    #[test]
    fn empty_graph_returns_empty_membership() {
        let g = Graph::with_vertices(0);
        let r = label_propagation(&g).unwrap();
        assert_eq!(r.membership.len(), 0);
        assert_eq!(r.nb_clusters, 0);
    }

    #[test]
    fn isolated_vertices_each_get_own_label() {
        let g = Graph::with_vertices(4);
        let r = label_propagation(&g).unwrap();
        assert_dense_labels(&r, 4);
        assert_eq!(r.nb_clusters, 4);
    }

    #[test]
    fn two_triangles_bridge_split() {
        let mut g = Graph::with_vertices(6);
        add_edges(
            &mut g,
            &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
        );
        let r = label_propagation(&g).unwrap();
        // Triangles should be intact; the partition itself can vary by
        // seed but each clique must stay together.
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[0], r.membership[2]);
        assert_eq!(r.membership[3], r.membership[4]);
        assert_eq!(r.membership[3], r.membership[5]);
        assert_dense_labels(&r, 6);
    }

    #[test]
    fn directed_input_rejected() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(label_propagation(&g).is_err());
        assert!(label_propagation_weighted(&g, &[1.0]).is_err());
        assert!(label_propagation_with_options(&g, None, &LpaOptions::default()).is_err());
    }

    #[test]
    fn weight_validation() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // wrong length
        assert!(label_propagation_weighted(&g, &[1.0]).is_err());
        // NaN
        assert!(label_propagation_weighted(&g, &[1.0, f64::NAN]).is_err());
        // negative
        assert!(label_propagation_weighted(&g, &[1.0, -0.1]).is_err());
    }

    #[test]
    fn determinism_under_seed_fast() {
        let mut g = Graph::with_vertices(8);
        for &(u, v) in &[
            (0, 1),
            (0, 2),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (4, 6),
            (5, 6),
            (5, 7),
        ] {
            g.add_edge(u, v).unwrap();
        }
        let opts = LpaOptions {
            seed: 12345,
            ..LpaOptions::default()
        };
        let a = label_propagation_with_options(&g, None, &opts).unwrap();
        let b = label_propagation_with_options(&g, None, &opts).unwrap();
        assert_eq!(a.membership, b.membership);
    }

    #[test]
    fn determinism_under_seed_dominance() {
        let mut g = Graph::with_vertices(6);
        add_edges(
            &mut g,
            &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
        );
        let opts = LpaOptions {
            variant: LpaVariant::Dominance,
            seed: 99,
            ..LpaOptions::default()
        };
        let a = label_propagation_with_options(&g, None, &opts).unwrap();
        let b = label_propagation_with_options(&g, None, &opts).unwrap();
        assert_eq!(a.membership, b.membership);
    }

    #[test]
    fn determinism_under_seed_retention() {
        let mut g = Graph::with_vertices(6);
        add_edges(
            &mut g,
            &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
        );
        let opts = LpaOptions {
            variant: LpaVariant::Retention,
            seed: 7,
            ..LpaOptions::default()
        };
        let a = label_propagation_with_options(&g, None, &opts).unwrap();
        let b = label_propagation_with_options(&g, None, &opts).unwrap();
        assert_eq!(a.membership, b.membership);
    }

    #[test]
    fn unit_weights_match_unweighted() {
        let mut g = Graph::with_vertices(6);
        add_edges(
            &mut g,
            &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
        );
        let opts = LpaOptions {
            seed: 42,
            ..LpaOptions::default()
        };
        let a = label_propagation_with_options(&g, None, &opts).unwrap();
        let ones = vec![1.0; g.ecount()];
        let b = label_propagation_with_options(&g, Some(&ones), &opts).unwrap();
        assert_eq!(a.membership, b.membership);
    }

    #[test]
    fn initial_validation() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        // wrong length
        let opts = LpaOptions {
            initial: Some(vec![0, 1]),
            ..LpaOptions::default()
        };
        assert!(label_propagation_with_options(&g, None, &opts).is_err());
        // label out of range
        let opts = LpaOptions {
            initial: Some(vec![0, 1, 99]),
            ..LpaOptions::default()
        };
        assert!(label_propagation_with_options(&g, None, &opts).is_err());
    }

    #[test]
    fn fixed_validation() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        // wrong length
        let opts = LpaOptions {
            initial: Some(vec![0, 1, 2]),
            fixed: Some(vec![false, false]),
            ..LpaOptions::default()
        };
        assert!(label_propagation_with_options(&g, None, &opts).is_err());
    }

    #[test]
    fn fixed_vertices_keep_their_labels() {
        // K3 with labels [7, 7, 9]; fix all three.
        let mut g = Graph::with_vertices(3);
        add_edges(&mut g, &[(0, 1), (0, 2), (1, 2)]);
        let opts = LpaOptions {
            initial: Some(vec![0, 0, 1]),
            fixed: Some(vec![true, true, true]),
            ..LpaOptions::default()
        };
        let r = label_propagation_with_options(&g, None, &opts).unwrap();
        assert_eq!(r.membership.len(), 3);
        // Labels are compacted, but the co-membership relation must
        // be preserved: vertex 0 == 1, vertex 2 ≠ them.
        assert_eq!(r.membership[0], r.membership[1]);
        assert_ne!(r.membership[0], r.membership[2]);
    }

    #[test]
    fn unlabelled_component_gets_fresh_label() {
        // Three disconnected K2s. Start with one labelled, two not.
        let mut g = Graph::with_vertices(6);
        add_edges(&mut g, &[(0, 1), (2, 3), (4, 5)]);
        let opts = LpaOptions {
            initial: Some(vec![0, 0, -1, -1, -1, -1]),
            ..LpaOptions::default()
        };
        let r = label_propagation_with_options(&g, None, &opts).unwrap();
        // K2 0-1 keeps the original label; the other two components
        // get fresh labels each. Total 3 communities.
        assert_eq!(r.nb_clusters, 3);
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[2], r.membership[3]);
        assert_eq!(r.membership[4], r.membership[5]);
    }

    #[test]
    fn ignore_fixed_when_no_initial() {
        // fixed is silently ignored if initial is None — matches
        // upstream warning behaviour.
        let mut g = Graph::with_vertices(3);
        add_edges(&mut g, &[(0, 1), (1, 2)]);
        let opts = LpaOptions {
            fixed: Some(vec![true, true, true]),
            ..LpaOptions::default()
        };
        // Should run successfully and treat all vertices as updatable.
        let r = label_propagation_with_options(&g, None, &opts).unwrap();
        assert_eq!(r.membership.len(), 3);
    }

    #[test]
    fn self_loops_handled() {
        // Self-loops count once toward the vertex's own label (matches
        // IGRAPH_LOOPS_ONCE). The result must still be a well-formed
        // partition.
        let mut g = Graph::with_vertices(3);
        add_edges(&mut g, &[(0, 0), (0, 1), (1, 2)]);
        let r = label_propagation(&g).unwrap();
        assert_dense_labels(&r, 3);
    }

    #[test]
    fn three_variants_all_run_on_karate_size_input() {
        // A bigger graph: 10 vertices in two K5s + bridge.
        let mut g = Graph::with_vertices(10);
        for c in 0..2 {
            let base = c * 5;
            for u in 0..5 {
                for v in (u + 1)..5 {
                    g.add_edge(base + u, base + v).unwrap();
                }
            }
        }
        g.add_edge(4, 5).unwrap();

        for variant in [
            LpaVariant::Fast,
            LpaVariant::Dominance,
            LpaVariant::Retention,
        ] {
            let opts = LpaOptions {
                variant,
                seed: 0,
                ..LpaOptions::default()
            };
            let r = label_propagation_with_options(&g, None, &opts).unwrap();
            assert_dense_labels(&r, 10);
            // Each K5 internally must stay together.
            for u in 0..5 {
                assert_eq!(r.membership[u], r.membership[0]);
                assert_eq!(r.membership[u + 5], r.membership[5]);
            }
        }
    }
}
