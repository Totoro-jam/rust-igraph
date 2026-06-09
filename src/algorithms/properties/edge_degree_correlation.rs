//! Edge degree correlation indices (ALGO-TR-093).
//!
//! Correlation-style measures between degrees at edge endpoints:
//!
//! - **Degree covariance (edges)** — Cov(d(u), d(v)) over edges
//! - **Degree Pearson (edges)** — Pearson r of (d(u), d(v)) pairs
//! - **Degree cosine similarity** — cosine of degree vectors over edges
//! - **Degree discrepancy** — Σ (d(u) - d(v))² / (4m) normalized squared diff

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the degree covariance across edges.
///
/// `Cov = (1/m) Σ_{(u,v)∈E} d(u)·d(v) - [(1/m) Σ d(u)]·[(1/m) Σ d(v)]`
///
/// where each edge contributes a pair (d(u), d(v)). Positive covariance
/// means high-degree vertices tend to connect to high-degree vertices
/// (assortative). Returns 0.0 for the empty or single-edge graph.
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_covariance};
///
/// // K_3: 3 edges, all (2,2) → Cov = 4 - 2·2 = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(edge_degree_covariance(&g).unwrap().abs() < 1e-10);
/// ```
pub fn edge_degree_covariance(graph: &Graph) -> IgraphResult<f64> {
    let mut m = 0_u64;
    let mut sum_u = 0.0_f64;
    let mut sum_v = 0.0_f64;
    let mut sum_uv = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        sum_u += du;
        sum_v += dv;
        sum_uv += du * dv;
        m += 1;
    }

    if m == 0 {
        return Ok(0.0);
    }

    let mf = m as f64;
    let mean_u = sum_u / mf;
    let mean_v = sum_v / mf;
    let cov = sum_uv / mf - mean_u * mean_v;

    Ok(cov)
}

/// Compute the Pearson correlation of endpoint degrees across edges.
///
/// The standard Pearson r between the two degree sequences formed by
/// edge endpoints. Related to but not identical to `degree_assortativity`
/// (Newman's definition uses a different normalization).
///
/// Returns 0.0 for the empty graph or when variance is zero (regular
/// graphs). Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_pearson};
///
/// // K_3: all (2,2) → Var=0 → r=0.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(edge_degree_pearson(&g).unwrap().abs() < 1e-10);
/// ```
pub fn edge_degree_pearson(graph: &Graph) -> IgraphResult<f64> {
    let mut m = 0_u64;
    let mut sum_u = 0.0_f64;
    let mut sum_v = 0.0_f64;
    let mut sum_uu = 0.0_f64;
    let mut sum_vv = 0.0_f64;
    let mut sum_uv = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        sum_u += du;
        sum_v += dv;
        sum_uu += du * du;
        sum_vv += dv * dv;
        sum_uv += du * dv;
        m += 1;
    }

    if m == 0 {
        return Ok(0.0);
    }

    let mf = m as f64;
    let var_u = sum_uu / mf - (sum_u / mf) * (sum_u / mf);
    let var_v = sum_vv / mf - (sum_v / mf) * (sum_v / mf);
    let cov = sum_uv / mf - (sum_u / mf) * (sum_v / mf);

    let denom = (var_u * var_v).sqrt();
    if denom < 1e-15 {
        return Ok(0.0);
    }

    Ok(cov / denom)
}

/// Compute the cosine similarity of degree vectors across edges.
///
/// `cos = Σ d(u)·d(v) / √(Σ d(u)² · Σ d(v)²)`
///
/// where the sums run over all edges. Treats each edge endpoint as
/// a component of two vectors. Returns 0.0 for the empty graph.
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_cosine};
///
/// // K_3: all (2,2) → cos = 3·4 / √(3·4 · 3·4) = 12/12 = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_degree_cosine(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn edge_degree_cosine(graph: &Graph) -> IgraphResult<f64> {
    let mut sum_uv = 0.0_f64;
    let mut sum_uu = 0.0_f64;
    let mut sum_vv = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        sum_uv += du * dv;
        sum_uu += du * du;
        sum_vv += dv * dv;
    }

    let denom = (sum_uu * sum_vv).sqrt();
    if denom < 1e-15 {
        return Ok(0.0);
    }

    Ok(sum_uv / denom)
}

/// Compute the normalized degree discrepancy across edges.
///
/// `D = Σ_{(u,v)∈E} (d(u) - d(v))² / (4m)`
///
/// Measures how differently-connected each edge's endpoints are,
/// normalized by edge count. Zero for regular graphs. Self-loops
/// are skipped. Returns 0.0 for the empty graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_discrepancy};
///
/// // K_3: all (2,2) → (0)²/12 = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(edge_degree_discrepancy(&g).unwrap().abs() < 1e-10);
/// ```
pub fn edge_degree_discrepancy(graph: &Graph) -> IgraphResult<f64> {
    let mut m = 0_u64;
    let mut sum_sq = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let diff = du - dv;
        sum_sq += diff * diff;
        m += 1;
    }

    if m == 0 {
        return Ok(0.0);
    }

    Ok(sum_sq / (4.0 * m as f64))
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

    // --- edge_degree_covariance ---

    #[test]
    fn cov_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_degree_covariance(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cov_isolated() {
        let g = Graph::with_vertices(5);
        assert!(edge_degree_covariance(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cov_regular() {
        // Regular: all (r,r), Cov = r² - r·r = 0
        assert!(edge_degree_covariance(&k3()).unwrap().abs() < 1e-10);
        assert!(edge_degree_covariance(&k4()).unwrap().abs() < 1e-10);
        assert!(edge_degree_covariance(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cov_single_edge() {
        // (1,1) → Cov = 1 - 1·1 = 0
        assert!(edge_degree_covariance(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cov_star5() {
        // 4 edges all (4,1): sum_uv=4·4=16, sum_u=4·4=16, sum_v=4·1=4
        // Cov = 16/4 - (16/4)·(4/4) = 4 - 4·1 = 0
        assert!(edge_degree_covariance(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cov_path3() {
        // 2 edges: (0,1)→(1,2), (1,2)→(2,1) [stored u<v]
        // Edge (0,1): d(0)=1, d(1)=2
        // Edge (1,2): d(1)=2, d(2)=1
        // sum_u = 1+2=3, sum_v = 2+1=3
        // sum_uv = 1·2 + 2·1 = 4
        // Cov = 4/2 - (3/2)·(3/2) = 2 - 2.25 = -0.25
        assert!((edge_degree_covariance(&path3()).unwrap() - (-0.25)).abs() < 1e-10);
    }

    #[test]
    fn cov_paw() {
        // Edges (u<v): (0,1),(0,2),(1,2),(2,3)
        // d: 0→2, 1→2, 2→3, 3→1
        // (0,1): (2,2), (0,2): (2,3), (1,2): (2,3), (2,3): (3,1)
        // sum_u = 2+2+2+3=9, sum_v = 2+3+3+1=9
        // sum_uv = 4+6+6+3=19
        // Cov = 19/4 - (9/4)² = 4.75 - 5.0625 = -0.3125
        assert!((edge_degree_covariance(&paw()).unwrap() - (-0.3125)).abs() < 1e-10);
    }

    // --- edge_degree_pearson ---

    #[test]
    fn pearson_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_degree_pearson(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn pearson_regular() {
        // Var=0 → r=0
        assert!(edge_degree_pearson(&k3()).unwrap().abs() < 1e-10);
        assert!(edge_degree_pearson(&k4()).unwrap().abs() < 1e-10);
        assert!(edge_degree_pearson(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn pearson_single_edge() {
        // Var=0 → r=0
        assert!(edge_degree_pearson(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn pearson_star5() {
        // Edges (4,1),(4,1),(4,1),(4,1): u always 4, Var_u=0 → r=0
        assert!(edge_degree_pearson(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn pearson_path3() {
        // u=[1,2], v=[2,1]: Cov=-0.25, Var_u=0.25, Var_v=0.25
        // r = -0.25/sqrt(0.25·0.25) = -0.25/0.25 = -1.0
        assert!((edge_degree_pearson(&path3()).unwrap() - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn pearson_in_range() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let r = edge_degree_pearson(g).unwrap();
            assert!(r >= -1.0 - 1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- edge_degree_cosine ---

    #[test]
    fn cos_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_degree_cosine(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cos_regular() {
        // All identical → cos = 1.0
        assert!((edge_degree_cosine(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((edge_degree_cosine(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((edge_degree_cosine(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cos_single_edge() {
        // (1,1): cos = 1/(1·1) = 1.0
        assert!((edge_degree_cosine(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cos_star5() {
        // u=[4,4,4,4], v=[1,1,1,1]
        // sum_uv=16, sum_uu=64, sum_vv=4
        // cos = 16/sqrt(64·4) = 16/16 = 1.0
        assert!((edge_degree_cosine(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cos_path3() {
        // u=[1,2], v=[2,1]
        // sum_uv=1·2+2·1=4, sum_uu=1+4=5, sum_vv=4+1=5
        // cos = 4/sqrt(25) = 4/5 = 0.8
        assert!((edge_degree_cosine(&path3()).unwrap() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn cos_paw() {
        // u=[2,2,2,3], v=[2,3,3,1]
        // sum_uv=4+6+6+3=19, sum_uu=4+4+4+9=21, sum_vv=4+9+9+1=23
        // cos = 19/sqrt(21·23) = 19/sqrt(483)
        let expected = 19.0 / (21.0_f64 * 23.0).sqrt();
        assert!((edge_degree_cosine(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn cos_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let c = edge_degree_cosine(g).unwrap();
            assert!(c >= -1e-10);
            assert!(c <= 1.0 + 1e-10);
        }
    }

    // --- edge_degree_discrepancy ---

    #[test]
    fn disc_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_degree_discrepancy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn disc_regular() {
        // All diffs=0 → 0
        assert!(edge_degree_discrepancy(&k3()).unwrap().abs() < 1e-10);
        assert!(edge_degree_discrepancy(&k4()).unwrap().abs() < 1e-10);
        assert!(edge_degree_discrepancy(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn disc_single_edge() {
        // (1,1) → 0²/(4·1) = 0
        assert!(edge_degree_discrepancy(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn disc_star5() {
        // 4 edges, each (4-1)²=9
        // D = 4·9 / (4·4) = 36/16 = 2.25
        assert!((edge_degree_discrepancy(&star5()).unwrap() - 2.25).abs() < 1e-10);
    }

    #[test]
    fn disc_path3() {
        // 2 edges, each (1-2)²=1
        // D = 2·1 / (4·2) = 2/8 = 0.25
        assert!((edge_degree_discrepancy(&path3()).unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn disc_paw() {
        // (0,1):(2-2)²=0, (0,2):(2-3)²=1, (1,2):(2-3)²=1, (2,3):(3-1)²=4
        // D = (0+1+1+4) / (4·4) = 6/16 = 3/8
        assert!((edge_degree_discrepancy(&paw()).unwrap() - 3.0 / 8.0).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn disc_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(edge_degree_discrepancy(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn cov_zero_implies_disc_related() {
        // When cov=0 (regular), discrepancy=0 too
        for g in &[k3(), k4(), cycle4()] {
            assert!(edge_degree_covariance(g).unwrap().abs() < 1e-10);
            assert!(edge_degree_discrepancy(g).unwrap().abs() < 1e-10);
        }
    }

    #[test]
    fn cos_ge_pearson_squared() {
        // cosine ≥ 0 always (degrees are positive)
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(edge_degree_cosine(g).unwrap() >= -1e-10);
        }
    }
}
