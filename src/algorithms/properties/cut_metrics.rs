//! Graph cut quality metrics (ALGO-TR-018).
//!
//! Given a graph and a vertex partition (membership vector), compute
//! standard cut quality metrics used in spectral clustering evaluation
//! and community detection:
//!
//! - **Cut size**: number of edges (or sum of weights) crossing between
//!   different partitions.
//! - **Normalized cut (`NCut`)**: `Σ_k cut(S_k, V\S_k) / vol(S_k)` where
//!   `vol(S)` is the sum of degrees in `S`. (Shi & Malik, 2000)
//! - **Ratio cut**: `Σ_k cut(S_k, V\S_k) / |S_k|`. (Hagen & Kahng, 1992)
//! - **Conductance**: For a 2-way partition, `cut(S, V\S) / min(vol(S), vol(V\S))`.
//!   For k-way, the minimum conductance over all clusters.
//! - **Expansion**: `cut(S, V\S) / min(|S|, |V\S|)` (isoperimetric number).

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::core::{Graph, IgraphError, IgraphResult};

/// Compute the cut size of a partition.
///
/// Counts the number of edges (or sum of edge weights) that cross between
/// different clusters.
///
/// # Parameters
///
/// - `graph` — The input graph.
/// - `membership` — Cluster assignment for each vertex (`membership[v]` is the
///   cluster id of vertex `v`). Length must equal `vcount`.
/// - `weights` — Optional edge weights. If `None`, each edge has weight 1.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, cut_size};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// // Partition: {0,1} vs {2,3}
/// let membership = vec![0u32, 0, 1, 1];
/// let cs = cut_size(&g, &membership, None).unwrap();
/// assert!((cs - 1.0).abs() < 1e-10); // only edge 1-2 crosses
/// ```
pub fn cut_size(graph: &Graph, membership: &[u32], weights: Option<&[f64]>) -> IgraphResult<f64> {
    let nv = graph.vcount() as usize;
    let ne = graph.ecount();

    if membership.len() != nv {
        return Err(IgraphError::InvalidArgument(format!(
            "membership length {} does not match vcount {nv}",
            membership.len()
        )));
    }

    if let Some(w) = weights {
        if w.len() != ne {
            return Err(IgraphError::InvalidArgument(format!(
                "weights length {} does not match ecount {ne}",
                w.len()
            )));
        }
    }

    let mut total_cut = 0.0_f64;
    for (eid, (u, v)) in graph.edges().enumerate() {
        if membership[u as usize] != membership[v as usize] {
            let w = weights.map_or(1.0, |ws| ws[eid]);
            total_cut += w;
        }
    }

    Ok(total_cut)
}

/// Compute the normalized cut (`NCut`) of a partition.
///
/// `NCut = Σ_k cut(S_k, V\S_k) / vol(S_k)`
///
/// where `vol(S_k)` is the sum of (weighted) degrees of vertices in cluster `k`.
/// Lower values indicate better balanced cuts.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, normalized_cut};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,0)], false, Some(4)
/// ).unwrap();
/// // Balanced partition: {0,1} vs {2,3}
/// let membership = vec![0u32, 0, 1, 1];
/// let nc = normalized_cut(&g, &membership, None).unwrap();
/// assert!(nc > 0.0);
/// ```
pub fn normalized_cut(
    graph: &Graph,
    membership: &[u32],
    weights: Option<&[f64]>,
) -> IgraphResult<f64> {
    let nv = graph.vcount() as usize;
    let ne = graph.ecount();

    if membership.len() != nv {
        return Err(IgraphError::InvalidArgument(format!(
            "membership length {} does not match vcount {nv}",
            membership.len()
        )));
    }

    if let Some(w) = weights {
        if w.len() != ne {
            return Err(IgraphError::InvalidArgument(format!(
                "weights length {} does not match ecount {ne}",
                w.len()
            )));
        }
    }

    let k = partition_count(membership);
    if k == 0 {
        return Ok(0.0);
    }

    // Compute volume of each cluster (sum of weighted degrees)
    let mut vol = vec![0.0_f64; k];
    let mut cut_per_cluster = vec![0.0_f64; k];

    for (eid, (u, v)) in graph.edges().enumerate() {
        let w = weights.map_or(1.0, |ws| ws[eid]);
        let cu = membership[u as usize] as usize;
        let cv = membership[v as usize] as usize;

        // Each edge contributes to the volume of its endpoints' clusters
        vol[cu] += w;
        vol[cv] += w;

        if cu != cv {
            cut_per_cluster[cu] += w;
            cut_per_cluster[cv] += w;
        }
    }

    let mut ncut = 0.0_f64;
    for c in 0..k {
        if vol[c] > 0.0 {
            ncut += cut_per_cluster[c] / vol[c];
        }
    }

    Ok(ncut)
}

/// Compute the ratio cut of a partition.
///
/// `RatioCut = Σ_k cut(S_k, V\S_k) / |S_k|`
///
/// where `|S_k|` is the number of vertices in cluster `k`.
/// Lower values indicate better balanced cuts.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, ratio_cut};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,0)], false, Some(4)
/// ).unwrap();
/// let membership = vec![0u32, 0, 1, 1];
/// let rc = ratio_cut(&g, &membership, None).unwrap();
/// assert!(rc > 0.0);
/// ```
pub fn ratio_cut(graph: &Graph, membership: &[u32], weights: Option<&[f64]>) -> IgraphResult<f64> {
    let nv = graph.vcount() as usize;
    let ne = graph.ecount();

    if membership.len() != nv {
        return Err(IgraphError::InvalidArgument(format!(
            "membership length {} does not match vcount {nv}",
            membership.len()
        )));
    }

    if let Some(w) = weights {
        if w.len() != ne {
            return Err(IgraphError::InvalidArgument(format!(
                "weights length {} does not match ecount {ne}",
                w.len()
            )));
        }
    }

    let k = partition_count(membership);
    if k == 0 {
        return Ok(0.0);
    }

    let mut sizes = vec![0usize; k];
    for &c in membership {
        sizes[c as usize] += 1;
    }

    let mut cut_per_cluster = vec![0.0_f64; k];
    for (eid, (u, v)) in graph.edges().enumerate() {
        let w = weights.map_or(1.0, |ws| ws[eid]);
        let cu = membership[u as usize] as usize;
        let cv = membership[v as usize] as usize;

        if cu != cv {
            cut_per_cluster[cu] += w;
            cut_per_cluster[cv] += w;
        }
    }

    let mut rcut = 0.0_f64;
    for c in 0..k {
        if sizes[c] > 0 {
            rcut += cut_per_cluster[c] / sizes[c] as f64;
        }
    }

    Ok(rcut)
}

/// Compute the conductance of a partition.
///
/// For a 2-way partition: `cut(S, V\S) / min(vol(S), vol(V\S))`.
/// For k-way: the maximum conductance (worst cluster), defined as
/// `max_k cut(S_k, V\S_k) / vol(S_k)` over clusters with `vol > 0`.
///
/// Lower conductance means better separation.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, conductance};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,0),(0,2)], false, Some(4)
/// ).unwrap();
/// let membership = vec![0u32, 0, 1, 1];
/// let c = conductance(&g, &membership, None).unwrap();
/// assert!(c > 0.0 && c <= 1.0);
/// ```
pub fn conductance(
    graph: &Graph,
    membership: &[u32],
    weights: Option<&[f64]>,
) -> IgraphResult<f64> {
    let nv = graph.vcount() as usize;
    let ne = graph.ecount();

    if membership.len() != nv {
        return Err(IgraphError::InvalidArgument(format!(
            "membership length {} does not match vcount {nv}",
            membership.len()
        )));
    }

    if let Some(w) = weights {
        if w.len() != ne {
            return Err(IgraphError::InvalidArgument(format!(
                "weights length {} does not match ecount {ne}",
                w.len()
            )));
        }
    }

    let k = partition_count(membership);
    if k == 0 {
        return Ok(0.0);
    }

    let mut vol = vec![0.0_f64; k];
    let mut cut_per_cluster = vec![0.0_f64; k];

    for (eid, (u, v)) in graph.edges().enumerate() {
        let w = weights.map_or(1.0, |ws| ws[eid]);
        let cu = membership[u as usize] as usize;
        let cv = membership[v as usize] as usize;

        vol[cu] += w;
        vol[cv] += w;

        if cu != cv {
            cut_per_cluster[cu] += w;
            cut_per_cluster[cv] += w;
        }
    }

    // For 2-way partition, use the standard definition
    if k == 2 {
        let total_cut = cut_per_cluster[0]; // same as cut_per_cluster[1]
        let min_vol = vol[0].min(vol[1]);
        if min_vol > 0.0 {
            return Ok(total_cut / min_vol);
        }
        return Ok(0.0);
    }

    // For k-way, return max conductance over all clusters
    let mut max_cond = 0.0_f64;
    for c in 0..k {
        if vol[c] > 0.0 {
            let cond = cut_per_cluster[c] / vol[c];
            if cond > max_cond {
                max_cond = cond;
            }
        }
    }

    Ok(max_cond)
}

/// Compute the expansion (isoperimetric number) of a partition.
///
/// For a 2-way partition: `cut(S, V\S) / min(|S|, |V\S|)`.
/// For k-way: the maximum expansion over all clusters.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, expansion};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,0)], false, Some(4)
/// ).unwrap();
/// let membership = vec![0u32, 0, 1, 1];
/// let e = expansion(&g, &membership, None).unwrap();
/// assert!(e > 0.0);
/// ```
pub fn expansion(graph: &Graph, membership: &[u32], weights: Option<&[f64]>) -> IgraphResult<f64> {
    let nv = graph.vcount() as usize;
    let ne = graph.ecount();

    if membership.len() != nv {
        return Err(IgraphError::InvalidArgument(format!(
            "membership length {} does not match vcount {nv}",
            membership.len()
        )));
    }

    if let Some(w) = weights {
        if w.len() != ne {
            return Err(IgraphError::InvalidArgument(format!(
                "weights length {} does not match ecount {ne}",
                w.len()
            )));
        }
    }

    let k = partition_count(membership);
    if k == 0 {
        return Ok(0.0);
    }

    let mut sizes = vec![0usize; k];
    for &c in membership {
        sizes[c as usize] += 1;
    }

    let mut cut_per_cluster = vec![0.0_f64; k];
    for (eid, (u, v)) in graph.edges().enumerate() {
        let w = weights.map_or(1.0, |ws| ws[eid]);
        let cu = membership[u as usize] as usize;
        let cv = membership[v as usize] as usize;

        if cu != cv {
            cut_per_cluster[cu] += w;
            cut_per_cluster[cv] += w;
        }
    }

    if k == 2 {
        let total_cut = cut_per_cluster[0];
        let min_size = sizes[0].min(sizes[1]);
        if min_size > 0 {
            return Ok(total_cut / min_size as f64);
        }
        return Ok(0.0);
    }

    let mut max_exp = 0.0_f64;
    for c in 0..k {
        if sizes[c] > 0 {
            let exp_c = cut_per_cluster[c] / sizes[c] as f64;
            if exp_c > max_exp {
                max_exp = exp_c;
            }
        }
    }

    Ok(max_exp)
}

// --- Internal helpers ---

fn partition_count(membership: &[u32]) -> usize {
    if membership.is_empty() {
        return 0;
    }
    membership
        .iter()
        .copied()
        .max()
        .map_or(0, |m| m as usize + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    fn complete4() -> Graph {
        Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap()
    }

    // --- cut_size tests ---

    #[test]
    fn cut_size_path_balanced() {
        let g = path4();
        let m = vec![0u32, 0, 1, 1];
        let cs = cut_size(&g, &m, None).unwrap();
        assert!((cs - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cut_size_all_same_cluster() {
        let g = cycle4();
        let m = vec![0u32, 0, 0, 0];
        let cs = cut_size(&g, &m, None).unwrap();
        assert!(cs.abs() < 1e-10);
    }

    #[test]
    fn cut_size_all_different() {
        let g = cycle4();
        let m = vec![0u32, 1, 2, 3];
        let cs = cut_size(&g, &m, None).unwrap();
        assert!((cs - 4.0).abs() < 1e-10);
    }

    #[test]
    fn cut_size_weighted() {
        let g = path4();
        let m = vec![0u32, 0, 1, 1];
        let w = vec![1.0, 3.0, 1.0]; // edge 1-2 has weight 3
        let cs = cut_size(&g, &m, Some(&w)).unwrap();
        assert!((cs - 3.0).abs() < 1e-10);
    }

    #[test]
    fn cut_size_empty_graph() {
        let g = Graph::with_vertices(3);
        let m = vec![0u32, 1, 2];
        let cs = cut_size(&g, &m, None).unwrap();
        assert!(cs.abs() < 1e-10);
    }

    #[test]
    fn cut_size_invalid_membership() {
        let g = path4();
        let m = vec![0u32, 1]; // too short
        assert!(cut_size(&g, &m, None).is_err());
    }

    #[test]
    fn cut_size_invalid_weights() {
        let g = path4();
        let m = vec![0u32, 0, 1, 1];
        let w = vec![1.0]; // too short
        assert!(cut_size(&g, &m, Some(&w)).is_err());
    }

    // --- normalized_cut tests ---

    #[test]
    fn ncut_cycle_balanced() {
        let g = cycle4();
        let m = vec![0u32, 0, 1, 1];
        let nc = normalized_cut(&g, &m, None).unwrap();
        // cut(S0, S1) = 2 (edges 1-2 and 3-0)
        // vol(S0) = deg(0)+deg(1) = 2+2 = 4
        // vol(S1) = deg(2)+deg(3) = 2+2 = 4
        // NCut = 2/4 + 2/4 = 1.0
        assert!((nc - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ncut_all_same_cluster() {
        let g = cycle4();
        let m = vec![0u32, 0, 0, 0];
        let nc = normalized_cut(&g, &m, None).unwrap();
        assert!(nc.abs() < 1e-10);
    }

    #[test]
    fn ncut_complete_balanced() {
        let g = complete4();
        let m = vec![0u32, 0, 1, 1];
        let nc = normalized_cut(&g, &m, None).unwrap();
        // cut edges: 0-2, 0-3, 1-2, 1-3 = 4
        // vol(S0) = 3+3 = 6, vol(S1) = 3+3 = 6
        // NCut = 4/6 + 4/6 = 8/6 = 4/3
        assert!((nc - 4.0 / 3.0).abs() < 1e-10);
    }

    // --- ratio_cut tests ---

    #[test]
    fn rcut_cycle_balanced() {
        let g = cycle4();
        let m = vec![0u32, 0, 1, 1];
        let rc = ratio_cut(&g, &m, None).unwrap();
        // cut = 2, |S0| = 2, |S1| = 2
        // RatioCut = 2/2 + 2/2 = 2.0
        assert!((rc - 2.0).abs() < 1e-10);
    }

    #[test]
    fn rcut_all_same_cluster() {
        let g = cycle4();
        let m = vec![0u32, 0, 0, 0];
        let rc = ratio_cut(&g, &m, None).unwrap();
        assert!(rc.abs() < 1e-10);
    }

    #[test]
    fn rcut_path_unbalanced() {
        let g = path4();
        let m = vec![0u32, 0, 0, 1]; // {0,1,2} vs {3}
        let rc = ratio_cut(&g, &m, None).unwrap();
        // cut = 1 (edge 2-3), |S0| = 3, |S1| = 1
        // RatioCut = 1/3 + 1/1 = 4/3
        assert!((rc - 4.0 / 3.0).abs() < 1e-10);
    }

    // --- conductance tests ---

    #[test]
    fn conductance_cycle_balanced() {
        let g = cycle4();
        let m = vec![0u32, 0, 1, 1];
        let c = conductance(&g, &m, None).unwrap();
        // cut = 2, vol(S0) = 4, vol(S1) = 4
        // conductance = 2 / min(4, 4) = 0.5
        assert!((c - 0.5).abs() < 1e-10);
    }

    #[test]
    fn conductance_all_same() {
        let g = cycle4();
        let m = vec![0u32, 0, 0, 0];
        let c = conductance(&g, &m, None).unwrap();
        assert!(c.abs() < 1e-10);
    }

    #[test]
    fn conductance_bounded() {
        let g = complete4();
        let m = vec![0u32, 0, 1, 1];
        let c = conductance(&g, &m, None).unwrap();
        assert!(c >= 0.0);
        assert!(c <= 1.0);
    }

    // --- expansion tests ---

    #[test]
    fn expansion_cycle_balanced() {
        let g = cycle4();
        let m = vec![0u32, 0, 1, 1];
        let e = expansion(&g, &m, None).unwrap();
        // cut = 2, min(|S0|, |S1|) = min(2, 2) = 2
        // expansion = 2/2 = 1.0
        assert!((e - 1.0).abs() < 1e-10);
    }

    #[test]
    fn expansion_path_unbalanced() {
        let g = path4();
        let m = vec![0u32, 0, 0, 1];
        let e = expansion(&g, &m, None).unwrap();
        // cut = 1, min(3, 1) = 1
        // expansion = 1/1 = 1.0
        assert!((e - 1.0).abs() < 1e-10);
    }

    #[test]
    fn expansion_all_same() {
        let g = cycle4();
        let m = vec![0u32, 0, 0, 0];
        let e = expansion(&g, &m, None).unwrap();
        assert!(e.abs() < 1e-10);
    }

    // --- directed graph tests ---

    #[test]
    fn cut_size_directed() {
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], true, Some(3)).unwrap();
        let m = vec![0u32, 0, 1];
        let cs = cut_size(&g, &m, None).unwrap();
        // Edges crossing: 1→2 and 2→0 = 2
        assert!((cs - 2.0).abs() < 1e-10);
    }

    // --- empty graph tests ---

    #[test]
    fn metrics_on_empty_partition() {
        let g = Graph::with_vertices(0);
        let m: Vec<u32> = vec![];
        assert!((cut_size(&g, &m, None).unwrap()).abs() < 1e-10);
        assert!((normalized_cut(&g, &m, None).unwrap()).abs() < 1e-10);
        assert!((ratio_cut(&g, &m, None).unwrap()).abs() < 1e-10);
        assert!((conductance(&g, &m, None).unwrap()).abs() < 1e-10);
        assert!((expansion(&g, &m, None).unwrap()).abs() < 1e-10);
    }
}
