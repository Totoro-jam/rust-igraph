//! Edge neighborhood overlap aggregates (ALGO-TR-092).
//!
//! Graph-level indices that aggregate the neighborhood overlap of each
//! edge's endpoints:
//!
//! - **Common neighbor sum** `Σ |N(u) ∩ N(v)|` — total shared neighbors
//! - **Jaccard sum** `Σ |N(u) ∩ N(v)| / |N(u) ∪ N(v)|` — average overlap
//! - **Overlap coefficient sum** `Σ |N(u) ∩ N(v)| / min(d(u),d(v))` — relative overlap
//! - **Adamic–Adar sum** `Σ_{(u,v)∈E} Σ_{w∈N(u)∩N(v)} 1/ln(d(w))` — rare-neighbor weight

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

fn common_neighbors(graph: &Graph, u: u32, v: u32) -> IgraphResult<Vec<u32>> {
    let nu = graph.neighbors(u)?;
    let nv = graph.neighbors(v)?;
    let mut nu_sorted = nu;
    let mut nv_sorted = nv;
    nu_sorted.sort_unstable();
    nv_sorted.sort_unstable();

    let mut common = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < nu_sorted.len() && j < nv_sorted.len() {
        match nu_sorted[i].cmp(&nv_sorted[j]) {
            std::cmp::Ordering::Equal => {
                let w = nu_sorted[i];
                if w != u && w != v {
                    common.push(w);
                }
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
    Ok(common)
}

/// Compute the sum of common neighbor counts across all edges.
///
/// `Σ_{(u,v)∈E} |N(u) ∩ N(v) \ {u,v}|`
///
/// Counts the total number of triangles-closing common neighbors.
/// Returns 0 for edgeless or tree-like graphs. Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_common_neighbor_sum};
///
/// // K_3: each edge shares 1 common neighbor → 3·1 = 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(edge_common_neighbor_sum(&g).unwrap(), 3);
/// ```
pub fn edge_common_neighbor_sum(graph: &Graph) -> IgraphResult<u64> {
    let mut result = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let cn = common_neighbors(graph, u, v)?;
        result = result.saturating_add(cn.len() as u64);
    }

    Ok(result)
}

/// Compute the sum of Jaccard overlap coefficients across all edges.
///
/// `Σ_{(u,v)∈E} |N(u) ∩ N(v) \ {u,v}| / |N(u) ∪ N(v) \ {u,v}|`
///
/// Each edge contributes a value in [0, 1]. Returns 0.0 for the empty
/// graph. Self-loops are skipped. Edges where the union is empty
/// (isolated edge components) contribute 0.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_jaccard_sum};
///
/// // K_3: each edge: |{w}|/|{w}| = 1 → 3·1 = 3.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_jaccard_sum(&g).unwrap() - 3.0).abs() < 1e-10);
/// ```
pub fn edge_jaccard_sum(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let nu = graph.neighbors(u)?;
        let nv = graph.neighbors(v)?;

        let mut nu_set: Vec<u32> = nu.into_iter().filter(|&w| w != u && w != v).collect();
        let mut nv_set: Vec<u32> = nv.into_iter().filter(|&w| w != u && w != v).collect();
        nu_set.sort_unstable();
        nv_set.sort_unstable();

        let mut inter = 0_usize;
        let (mut i, mut j) = (0, 0);
        while i < nu_set.len() && j < nv_set.len() {
            match nu_set[i].cmp(&nv_set[j]) {
                std::cmp::Ordering::Equal => {
                    inter += 1;
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
            }
        }

        let union_size = nu_set.len() + nv_set.len() - inter;
        if union_size > 0 {
            result += inter as f64 / union_size as f64;
        }
    }

    Ok(result)
}

/// Compute the sum of overlap coefficients across all edges.
///
/// `Σ_{(u,v)∈E} |N(u) ∩ N(v) \ {u,v}| / min(|N(u)\{v}|, |N(v)\{u}|)`
///
/// Each edge contributes a value in [0, 1]. Edges where min neighbor
/// count (excluding the other endpoint) is zero contribute 0.
/// Self-loops are skipped. Returns 0.0 for the empty graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_overlap_sum};
///
/// // K_3: each edge: |{w}| / min(1,1) = 1 → 3·1 = 3.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_overlap_sum(&g).unwrap() - 3.0).abs() < 1e-10);
/// ```
pub fn edge_overlap_sum(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let cn = common_neighbors(graph, u, v)?;
        let du = graph.degree(u)?;
        let dv = graph.degree(v)?;
        let du_excl = du.saturating_sub(1);
        let dv_excl = dv.saturating_sub(1);
        let min_d = du_excl.min(dv_excl);
        if min_d == 0 {
            continue;
        }
        result += cn.len() as f64 / min_d as f64;
    }

    Ok(result)
}

/// Compute the sum of Adamic–Adar indices across all edges.
///
/// `Σ_{(u,v)∈E} Σ_{w ∈ N(u) ∩ N(v)} 1/ln(d(w))`
///
/// Common neighbors with higher degree contribute less (rare shared
/// neighbors are more informative). Returns 0.0 for the empty graph.
/// Vertices with degree ≤ 1 contribute infinity in theory; we skip
/// them (they can't be common neighbors of an edge's endpoints in
/// a simple graph with ≥ 3 vertices). Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_adamic_adar_sum};
///
/// // K_3: each edge has 1 common neighbor of degree 2
/// // → 3 · (1/ln(2)) ≈ 3/0.6931 ≈ 4.328
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let expected = 3.0 / 2.0_f64.ln();
/// assert!((edge_adamic_adar_sum(&g).unwrap() - expected).abs() < 1e-10);
/// ```
pub fn edge_adamic_adar_sum(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let cn = common_neighbors(graph, u, v)?;
        for &w in &cn {
            let dw = graph.degree(w)?;
            if dw >= 2 {
                result += 1.0 / (dw as f64).ln();
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_edge() -> Graph {
        Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap()
    }

    fn path3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap()
    }

    fn k3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap()
    }

    fn k4() -> Graph {
        Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap()
    }

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- edge_common_neighbor_sum ---

    #[test]
    fn cn_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(edge_common_neighbor_sum(&g).unwrap(), 0);
    }

    #[test]
    fn cn_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(edge_common_neighbor_sum(&g).unwrap(), 0);
    }

    #[test]
    fn cn_single_edge() {
        // No common neighbors
        assert_eq!(edge_common_neighbor_sum(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn cn_path3() {
        // No edge shares a common neighbor
        assert_eq!(edge_common_neighbor_sum(&path3()).unwrap(), 0);
    }

    #[test]
    fn cn_k3() {
        // Each of 3 edges has 1 common neighbor → 3
        assert_eq!(edge_common_neighbor_sum(&k3()).unwrap(), 3);
    }

    #[test]
    fn cn_k4() {
        // Each of 6 edges has 2 common neighbors → 12
        assert_eq!(edge_common_neighbor_sum(&k4()).unwrap(), 12);
    }

    #[test]
    fn cn_star5() {
        // No edge shares a common neighbor (leaves not connected)
        assert_eq!(edge_common_neighbor_sum(&star5()).unwrap(), 0);
    }

    #[test]
    fn cn_cycle4() {
        // Opposite edges share 0 cn; adjacent shares 0 cn
        // (0,1) cn with {3,0}∩{0,2}\{0,1} → check: N(0)\{1}={3}, N(1)\{0}={2} → ∩=∅
        // Actually all edges of C4: each pair has no common neighbor
        assert_eq!(edge_common_neighbor_sum(&cycle4()).unwrap(), 0);
    }

    #[test]
    fn cn_paw() {
        // (0,1): N(0)\{1}={2}, N(1)\{0}={2} → {2} → 1
        // (0,2): N(0)\{2}={1}, N(2)\{0}={1,3} → {1} → 1
        // (1,2): N(1)\{2}={0}, N(2)\{1}={0,3} → {0} → 1
        // (2,3): N(2)\{3}={0,1}, N(3)\{2}={} → {} → 0
        assert_eq!(edge_common_neighbor_sum(&paw()).unwrap(), 3);
    }

    // --- edge_jaccard_sum ---

    #[test]
    fn jac_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_jaccard_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn jac_single_edge() {
        // N(0)\{1} = ∅, N(1)\{0} = ∅, union=0 → contribute 0
        assert!(edge_jaccard_sum(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn jac_path3() {
        // (0,1): N(0)\{1}=∅, N(1)\{0}={2} → inter=0, union=1 → 0
        // (1,2): N(1)\{2}={0}, N(2)\{1}=∅ → inter=0, union=1 → 0
        assert!(edge_jaccard_sum(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn jac_k3() {
        // Each edge: N(u)\{v}={w}, N(v)\{u}={w} → inter=1, union=1 → 1.0
        // 3 edges → 3.0
        assert!((edge_jaccard_sum(&k3()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn jac_k4() {
        // Each edge: |CN|=2, union=|N(u)\{v}|+|N(v)\{u}|-|CN|=2+2-2=2
        // Jaccard = 2/2 = 1.0, 6 edges → 6.0
        assert!((edge_jaccard_sum(&k4()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn jac_star5() {
        // (0,k): N(0)\{k}={others 3 leaves}, N(k)\{0}=∅ → inter=0, union=3 → 0
        assert!(edge_jaccard_sum(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn jac_paw() {
        // (0,1): N(0)\{1}={2}, N(1)\{0}={2} → inter=1, union=1 → 1.0
        // (0,2): N(0)\{2}={1}, N(2)\{0}={1,3} → inter=1, union=2 → 0.5
        // (1,2): N(1)\{2}={0}, N(2)\{1}={0,3} → inter=1, union=2 → 0.5
        // (2,3): N(2)\{3}={0,1}, N(3)\{2}=∅ → inter=0, union=2 → 0
        let expected = 1.0 + 0.5 + 0.5 + 0.0;
        assert!((edge_jaccard_sum(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- edge_overlap_sum ---

    #[test]
    fn ovl_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_overlap_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ovl_single_edge() {
        // min(d-1, d-1) = min(0,0) = 0 → skip → 0
        assert!(edge_overlap_sum(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ovl_path3() {
        // (0,1): min(0,1)=0 → skip
        // (1,2): min(1,0)=0 → skip
        assert!(edge_overlap_sum(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ovl_k3() {
        // Each edge: |CN|=1, min(d-1,d-1)=min(1,1)=1 → 1/1=1
        // 3 edges → 3.0
        assert!((edge_overlap_sum(&k3()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn ovl_k4() {
        // Each edge: |CN|=2, min(d-1,d-1)=min(2,2)=2 → 2/2=1
        // 6 edges → 6.0
        assert!((edge_overlap_sum(&k4()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn ovl_star5() {
        // (0,k): |CN|=0, min(3,0)=0 → skip
        assert!(edge_overlap_sum(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ovl_paw() {
        // (0,1): |CN|=1, min(1,1)=1 → 1.0
        // (0,2): |CN|=1, min(1,2)=1 → 1.0
        // (1,2): |CN|=1, min(1,2)=1 → 1.0
        // (2,3): |CN|=0, min(2,0)=0 → skip
        assert!((edge_overlap_sum(&paw()).unwrap() - 3.0).abs() < 1e-10);
    }

    // --- edge_adamic_adar_sum ---

    #[test]
    fn aa_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_adamic_adar_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn aa_single_edge() {
        assert!(edge_adamic_adar_sum(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn aa_path3() {
        assert!(edge_adamic_adar_sum(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn aa_k3() {
        // Each edge: 1 common neighbor of degree 2
        // 3 · 1/ln(2)
        let expected = 3.0 / 2.0_f64.ln();
        assert!((edge_adamic_adar_sum(&k3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn aa_k4() {
        // Each edge: 2 common neighbors each of degree 3
        // 6 · 2/ln(3) = 12/ln(3)
        let expected = 12.0 / 3.0_f64.ln();
        assert!((edge_adamic_adar_sum(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn aa_star5() {
        assert!(edge_adamic_adar_sum(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn aa_paw() {
        // (0,1): CN={2}, d(2)=3 → 1/ln(3)
        // (0,2): CN={1}, d(1)=2 → 1/ln(2)
        // (1,2): CN={0}, d(0)=2 → 1/ln(2)
        // (2,3): CN=∅ → 0
        let expected = 1.0 / 3.0_f64.ln() + 2.0 / 2.0_f64.ln();
        assert!((edge_adamic_adar_sum(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn cn_eq_3_times_triangles_for_complete() {
        // In K_n, each edge has (n-2) common neighbors
        // total = C(n,2)·(n-2) = n(n-1)(n-2)/2
        // K_3: 3·1=3, K_4: 6·2=12
        assert_eq!(edge_common_neighbor_sum(&k3()).unwrap(), 3);
        assert_eq!(edge_common_neighbor_sum(&k4()).unwrap(), 12);
    }

    #[test]
    fn jaccard_perfect_for_complete() {
        // In K_n, Jaccard = 1.0 for every edge → sum = m
        assert!((edge_jaccard_sum(&k3()).unwrap() - 3.0).abs() < 1e-10);
        assert!((edge_jaccard_sum(&k4()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn overlap_perfect_for_complete() {
        // In K_n, overlap coeff = 1.0 for every edge → sum = m
        assert!((edge_overlap_sum(&k3()).unwrap() - 3.0).abs() < 1e-10);
        assert!((edge_overlap_sum(&k4()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn cn_bounded_by_m_times_n() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let cn = edge_common_neighbor_sum(g).unwrap();
            let bound = g.ecount() as u64 * u64::from(g.vcount());
            assert!(cn <= bound);
        }
    }

    #[test]
    fn jaccard_in_0_m() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let j = edge_jaccard_sum(g).unwrap();
            let m = g.ecount() as f64;
            assert!(j >= -1e-10);
            assert!(j <= m + 1e-10);
        }
    }

    #[test]
    fn aa_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(edge_adamic_adar_sum(g).unwrap() >= -1e-10);
        }
    }
}
