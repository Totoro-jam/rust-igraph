//! Local structure ratio indices (ALGO-TR-113).
//!
//! Measures capturing local structural patterns around vertices:
//!
//! - **Local density ratio** — mean density of ego-networks (1-hop neighborhoods)
//! - **Neighbor connectivity** — mean min-degree among neighbors / max-degree
//!   among neighbors, averaged over vertices
//! - **Degree-neighbor correlation** — Pearson r between vertex degree and
//!   mean neighbor degree

#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the local density ratio.
///
/// Mean density of ego-networks: for each vertex v with degree ≥ 2,
/// compute the density of the subgraph induced by v's neighbors
/// (edges among neighbors / possible edges among neighbors). Average
/// over all qualifying vertices. This equals the mean local clustering
/// coefficient. Returns 0.0 for trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, local_density_ratio};
///
/// // K_4: every vertex's neighborhood is K_3 → density 1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((local_density_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn local_density_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let mut sum_density = 0.0_f64;
    let mut count = 0_u64;

    for v in 0..n {
        let nbrs = graph.neighbors(v as u32)?;
        let d = nbrs.len();
        if d < 2 {
            continue;
        }

        let max_edges = d * (d - 1) / 2;
        let mut actual_edges = 0_u64;
        for i in 0..d {
            for j in (i + 1)..d {
                if graph.has_edge(nbrs[i], nbrs[j]) {
                    actual_edges += 1;
                }
            }
        }

        sum_density += actual_edges as f64 / max_edges as f64;
        count += 1;
    }

    if count == 0 {
        return Ok(0.0);
    }

    Ok(sum_density / count as f64)
}

/// Compute the neighbor connectivity ratio.
///
/// For each vertex v with degree ≥ 1, computes
/// `min_neighbor_degree / max_neighbor_degree`. Averages over all
/// qualifying vertices. Values near 1 indicate homogeneous neighbor
/// degrees; values near 0 indicate high disparity. Returns 0.0 for
/// edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, neighbor_connectivity_ratio};
///
/// // K_3: all neighbor degrees = 2 → min/max = 1.0 per vertex
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((neighbor_connectivity_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn neighbor_connectivity_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }

    let mut sum_ratio = 0.0_f64;
    let mut count = 0_u64;

    for v in 0..n {
        if degrees[v] == 0 {
            continue;
        }
        let nbrs = graph.neighbors(v as u32)?;
        let mut min_d = usize::MAX;
        let mut max_d = 0_usize;
        for &u in &nbrs {
            let du = degrees[u as usize];
            if du < min_d {
                min_d = du;
            }
            if du > max_d {
                max_d = du;
            }
        }
        if max_d > 0 {
            sum_ratio += min_d as f64 / max_d as f64;
            count += 1;
        }
    }

    if count == 0 {
        return Ok(0.0);
    }

    Ok(sum_ratio / count as f64)
}

/// Compute the degree-neighbor correlation.
///
/// Pearson correlation between vertex degree and mean neighbor degree
/// across all vertices with degree ≥ 1. Negative values indicate
/// disassortative mixing (high-degree nodes connect to low-degree
/// nodes). Returns 0.0 for trivial graphs or when variance is zero.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_neighbor_correlation};
///
/// // K_4: all degrees and mean-neighbor-degrees equal → 0.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!(degree_neighbor_correlation(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_neighbor_correlation(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }

    let mut x_vals = Vec::new(); // degree
    let mut y_vals = Vec::new(); // mean neighbor degree

    for v in 0..n {
        let dv = degrees[v];
        if dv == 0 {
            continue;
        }
        let nbrs = graph.neighbors(v as u32)?;
        let mean_nbr: f64 = nbrs
            .iter()
            .map(|&u| degrees[u as usize] as f64)
            .sum::<f64>()
            / dv as f64;
        x_vals.push(dv as f64);
        y_vals.push(mean_nbr);
    }

    if x_vals.len() < 2 {
        return Ok(0.0);
    }

    let count = x_vals.len() as f64;
    let mean_x: f64 = x_vals.iter().sum::<f64>() / count;
    let mean_y: f64 = y_vals.iter().sum::<f64>() / count;

    let mut cov = 0.0_f64;
    let mut var_x = 0.0_f64;
    let mut var_y = 0.0_f64;

    for i in 0..x_vals.len() {
        let dx = x_vals[i] - mean_x;
        let dy = y_vals[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x < 1e-30 || var_y < 1e-30 {
        return Ok(0.0);
    }

    Ok(cov / (var_x.sqrt() * var_y.sqrt()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> Graph {
        Graph::with_vertices(0)
    }

    fn single() -> Graph {
        Graph::with_vertices(1)
    }

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

    // --- local_density_ratio ---

    #[test]
    fn ldr_empty() {
        assert!(local_density_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ldr_single() {
        assert!(local_density_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ldr_single_edge() {
        // deg < 2 for both → 0.0
        assert!(local_density_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ldr_k3() {
        // Each vertex has 2 neighbors connected → density 1.0
        assert!((local_density_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ldr_k4() {
        // Each vertex has 3 neighbors forming K_3 → density 1.0
        assert!((local_density_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ldr_cycle4() {
        // Each vertex has 2 neighbors NOT connected → density 0.0
        assert!(local_density_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ldr_star5() {
        // Center: 4 neighbors, no edges among them → density 0.0
        // Leaves: degree 1 < 2 → excluded
        // Only center qualifies → 0.0
        assert!(local_density_ratio(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ldr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = local_density_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- neighbor_connectivity_ratio ---

    #[test]
    fn ncr_empty() {
        assert!(neighbor_connectivity_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ncr_single() {
        assert!(neighbor_connectivity_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ncr_single_edge() {
        // Both: min=max=1 → 1.0
        assert!((neighbor_connectivity_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ncr_k3() {
        assert!((neighbor_connectivity_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ncr_k4() {
        assert!((neighbor_connectivity_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ncr_cycle4() {
        // All neighbor degrees = 2 → 1.0
        assert!((neighbor_connectivity_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ncr_star5() {
        // Center: neighbors all deg=1 → min/max=1/1=1.0
        // Leaves: neighbor is center deg=4 → min/max=4/4=1.0
        assert!((neighbor_connectivity_ratio(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ncr_paw() {
        // Degrees: 0→2, 1→2, 2→3, 3→1
        // v0: nbrs={1(2),2(3)} → min=2,max=3 → 2/3
        // v1: nbrs={0(2),2(3)} → 2/3
        // v2: nbrs={0(2),1(2),3(1)} → 1/2
        // v3: nbrs={2(3)} → 3/3=1
        // mean = (2/3 + 2/3 + 1/2 + 1)/4 = (8/12 + 8/12 + 6/12 + 12/12)/4 = 34/48 = 17/24
        let r = neighbor_connectivity_ratio(&paw()).unwrap();
        assert!((r - 17.0 / 24.0).abs() < 1e-10);
    }

    #[test]
    fn ncr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = neighbor_connectivity_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- degree_neighbor_correlation ---

    #[test]
    fn dnc_empty() {
        assert!(degree_neighbor_correlation(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dnc_single() {
        assert!(degree_neighbor_correlation(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dnc_k3() {
        // All same degree → zero variance → 0.0
        assert!(degree_neighbor_correlation(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dnc_k4() {
        assert!(degree_neighbor_correlation(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dnc_cycle4() {
        assert!(degree_neighbor_correlation(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dnc_star5() {
        // Center: deg=4, mean_nbr_deg=1
        // Leaves: deg=1, mean_nbr_deg=4
        // Perfect negative correlation → -1.0
        assert!((degree_neighbor_correlation(&star5()).unwrap() + 1.0).abs() < 1e-10);
    }

    #[test]
    fn dnc_in_range() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = degree_neighbor_correlation(g).unwrap();
            assert!(r >= -1.0 - 1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn regular_zero_correlation() {
        assert!(degree_neighbor_correlation(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_neighbor_correlation(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_neighbor_correlation(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn complete_full_local_density() {
        assert!((local_density_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((local_density_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }
}
