//! Neighborhood density measures (ALGO-TR-100).
//!
//! Measures capturing local neighborhood structure:
//!
//! - **Average neighbor degree ratio** — mean ratio of each vertex's
//!   average neighbor degree to its own degree
//! - **Hub ratio** — fraction of vertices whose degree exceeds the average
//! - **Leaf-to-hub ratio** — degree-1 vertices divided by hub vertices
//! - **Degree centralization** — Freeman's degree centralization

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the average neighbor degree ratio.
///
/// For each vertex with degree >= 1, compute the ratio of the average
/// degree of its neighbors to its own degree. Return the mean of these
/// ratios. Returns 0.0 for graphs where no vertex has degree >= 1.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, avg_neighbor_degree_ratio};
///
/// // K_3: each v has d=2, neighbors avg d=2, ratio=1.0 → avg=1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((avg_neighbor_degree_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn avg_neighbor_degree_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }

    let mut sum = 0.0_f64;
    let mut count = 0_usize;

    for v in 0..n {
        let dv = degrees[v];
        if dv == 0 {
            continue;
        }

        let neighbors = graph.neighbors(v as u32)?;
        let mut neighbor_deg_sum = 0_usize;
        for &u in &neighbors {
            neighbor_deg_sum += degrees[u as usize];
        }

        let avg_neighbor_deg = neighbor_deg_sum as f64 / dv as f64;
        sum += avg_neighbor_deg / dv as f64;
        count += 1;
    }

    if count == 0 {
        return Ok(0.0);
    }

    Ok(sum / count as f64)
}

/// Compute the hub ratio.
///
/// Fraction of vertices whose degree exceeds the average degree.
/// Returns 0.0 for empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, hub_ratio};
///
/// // K_3: all d=2, avg=2, no vertex exceeds → 0.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(hub_ratio(&g).unwrap().abs() < 1e-10);
/// ```
pub fn hub_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut sum_deg = 0_usize;
    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        degrees.push(d);
        sum_deg += d;
    }

    let avg_deg = sum_deg as f64 / n as f64;

    let hub_count = degrees
        .iter()
        .filter(|&&d| d as f64 > avg_deg + 1e-15)
        .count();

    Ok(hub_count as f64 / n as f64)
}

/// Compute the leaf-to-hub ratio.
///
/// Number of degree-1 vertices divided by number of vertices whose
/// degree exceeds the average. Returns 0.0 if there are no hub
/// vertices or no leaves.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, leaf_to_hub_ratio};
///
/// // Star S_5: 4 leaves, 1 hub (center d=4, avg=8/5=1.6) → 4/1 = 4.0
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert!((leaf_to_hub_ratio(&g).unwrap() - 4.0).abs() < 1e-10);
/// ```
pub fn leaf_to_hub_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut sum_deg = 0_usize;
    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        degrees.push(d);
        sum_deg += d;
    }

    let avg_deg = sum_deg as f64 / n as f64;

    let leaf_count = degrees.iter().filter(|&&d| d == 1).count();
    let hub_count = degrees
        .iter()
        .filter(|&&d| d as f64 > avg_deg + 1e-15)
        .count();

    if hub_count == 0 || leaf_count == 0 {
        return Ok(0.0);
    }

    Ok(leaf_count as f64 / hub_count as f64)
}

/// Compute Freeman's degree centralization.
///
/// `C_D = Σ (d_max - d(v)) / ((n-1)(n-2))`
///
/// The sum of deviations from the maximum degree, normalized by
/// the theoretical maximum for a star graph. Ranges from 0 (regular
/// graphs) to 1 (star-like). Returns 0.0 for graphs with fewer
/// than 3 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, freeman_degree_centralization};
///
/// // Star S_5: max=4, deviations=[0,3,3,3,3]=12, denom=(4)(3)=12 → 1.0
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert!((freeman_degree_centralization(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn freeman_degree_centralization(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let mut max_deg = 0_usize;
    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        degrees.push(d);
        if d > max_deg {
            max_deg = d;
        }
    }

    let mut deviation_sum = 0_usize;
    for &d in &degrees {
        deviation_sum += max_deg - d;
    }

    let denom = (n - 1) * (n - 2);
    if denom == 0 {
        return Ok(0.0);
    }

    Ok(deviation_sum as f64 / denom as f64)
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

    // --- avg_neighbor_degree_ratio ---

    #[test]
    fn andr_empty() {
        let g = Graph::with_vertices(0);
        assert!(avg_neighbor_degree_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn andr_isolated() {
        let g = Graph::with_vertices(5);
        assert!(avg_neighbor_degree_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn andr_single_edge() {
        // Both d=1, neighbor d=1, ratio=1/1=1 → avg=1.0
        assert!((avg_neighbor_degree_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn andr_k3() {
        // All d=2, avg_neigh=2, ratio=2/2=1 → avg=1.0
        assert!((avg_neighbor_degree_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn andr_k4() {
        // All d=3, ratio=1 → avg=1.0
        assert!((avg_neighbor_degree_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn andr_cycle4() {
        // All d=2, ratio=1 → avg=1.0
        assert!((avg_neighbor_degree_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn andr_star5() {
        // Center(d=4): avg_neigh=1, ratio=1/4=0.25
        // Each leaf(d=1): avg_neigh=4, ratio=4/1=4
        // avg = (0.25 + 4*4) / 5 = (0.25 + 16) / 5 = 16.25/5 = 3.25
        assert!((avg_neighbor_degree_ratio(&star5()).unwrap() - 3.25).abs() < 1e-10);
    }

    // --- hub_ratio ---

    #[test]
    fn hr_empty() {
        let g = Graph::with_vertices(0);
        assert!(hub_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hr_isolated() {
        let g = Graph::with_vertices(5);
        assert!(hub_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hr_k3() {
        // All d=2, avg=2, none exceed → 0
        assert!(hub_ratio(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hr_k4() {
        // All d=3, avg=3, none exceed → 0
        assert!(hub_ratio(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hr_cycle4() {
        // All d=2, none exceed → 0
        assert!(hub_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hr_star5() {
        // degrees: [4,1,1,1,1], avg=8/5=1.6
        // Center d=4 > 1.6 → 1 hub, 1/5 = 0.2
        assert!((hub_ratio(&star5()).unwrap() - 0.2).abs() < 1e-10);
    }

    #[test]
    fn hr_path3() {
        // degrees: [1,2,1], avg=4/3≈1.33
        // v1: d=2 > 1.33 → 1 hub, 1/3
        assert!((hub_ratio(&path3()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn hr_paw() {
        // degrees: [2,2,3,1], avg=8/4=2
        // v2: d=3 > 2 → 1 hub, 1/4 = 0.25
        assert!((hub_ratio(&paw()).unwrap() - 0.25).abs() < 1e-10);
    }

    // --- leaf_to_hub_ratio ---

    #[test]
    fn lthr_empty() {
        let g = Graph::with_vertices(0);
        assert!(leaf_to_hub_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn lthr_isolated() {
        let g = Graph::with_vertices(5);
        assert!(leaf_to_hub_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn lthr_k3() {
        // No leaves, no hubs → 0
        assert!(leaf_to_hub_ratio(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn lthr_star5() {
        // 4 leaves, 1 hub → 4/1 = 4.0
        assert!((leaf_to_hub_ratio(&star5()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn lthr_path3() {
        // degrees: [1,2,1], avg≈1.33
        // 2 leaves, 1 hub → 2/1 = 2.0
        assert!((leaf_to_hub_ratio(&path3()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn lthr_paw() {
        // degrees: [2,2,3,1], avg=2
        // 1 leaf (v3), 1 hub (v2 d=3 > 2) → 1/1 = 1.0
        assert!((leaf_to_hub_ratio(&paw()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn lthr_k4() {
        // No leaves → 0
        assert!(leaf_to_hub_ratio(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn lthr_cycle4() {
        // No leaves → 0
        assert!(leaf_to_hub_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    // --- freeman_degree_centralization ---

    #[test]
    fn fdc_empty() {
        let g = Graph::with_vertices(0);
        assert!(freeman_degree_centralization(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fdc_single() {
        let g = Graph::with_vertices(1);
        assert!(freeman_degree_centralization(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fdc_two() {
        let g = Graph::with_vertices(2);
        assert!(freeman_degree_centralization(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fdc_k3() {
        // Regular → 0
        assert!(freeman_degree_centralization(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fdc_k4() {
        // Regular → 0
        assert!(freeman_degree_centralization(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fdc_cycle4() {
        // Regular → 0
        assert!(freeman_degree_centralization(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fdc_star5() {
        // max=4, deviations=[0,3,3,3,3]=12, denom=(4)(3)=12 → 1.0
        assert!((freeman_degree_centralization(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn fdc_path3() {
        // max=2, deviations=[1,0,1]=2, denom=(2)(1)=2 → 1.0
        assert!((freeman_degree_centralization(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn fdc_paw() {
        // max=3, deviations=[1,1,0,2]=4, denom=(3)(2)=6 → 4/6 = 2/3
        assert!((freeman_degree_centralization(&paw()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn andr_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(avg_neighbor_degree_ratio(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn hr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = hub_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn lthr_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(leaf_to_hub_ratio(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn fdc_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = freeman_degree_centralization(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn regular_graphs_zero_centralization() {
        assert!(freeman_degree_centralization(&k3()).unwrap().abs() < 1e-10);
        assert!(freeman_degree_centralization(&k4()).unwrap().abs() < 1e-10);
        assert!(freeman_degree_centralization(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn regular_graphs_unit_ratio() {
        // In regular graphs, avg_neighbor_degree_ratio = 1.0
        assert!((avg_neighbor_degree_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((avg_neighbor_degree_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((avg_neighbor_degree_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }
}
