//! Spectral ratio indices (ALGO-TR-099).
//!
//! Lightweight spectral-inspired measures derived from degree sequences
//! (no eigenvalue computation required):
//!
//! - **Degree-based spectral gap estimate** — max degree minus second-max degree
//!   normalized by max degree
//! - **Degree variance ratio** — degree variance / max possible variance
//! - **Edge-vertex ratio** — m / n (average half-degree)
//! - **Cyclomatic density** — circuit rank / n

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the degree-based spectral gap estimate.
///
/// `(d_max - d_second) / d_max` where `d_max` is the maximum degree
/// and `d_second` is the second-largest degree. This is a rough proxy
/// for the spectral gap (difference between the two largest eigenvalues
/// of the adjacency matrix). Returns 0.0 for graphs with fewer than 2
/// vertices or where max degree is 0.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_spectral_gap_estimate};
///
/// // Star S_5: max_deg=4, second=1 → (4-1)/4 = 0.75
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert!((degree_spectral_gap_estimate(&g).unwrap() - 0.75).abs() < 1e-10);
/// ```
pub fn degree_spectral_gap_estimate(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let mut max_deg = 0_usize;
    let mut second_deg = 0_usize;

    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d >= max_deg {
            second_deg = max_deg;
            max_deg = d;
        } else if d > second_deg {
            second_deg = d;
        }
    }

    if max_deg == 0 {
        return Ok(0.0);
    }

    Ok((max_deg - second_deg) as f64 / max_deg as f64)
}

/// Compute the degree variance ratio.
///
/// `Var(d) / Var_max` where `Var_max = (n-1)² · n / (4n)` for simple
/// undirected graphs (the variance of a star on n vertices). Simplifies
/// to `Var(d) · 4 / (n-1)²`. Returns 0.0 for graphs with fewer than
/// 2 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_variance_ratio};
///
/// // K_3: all d=2, Var=0 → 0.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_variance_ratio(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_variance_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let mut sum = 0_u64;
    let mut sum_sq = 0_u64;

    for v in 0..n {
        let d = graph.degree(v as u32)? as u64;
        sum += d;
        sum_sq += d * d;
    }

    let nf = n as f64;
    let mean = sum as f64 / nf;
    let variance = sum_sq as f64 / nf - mean * mean;

    let max_deg = (n - 1) as f64;
    let var_max = max_deg * max_deg / 4.0;

    if var_max < 1e-15 {
        return Ok(0.0);
    }

    Ok((variance / var_max).min(1.0))
}

/// Compute the edge-vertex ratio.
///
/// `m / n` — the number of edges per vertex (half the average degree).
/// Returns 0.0 for graphs with no vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_vertex_ratio};
///
/// // K_3: m=3, n=3, ratio=1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_vertex_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn edge_vertex_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let m = graph.ecount() as f64;
    Ok(m / f64::from(n))
}

/// Compute the cyclomatic density.
///
/// `(m - n + c) / n` where c is the number of connected components.
/// The circuit rank divided by the number of vertices gives the
/// density of independent cycles per vertex. Returns 0.0 for graphs
/// with no vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, cyclomatic_density};
///
/// // K_3: m=3, n=3, c=1, circuit_rank=1 → 1/3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((cyclomatic_density(&g).unwrap() - 1.0/3.0).abs() < 1e-10);
/// ```
pub fn cyclomatic_density(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let m = graph.ecount() as i64;
    let ni = n as i64;
    let c = count_components(graph)? as i64;
    let circuit_rank = (m - ni + c).max(0);

    Ok(circuit_rank as f64 / n as f64)
}

fn count_components(graph: &Graph) -> IgraphResult<usize> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let mut visited = vec![false; n];
    let mut count = 0_usize;

    for start in 0..n {
        if visited[start] {
            continue;
        }
        count += 1;
        let mut stack = vec![start];
        visited[start] = true;
        while let Some(v) = stack.pop() {
            let neighbors = graph.neighbors(v as u32)?;
            for &u in &neighbors {
                let ui = u as usize;
                if !visited[ui] {
                    visited[ui] = true;
                    stack.push(ui);
                }
            }
        }
    }

    Ok(count)
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

    // --- degree_spectral_gap_estimate ---

    #[test]
    fn dsge_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_spectral_gap_estimate(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dsge_single() {
        let g = Graph::with_vertices(1);
        assert!(degree_spectral_gap_estimate(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dsge_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_spectral_gap_estimate(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dsge_single_edge() {
        // max=1, second=1 → (1-1)/1 = 0
        assert!(degree_spectral_gap_estimate(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dsge_k3() {
        // All d=2 → gap=0
        assert!(degree_spectral_gap_estimate(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dsge_k4() {
        // All d=3 → gap=0
        assert!(degree_spectral_gap_estimate(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dsge_cycle4() {
        // All d=2 → gap=0
        assert!(degree_spectral_gap_estimate(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dsge_star5() {
        // max=4, second=1 → (4-1)/4 = 0.75
        assert!((degree_spectral_gap_estimate(&star5()).unwrap() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn dsge_paw() {
        // degrees: [2,2,3,1] → max=3, second=2 → (3-2)/3 = 1/3
        assert!((degree_spectral_gap_estimate(&paw()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn dsge_path3() {
        // degrees: [1,2,1] → max=2, second=1 → (2-1)/2 = 0.5
        assert!((degree_spectral_gap_estimate(&path3()).unwrap() - 0.5).abs() < 1e-10);
    }

    // --- degree_variance_ratio ---

    #[test]
    fn dvr_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_variance_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dvr_single() {
        let g = Graph::with_vertices(1);
        assert!(degree_variance_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dvr_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_variance_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dvr_k3() {
        // All d=2, variance=0 → 0
        assert!(degree_variance_ratio(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dvr_k4() {
        // All d=3, variance=0 → 0
        assert!(degree_variance_ratio(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dvr_cycle4() {
        // All d=2, variance=0 → 0
        assert!(degree_variance_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dvr_star5() {
        // degrees: [4,1,1,1,1], mean=8/5=1.6
        // variance = (4-1.6)²·1/5 + (1-1.6)²·4/5 = (2.4²·1+0.6²·4)/5 = (5.76+1.44)/5 = 7.2/5 = 1.44
        // var_max = (n-1)²/4 = 16/4 = 4
        // ratio = 1.44/4 = 0.36
        assert!((degree_variance_ratio(&star5()).unwrap() - 0.36).abs() < 1e-10);
    }

    #[test]
    fn dvr_single_edge() {
        // degrees: [1,1], variance=0 → 0
        assert!(degree_variance_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dvr_paw() {
        // degrees: [2,2,3,1], mean=8/4=2
        // variance = ((2-2)²+(2-2)²+(3-2)²+(1-2)²)/4 = (0+0+1+1)/4 = 0.5
        // var_max = (4-1)²/4 = 9/4 = 2.25
        // ratio = 0.5/2.25 = 2/9
        assert!((degree_variance_ratio(&paw()).unwrap() - 2.0 / 9.0).abs() < 1e-10);
    }

    // --- edge_vertex_ratio ---

    #[test]
    fn evr_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_vertex_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn evr_isolated() {
        let g = Graph::with_vertices(5);
        assert!(edge_vertex_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn evr_single_edge() {
        // m=1, n=2 → 0.5
        assert!((edge_vertex_ratio(&single_edge()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn evr_k3() {
        // m=3, n=3 → 1.0
        assert!((edge_vertex_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn evr_k4() {
        // m=6, n=4 → 1.5
        assert!((edge_vertex_ratio(&k4()).unwrap() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn evr_cycle4() {
        // m=4, n=4 → 1.0
        assert!((edge_vertex_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn evr_star5() {
        // m=4, n=5 → 0.8
        assert!((edge_vertex_ratio(&star5()).unwrap() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn evr_path3() {
        // m=2, n=3 → 2/3
        assert!((edge_vertex_ratio(&path3()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn evr_paw() {
        // m=4, n=4 → 1.0
        assert!((edge_vertex_ratio(&paw()).unwrap() - 1.0).abs() < 1e-10);
    }

    // --- cyclomatic_density ---

    #[test]
    fn cd_empty() {
        let g = Graph::with_vertices(0);
        assert!(cyclomatic_density(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cd_isolated() {
        let g = Graph::with_vertices(5);
        assert!(cyclomatic_density(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cd_single_edge() {
        // m=1, n=2, c=1, cr=0 → 0/2 = 0
        assert!(cyclomatic_density(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cd_path3() {
        // Tree → cr=0 → 0
        assert!(cyclomatic_density(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cd_k3() {
        // m=3, n=3, c=1, cr=1 → 1/3
        assert!((cyclomatic_density(&k3()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn cd_k4() {
        // m=6, n=4, c=1, cr=3 → 3/4 = 0.75
        assert!((cyclomatic_density(&k4()).unwrap() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn cd_cycle4() {
        // m=4, n=4, c=1, cr=1 → 1/4 = 0.25
        assert!((cyclomatic_density(&cycle4()).unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn cd_star5() {
        // Tree → cr=0 → 0
        assert!(cyclomatic_density(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cd_paw() {
        // m=4, n=4, c=1, cr=1 → 1/4 = 0.25
        assert!((cyclomatic_density(&paw()).unwrap() - 0.25).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn dsge_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = degree_spectral_gap_estimate(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn dvr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = degree_variance_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn evr_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(edge_vertex_ratio(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn cd_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(cyclomatic_density(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn regular_graphs_zero_variance() {
        assert!(degree_variance_ratio(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_variance_ratio(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_variance_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn regular_graphs_zero_gap() {
        assert!(degree_spectral_gap_estimate(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_spectral_gap_estimate(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_spectral_gap_estimate(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn trees_zero_cyclomatic() {
        assert!(cyclomatic_density(&path3()).unwrap().abs() < 1e-10);
        assert!(cyclomatic_density(&star5()).unwrap().abs() < 1e-10);
        assert!(cyclomatic_density(&single_edge()).unwrap().abs() < 1e-10);
    }
}
