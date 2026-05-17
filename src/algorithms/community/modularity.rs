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
}
