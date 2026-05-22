//! Edge-betweenness community detection (ALGO-CO-006).
//!
//! Counterpart of `igraph_community_edge_betweenness()` from
//! `references/igraph/src/community/edge_betweenness.c`.
//!
//! Girvan-Newman (2002): iteratively remove the edge with the highest
//! current betweenness; the order in which removals split the graph is
//! a binary dendrogram whose best cut (by modularity) yields a
//! community partition.
//!
//! References:
//! - M. Girvan, M. E. J. Newman. *Community Structure in Social and
//!   Biological Networks*, PNAS 99, 7821 (2002).
//!   <https://doi.org/10.1073/pnas.122653799>
//! - M. E. J. Newman. *Analysis of Weighted Networks*,
//!   Phys. Rev. E 70 (2004). <https://doi.org/10.1103/PhysRevE.70.056131>
//!
//! Handles **unweighted, undirected + directed** graphs. The weighted
//! sibling is in `edge_betweenness_community_weighted.rs` (CO-006b for
//! undirected, CO-006c for directed). Length-aware (separate length
//! vector) remains a future AWU.
//!
//! Directed handling (CO-006c) follows the C reference:
//! - traversal uses the OUT-incidence list (`Graph::incident`), back-edge
//!   dependency accumulation uses the IN-incidence list (`Graph::incident_in`);
//! - `edge_betweenness[i]` is **not** halved for directed (per the upstream
//!   centrality convention — `if (!directed) eb /= 2.0;`);
//! - per-level modularity uses `modularity_directed` (Leicht-Newman 2008)
//!   so the best-Q cut reflects the directed adjacency.
//!
//! Pipeline mirrors the upstream loop:
//! 1. Build a private mutable incidence list (one `Vec<EdgeId>` per
//!    vertex). Edge IDs from the original graph stay valid — we only
//!    mask removed edges instead of mutating the graph.
//! 2. Repeat `m` times: run the Brandes unweighted edge-betweenness
//!    pass over the **active** edges only, pick the edge with the
//!    largest betweenness (ties broken by smallest id, matching the
//!    upstream `igraph_i_which_max_active_ratio` tie behaviour), record
//!    it in `removed_edges`, mark it passive, and pop it from both
//!    endpoints' incidence lists.
//! 3. Replay the removals in reverse to build the merge dendrogram.
//!    Each removal that re-joins two distinct components is a *merge*;
//!    cumulative modularity is evaluated after every merge and the
//!    best-Q membership is reported.
//!
//! Complexity: `O(|V| * |E|^2)` — the Brandes pass per removal is the
//! dominant cost. Matches the C reference.

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

use std::collections::VecDeque;

use crate::algorithms::community::modularity::{modularity, modularity_directed};
use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Result of [`edge_betweenness_community`].
#[derive(Debug, Clone)]
pub struct EdgeBetweennessResult {
    /// Per-vertex community label of the **best** partition along the
    /// dendrogram (the one maximising modularity). Labels are densely
    /// numbered in `0..nb_clusters`.
    pub membership: Vec<u32>,
    /// Number of distinct communities in `membership`.
    pub nb_clusters: u32,
    /// Edge IDs in the order they were removed (length = `ecount`).
    /// Suitable as input to a separate dendrogram replay if a caller
    /// wants to recompute partitions at other cut points.
    pub removed_edges: Vec<EdgeId>,
    /// Betweenness of each removed edge **at the moment of removal**
    /// (same length and order as [`removed_edges`](Self::removed_edges)).
    /// Halved for undirected graphs to match the centrality convention.
    /// For directed graphs this is left un-halved, matching the upstream
    /// `if (!directed) eb /= 2.0;` rule.
    pub edge_betweenness: Vec<f64>,
    /// Merges in dendrogram order. Each row `[c1, c2]` means clusters
    /// `c1` and `c2` are merged into the new cluster `n + i` (where
    /// `i` is the merge index and `n` is `vcount`). Same encoding as
    /// `igraph_community_eb_get_merges()` / Walktrap.
    pub merges: Vec<[u32; 2]>,
    /// `bridges[i]` is the index into [`removed_edges`](Self::removed_edges)
    /// of the edge whose removal triggered the *i*-th merge (i.e. the
    /// edge that disconnected the network into one more component).
    /// Reverse-order count: equal to the number of merges.
    pub bridges: Vec<u32>,
    /// Modularity at each level of the dendrogram. `modularity[0]` is
    /// the modularity of the all-singletons partition, then one entry
    /// per merge. Length = `merges.len() + 1`.
    pub modularity: Vec<f64>,
}

/// Run edge-betweenness community detection on `graph`.
///
/// Returns the partition with the highest modularity along the
/// Girvan-Newman dendrogram, plus the full removal history and merges
/// so callers can replay alternative cuts. Accepts both undirected and
/// directed graphs: the directed branch uses directed shortest paths
/// (OUT-incidence forward, IN-incidence backward) and directed
/// modularity (Leicht-Newman 2008) for the per-level Q.
///
/// # Errors
/// Returns `IgraphError` only for internal-consistency failures (edge
/// id overflow in the dendrogram replay, dangling adjacency entries).
/// Both graph orientations are supported.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_betweenness_community};
///
/// // Two K3 triangles bridged by a single edge (0-1-2)-(3-4-5).
/// // Girvan-Newman removes the bridge first, splitting into the two
/// // expected communities.
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let r = edge_betweenness_community(&g).unwrap();
/// assert_eq!(r.nb_clusters, 2);
/// assert_eq!(r.membership[0], r.membership[1]);
/// assert_eq!(r.membership[3], r.membership[5]);
/// assert_ne!(r.membership[0], r.membership[3]);
/// ```
pub fn edge_betweenness_community(graph: &Graph) -> IgraphResult<EdgeBetweennessResult> {
    let directed = graph.is_directed();
    let n = graph.vcount();
    let m_us = graph.ecount();
    let n_us = n as usize;

    // Null and edgeless graphs are well-defined: no merges, every vertex its
    // own community, no removal history.
    if n == 0 {
        return Ok(EdgeBetweennessResult {
            membership: Vec::new(),
            nb_clusters: 0,
            removed_edges: Vec::new(),
            edge_betweenness: Vec::new(),
            merges: Vec::new(),
            bridges: Vec::new(),
            modularity: Vec::new(),
        });
    }
    if m_us == 0 {
        // All singletons. Modularity is undefined (no edges); upstream
        // returns NaN — we surface 0.0 for the single trivial level so
        // downstream code can read `.modularity[0]` without flinching.
        return Ok(EdgeBetweennessResult {
            membership: (0..n).collect(),
            nb_clusters: n,
            removed_edges: Vec::new(),
            edge_betweenness: Vec::new(),
            merges: Vec::new(),
            bridges: Vec::new(),
            modularity: vec![0.0],
        });
    }

    // --- Stage 1: Girvan-Newman removal order ---
    //
    // Build private mutable incidence lists keyed on the original edge
    // ids. We never call `graph.delete_edges()`, so the original edge
    // ids stay valid throughout.
    //
    // Directed graphs need two lists: `inc_out` for the BFS forward pass
    // (`elist_out_p` in the C source) and `inc_in` for the backward
    // dependency-accumulation pass (`elist_in_p`). Undirected graphs use
    // a single list for both roles, exactly as the C aliases
    // `elist_out_p = elist_in_p = &elist_out`.
    let mut inc_out: Vec<Vec<EdgeId>> = (0..n)
        .map(|v| graph.incident(v))
        .collect::<IgraphResult<Vec<_>>>()?;
    let mut inc_in: Vec<Vec<EdgeId>> = if directed {
        (0..n)
            .map(|v| graph.incident_in(v))
            .collect::<IgraphResult<Vec<_>>>()?
    } else {
        Vec::new()
    };
    let mut passive: Vec<bool> = vec![false; m_us];

    let mut removed_edges: Vec<EdgeId> = Vec::with_capacity(m_us);
    let mut edge_betweenness_history: Vec<f64> = Vec::with_capacity(m_us);

    // Scratch buffers for the Brandes pass (reused across iterations).
    let mut sigma = vec![0.0_f64; n_us];
    let mut dist = vec![-1_i64; n_us];
    let mut pred: Vec<Vec<(VertexId, EdgeId)>> = vec![Vec::new(); n_us];
    let mut stack: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut delta_v = vec![0.0_f64; n_us];
    let mut eb_now = vec![0.0_f64; m_us];
    let mut queue: VecDeque<VertexId> = VecDeque::with_capacity(n_us);

    for _ in 0..m_us {
        // Reset per-iteration edge-betweenness scores. Passive edges
        // will simply never be incremented; the selection step skips
        // them explicitly.
        eb_now.fill(0.0);

        // Brandes over active edges.
        for s in 0..n {
            sigma.fill(0.0);
            dist.fill(-1);
            for slot in &mut pred {
                slot.clear();
            }
            stack.clear();
            delta_v.fill(0.0);
            queue.clear();

            sigma[s as usize] = 1.0;
            dist[s as usize] = 0;
            queue.push_back(s);

            while let Some(v) = queue.pop_front() {
                stack.push(v);
                // OUT-incidence forward (directed) or full incidence
                // (undirected). For directed edges, the neighbour is the
                // target `to`, not `edge_other`; for undirected, both
                // endpoints appear and `edge_other` returns the correct
                // one — see the C reference (`elist_out_p`) which uses
                // OUT-incidence with `IGRAPH_LOOPS_ONCE` for directed and
                // ALL-incidence with `IGRAPH_LOOPS_TWICE` for undirected.
                for &e in &inc_out[v as usize] {
                    let w = if directed {
                        let (_from, to) = graph.edge(e)?;
                        to
                    } else {
                        graph.edge_other(e, v)?
                    };
                    if dist[w as usize] < 0 {
                        queue.push_back(w);
                        dist[w as usize] = dist[v as usize] + 1;
                    }
                    if dist[w as usize] == dist[v as usize] + 1 {
                        sigma[w as usize] += sigma[v as usize];
                        pred[w as usize].push((v, e));
                    }
                }
            }

            while let Some(w) = stack.pop() {
                for &(v, e) in &pred[w as usize] {
                    let c = (sigma[v as usize] / sigma[w as usize]) * (1.0 + delta_v[w as usize]);
                    delta_v[v as usize] += c;
                    eb_now[e as usize] += c;
                }
            }
        }

        // Find the largest active betweenness. Ties broken by lowest
        // edge id (matches the upstream linear scan from the first
        // active index).
        let mut max_eid: Option<EdgeId> = None;
        let mut max_val = f64::NEG_INFINITY;
        for e in 0..m_us {
            if passive[e] {
                continue;
            }
            let val = eb_now[e];
            if val > max_val {
                max_val = val;
                max_eid = Some(e as EdgeId);
            }
        }
        let eid = max_eid.ok_or(IgraphError::Internal(
            "edge_betweenness_community: no active edge to remove",
        ))?;
        removed_edges.push(eid);
        // Undirected: every unordered pair was counted from both ends —
        // halve to match the centrality convention. Directed: keep the
        // raw count (matches the C `if (!directed) eb /= 2.0;` rule).
        edge_betweenness_history.push(if directed { max_val } else { max_val / 2.0 });
        passive[eid as usize] = true;

        // Detach the chosen edge from the incidence lists. Directed: pop
        // it from `inc_out[from]` and `inc_in[to]`. Undirected: pop from
        // both endpoints' `inc_out` (the only list). Self-loops appear
        // twice in undirected incidence; we strip every occurrence.
        let (from, to) = graph.edge(eid)?;
        if directed {
            inc_out[from as usize].retain(|&e| e != eid);
            inc_in[to as usize].retain(|&e| e != eid);
        } else {
            for endpoint in [from, to] {
                inc_out[endpoint as usize].retain(|&e| e != eid);
            }
        }
    }

    // --- Stage 2: replay merges + compute modularity per level ---

    // `parent[i]` (1-indexed slot) is the union-find pointer used by
    // the C `igraph_community_eb_get_merges` to walk to the cluster
    // root; we also track the canonical "current cluster id" per
    // vertex for the modularity-tracking variant.
    //
    // We need both `membership_now` (per-vertex current cluster id,
    // used to compute modularity at each step) and the dendrogram
    // pointer table.
    let mut membership_now: Vec<u32> = (0..n).collect();
    let mut merges: Vec<[u32; 2]> = Vec::new();
    let mut bridges: Vec<u32> = Vec::new();
    let mut modularity_levels: Vec<f64> = Vec::new();

    // Initial all-singletons modularity (always defined for m > 0).
    // Directed graphs use `modularity_directed` (Leicht-Newman 2008);
    // undirected fall through `modularity_directed` to `modularity`, but
    // we dispatch explicitly so the unweighted/undirected fast path
    // stays a single hop.
    let level_q = |mem: &[u32]| -> IgraphResult<f64> {
        let opt = if directed {
            modularity_directed(graph, mem, 1.0)?
        } else {
            modularity(graph, mem, 1.0)?
        };
        Ok(opt.unwrap_or(0.0))
    };
    let q0 = level_q(&membership_now)?;
    modularity_levels.push(q0);
    let mut max_mod = q0;
    let mut best_membership: Vec<u32> = membership_now.clone();

    // We need the cluster-id pointer too: vertex `v` currently sits in
    // cluster `find(v)`. The C code holds this implicitly via the
    // `mymembership` vector and rewrites it on each merge.
    for (step, &eid) in removed_edges.iter().enumerate().rev() {
        let (from, to) = graph.edge(eid)?;
        let c1 = membership_now[from as usize];
        let c2 = membership_now[to as usize];
        if c1 == c2 {
            continue;
        }

        // Record the merge: the new cluster gets id n + merge_index.
        let merge_index = merges.len();
        let new_cluster = n
            .checked_add(merge_index as u32)
            .ok_or(IgraphError::Internal(
                "edge_betweenness_community: merge index overflow",
            ))?;
        merges.push([c1, c2]);
        bridges.push(step as u32);

        // Re-label everyone in c1 or c2 to the new cluster id.
        for slot in &mut membership_now {
            if *slot == c1 || *slot == c2 {
                *slot = new_cluster;
            }
        }

        let q = level_q(&membership_now)?;
        modularity_levels.push(q);
        if q > max_mod {
            max_mod = q;
            best_membership.clone_from(&membership_now);
        }
    }

    // Densify the chosen partition's labels onto 0..nb_clusters.
    let (membership_dense, nb_clusters) = densify_labels(&best_membership);

    Ok(EdgeBetweennessResult {
        membership: membership_dense,
        nb_clusters,
        removed_edges,
        edge_betweenness: edge_betweenness_history,
        merges,
        bridges,
        modularity: modularity_levels,
    })
}

/// Reindex `labels` so the distinct values become `0..nb_clusters`,
/// preserving first-appearance order. Returns the dense membership and
/// the cluster count.
fn densify_labels(labels: &[u32]) -> (Vec<u32>, u32) {
    let mut remap: Vec<(u32, u32)> = Vec::new();
    let mut out: Vec<u32> = Vec::with_capacity(labels.len());
    for &lbl in labels {
        let dense = if let Some(&(_, d)) = remap.iter().find(|(orig, _)| *orig == lbl) {
            d
        } else {
            let d = remap.len() as u32;
            remap.push((lbl, d));
            d
        };
        out.push(dense);
    }
    let n_clusters = remap.len() as u32;
    (out, n_clusters)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn well_formed(r: &EdgeBetweennessResult, n: u32, m: usize) {
        assert_eq!(r.membership.len() as u32, n, "membership length");
        assert_eq!(r.removed_edges.len(), m, "removed_edges length");
        assert_eq!(r.edge_betweenness.len(), m, "history length");
        assert_eq!(r.merges.len(), r.bridges.len(), "merges/bridges");
        assert_eq!(
            r.modularity.len(),
            r.merges.len() + 1,
            "modularity = merges + 1"
        );
        for &lbl in &r.membership {
            assert!(lbl < r.nb_clusters, "dense label in range");
        }
    }

    #[test]
    fn empty_graph_returns_empty_result() {
        let g = Graph::with_vertices(0);
        let r = edge_betweenness_community(&g).unwrap();
        assert_eq!(r.nb_clusters, 0);
        assert!(r.removed_edges.is_empty());
        assert!(r.modularity.is_empty());
    }

    #[test]
    fn edgeless_graph_returns_singletons() {
        let g = Graph::with_vertices(4);
        let r = edge_betweenness_community(&g).unwrap();
        assert_eq!(r.nb_clusters, 4);
        for v in 0..4 {
            assert_eq!(r.membership[v as usize], v);
        }
        assert!(r.removed_edges.is_empty());
        assert_eq!(r.modularity, vec![0.0]);
    }

    #[test]
    fn two_triangles_bridge_splits_into_two() {
        let mut g = Graph::with_vertices(6);
        for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let r = edge_betweenness_community(&g).unwrap();
        well_formed(&r, 6, 7);
        assert_eq!(r.nb_clusters, 2);
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[1], r.membership[2]);
        assert_eq!(r.membership[3], r.membership[4]);
        assert_eq!(r.membership[4], r.membership[5]);
        assert_ne!(r.membership[0], r.membership[3]);
        // Bridge edge (2,3) carries the largest betweenness on iter 1,
        // so it is the first removal.
        let (from0, to0) = g.edge(r.removed_edges[0]).unwrap();
        assert!(
            (from0, to0) == (2, 3) || (from0, to0) == (3, 2),
            "first removed must be the bridge, got ({from0}, {to0})"
        );
    }

    #[test]
    fn path_4_splits_at_middle_edge() {
        // 0-1-2-3: edge (1,2) has the highest betweenness → first removal,
        // splitting into {0,1} and {2,3}. Best modularity should match.
        let mut g = Graph::with_vertices(4);
        for i in 0..3u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let r = edge_betweenness_community(&g).unwrap();
        well_formed(&r, 4, 3);
        let (from0, to0) = g.edge(r.removed_edges[0]).unwrap();
        assert!((from0, to0) == (1, 2) || (from0, to0) == (2, 1));
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[2], r.membership[3]);
        assert_ne!(r.membership[0], r.membership[2]);
    }

    #[test]
    fn triangle_modularity_levels_are_in_range() {
        // K3 has no good partition; modularity is bounded in [-1/2, 0].
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let r = edge_betweenness_community(&g).unwrap();
        well_formed(&r, 3, 3);
        for &q in &r.modularity {
            assert!((-0.501..=0.001).contains(&q), "modularity {q} out of range");
        }
    }

    #[test]
    fn two_components_already_split_no_bridges_between_them() {
        // 0-1 and 2-3-4 → already two components; the algorithm still
        // strips every edge, but the best partition matches the natural
        // split into {0,1} and {2,3,4}.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let r = edge_betweenness_community(&g).unwrap();
        well_formed(&r, 5, 3);
        assert!(r.nb_clusters >= 2);
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[2], r.membership[3]);
        assert_eq!(r.membership[3], r.membership[4]);
        assert_ne!(r.membership[0], r.membership[2]);
    }

    #[test]
    fn dendrogram_merges_at_most_n_minus_components() {
        // 4-cycle: 0-1-2-3-0. One component, so at most n-1 = 3 merges.
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        let r = edge_betweenness_community(&g).unwrap();
        assert!(r.merges.len() <= 3);
        well_formed(&r, 4, 4);
    }

    #[test]
    fn directed_path_6_middle_edge_first_split() {
        // Directed 6-path 0→1→2→3→4→5: edge (2,3) is uniquely the
        // highest-betweenness edge (it lies on 9 of the 15 reachable
        // (s,t) pairs vs. 8 for (1,2)/(3,4)). It is removed first and
        // splits into {0,1,2}|{3,4,5}. Per-level directed modularity
        // peaks at 8/25 = 0.32 there.
        let mut g = Graph::new(6, true).unwrap();
        for i in 0..5u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let r = edge_betweenness_community(&g).unwrap();
        well_formed(&r, 6, 5);
        let (from0, to0) = g.edge(r.removed_edges[0]).unwrap();
        assert_eq!((from0, to0), (2, 3), "bridge edge must be removed first");
        assert_eq!(r.nb_clusters, 2);
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[1], r.membership[2]);
        assert_eq!(r.membership[3], r.membership[4]);
        assert_eq!(r.membership[4], r.membership[5]);
        assert_ne!(r.membership[0], r.membership[3]);
        // Hand-checked best directed Q.
        let best = r
            .modularity
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            (best - 8.0 / 25.0).abs() < 1e-9,
            "expected best directed Q ≈ 8/25, got {best}"
        );
    }

    #[test]
    fn directed_two_triangles_bridge_runs_cleanly() {
        // Directed analogue of two-K3-bridge: 0→1→2→0 + 3→4→5→3 + 2→3.
        // The cycle structure produces a 3-way Brandes tie at the
        // central edges (1,2)/(2,3)/(3,4), so tie-breaking by lowest
        // edge id removes (1,2) first rather than the bridge — every
        // intermediate dendrogram level then has negative directed Q,
        // so the algorithm correctly picks the trivial all-one cluster.
        // Documented here so a future change in tie-breaking is caught.
        let mut g = Graph::new(6, true).unwrap();
        for &(u, v) in &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let r = edge_betweenness_community(&g).unwrap();
        well_formed(&r, 6, 7);
        for &q in &r.modularity {
            assert!(q.is_finite(), "directed modularity is finite");
            assert!((-1.0..=1.0).contains(&q), "directed Q in plausible range");
        }
    }

    #[test]
    fn directed_betweenness_is_not_halved() {
        // Directed path 0→1→2→3: edge (1,2) lies on the directed shortest
        // paths 0→{2,3} and 1→{2,3} → unhalved count is 4.0 (vs. 2.0 for
        // an undirected path, which would be halved).
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = edge_betweenness_community(&g).unwrap();
        well_formed(&r, 4, 3);
        // First removal is the middle edge with max betweenness.
        let (from0, to0) = g.edge(r.removed_edges[0]).unwrap();
        assert_eq!((from0, to0), (1, 2));
        // Unhalved Brandes count for (1,2) on this 4-path is 4
        // (4 source-target pairs route through it: 0→2, 0→3, 1→2, 1→3).
        assert!(
            (r.edge_betweenness[0] - 4.0).abs() < 1e-9,
            "expected unhalved eb=4.0, got {}",
            r.edge_betweenness[0]
        );
    }

    #[test]
    fn directed_disconnected_components_yield_singletons_or_components() {
        // 0→1 and 2→3→4 — two weakly connected components, no bridges
        // between them. The natural directed cut keeps each component.
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let r = edge_betweenness_community(&g).unwrap();
        well_formed(&r, 5, 3);
        assert!(r.nb_clusters >= 2);
        assert_ne!(r.membership[0], r.membership[2]);
    }

    #[test]
    fn densify_labels_preserves_first_appearance_order() {
        let (dense, n) = densify_labels(&[7, 7, 4, 4, 7, 9, 4, 9]);
        assert_eq!(n, 3);
        assert_eq!(dense, vec![0, 0, 1, 1, 0, 2, 1, 2]);
    }
}
