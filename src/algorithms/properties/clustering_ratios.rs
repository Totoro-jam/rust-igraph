//! Clustering-based ratio indices (ALGO-TR-104).
//!
//! Ratios capturing clustering and triangle structure:
//!
//! - **Clustering-degree correlation** — Pearson r(degree, local clustering)
//! - **Transitivity gap** — global transitivity - avg local transitivity
//! - **Closed triplet ratio** — closed triplets / total triplets (== global transitivity)
//! - **Square clustering ratio** — fraction of connected 4-paths that close into `C_4`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the clustering-degree correlation.
///
/// Pearson correlation between vertex degree and local clustering
/// coefficient. Returns 0.0 for graphs where the correlation is
/// undefined (constant degree or constant clustering, or fewer
/// than 2 vertices with degree >= 2).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, clustering_degree_correlation};
///
/// // K_4: all same degree and clustering → 0.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!(clustering_degree_correlation(&g).unwrap().abs() < 1e-10);
/// ```
pub fn clustering_degree_correlation(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    let mut clusterings = Vec::with_capacity(n);

    for v in 0..n {
        let d = graph.degree(v as u32)?;
        degrees.push(d);

        if d < 2 {
            clusterings.push(0.0_f64);
            continue;
        }

        let neighbors = graph.neighbors(v as u32)?;
        let mut triangles = 0_u64;
        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                if graph.has_edge(neighbors[i], neighbors[j]) {
                    triangles += 1;
                }
            }
        }

        let possible = (d * (d - 1)) / 2;
        clusterings.push(triangles as f64 / possible as f64);
    }

    let eligible: Vec<usize> = (0..n).filter(|&v| degrees[v] >= 2).collect();
    if eligible.len() < 2 {
        return Ok(0.0);
    }

    let mean_deg = eligible.iter().map(|&v| degrees[v] as f64).sum::<f64>() / eligible.len() as f64;
    let mean_cc = eligible.iter().map(|&v| clusterings[v]).sum::<f64>() / eligible.len() as f64;

    let mut cov = 0.0_f64;
    let mut var_deg = 0.0_f64;
    let mut var_cc = 0.0_f64;

    for &v in &eligible {
        let dd = degrees[v] as f64 - mean_deg;
        let dc = clusterings[v] - mean_cc;
        cov += dd * dc;
        var_deg += dd * dd;
        var_cc += dc * dc;
    }

    if var_deg < 1e-30 || var_cc < 1e-30 {
        return Ok(0.0);
    }

    Ok(cov / (var_deg.sqrt() * var_cc.sqrt()))
}

/// Compute the transitivity gap.
///
/// `global_transitivity - avg_local_transitivity` — the difference
/// between the global clustering coefficient (fraction of closed
/// triplets) and the mean local clustering coefficient. Can be
/// positive or negative. Returns 0.0 for graphs with no triplets.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, transitivity_gap};
///
/// // K_4: global=1.0, all local=1.0 → gap=0.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!(transitivity_gap(&g).unwrap().abs() < 1e-10);
/// ```
pub fn transitivity_gap(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let mut total_triplets = 0_u64;
    let mut closed_triplets = 0_u64;
    let mut local_cc_sum = 0.0_f64;
    let mut local_cc_count = 0_usize;

    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d < 2 {
            continue;
        }

        let neighbors = graph.neighbors(v as u32)?;
        let mut triangles = 0_u64;
        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                if graph.has_edge(neighbors[i], neighbors[j]) {
                    triangles += 1;
                }
            }
        }

        let possible = (d * (d - 1)) / 2;
        total_triplets += possible as u64;
        closed_triplets += triangles;

        local_cc_sum += triangles as f64 / possible as f64;
        local_cc_count += 1;
    }

    if total_triplets == 0 || local_cc_count == 0 {
        return Ok(0.0);
    }

    let global_cc = closed_triplets as f64 / total_triplets as f64;
    let avg_local_cc = local_cc_sum / local_cc_count as f64;

    Ok(global_cc - avg_local_cc)
}

/// Compute the closed triplet ratio (global transitivity).
///
/// `3 * triangles / connected_triples` — the fraction of connected
/// triples that are closed into triangles. This is the standard
/// global clustering coefficient. Returns 0.0 for graphs with no
/// connected triples.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, closed_triplet_ratio};
///
/// // K_3: 1 triangle, 3 connected triples → 3*1/3 = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((closed_triplet_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn closed_triplet_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let mut total_triplets = 0_u64;
    let mut closed_triplets = 0_u64;

    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d < 2 {
            continue;
        }

        let neighbors = graph.neighbors(v as u32)?;
        let mut triangles = 0_u64;
        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                if graph.has_edge(neighbors[i], neighbors[j]) {
                    triangles += 1;
                }
            }
        }

        let possible = (d * (d - 1)) / 2;
        total_triplets += possible as u64;
        closed_triplets += triangles;
    }

    if total_triplets == 0 {
        return Ok(0.0);
    }

    Ok(closed_triplets as f64 / total_triplets as f64)
}

/// Compute the square clustering ratio.
///
/// Average over all vertices of the fraction of pairs of neighbors
/// that share a second common neighbor (forming a 4-cycle through
/// v). For vertex v with neighbors N(v), counts pairs (a,b) in N(v)
/// that have a common neighbor w != v, divided by total pairs.
/// Returns 0.0 for graphs where no vertex has degree >= 2.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, square_clustering_ratio};
///
/// // K_4: every pair of neighbors shares 1 other common neighbor → 1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((square_clustering_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn square_clustering_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let mut sum = 0.0_f64;
    let mut count = 0_usize;

    for v in 0..n {
        let neighbors = graph.neighbors(v as u32)?;
        let nn = neighbors.len();
        if nn < 2 {
            continue;
        }

        let mut closed = 0_u64;
        let total = (nn * (nn - 1)) / 2;

        for i in 0..nn {
            let ni = neighbors[i];
            let ni_neighbors = graph.neighbors(ni)?;
            for j in (i + 1)..nn {
                let nj = neighbors[j];
                let has_common = ni_neighbors
                    .iter()
                    .any(|&w| w != v as u32 && graph.has_edge(w, nj));
                if has_common {
                    closed += 1;
                }
            }
        }

        sum += closed as f64 / total as f64;
        count += 1;
    }

    if count == 0 {
        return Ok(0.0);
    }

    Ok(sum / count as f64)
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

    // --- clustering_degree_correlation ---

    #[test]
    fn cdc_empty() {
        let g = Graph::with_vertices(0);
        assert!(clustering_degree_correlation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cdc_single() {
        let g = Graph::with_vertices(1);
        assert!(clustering_degree_correlation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cdc_k3() {
        // All deg=2, all cc=1 → constant → 0
        assert!(clustering_degree_correlation(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cdc_k4() {
        assert!(clustering_degree_correlation(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cdc_cycle4() {
        // All deg=2, all cc=0 → constant → 0
        assert!(clustering_degree_correlation(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cdc_path3() {
        // Only v1 has deg>=2, so only 1 eligible → 0
        assert!(clustering_degree_correlation(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cdc_in_range() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = clustering_degree_correlation(g).unwrap();
            assert!(r >= -1.0 - 1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- transitivity_gap ---

    #[test]
    fn tg_empty() {
        let g = Graph::with_vertices(0);
        assert!(transitivity_gap(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tg_single() {
        let g = Graph::with_vertices(1);
        assert!(transitivity_gap(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tg_k3() {
        // Global=1, avg_local=1 → gap=0
        assert!(transitivity_gap(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tg_k4() {
        // Global=1, avg_local=1 → gap=0
        assert!(transitivity_gap(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tg_cycle4() {
        // Global=0 (no triangles), avg_local=0 → gap=0
        assert!(transitivity_gap(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tg_star5() {
        // Global=0, avg_local: only center has deg>=2, cc=0 → gap=0
        assert!(transitivity_gap(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tg_paw() {
        // v0: d=2, neighbors={1,2}, has_edge(1,2)=yes → cc=1.0
        // v1: d=2, neighbors={0,2}, has_edge(0,2)=yes → cc=1.0
        // v2: d=3, neighbors={0,1,3}, edges: (0,1)=yes, (0,3)=no, (1,3)=no → cc=1/3
        // v3: d=1, skip
        // total_triplets: 1+1+3=5, closed: 1+1+1=3
        // global=3/5=0.6, avg_local=(1+1+1/3)/3=7/9≈0.778
        // gap = 0.6 - 7/9 ≈ -0.178
        let r = transitivity_gap(&paw()).unwrap();
        assert!(r < 0.0);
    }

    // --- closed_triplet_ratio ---

    #[test]
    fn ctr_empty() {
        let g = Graph::with_vertices(0);
        assert!(closed_triplet_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ctr_single() {
        let g = Graph::with_vertices(1);
        assert!(closed_triplet_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ctr_k3() {
        assert!((closed_triplet_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ctr_k4() {
        assert!((closed_triplet_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ctr_cycle4() {
        assert!(closed_triplet_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ctr_star5() {
        assert!(closed_triplet_ratio(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ctr_paw() {
        // total_triplets=5, closed=3 → 3/5=0.6
        assert!((closed_triplet_ratio(&paw()).unwrap() - 0.6).abs() < 1e-10);
    }

    #[test]
    fn ctr_path3() {
        // v1: d=2, neighbors={0,2}, no edge → 0/1
        // total_triplets=1, closed=0 → 0
        assert!(closed_triplet_ratio(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ctr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = closed_triplet_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- square_clustering_ratio ---

    #[test]
    fn scr_empty() {
        let g = Graph::with_vertices(0);
        assert!(square_clustering_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn scr_single() {
        let g = Graph::with_vertices(1);
        assert!(square_clustering_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn scr_k4() {
        // Every pair of neighbors shares another common neighbor → 1.0
        assert!((square_clustering_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn scr_cycle4() {
        // v0: neighbors={1,3}, common neighbor w!=0 between 1 and 3?
        // 1's neighbors: {0,2}, 3's neighbors: {0,2}
        // w=2: has_edge(2,3)=yes → closed=1, total=1 → 1.0
        // Same for all vertices → avg=1.0
        assert!((square_clustering_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn scr_star5() {
        // Center: d=4, pairs of leaves (a,b): their neighbors are just {center}
        // So w != center that has edge to both a and b? No such w exists → 0
        assert!(square_clustering_ratio(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn scr_path3() {
        // v1: d=2, neighbors={0,2}, need common neighbor w!=1 between 0 and 2
        // 0's neighbors={1}, 2's neighbors={1}, only w=1 but w==v → no → 0
        assert!(square_clustering_ratio(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn scr_single_edge() {
        // No vertex with deg>=2 → 0
        assert!(square_clustering_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn scr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = square_clustering_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn complete_graphs_full_transitivity() {
        assert!((closed_triplet_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((closed_triplet_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn complete_graphs_zero_gap() {
        assert!(transitivity_gap(&k3()).unwrap().abs() < 1e-10);
        assert!(transitivity_gap(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn triangle_free_zero_ctr() {
        assert!(closed_triplet_ratio(&cycle4()).unwrap().abs() < 1e-10);
        assert!(closed_triplet_ratio(&star5()).unwrap().abs() < 1e-10);
        assert!(closed_triplet_ratio(&path3()).unwrap().abs() < 1e-10);
    }
}
