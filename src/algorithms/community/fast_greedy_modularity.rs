//! Fast greedy modularity community detection (ALGO-CO-007).
//!
//! Counterpart of `igraph_community_fastgreedy()` from
//! `references/igraph/src/community/fast_modularity.c`.
//!
//! Clauset-Newman-Moore (2004): start with every vertex its own
//! community, then greedily merge the community-pair whose merger
//! yields the largest improvement in Newman-Girvan modularity. The
//! Wakita-Tsurumi (2007) data-structure trick maintains a global
//! max-heap over per-community max-`ΔQ` so each merge runs in
//! `O((|N(to)| + |N(from)|) log V)`.
//!
//! References:
//! - A. Clauset, M. E. J. Newman, C. Moore. *Finding community
//!   structure in very large networks*, Phys. Rev. E 70 (2004).
//!   <https://doi.org/10.1103/PhysRevE.70.066111>
//! - K. Wakita, T. Tsurumi. *Finding community structure in mega-scale
//!   social networks*, arXiv:cs/0702048.
//!
//! Phase-1 scope: **undirected**, optionally weighted with non-negative
//! finite weights; rejects multi-edges (matches the upstream constraint)
//! and directed graphs. Self-loops are accepted and contribute to the
//! `loop_weight_sum` term, exactly as in C.
//!
//! Per-pair `ΔQ` update rules on each merge of `from → to`:
//! - **Triangle** (both `to` and `from` already know `k`): the new
//!   pair `(to, k)` has `ΔQ' = ΔQ(to, k) + ΔQ(from, k)`.
//! - **Chain case 1** (only `to` knows `k`): `ΔQ' = ΔQ(to, k) -
//!   2 · a[from] · a[k]` (the old `from`-`k` non-edge is now part of
//!   `to`'s neighbourhood with implicit edge weight 0).
//! - **Chain case 2** (only `from` knows `k`): `ΔQ' = ΔQ(from, k) -
//!   2 · a[to] · a[k]`.
//!
//! `a[c]` is the fraction of total edge weight incident to community
//! `c`. The running modularity `q` is updated as `q += ΔQ` on each
//! accepted merge. The C drives the merge to completion (full
//! dendrogram of `n-1` rows) and reports the partition at the highest
//! Q-step; we do the same.
//!
//! Complexity: `O(|E| · log V)` per merge × up to `|V|` merges, so
//! `O(|V| · |E| · log V)` worst case — matches the C reference.

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

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use crate::algorithms::properties::multiplicity::has_multiple;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of [`fast_greedy_modularity`] / [`fast_greedy_modularity_weighted`].
#[derive(Debug, Clone)]
pub struct FastGreedyResult {
    /// Per-vertex community label of the best-modularity partition,
    /// densified to `0..nb_clusters`.
    pub membership: Vec<u32>,
    /// Number of distinct communities in `membership`.
    pub nb_clusters: u32,
    /// Merges in dendrogram order. Each row `[c1, c2]` merges clusters
    /// `c1` and `c2` into the new cluster `n + i` where `i` is the
    /// merge index. Same encoding as
    /// `igraph_community_fastgreedy()` / Walktrap / EB.
    pub merges: Vec<[u32; 2]>,
    /// Modularity trajectory. `modularity[i]` is the modularity *after*
    /// `i` merges have been applied to the all-singletons start
    /// partition. Length = `merges.len() + 1`. For an edgeless graph
    /// the single entry is `NaN`, matching the C convention.
    pub modularity: Vec<f64>,
}

/// Run fast greedy modularity community detection on `graph` (unweighted).
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph.is_directed()`.
/// - [`IgraphError::InvalidArgument`] if the graph has multi-edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, fast_greedy_modularity};
///
/// // Two K5 cliques joined by a single bridge edge (0,5).
/// let mut g = Graph::with_vertices(10);
/// for &(u, v) in &[
///     (0, 1), (0, 2), (0, 3), (0, 4), (1, 2), (1, 3), (1, 4),
///     (2, 3), (2, 4), (3, 4),
///     (5, 6), (5, 7), (5, 8), (5, 9), (6, 7), (6, 8), (6, 9),
///     (7, 8), (7, 9), (8, 9),
///     (0, 5),
/// ] {
///     g.add_edge(u, v).unwrap();
/// }
/// let r = fast_greedy_modularity(&g).unwrap();
/// assert_eq!(r.nb_clusters, 2);
/// assert!((r.modularity.iter().copied().fold(f64::NEG_INFINITY, f64::max) - 0.452_381).abs() < 1e-5);
/// ```
pub fn fast_greedy_modularity(graph: &Graph) -> IgraphResult<FastGreedyResult> {
    fast_greedy_modularity_impl(graph, None)
}

/// Run fast greedy modularity community detection on `graph` with edge `weights`.
///
/// `weights[i]` is the weight of edge id `i`; length must equal
/// `graph.ecount()`. Weights must be finite and non-negative.
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph.is_directed()`.
/// - [`IgraphError::InvalidArgument`] if the graph has multi-edges,
///   `weights.len() != graph.ecount()`, or any weight is negative / NaN.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, fast_greedy_modularity_weighted};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let r = fast_greedy_modularity_weighted(&g, &[1.0, 2.0, 1.0]).unwrap();
/// assert!(r.nb_clusters >= 1);
/// ```
pub fn fast_greedy_modularity_weighted(
    graph: &Graph,
    weights: &[f64],
) -> IgraphResult<FastGreedyResult> {
    fast_greedy_modularity_impl(graph, Some(weights))
}

fn fast_greedy_modularity_impl(
    graph: &Graph,
    weights: Option<&[f64]>,
) -> IgraphResult<FastGreedyResult> {
    if graph.is_directed() {
        return Err(IgraphError::Unsupported(
            "fast_greedy_modularity is undirected-only; directed variant is a follow-up AWU",
        ));
    }
    let n = graph.vcount();
    let n_us = n as usize;
    let m_us = graph.ecount();

    if let Some(w) = weights {
        if w.len() != m_us {
            return Err(IgraphError::InvalidArgument(
                "weights length must equal graph.ecount()".into(),
            ));
        }
        for &x in w {
            if x.is_nan() {
                return Err(IgraphError::InvalidArgument(
                    "weights must not be NaN".into(),
                ));
            }
            if x < 0.0 {
                return Err(IgraphError::InvalidArgument(
                    "weights must be non-negative".into(),
                ));
            }
        }
    }
    if has_multiple(graph)? {
        return Err(IgraphError::InvalidArgument(
            "fast_greedy_modularity requires graphs without multi-edges".into(),
        ));
    }

    if n == 0 {
        return Ok(FastGreedyResult {
            membership: Vec::new(),
            nb_clusters: 0,
            merges: Vec::new(),
            modularity: Vec::new(),
        });
    }

    // a[c] starts as raw incident strength; normalised by 2W below.
    let mut a = vec![0.0_f64; n_us];
    let mut loop_weight_sum = 0.0_f64;
    let mut weight_sum = 0.0_f64;

    for e in 0..m_us {
        let (u, v) = graph.edge(e as u32)?;
        let w = match weights {
            Some(ws) => ws[e],
            None => 1.0,
        };
        weight_sum += w;
        a[u as usize] += w;
        a[v as usize] += w;
        if u == v {
            loop_weight_sum += 2.0 * w;
        }
    }

    if m_us == 0 {
        // No edges: all singletons; Q undefined → NaN (matches C).
        return Ok(FastGreedyResult {
            membership: (0..n).collect(),
            nb_clusters: n,
            merges: Vec::new(),
            modularity: vec![f64::NAN],
        });
    }

    let two_w = 2.0 * weight_sum;
    // a[i] /= 2W
    if two_w > 0.0 {
        for slot in &mut a {
            *slot /= two_w;
        }
    }

    // Per-community sorted neighbour map (neighbour id -> ΔQ).
    let mut neis: Vec<BTreeMap<u32, f64>> = vec![BTreeMap::new(); n_us];

    // Initial ΔQ for each non-loop edge.
    for e in 0..m_us {
        let (u, v) = graph.edge(e as u32)?;
        if u == v {
            continue;
        }
        let w = match weights {
            Some(ws) => ws[e],
            None => 1.0,
        };
        // dq = 2 * (w / 2W - a[u] * a[v])   (after a has been normalised by 2W)
        let dq = 2.0 * (w / two_w - a[u as usize] * a[v as usize]);
        // No multi-edges by precondition: assert by inserting.
        neis[u as usize].insert(v, dq);
        neis[v as usize].insert(u, dq);
    }

    // Initial modularity: loop_weight_sum / 2W - sum(a^2).
    let mut q = if two_w > 0.0 {
        let mut s = loop_weight_sum / two_w;
        for &ai in &a {
            s -= ai * ai;
        }
        s
    } else {
        0.0
    };

    // Build heap of (dq, c1, c2) for each canonical (c1 < c2) live edge.
    let mut heap: BinaryHeap<HeapEntry> = BinaryHeap::new();
    for c in 0..n_us {
        for (&k, &dq) in &neis[c] {
            if (c as u32) < k {
                heap.push(HeapEntry {
                    dq,
                    c1: c as u32,
                    c2: k,
                });
            }
        }
    }

    let mut alive = vec![true; n_us];
    let mut size = vec![1_u32; n_us];
    let mut id: Vec<u32> = (0..n).collect();

    let total_merges_cap = n_us.saturating_sub(1);
    let mut merges: Vec<[u32; 2]> = Vec::with_capacity(total_merges_cap);
    let mut modularity_traj: Vec<f64> = Vec::with_capacity(total_merges_cap + 1);
    modularity_traj.push(q);

    let mut best_q = q;
    let mut best_n_merges = 0_usize;

    while merges.len() < total_merges_cap {
        // Pop until we find a non-stale entry.
        let chosen = loop {
            let Some(e) = heap.pop() else {
                break None;
            };
            if !alive[e.c1 as usize] || !alive[e.c2 as usize] {
                continue;
            }
            // Validate the stored dq still matches.
            if let Some(&cur) = neis[e.c1 as usize].get(&e.c2) {
                if cur == e.dq {
                    break Some(e);
                }
            }
        };
        let Some(entry) = chosen else {
            // No more inter-community edges (disconnected components remain).
            break;
        };

        // Choose `to` = smaller id, `from` = larger id. This matches the
        // common convention; the dendrogram encoding stores [to, from].
        let (to, from) = if entry.c1 < entry.c2 {
            (entry.c1, entry.c2)
        } else {
            (entry.c2, entry.c1)
        };

        q += entry.dq;

        // Take both neighbour maps out so we can iterate freely without
        // overlapping borrows.
        let to_neis = std::mem::take(&mut neis[to as usize]);
        let from_neis = std::mem::take(&mut neis[from as usize]);

        // Collect the union of neighbour keys, skipping the two endpoints.
        let mut all_keys: BTreeSet<u32> = BTreeSet::new();
        for &k in to_neis.keys() {
            if k != from {
                all_keys.insert(k);
            }
        }
        for &k in from_neis.keys() {
            if k != to {
                all_keys.insert(k);
            }
        }

        let mut new_to_neis: BTreeMap<u32, f64> = BTreeMap::new();
        for k in all_keys {
            let in_to = to_neis.get(&k).copied();
            let in_from = from_neis.get(&k).copied();
            let new_dq = match (in_to, in_from) {
                (Some(t), Some(f)) => t + f, // triangle
                (Some(t), None) => t - 2.0 * a[from as usize] * a[k as usize], // chain 1
                (None, Some(f)) => f - 2.0 * a[to as usize] * a[k as usize], // chain 2
                (None, None) => unreachable!(),
            };
            new_to_neis.insert(k, new_dq);

            // Mirror in k's map: drop `from`, set `to` -> new_dq.
            let km = &mut neis[k as usize];
            km.remove(&from);
            km.insert(to, new_dq);

            // Push the updated edge onto the heap in canonical (c1<c2) order.
            let (c1, c2) = if to < k { (to, k) } else { (k, to) };
            heap.push(HeapEntry { dq: new_dq, c1, c2 });
        }
        neis[to as usize] = new_to_neis;

        alive[from as usize] = false;
        size[to as usize] += size[from as usize];
        a[to as usize] += a[from as usize];
        a[from as usize] = 0.0;

        merges.push([id[to as usize], id[from as usize]]);
        id[to as usize] = u32::try_from(n_us)
            .map_err(|_| IgraphError::Internal("vcount exceeds u32::MAX"))?
            + u32::try_from(merges.len() - 1)
                .map_err(|_| IgraphError::Internal("merges.len exceeds u32::MAX"))?;

        modularity_traj.push(q);

        if q > best_q {
            best_q = q;
            best_n_merges = merges.len();
        }
    }

    // Build membership by applying the first best_n_merges merges to a
    // singleton labelling. label[v] = current cluster label.
    let mut label: Vec<u32> = (0..n).collect();
    // Walk merges in chronological order; each merge produces a fresh
    // cluster id n+i. Rewrite all vertices whose current label matches
    // either side of the merge to the new id.
    for (i, m) in merges.iter().take(best_n_merges).enumerate() {
        let [c_a, c_b] = *m;
        let new_id = n + u32::try_from(i)
            .map_err(|_| IgraphError::Internal("merge index exceeds u32::MAX"))?;
        for slot in &mut label {
            if *slot == c_a || *slot == c_b {
                *slot = new_id;
            }
        }
    }

    let (membership, nb_clusters) = densify_labels(&label);

    Ok(FastGreedyResult {
        membership,
        nb_clusters,
        merges,
        modularity: modularity_traj,
    })
}

fn densify_labels(labels: &[u32]) -> (Vec<u32>, u32) {
    let mut remap: Vec<(u32, u32)> = Vec::new();
    let mut out = Vec::with_capacity(labels.len());
    for &l in labels {
        let dense = if let Some(&(_, d)) = remap.iter().find(|&&(orig, _)| orig == l) {
            d
        } else {
            let d = u32::try_from(remap.len()).unwrap_or(u32::MAX);
            remap.push((l, d));
            d
        };
        out.push(dense);
    }
    let k = u32::try_from(remap.len()).unwrap_or(u32::MAX);
    (out, k)
}

/// Heap entry: max-heap on `dq`, ties broken by smaller `(c1, c2)`
/// winning, so the greedy choice is deterministic across runs.
#[derive(Debug, Clone, Copy)]
struct HeapEntry {
    dq: f64,
    c1: u32,
    c2: u32,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.dq.to_bits() == other.dq.to_bits() && self.c1 == other.c1 && self.c2 == other.c2
    }
}
impl Eq for HeapEntry {}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // primary: higher dq is "greater" (max-heap).
        match self.dq.partial_cmp(&other.dq) {
            Some(Ordering::Equal) | None => {
                // tie: smaller (c1, c2) is "greater" so pops first.
                other.c1.cmp(&self.c1).then_with(|| other.c2.cmp(&self.c2))
            }
            Some(o) => o,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_k5_bridge() -> Graph {
        let mut g = Graph::with_vertices(10);
        for u in 0..5u32 {
            for v in (u + 1)..5 {
                g.add_edge(u, v).expect("clique edge");
            }
        }
        for u in 5..10u32 {
            for v in (u + 1)..10 {
                g.add_edge(u, v).expect("clique edge");
            }
        }
        g.add_edge(0, 5).expect("bridge");
        g
    }

    #[test]
    fn empty_graph_returns_empty() {
        let g = Graph::with_vertices(0);
        let r = fast_greedy_modularity(&g).unwrap();
        assert!(r.membership.is_empty());
        assert_eq!(r.nb_clusters, 0);
        assert!(r.merges.is_empty());
        assert!(r.modularity.is_empty());
    }

    #[test]
    fn edgeless_graph_yields_singletons_with_nan() {
        let g = Graph::with_vertices(5);
        let r = fast_greedy_modularity(&g).unwrap();
        assert_eq!(r.membership, vec![0, 1, 2, 3, 4]);
        assert_eq!(r.nb_clusters, 5);
        assert!(r.merges.is_empty());
        assert_eq!(r.modularity.len(), 1);
        assert!(r.modularity[0].is_nan());
    }

    #[test]
    fn single_vertex_no_edges() {
        let g = Graph::with_vertices(1);
        let r = fast_greedy_modularity(&g).unwrap();
        assert_eq!(r.membership, vec![0]);
        assert_eq!(r.nb_clusters, 1);
        assert!(r.merges.is_empty());
    }

    #[test]
    fn two_k5_bridge_q_matches_upstream() {
        let g = two_k5_bridge();
        let r = fast_greedy_modularity(&g).unwrap();
        let best_q = r
            .modularity
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            (best_q - 0.452_381).abs() < 1e-5,
            "best Q = {best_q}, expected ≈ 0.452381"
        );
        assert_eq!(r.nb_clusters, 2);
        // Vertices 0..4 share one community, 5..9 the other.
        for v in 1..5 {
            assert_eq!(r.membership[v], r.membership[0]);
        }
        for v in 6..10 {
            assert_eq!(r.membership[v], r.membership[5]);
        }
        assert_ne!(r.membership[0], r.membership[5]);
    }

    #[test]
    fn two_k5_bridge_with_uniform_weights_matches_unweighted() {
        let g = two_k5_bridge();
        let weights = vec![2.0_f64; g.ecount()];
        let r = fast_greedy_modularity_weighted(&g, &weights).unwrap();
        let best_q = r
            .modularity
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((best_q - 0.452_381).abs() < 1e-5);
        assert_eq!(r.nb_clusters, 2);
    }

    #[test]
    fn dendrogram_size_bounded_by_n_minus_1() {
        let g = two_k5_bridge();
        let r = fast_greedy_modularity(&g).unwrap();
        assert!(r.merges.len() <= 9);
        assert_eq!(r.modularity.len(), r.merges.len() + 1);
    }

    #[test]
    fn two_disjoint_k4_yields_two_components() {
        let mut g = Graph::with_vertices(8);
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
        let r = fast_greedy_modularity(&g).unwrap();
        // Disconnected → algorithm stops before crossing components.
        assert_eq!(r.nb_clusters, 2);
        for v in 1..4 {
            assert_eq!(r.membership[v], r.membership[0]);
        }
        for v in 5..8 {
            assert_eq!(r.membership[v], r.membership[4]);
        }
        assert_ne!(r.membership[0], r.membership[4]);
    }

    #[test]
    fn rejects_directed_graph() {
        let mut g = Graph::new(3, true).expect("3v directed graph");
        g.add_edge(0, 1).expect("0-1");
        let err = fast_greedy_modularity(&g).unwrap_err();
        assert!(matches!(err, IgraphError::Unsupported(_)));
    }

    #[test]
    fn rejects_multi_edges() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let err = fast_greedy_modularity(&g).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rejects_negative_weight() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let err = fast_greedy_modularity_weighted(&g, &[1.0, -0.5]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rejects_weight_length_mismatch() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let err = fast_greedy_modularity_weighted(&g, &[1.0]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn densify_labels_basic() {
        let (m, k) = densify_labels(&[5, 5, 7, 5, 9, 7]);
        assert_eq!(m, vec![0, 0, 1, 0, 2, 1]);
        assert_eq!(k, 3);
    }
}
