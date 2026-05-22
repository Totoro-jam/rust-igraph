//! Weighted edge-betweenness community detection (ALGO-CO-006b).
//!
//! Counterpart of `igraph_community_edge_betweenness(..., weights=&w, ...)`
//! from `references/igraph/src/community/edge_betweenness.c`.
//!
//! Same Girvan-Newman framework as the unweighted CO-006
//! (`edge_betweenness_community`): iteratively strip the edge with the
//! highest current betweenness, then replay removals in reverse to
//! build the binary dendrogram and surface the best-modularity
//! partition. The only differences against the unweighted slice:
//!
//! - Per-removal betweenness is computed via Brandes over Dijkstra
//!   shortest paths (`edge_betweenness_weighted` style) rather than the
//!   BFS shortest paths used in CO-006.
//! - Modularity at every dendrogram level uses [`modularity_weighted`]
//!   so the merge score reflects the weighted edge sums (`m =
//!   Σ_e w_e`).
//! - Weights must be non-negative + finite; weight vector length must
//!   equal `graph.ecount()`. Both constraints surface as
//!   `IgraphError::InvalidArgument`. Directed graphs are still
//!   unsupported (delegated to CO-006c, just like the unweighted
//!   slice).
//!
//! Complexity: `O(|V| * |E| * (|E| + |V| log |V|))` — the per-removal
//! Dijkstra-Brandes pass dominates.

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
use std::collections::BinaryHeap;

use crate::algorithms::community::edge_betweenness_community::EdgeBetweennessResult;
use crate::algorithms::community::modularity::modularity_weighted;
use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Min-heap entry. Reversed ordering so `BinaryHeap` (max-heap) pops
/// the smallest distance first. NaN / negative weights are rejected by
/// the entry point so `total_cmp` is safe.
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
        other.0.total_cmp(&self.0).then(other.1.cmp(&self.1))
    }
}
impl PartialOrd for Frontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Run weighted edge-betweenness community detection on `graph` with
/// per-edge `weights`.
///
/// Returns the same [`EdgeBetweennessResult`] shape as the unweighted
/// CO-006 entrypoint. `edge_betweenness[i]` is the **weighted**
/// betweenness of the *i*-th removed edge at the moment of removal
/// (halved for undirected to match the centrality convention). The
/// per-level modularity uses [`modularity_weighted`] so the best-Q
/// partition reflects edge weights, not just edge counts.
///
/// # Errors
/// - [`IgraphError::Unsupported`] if `graph.is_directed()` (the
///   directed variant lands in CO-006c).
/// - [`IgraphError::InvalidArgument`] if `weights.len() != ecount`,
///   or if any weight is NaN, negative, or non-finite.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_betweenness_community_weighted};
///
/// // Two K3 triangles bridged by edge (2,3). Weights = 1.0 everywhere
/// // ⇒ identical result to the unweighted slice (CO-006).
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let w = vec![1.0_f64; g.ecount()];
/// let r = edge_betweenness_community_weighted(&g, &w).unwrap();
/// assert_eq!(r.nb_clusters, 2);
/// assert_eq!(r.membership[0], r.membership[1]);
/// assert_eq!(r.membership[3], r.membership[5]);
/// assert_ne!(r.membership[0], r.membership[3]);
/// ```
pub fn edge_betweenness_community_weighted(
    graph: &Graph,
    weights: &[f64],
) -> IgraphResult<EdgeBetweennessResult> {
    if graph.is_directed() {
        return Err(IgraphError::Unsupported(
            "directed edge_betweenness_community_weighted is ALGO-CO-006c; not yet ported",
        ));
    }

    let n = graph.vcount();
    let m_us = graph.ecount();
    let n_us = n as usize;

    // Null and edgeless graphs follow the same well-defined trivial-result
    // contract as the unweighted slice.
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
        if !weights.is_empty() {
            return Err(IgraphError::InvalidArgument(format!(
                "weights vector size ({}) differs from edge count (0)",
                weights.len(),
            )));
        }
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

    // Validate weights up front: same contract as edge_betweenness_weighted
    // / modularity_weighted.
    if weights.len() != m_us {
        return Err(IgraphError::InvalidArgument(format!(
            "weights vector size ({}) differs from edge count ({})",
            weights.len(),
            m_us
        )));
    }
    for (e, &w) in weights.iter().enumerate() {
        if w.is_nan() || !w.is_finite() || w < 0.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} must be non-negative and finite (got {w})"
            )));
        }
    }

    // --- Stage 1: weighted Girvan-Newman removal order ---

    let mut inc: Vec<Vec<EdgeId>> = (0..n)
        .map(|v| graph.incident(v))
        .collect::<IgraphResult<Vec<_>>>()?;
    let mut passive: Vec<bool> = vec![false; m_us];

    let mut removed_edges: Vec<EdgeId> = Vec::with_capacity(m_us);
    let mut edge_betweenness_history: Vec<f64> = Vec::with_capacity(m_us);

    // Brandes-Dijkstra scratch buffers (reused across removals).
    let mut sigma = vec![0.0_f64; n_us];
    let mut dist = vec![f64::INFINITY; n_us];
    let mut visited = vec![false; n_us];
    let mut pred: Vec<Vec<(VertexId, EdgeId)>> = vec![Vec::new(); n_us];
    let mut stack: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut delta_v = vec![0.0_f64; n_us];
    let mut eb_now = vec![0.0_f64; m_us];

    for _ in 0..m_us {
        eb_now.fill(0.0);

        // Brandes over Dijkstra shortest paths, active edges only.
        for s in 0..n {
            sigma.fill(0.0);
            dist.fill(f64::INFINITY);
            visited.fill(false);
            for slot in &mut pred {
                slot.clear();
            }
            stack.clear();
            delta_v.fill(0.0);

            sigma[s as usize] = 1.0;
            dist[s as usize] = 0.0;
            let mut heap: BinaryHeap<Frontier> = BinaryHeap::new();
            heap.push(Frontier(0.0, s));

            while let Some(Frontier(d, v)) = heap.pop() {
                let v_us = v as usize;
                if visited[v_us] {
                    continue;
                }
                visited[v_us] = true;
                stack.push(v);

                for &eid in &inc[v_us] {
                    let w_edge = weights[eid as usize];
                    let other = graph.edge_other(eid, v)?;
                    let other_us = other as usize;
                    let nd = d + w_edge;
                    match nd.partial_cmp(&dist[other_us]) {
                        Some(Ordering::Less) => {
                            dist[other_us] = nd;
                            sigma[other_us] = sigma[v_us];
                            pred[other_us].clear();
                            pred[other_us].push((v, eid));
                            heap.push(Frontier(nd, other));
                        }
                        Some(Ordering::Equal) => {
                            sigma[other_us] += sigma[v_us];
                            pred[other_us].push((v, eid));
                        }
                        _ => {}
                    }
                }
            }

            while let Some(w) = stack.pop() {
                let w_us = w as usize;
                for &(v, e) in &pred[w_us] {
                    let c = (sigma[v as usize] / sigma[w_us]) * (1.0 + delta_v[w_us]);
                    delta_v[v as usize] += c;
                    eb_now[e as usize] += c;
                }
            }
        }

        // Tie-break: largest weighted betweenness, ties → smallest active
        // edge id (matches the upstream linear scan).
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
            "edge_betweenness_community_weighted: no active edge to remove",
        ))?;
        removed_edges.push(eid);
        edge_betweenness_history.push(max_val / 2.0);
        passive[eid as usize] = true;

        let (from, to) = graph.edge(eid)?;
        for endpoint in [from, to] {
            let list = &mut inc[endpoint as usize];
            list.retain(|&e| e != eid);
        }
    }

    // --- Stage 2: replay merges + weighted modularity per level ---

    let mut membership_now: Vec<u32> = (0..n).collect();
    let mut merges: Vec<[u32; 2]> = Vec::new();
    let mut bridges: Vec<u32> = Vec::new();
    let mut modularity_levels: Vec<f64> = Vec::new();

    let q0 = modularity_weighted(graph, &membership_now, 1.0, weights)?.unwrap_or(0.0);
    modularity_levels.push(q0);
    let mut max_mod = q0;
    let mut best_membership: Vec<u32> = membership_now.clone();

    for (step, &eid) in removed_edges.iter().enumerate().rev() {
        let (from, to) = graph.edge(eid)?;
        let c1 = membership_now[from as usize];
        let c2 = membership_now[to as usize];
        if c1 == c2 {
            continue;
        }

        let merge_index = merges.len();
        let new_cluster = n
            .checked_add(merge_index as u32)
            .ok_or(IgraphError::Internal(
                "edge_betweenness_community_weighted: merge index overflow",
            ))?;
        merges.push([c1, c2]);
        bridges.push(step as u32);

        for slot in &mut membership_now {
            if *slot == c1 || *slot == c2 {
                *slot = new_cluster;
            }
        }

        let q = modularity_weighted(graph, &membership_now, 1.0, weights)?.unwrap_or(0.0);
        modularity_levels.push(q);
        if q > max_mod {
            max_mod = q;
            best_membership.clone_from(&membership_now);
        }
    }

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

/// Reindex `labels` so distinct values become `0..nb_clusters`,
/// preserving first-appearance order. (Mirror of the helper in the
/// unweighted module — kept private here so the two SOP slices stay
/// independent.)
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
    use crate::algorithms::community::edge_betweenness_community::edge_betweenness_community;

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
        let r = edge_betweenness_community_weighted(&g, &[]).unwrap();
        assert_eq!(r.nb_clusters, 0);
        assert!(r.removed_edges.is_empty());
        assert!(r.modularity.is_empty());
    }

    #[test]
    fn edgeless_graph_returns_singletons() {
        let g = Graph::with_vertices(4);
        let r = edge_betweenness_community_weighted(&g, &[]).unwrap();
        assert_eq!(r.nb_clusters, 4);
        for v in 0..4 {
            assert_eq!(r.membership[v as usize], v);
        }
        assert!(r.removed_edges.is_empty());
        assert_eq!(r.modularity, vec![0.0]);
    }

    #[test]
    fn two_triangles_bridge_unit_weights_split_into_two() {
        let mut g = Graph::with_vertices(6);
        for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let w = vec![1.0_f64; g.ecount()];
        let r = edge_betweenness_community_weighted(&g, &w).unwrap();
        well_formed(&r, 6, 7);
        assert_eq!(r.nb_clusters, 2);
        let (from0, to0) = g.edge(r.removed_edges[0]).unwrap();
        assert!(
            (from0, to0) == (2, 3) || (from0, to0) == (3, 2),
            "first removed must be the bridge, got ({from0}, {to0})"
        );
    }

    #[test]
    fn unit_weights_match_unweighted_path_5() {
        let mut g = Graph::with_vertices(5);
        for i in 0..4u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let w = vec![1.0_f64; g.ecount()];
        let rw = edge_betweenness_community_weighted(&g, &w).unwrap();
        let ru = edge_betweenness_community(&g).unwrap();
        assert_eq!(rw.nb_clusters, ru.nb_clusters);
        assert_eq!(rw.merges, ru.merges);
        assert_eq!(rw.removed_edges, ru.removed_edges);
        // Per-step modularity is the unweighted-equivalent (m = ecount).
        for (a, b) in rw.modularity.iter().zip(ru.modularity.iter()) {
            assert!((a - b).abs() < 1e-9, "modularity mismatch: {a} vs {b}");
        }
    }

    #[test]
    fn cheap_bridge_still_removed_first() {
        // Two K3 triangles joined by edge (2,3). Bridge weight 0.1 means
        // it is the cheapest path between any (0..2) and (3..5) vertex,
        // so it sits on every cross-triangle shortest path and carries
        // the largest weighted betweenness ⇒ first removal.
        let mut g = Graph::with_vertices(6);
        for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let weights = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.1];
        let r = edge_betweenness_community_weighted(&g, &weights).unwrap();
        well_formed(&r, 6, 7);
        let (from0, to0) = g.edge(r.removed_edges[0]).unwrap();
        assert!(
            (from0, to0) == (2, 3) || (from0, to0) == (3, 2),
            "weighted-first-removed must be the bridge, got ({from0}, {to0})"
        );
    }

    #[test]
    fn directed_graph_rejected() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let err = edge_betweenness_community_weighted(&g, &[1.0, 1.0]).unwrap_err();
        assert!(matches!(err, IgraphError::Unsupported(_)));
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(edge_betweenness_community_weighted(&g, &[]).is_err());
    }

    #[test]
    fn negative_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = edge_betweenness_community_weighted(&g, &[-1.0]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = edge_betweenness_community_weighted(&g, &[f64::NAN]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn dendrogram_at_most_n_minus_components() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        let r = edge_betweenness_community_weighted(&g, &[1.0; 4]).unwrap();
        assert!(r.merges.len() <= 3);
    }

    #[test]
    fn densify_labels_preserves_first_appearance_order() {
        let (dense, n) = densify_labels(&[7, 7, 4, 4, 7, 9, 4, 9]);
        assert_eq!(n, 3);
        assert_eq!(dense, vec![0, 0, 1, 1, 0, 2, 1, 2]);
    }
}
