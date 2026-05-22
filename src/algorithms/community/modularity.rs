//! Modularity (ALGO-CO-001).
//!
//! Counterpart of `igraph_modularity()` from
//! `references/igraph/src/community/modularity.c`.
//!
//! Newman-Girvan modularity of a partition `c`:
//!
//! ```text
//! Q = (1/2m) Σ_{ij} (A_ij − γ k_i k_j / 2m) δ(c_i, c_j)
//! ```
//!
//! where `m = ecount`, `A_ij` is the adjacency matrix (each undirected
//! edge counts 2 — diagonal counts twice the number of self-loops),
//! `k_i` is the degree of vertex `i`, and `γ` is the resolution
//! parameter (1.0 recovers the classical definition).
//!
//! Phase-1 minimal slice: **undirected, unweighted**. Directed and
//! weighted variants land later (ALGO-CO-001b/c) — directed needs the
//! `(k_out_i k_in_j) / m` denominator (Leicht-Newman 2008) and
//! weighted needs the strength sum to replace `m`. Both stem from the
//! same loop body so the extension is mechanical once `Graph` exposes
//! in/out degree separately.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Modularity of `graph` with respect to community assignment `membership`.
///
/// `membership[v]` is the integer community label of vertex `v`; labels
/// need not be consecutive (we reindex internally). Returns `None` for
/// `ecount == 0` (modularity is undefined — matches upstream's NaN).
///
/// # Errors
/// - [`IgraphError::InvalidArgument`] if `membership.len() != vcount()`
///   or if `resolution < 0`.
/// - [`IgraphError::Unsupported`] for directed graphs (Phase-1 ships
///   the undirected slice; directed mode lands in ALGO-CO-001b).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, modularity};
///
/// // Two K3 triangles plus a single bridge edge: communities [0,0,0,1,1,1]
/// // give a high (positive) modularity.
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let q = modularity(&g, &[0, 0, 0, 1, 1, 1], 1.0).unwrap();
/// assert!(q.is_some());
/// assert!(q.unwrap() > 0.3);
/// ```
pub fn modularity(graph: &Graph, membership: &[u32], resolution: f64) -> IgraphResult<Option<f64>> {
    if graph.is_directed() {
        return Err(IgraphError::Unsupported(
            "directed modularity is ALGO-CO-001b; not yet ported",
        ));
    }
    let n = graph.vcount() as usize;
    if membership.len() != n {
        return Err(IgraphError::InvalidArgument(
            "membership vector size differs from number of vertices".to_string(),
        ));
    }
    if !resolution.is_finite() || resolution < 0.0 {
        return Err(IgraphError::InvalidArgument(
            "resolution parameter must be non-negative and finite".to_string(),
        ));
    }

    let ecount = graph.ecount();
    if ecount == 0 {
        return Ok(None);
    }

    // Reindex labels onto [0, no_of_partitions).
    let max_label = membership.iter().copied().max().unwrap_or(0);
    let mut remap: Vec<Option<u32>> = vec![None; max_label as usize + 1];
    let mut next_id: u32 = 0;
    let mut reindexed: Vec<u32> = Vec::with_capacity(n);
    for &lbl in membership {
        let slot = lbl as usize;
        if remap[slot].is_none() {
            remap[slot] = Some(next_id);
            next_id += 1;
        }
        reindexed.push(remap[slot].expect("just assigned"));
    }
    let no_of_partitions = next_id as usize;

    // Sum of degrees per partition (k_out + k_in for undirected graphs;
    // we accumulate into a single `k` since the contributions are equal).
    let mut k_part = vec![0.0_f64; no_of_partitions];
    let mut e_internal = 0.0_f64; // edges with both endpoints in same partition

    let m_u32 =
        u32::try_from(ecount).map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
    for eid in 0..m_u32 {
        let (src, dst) = graph.edge(eid as EdgeId)?;
        let cu = reindexed[src as usize];
        let cv = reindexed[dst as usize];
        if cu == cv {
            // Undirected: each internal edge contributes `directed_multiplier=2`
            // to e (matches C reference at modularity.c:198).
            e_internal += 2.0;
        }
        // For undirected, k_out and k_in collapse into the same vector
        // after the C code's `igraph_vector_add(&k_out, &k_in)`. We
        // anticipate that by adding 1 for each endpoint here.
        k_part[cu as usize] += 1.0;
        k_part[cv as usize] += 1.0;
    }

    #[allow(clippy::cast_precision_loss)]
    let m_f = ecount as f64;
    let two_m = 2.0 * m_f;
    // Normalise by 2m.
    for slot in &mut k_part {
        *slot /= two_m;
    }
    let e_norm = e_internal / two_m;

    let mut q = e_norm;
    for &kc in &k_part {
        q -= resolution * kc * kc;
    }
    Ok(Some(q))
}

/// Weighted modularity (ALGO-CO-001c).
///
/// Counterpart of `igraph_modularity(_, _, &weights, resolution,
/// /*directed=*/false, _)`. Same Newman-Girvan formula as
/// [`modularity`] but with edge weights replacing the unit
/// adjacency: `s_v = Σ w_e` over incident edges (self-loops
/// contribute `2w` per upstream `IGRAPH_LOOPS`), `W = Σ w_e`, and
///
/// ```text
/// Q_w = (1 / 2W) Σ_{ij} (w_ij − γ s_i s_j / 2W) δ(c_i, c_j)
/// ```
///
/// `weights.len()` must equal `graph.ecount()`. Returns `None` for
/// graphs with `ecount == 0` or `W == 0` (both modularity-undefined,
/// matches upstream NaN). Weights must be non-negative and finite —
/// igraph rejects negatives outright because the bound `Q ∈ [-1, 1]`
/// stops holding.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, modularity_weighted};
///
/// // Unit weights collapse to unweighted modularity (CO-001).
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let weights = vec![1.0_f64; 7];
/// let q = modularity_weighted(&g, &[0, 0, 0, 1, 1, 1], 1.0, &weights).unwrap();
/// assert!(q.is_some());
/// assert!(q.unwrap() > 0.3);
/// ```
pub fn modularity_weighted(
    graph: &Graph,
    membership: &[u32],
    resolution: f64,
    weights: &[f64],
) -> IgraphResult<Option<f64>> {
    if graph.is_directed() {
        return Err(IgraphError::Unsupported(
            "directed weighted modularity is ALGO-CO-001b/c follow-up; not yet ported",
        ));
    }
    let n = graph.vcount() as usize;
    if membership.len() != n {
        return Err(IgraphError::InvalidArgument(
            "membership vector size differs from number of vertices".to_string(),
        ));
    }
    if !resolution.is_finite() || resolution < 0.0 {
        return Err(IgraphError::InvalidArgument(
            "resolution parameter must be non-negative and finite".to_string(),
        ));
    }
    let ecount = graph.ecount();
    if weights.len() != ecount {
        return Err(IgraphError::InvalidArgument(format!(
            "weights vector size ({}) differs from edge count ({})",
            weights.len(),
            ecount
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
                "weight at edge {e} is negative ({w}); modularity is undefined"
            )));
        }
        if !w.is_finite() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is not finite ({w})"
            )));
        }
    }
    if ecount == 0 {
        return Ok(None);
    }

    // Reindex labels (same as the unweighted entry above).
    let max_label = membership.iter().copied().max().unwrap_or(0);
    let mut remap: Vec<Option<u32>> = vec![None; max_label as usize + 1];
    let mut next_id: u32 = 0;
    let mut reindexed: Vec<u32> = Vec::with_capacity(n);
    for &lbl in membership {
        let slot = lbl as usize;
        if remap[slot].is_none() {
            remap[slot] = Some(next_id);
            next_id += 1;
        }
        reindexed.push(remap[slot].expect("just assigned"));
    }
    let no_of_partitions = next_id as usize;

    let mut s_part = vec![0.0_f64; no_of_partitions];
    let mut w_internal = 0.0_f64;
    let mut total_w = 0.0_f64;

    let m_u32 =
        u32::try_from(ecount).map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
    for eid in 0..m_u32 {
        let (src, tgt) = graph.edge(eid as EdgeId)?;
        let w = weights[eid as usize];
        let cu = reindexed[src as usize];
        let cv = reindexed[tgt as usize];
        if cu == cv {
            // Each internal undirected edge contributes 2w to e (two
            // ordered (i,j) pairs).
            w_internal += 2.0 * w;
        }
        // Strength: each endpoint accumulates `w`. Self-loops have
        // src == tgt so this naturally contributes 2w to that vertex's
        // strength (matches IGRAPH_LOOPS).
        s_part[cu as usize] += w;
        s_part[cv as usize] += w;
        total_w += w;
    }

    if total_w == 0.0 {
        return Ok(None);
    }

    let two_w = 2.0 * total_w;
    for slot in &mut s_part {
        *slot /= two_w;
    }
    let e_norm = w_internal / two_w;

    let mut q = e_norm;
    for &sc in &s_part {
        q -= resolution * sc * sc;
    }
    Ok(Some(q))
}

/// Directed modularity (Leicht-Newman, ALGO-CO-001b).
///
/// Counterpart of `igraph_modularity(_, _, NULL_weights, resolution,
/// /*directed=*/true, _)`. The directed analogue of [`modularity`]:
///
/// ```text
/// Q = (1 / m) Σ_{ij} (A_ij − γ k_out_i * k_in_j / m) δ(c_i, c_j)
/// ```
///
/// where `m = ecount`, `k_out_i = out_degree(i)`, `k_in_j = in_degree(j)`,
/// and `A_ij` is the directed adjacency matrix (each directed edge
/// contributes 1 to one entry, not symmetric). Reference: Leicht &
/// Newman (2008).
///
/// Undirected graphs route to [`modularity`] with the same membership
/// and resolution (matches python-igraph's "ignored on undirected"
/// semantics).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, modularity_directed};
///
/// // Two directed triangles connected by a single bridge:
/// // 0→1→2→0, 3→4→5→3, plus 2→3.
/// // Partition {0,1,2} vs {3,4,5} gives a positive Q (hand-checked
/// // value: 18/49 ≈ 0.367).
/// let mut g = Graph::new(6, true).unwrap();
/// for &(u, v) in &[(0u32, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let q = modularity_directed(&g, &[0, 0, 0, 1, 1, 1], 1.0).unwrap();
/// assert!(q.is_some());
/// assert!(q.unwrap() > 0.3);
/// ```
pub fn modularity_directed(
    graph: &Graph,
    membership: &[u32],
    resolution: f64,
) -> IgraphResult<Option<f64>> {
    if !graph.is_directed() {
        // Match python-igraph: directed flag is ignored on undirected.
        return modularity(graph, membership, resolution);
    }
    let n = graph.vcount() as usize;
    if membership.len() != n {
        return Err(IgraphError::InvalidArgument(
            "membership vector size differs from number of vertices".to_string(),
        ));
    }
    if !resolution.is_finite() || resolution < 0.0 {
        return Err(IgraphError::InvalidArgument(
            "resolution parameter must be non-negative and finite".to_string(),
        ));
    }
    let ecount = graph.ecount();
    if ecount == 0 {
        return Ok(None);
    }

    // Reindex labels.
    let max_label = membership.iter().copied().max().unwrap_or(0);
    let mut remap: Vec<Option<u32>> = vec![None; max_label as usize + 1];
    let mut next_id: u32 = 0;
    let mut reindexed: Vec<u32> = Vec::with_capacity(n);
    for &lbl in membership {
        let slot = lbl as usize;
        if remap[slot].is_none() {
            remap[slot] = Some(next_id);
            next_id += 1;
        }
        reindexed.push(remap[slot].expect("just assigned"));
    }
    let no_of_partitions = next_id as usize;

    // Per-partition out- and in-degree sums; e = count of edges with
    // both endpoints in the same partition (directed_multiplier = 1).
    let mut k_out = vec![0.0_f64; no_of_partitions];
    let mut k_in = vec![0.0_f64; no_of_partitions];
    let mut e_internal = 0.0_f64;

    let m_u32 =
        u32::try_from(ecount).map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
    for eid in 0..m_u32 {
        let (src, tgt) = graph.edge(eid as EdgeId)?;
        let cu = reindexed[src as usize];
        let cv = reindexed[tgt as usize];
        if cu == cv {
            e_internal += 1.0;
        }
        k_out[cu as usize] += 1.0;
        k_in[cv as usize] += 1.0;
    }

    #[allow(clippy::cast_precision_loss)]
    let m_f = ecount as f64;
    for slot in &mut k_out {
        *slot /= m_f;
    }
    for slot in &mut k_in {
        *slot /= m_f;
    }
    let e_norm = e_internal / m_f;

    let mut q = e_norm;
    for c in 0..no_of_partitions {
        q -= resolution * k_out[c] * k_in[c];
    }
    Ok(Some(q))
}

/// Directed *weighted* modularity (Leicht-Newman, ALGO-CO-006c
/// follow-up).
///
/// Combines the directed normalisation of [`modularity_directed`]
/// (split out-strength / in-strength) with the per-edge weighting of
/// [`modularity_weighted`]:
///
/// ```text
/// Q = (1 / W) Σ_{ij} (W_ij − γ s_out_i * s_in_j / W) δ(c_i, c_j)
/// ```
///
/// where `W = Σ_e w_e`, `s_out_i = Σ_{j: i→j} w_{ij}`, `s_in_j = Σ_{i: i→j} w_{ij}`,
/// and `W_ij` is the weighted adjacency. Undirected graphs route to
/// [`modularity_weighted`] (matching python-igraph's "directed flag
/// ignored on undirected" semantics).
pub fn modularity_weighted_directed(
    graph: &Graph,
    membership: &[u32],
    resolution: f64,
    weights: &[f64],
) -> IgraphResult<Option<f64>> {
    if !graph.is_directed() {
        return modularity_weighted(graph, membership, resolution, weights);
    }
    let n = graph.vcount() as usize;
    if membership.len() != n {
        return Err(IgraphError::InvalidArgument(
            "membership vector size differs from number of vertices".to_string(),
        ));
    }
    if !resolution.is_finite() || resolution < 0.0 {
        return Err(IgraphError::InvalidArgument(
            "resolution parameter must be non-negative and finite".to_string(),
        ));
    }
    let ecount = graph.ecount();
    if weights.len() != ecount {
        return Err(IgraphError::InvalidArgument(format!(
            "weights vector size ({}) differs from edge count ({})",
            weights.len(),
            ecount
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
                "weight at edge {e} is negative ({w}); modularity is undefined"
            )));
        }
        if !w.is_finite() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is not finite ({w})"
            )));
        }
    }
    if ecount == 0 {
        return Ok(None);
    }

    let max_label = membership.iter().copied().max().unwrap_or(0);
    let mut remap: Vec<Option<u32>> = vec![None; max_label as usize + 1];
    let mut next_id: u32 = 0;
    let mut reindexed: Vec<u32> = Vec::with_capacity(n);
    for &lbl in membership {
        let slot = lbl as usize;
        if remap[slot].is_none() {
            remap[slot] = Some(next_id);
            next_id += 1;
        }
        reindexed.push(remap[slot].expect("just assigned"));
    }
    let no_of_partitions = next_id as usize;

    let mut s_out = vec![0.0_f64; no_of_partitions];
    let mut s_in = vec![0.0_f64; no_of_partitions];
    let mut w_internal = 0.0_f64;
    let mut total_w = 0.0_f64;

    let m_u32 =
        u32::try_from(ecount).map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
    for eid in 0..m_u32 {
        let (src, tgt) = graph.edge(eid as EdgeId)?;
        let w = weights[eid as usize];
        let cu = reindexed[src as usize];
        let cv = reindexed[tgt as usize];
        if cu == cv {
            w_internal += w;
        }
        s_out[cu as usize] += w;
        s_in[cv as usize] += w;
        total_w += w;
    }

    if total_w == 0.0 {
        return Ok(None);
    }

    for slot in &mut s_out {
        *slot /= total_w;
    }
    for slot in &mut s_in {
        *slot /= total_w;
    }
    let e_norm = w_internal / total_w;

    let mut q = e_norm;
    for c in 0..no_of_partitions {
        q -= resolution * s_out[c] * s_in[c];
    }
    Ok(Some(q))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: f64, expected: f64, tol: f64) {
        assert!(
            (actual - expected).abs() < tol,
            "actual={actual} expected={expected}"
        );
    }

    #[test]
    fn empty_graph_yields_none() {
        let g = Graph::with_vertices(0);
        let q = modularity(&g, &[], 1.0).unwrap();
        assert!(q.is_none());
    }

    #[test]
    fn no_edges_yields_none() {
        let g = Graph::with_vertices(3);
        let q = modularity(&g, &[0, 0, 0], 1.0).unwrap();
        assert!(q.is_none());
    }

    #[test]
    fn single_partition_zero_for_well_separated_clusters() {
        // K3 ∪ K3 + bridge edge:
        // Putting all 6 vertices in one community means all edges are
        // "internal" → e/2m = 1.0; sum of k_c² = (2m)² / (2m)² = 1.0.
        // Q = 1.0 - 1.0 = 0.0 exactly.
        let mut g = Graph::with_vertices(6);
        for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let q = modularity(&g, &[0; 6], 1.0).unwrap().unwrap();
        close(q, 0.0, 1e-12);
    }

    #[test]
    fn k3_union_k3_with_bridge_two_communities_high_q() {
        // Two K3 triangles connected by a single bridge edge, partition
        // {0,1,2} vs {3,4,5} → Q ≈ 0.408 (matches python-igraph).
        let mut g = Graph::with_vertices(6);
        for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let q = modularity(&g, &[0, 0, 0, 1, 1, 1], 1.0).unwrap().unwrap();
        // Hand calculation:
        //   m = 7, 2m = 14
        //   internal edges = 6 (3 per triangle), bridge crosses → e = 12/14
        //   k_c0 = (2+2+3)*2/2m = 14/14 = 1? Wait: vertex degrees in {0..5}:
        //     deg(0)=2, deg(1)=2, deg(2)=3, deg(3)=3, deg(4)=2, deg(5)=2.
        //     sum(c0)=7, sum(c1)=7. k_c0 = 7*2/14 = 1.0 — no wait,
        //     k_c[c] = (sum endpoints in c across edges) / 2m.
        //     Each edge contributes 1 to k of each endpoint's community.
        //     For c0 = {0,1,2}: edges 01,02,12 contribute 2 each = 6,
        //     bridge 23 contributes 1 (to c0 from vertex 2) → total 7.
        //     k[c0] = 7/14 = 0.5. Same for c1.
        //   Q = 12/14 - 1.0*(0.25 + 0.25) = 6/7 - 0.5 ≈ 0.857 - 0.5 = 0.357.
        // Sanity: positive but not maximal. python-igraph confirms.
        close(q, 6.0_f64 / 7.0 - 0.5, 1e-12);
    }

    #[test]
    fn negative_q_for_singleton_cluster_in_dense_graph() {
        // K4 with each vertex in its own community → Q < 0.
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let q = modularity(&g, &[0, 1, 2, 3], 1.0).unwrap().unwrap();
        // K4: m = 6, 2m = 12; e = 0 (no internal edges); each k_c = 3/12 = 0.25.
        // Q = 0 - 4*(1/4)*(1/4)*1 = -0.25.
        close(q, -0.25, 1e-12);
    }

    #[test]
    fn membership_size_mismatch_errors() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert!(modularity(&g, &[0, 0], 1.0).is_err());
    }

    #[test]
    fn negative_resolution_errors() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert!(modularity(&g, &[0, 0, 0], -0.1).is_err());
    }

    #[test]
    fn directed_returns_unsupported() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(modularity(&g, &[0, 0], 1.0).is_err());
    }

    #[test]
    fn non_consecutive_labels_reindex_correctly() {
        // Sparse non-consecutive label set should reindex internally.
        // Same K3 ∪ K3 + bridge graph and same partition, just with
        // labels {7, 42}.
        let mut g = Graph::with_vertices(6);
        for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let q1 = modularity(&g, &[0, 0, 0, 1, 1, 1], 1.0).unwrap().unwrap();
        let q2 = modularity(&g, &[7, 7, 7, 42, 42, 42], 1.0)
            .unwrap()
            .unwrap();
        close(q1, q2, 1e-12);
    }

    #[test]
    fn resolution_zero_yields_density_term_only() {
        // γ = 0 turns Q into e/2m alone.
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        // K4 with [0,0,1,1]: 2 internal edges (01, 23), 4 cross.
        // Q(γ=0) = 2*2/(2*6) = 4/12 = 1/3.
        let q = modularity(&g, &[0, 0, 1, 1], 0.0).unwrap().unwrap();
        close(q, 4.0 / 12.0, 1e-12);
    }

    // ----- ALGO-CO-001c: weighted modularity -----

    #[test]
    fn weighted_unit_weights_match_unweighted() {
        // Unit weights → modularity_weighted == modularity.
        let mut g = Graph::with_vertices(6);
        for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let mem = [0, 0, 0, 1, 1, 1];
        let weights = vec![1.0_f64; 7];
        let qw = modularity_weighted(&g, &mem, 1.0, &weights)
            .unwrap()
            .unwrap();
        let q = modularity(&g, &mem, 1.0).unwrap().unwrap();
        close(qw, q, 1e-12);
    }

    #[test]
    fn weighted_balanced_heavy_internal_edges_increase_q() {
        // Same K3 ∪ K3 + bridge graph. SYMMETRICALLY boost every
        // internal edge to 10× and shrink the bridge to 0.1×: both
        // communities keep equal strength so the s² penalty stays
        // balanced and the heavier concentration of internal weight
        // makes Q go up vs. the unit-weight baseline.
        let mut g = Graph::with_vertices(6);
        for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let mem = [0, 0, 0, 1, 1, 1];
        let weights = vec![10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 0.1];
        let qw = modularity_weighted(&g, &mem, 1.0, &weights)
            .unwrap()
            .unwrap();
        let q_unit = modularity(&g, &mem, 1.0).unwrap().unwrap();
        assert!(
            qw > q_unit,
            "balanced-heavy Q ({qw}) should exceed unit-weight Q ({q_unit})"
        );
    }

    #[test]
    fn weighted_zero_total_weight_yields_none() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let q = modularity_weighted(&g, &[0, 0], 1.0, &[0.0]).unwrap();
        assert!(q.is_none());
    }

    #[test]
    fn weighted_negative_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(modularity_weighted(&g, &[0, 0], 1.0, &[-1.0]).is_err());
    }

    #[test]
    fn weighted_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(modularity_weighted(&g, &[0, 0], 1.0, &[1.0, 2.0]).is_err());
    }

    #[test]
    fn weighted_directed_returns_unsupported() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(modularity_weighted(&g, &[0, 0], 1.0, &[1.0]).is_err());
    }

    // ----- ALGO-CO-001b: directed modularity (Leicht-Newman) -----

    #[test]
    fn directed_two_triangles_with_bridge_high_q() {
        // 0→1→2→0, 3→4→5→3, 2→3. Partition {0,1,2}/{3,4,5}.
        // m = 7.
        // Internal edges in c0: (0→1), (1→2), (2→0) → e_c0 = 3
        // Internal edges in c1: (3→4), (4→5), (5→3) → e_c1 = 3
        // Cross: (2→3) → 1.
        // e_internal = 6. e_norm = 6/7.
        // k_out[c0] = (1+1+1)/7 = 3/7 (vertices 0,1,2 each have out-deg 1)
        // Actually vertex 2 has out-deg 2 (2→0 and 2→3) so k_out[c0] = 4/7.
        // k_out[c1] = 3/7 (each of 3,4,5 has out-deg 1).
        // k_in[c0] = 3/7 (each of 0,1,2 has in-deg 1).
        // k_in[c1] = 4/7 (3 has in-deg 2, 4 and 5 have in-deg 1).
        // Q = 6/7 - (k_out[c0]*k_in[c0] + k_out[c1]*k_in[c1])
        //   = 6/7 - (4/7 * 3/7 + 3/7 * 4/7)
        //   = 6/7 - 24/49 = 42/49 - 24/49 = 18/49
        //   ≈ 0.3673
        let mut g = Graph::new(6, true).unwrap();
        for &(u, v) in &[(0u32, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let q = modularity_directed(&g, &[0, 0, 0, 1, 1, 1], 1.0)
            .unwrap()
            .unwrap();
        close(q, 18.0 / 49.0, 1e-12);
    }

    #[test]
    fn directed_undirected_graph_routes_to_undirected_formula() {
        let mut g = Graph::with_vertices(6);
        for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let mem = [0u32, 0, 0, 1, 1, 1];
        let a = modularity(&g, &mem, 1.0).unwrap();
        let b = modularity_directed(&g, &mem, 1.0).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn directed_no_edges_yields_none() {
        let g = Graph::new(3, true).unwrap();
        let q = modularity_directed(&g, &[0, 0, 0], 1.0).unwrap();
        assert!(q.is_none());
    }

    #[test]
    fn directed_membership_size_mismatch_errors() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(modularity_directed(&g, &[0, 0], 1.0).is_err());
    }

    #[test]
    fn directed_negative_resolution_errors() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(modularity_directed(&g, &[0, 0, 0], -0.1).is_err());
    }

    #[test]
    fn directed_3_cycle_single_partition_zero() {
        // 0→1→2→0, partition [0,0,0]: all 3 edges internal.
        // m = 3, e = 3, e_norm = 1.0.
        // k_out[0] = 3/3 = 1.0; k_in[0] = 3/3 = 1.0.
        // Q = 1.0 - 1.0*1.0 = 0.0.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let q = modularity_directed(&g, &[0, 0, 0], 1.0).unwrap().unwrap();
        close(q, 0.0, 1e-12);
    }

    #[test]
    fn weighted_two_disjoint_edges_q_eq_half() {
        // Two disjoint edges with equal weights, partitioned
        // {0,1} vs {2,3}: w_internal = 2*(1+1) = 4 (two undirected
        // edges), W = 2, 2W = 4. e_norm = 4/4 = 1.0. s[c0] = 2/4 = 0.5,
        // s[c1] = 2/4 = 0.5. Q = 1.0 - (0.25 + 0.25) = 0.5.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let q = modularity_weighted(&g, &[0, 0, 1, 1], 1.0, &[1.0, 1.0])
            .unwrap()
            .unwrap();
        close(q, 0.5, 1e-12);
    }
}
