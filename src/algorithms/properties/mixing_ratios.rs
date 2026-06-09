//! Mixing-pattern ratio indices (ALGO-TR-107).
//!
//! Ratios capturing degree mixing and correlation patterns:
//!
//! - **Degree assortativity proxy** — Pearson r of degree at edge endpoints
//! - **Rich club density** — density among top-k% degree vertices
//! - **Degree mixing entropy** — Shannon entropy of the degree-pair
//!   distribution over edges
//! - **Hub dominance ratio** — fraction of edges incident to the
//!   highest-degree vertex

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the degree assortativity proxy.
///
/// Pearson correlation coefficient of degrees at edge endpoints.
/// Positive values indicate assortative mixing (high-degree nodes
/// connect to high-degree nodes). Returns 0.0 for graphs with
/// fewer than 2 edges or constant degree.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_assortativity_proxy};
///
/// // K_3: all degrees equal → 0.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_assortativity_proxy(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_assortativity_proxy(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    if m < 2 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }

    let mut sum_x = 0.0_f64;
    let mut sum_y = 0.0_f64;
    let mut sum_xx = 0.0_f64;
    let mut sum_yy = 0.0_f64;
    let mut sum_xy = 0.0_f64;
    let mut edge_count = 0_u64;

    for v in 0..n {
        let neighbors = graph.neighbors(v as u32)?;
        for &u in &neighbors {
            let ui = u as usize;
            if !graph.is_directed() && ui <= v {
                continue;
            }
            let dx = degrees[v] as f64;
            let dy = degrees[ui] as f64;
            sum_x += dx;
            sum_y += dy;
            sum_xx += dx * dx;
            sum_yy += dy * dy;
            sum_xy += dx * dy;
            edge_count += 1;
        }
    }

    if edge_count < 2 {
        return Ok(0.0);
    }

    let mf = edge_count as f64;
    let mean_x = sum_x / mf;
    let mean_y = sum_y / mf;
    let var_x = sum_xx / mf - mean_x * mean_x;
    let var_y = sum_yy / mf - mean_y * mean_y;

    if var_x < 1e-30 || var_y < 1e-30 {
        return Ok(0.0);
    }

    let cov = sum_xy / mf - mean_x * mean_y;
    Ok(cov / (var_x.sqrt() * var_y.sqrt()))
}

/// Compute the rich club density.
///
/// Density of the subgraph induced by the top-25% highest-degree
/// vertices (at least 2 vertices). This measures how interconnected
/// the hubs are. Returns 0.0 for trivial graphs or when fewer than
/// 2 vertices qualify.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, rich_club_density};
///
/// // K_4: all vertices in top 25% → density = 1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((rich_club_density(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn rich_club_density(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }

    let mut sorted_degs: Vec<(usize, usize)> = degrees.iter().copied().enumerate().collect();
    sorted_degs.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    let k = (n / 4).max(2).min(n);
    let rich_set: Vec<usize> = sorted_degs[..k].iter().map(|&(v, _)| v).collect();

    if rich_set.len() < 2 {
        return Ok(0.0);
    }

    let mut in_rich = vec![false; n];
    for &v in &rich_set {
        in_rich[v] = true;
    }

    let mut edges_in_rich = 0_u64;
    for &v in &rich_set {
        let neighbors = graph.neighbors(v as u32)?;
        for &u in &neighbors {
            let ui = u as usize;
            if in_rich[ui] && (graph.is_directed() || ui > v) {
                edges_in_rich += 1;
            }
        }
    }

    let rs = rich_set.len();
    let max_edges = if graph.is_directed() {
        rs * (rs - 1)
    } else {
        rs * (rs - 1) / 2
    };

    if max_edges == 0 {
        return Ok(0.0);
    }

    Ok(edges_in_rich as f64 / max_edges as f64)
}

/// Compute the degree mixing entropy.
///
/// Shannon entropy of the distribution of degree pairs `(d_u, d_v)`
/// over all edges, normalized by `log2(m)` where m is the edge count.
/// Higher values indicate more diverse degree mixing. Returns 0.0
/// for graphs with fewer than 2 edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_mixing_entropy};
///
/// // K_3: all edges have same degree pair (2,2) → entropy = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_mixing_entropy(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_mixing_entropy(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    if m < 2 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }

    let mut pair_counts = std::collections::HashMap::new();
    let mut edge_count = 0_u64;

    for v in 0..n {
        let neighbors = graph.neighbors(v as u32)?;
        for &u in &neighbors {
            let ui = u as usize;
            if !graph.is_directed() && ui <= v {
                continue;
            }
            let (da, db) = if degrees[v] <= degrees[ui] {
                (degrees[v], degrees[ui])
            } else {
                (degrees[ui], degrees[v])
            };
            *pair_counts.entry((da, db)).or_insert(0_u64) += 1;
            edge_count += 1;
        }
    }

    if edge_count < 2 {
        return Ok(0.0);
    }

    let mf = edge_count as f64;
    let log_m = mf.log2();
    if log_m < 1e-30 {
        return Ok(0.0);
    }

    let mut entropy = 0.0_f64;
    for &count in pair_counts.values() {
        let p = count as f64 / mf;
        if p > 0.0 {
            entropy -= p * p.log2();
        }
    }

    Ok(entropy / log_m)
}

/// Compute the hub dominance ratio.
///
/// Fraction of edges that are incident to the highest-degree vertex.
/// Measures how much the network depends on a single hub. Returns
/// 0.0 for empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, hub_dominance_ratio};
///
/// // Star_5: center has 4 edges out of 4 total → 1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(0,4)], false, Some(5)
/// ).unwrap();
/// assert!((hub_dominance_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn hub_dominance_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    if m == 0 || n == 0 {
        return Ok(0.0);
    }

    let mut max_deg = 0_usize;
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d > max_deg {
            max_deg = d;
        }
    }

    Ok(max_deg as f64 / m as f64)
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

    // --- degree_assortativity_proxy ---

    #[test]
    fn dap_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_assortativity_proxy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dap_single() {
        let g = Graph::with_vertices(1);
        assert!(degree_assortativity_proxy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dap_single_edge() {
        // Only 1 edge → 0
        assert!(degree_assortativity_proxy(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dap_k3() {
        // All same degree → 0
        assert!(degree_assortativity_proxy(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dap_k4() {
        assert!(degree_assortativity_proxy(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dap_cycle4() {
        assert!(degree_assortativity_proxy(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dap_star5() {
        // Star: all edges have same pattern (4,1) → zero variance → 0.0
        let r = degree_assortativity_proxy(&star5()).unwrap();
        assert!(r.abs() < 1e-10);
    }

    #[test]
    fn dap_in_range() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = degree_assortativity_proxy(g).unwrap();
            assert!(r >= -1.0 - 1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- rich_club_density ---

    #[test]
    fn rcd_empty() {
        let g = Graph::with_vertices(0);
        assert!(rich_club_density(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rcd_single() {
        let g = Graph::with_vertices(1);
        assert!(rich_club_density(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rcd_k4() {
        // n=4, top 25% = max(4/4, 2) = 2 vertices (top 2 by degree, all deg=3)
        // Those 2 are connected → density = 1/(1) = 1.0
        assert!((rich_club_density(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rcd_star5() {
        // n=5, top 25% = max(5/4=1, 2) = 2 vertices
        // Top 2 by degree: center(4) + one leaf(1)
        // They're connected → density = 1/1 = 1.0
        assert!((rich_club_density(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rcd_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = rich_club_density(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- degree_mixing_entropy ---

    #[test]
    fn dme_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_mixing_entropy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dme_single() {
        let g = Graph::with_vertices(1);
        assert!(degree_mixing_entropy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dme_single_edge() {
        assert!(degree_mixing_entropy(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dme_k3() {
        // All edges (2,2) → one bin → entropy=0
        assert!(degree_mixing_entropy(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dme_k4() {
        // All edges (3,3) → one bin → entropy=0
        assert!(degree_mixing_entropy(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dme_cycle4() {
        // All edges (2,2) → one bin → entropy=0
        assert!(degree_mixing_entropy(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dme_star5() {
        // All edges (1,4) → one bin → entropy=0
        assert!(degree_mixing_entropy(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dme_paw() {
        // Edges: (0,1)→(2,2), (0,2)→(2,3), (1,2)→(2,3), (2,3)→(3,1)→(1,3)
        // Degree pairs sorted: (2,2):1, (2,3):2, (1,3):1
        // 3 distinct bins, 4 edges, log2(4)=2
        // entropy = -(1/4*log2(1/4) + 2/4*log2(2/4) + 1/4*log2(1/4))/2
        //         = -(2*(-2/4) + (-1/2))/2 = (1 + 0.5)/2 = 1.5/2 = 0.75
        let r = degree_mixing_entropy(&paw()).unwrap();
        assert!(r > 0.0);
        assert!(r <= 1.0);
    }

    #[test]
    fn dme_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = degree_mixing_entropy(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- hub_dominance_ratio ---

    #[test]
    fn hdr_empty() {
        let g = Graph::with_vertices(0);
        assert!(hub_dominance_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hdr_single() {
        let g = Graph::with_vertices(1);
        assert!(hub_dominance_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hdr_single_edge() {
        // max_deg=1, m=1 → 1.0
        assert!((hub_dominance_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn hdr_k3() {
        // max_deg=2, m=3 → 2/3
        assert!((hub_dominance_ratio(&k3()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn hdr_k4() {
        // max_deg=3, m=6 → 3/6 = 0.5
        assert!((hub_dominance_ratio(&k4()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn hdr_star5() {
        // max_deg=4, m=4 → 1.0
        assert!((hub_dominance_ratio(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn hdr_cycle4() {
        // max_deg=2, m=4 → 0.5
        assert!((hub_dominance_ratio(&cycle4()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn hdr_path3() {
        // max_deg=2, m=2 → 1.0
        assert!((hub_dominance_ratio(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn hdr_positive() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(hub_dominance_ratio(g).unwrap() > 0.0);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn regular_zero_assortativity() {
        assert!(degree_assortativity_proxy(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_assortativity_proxy(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_assortativity_proxy(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn regular_zero_entropy() {
        assert!(degree_mixing_entropy(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_mixing_entropy(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_mixing_entropy(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn star_max_hub_dominance() {
        assert!((hub_dominance_ratio(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }
}
