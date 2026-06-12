//! Centrality diversity indices (ALGO-TR-121).
//!
//! Measures of how consistently different centrality measures rank vertices:
//!
//! - **Centrality entropy** — Shannon entropy of the normalized centrality
//!   distribution, measuring how evenly importance is spread
//! - **Centrality divergence** — Jensen-Shannon divergence between degree
//!   centrality and betweenness centrality distributions, measuring how
//!   differently these two perspectives rank vertices
//! - **Rank correlation** — Spearman rank correlation between degree and
//!   betweenness centrality, measuring monotonic agreement

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

/// Compute the Shannon entropy of the degree centrality distribution.
///
/// Normalizes degrees to a probability distribution and computes
/// `H = -sum(p_i * ln(p_i))`. Higher values indicate more evenly
/// distributed importance; lower values indicate concentration around
/// a few hubs. Returns 0.0 for trivial or edgeless graphs.
///
/// The result is normalized by `ln(n)` to give a value in `[0, 1]`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, centrality_entropy};
///
/// // K_4: all degrees equal → maximum entropy = 1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// let h = centrality_entropy(&g).unwrap();
/// assert!((h - 1.0).abs() < 1e-10);
/// ```
pub fn centrality_entropy(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    let mut sum = 0_u64;
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        degrees.push(d);
        sum += d as u64;
    }

    if sum == 0 {
        return Ok(0.0);
    }

    let sum_f = sum as f64;
    let mut entropy = 0.0_f64;
    for &d in &degrees {
        if d > 0 {
            let p = d as f64 / sum_f;
            entropy -= p * p.ln();
        }
    }

    // Normalize by ln(n) to get [0, 1]
    let max_entropy = (n as f64).ln();
    if max_entropy > 0.0 {
        Ok(entropy / max_entropy)
    } else {
        Ok(0.0)
    }
}

/// Compute the Jensen-Shannon divergence between degree centrality and
/// betweenness centrality distributions.
///
/// Both centrality vectors are normalized to probability distributions,
/// then JSD = (KL(P||M) + KL(Q||M)) / 2 where M = (P+Q)/2.
/// Returns a value in `[0, ln(2)]` (or 0.0 for trivial graphs).
/// Higher values indicate that degree and betweenness rank vertices
/// very differently.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, centrality_divergence};
///
/// // Star graph: hub has high degree AND high betweenness → low divergence
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(0,4)], false, Some(5)
/// ).unwrap();
/// let jsd = centrality_divergence(&g).unwrap();
/// assert!(jsd < 0.3); // relatively low divergence
/// ```
pub fn centrality_divergence(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    // Degree centrality
    let mut degrees = Vec::with_capacity(n);
    let mut deg_sum = 0_u64;
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        degrees.push(d as f64);
        deg_sum += d as u64;
    }

    if deg_sum == 0 {
        return Ok(0.0);
    }

    // Betweenness centrality
    let bc = crate::algorithms::properties::betweenness::betweenness(graph)?;

    let bc_sum: f64 = bc.iter().sum();

    // If betweenness is all zero (e.g., complete graph), divergence is 0
    if bc_sum <= 0.0 {
        return Ok(0.0);
    }

    // Normalize to probability distributions
    let deg_sum_f = deg_sum as f64;
    let p: Vec<f64> = degrees.iter().map(|&d| d / deg_sum_f).collect();
    let q: Vec<f64> = bc.iter().map(|&b| b / bc_sum).collect();

    // Jensen-Shannon divergence
    let mut jsd = 0.0_f64;
    for i in 0..n {
        let m_i = f64::midpoint(p[i], q[i]);
        if m_i > 0.0 {
            if p[i] > 0.0 {
                jsd += p[i] * (p[i] / m_i).ln();
            }
            if q[i] > 0.0 {
                jsd += q[i] * (q[i] / m_i).ln();
            }
        }
    }
    jsd /= 2.0;

    Ok(jsd)
}

/// Compute the Spearman rank correlation between degree centrality and
/// betweenness centrality.
///
/// Returns a value in `[-1, 1]`. Values near 1 indicate that vertices
/// with high degree also have high betweenness (consistent importance).
/// Values near 0 indicate no monotonic relationship. Returns 0.0 for
/// trivial graphs or graphs where one centrality is constant.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, centrality_rank_correlation};
///
/// // Path graph: degree and betweenness are inversely related at endpoints
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,4)], false, Some(5)
/// ).unwrap();
/// let rho = centrality_rank_correlation(&g).unwrap();
/// // For a path, internal vertices have both higher degree and betweenness
/// assert!(rho > 0.5);
/// ```
pub fn centrality_rank_correlation(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    // Degree centrality
    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)? as f64);
    }

    // Betweenness centrality
    let bc = crate::algorithms::properties::betweenness::betweenness(graph)?;

    // Compute ranks (average rank for ties)
    let deg_ranks = compute_ranks(&degrees);
    let bc_ranks = compute_ranks(&bc);

    // Spearman correlation = Pearson correlation of ranks
    Ok(pearson_correlation(&deg_ranks, &bc_ranks))
}

/// Compute average ranks with tie-breaking (average rank for tied values).
fn compute_ranks(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut indexed: Vec<(usize, f64)> = values.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut ranks = vec![0.0_f64; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        // Find all tied values
        while j < n && (indexed[j].1 - indexed[i].1).abs() < 1e-12 {
            j += 1;
        }
        // Average rank for the tied group (1-based): positions i..j → ranks (i+1)..=j
        // Average = (i + 1 + j) / 2
        let avg_rank = (i + 1 + j) as f64 / 2.0;
        for k in i..j {
            ranks[indexed[k].0] = avg_rank;
        }
        i = j;
    }
    ranks
}

/// Pearson correlation coefficient between two vectors.
fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len();
    if n == 0 {
        return 0.0;
    }

    let mean_x: f64 = x.iter().sum::<f64>() / n as f64;
    let mean_y: f64 = y.iter().sum::<f64>() / n as f64;

    let mut cov = 0.0_f64;
    let mut var_x = 0.0_f64;
    let mut var_y = 0.0_f64;

    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    let denom = (var_x * var_y).sqrt();
    if denom < 1e-15 {
        return 0.0;
    }

    cov / denom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entropy_empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(centrality_entropy(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn entropy_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(centrality_entropy(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn entropy_regular_graph_is_one() {
        // K_4: all degrees equal → max entropy
        let g =
            Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)], false, Some(4))
                .unwrap();
        let h = centrality_entropy(&g).unwrap();
        assert!((h - 1.0).abs() < 1e-10, "K4 entropy = {h}, expected 1.0");
    }

    #[test]
    fn entropy_star_is_low() {
        // Star: one hub with high degree, leaves with degree 1
        let g = Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4), (0, 5)], false, Some(6))
            .unwrap();
        let h = centrality_entropy(&g).unwrap();
        // Not maximum entropy
        assert!(h < 0.95, "Star entropy = {h}, should be < 0.95");
        assert!(h > 0.0, "Star entropy should be positive");
    }

    #[test]
    fn divergence_empty() {
        let g = Graph::with_vertices(2);
        assert!(centrality_divergence(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn divergence_complete_graph() {
        // K_4: betweenness is all zero → divergence is 0
        let g =
            Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)], false, Some(4))
                .unwrap();
        let jsd = centrality_divergence(&g).unwrap();
        assert!(jsd.abs() < 1e-12);
    }

    #[test]
    fn divergence_path_graph() {
        // Path: degree and betweenness differ
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)], false, Some(5)).unwrap();
        let jsd = centrality_divergence(&g).unwrap();
        assert!(jsd > 0.0, "Path JSD should be positive, got {jsd}");
        assert!(
            jsd < std::f64::consts::LN_2,
            "JSD should be < ln(2), got {jsd}"
        );
    }

    #[test]
    fn rank_correlation_empty() {
        let g = Graph::with_vertices(2);
        assert!(centrality_rank_correlation(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn rank_correlation_path() {
        // Path: internal vertices have both higher degree and betweenness
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)], false, Some(5)).unwrap();
        let rho = centrality_rank_correlation(&g).unwrap();
        assert!(
            rho > 0.5,
            "Path rank correlation should be > 0.5, got {rho}"
        );
    }

    #[test]
    fn rank_correlation_star() {
        // Star: hub has both max degree and max betweenness
        let g = Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap();
        let rho = centrality_rank_correlation(&g).unwrap();
        // All leaves have same degree and same betweenness → correlation should be high
        assert!(
            rho > 0.8,
            "Star rank correlation should be > 0.8, got {rho}"
        );
    }

    #[test]
    fn rank_correlation_edgeless() {
        // Edgeless: all degrees 0, all betweenness 0 → correlation 0
        let g = Graph::with_vertices(5);
        let rho = centrality_rank_correlation(&g).unwrap();
        assert!(rho.abs() < 1e-12);
    }

    #[test]
    fn compute_ranks_basic() {
        let values = vec![3.0, 1.0, 2.0, 1.0];
        let ranks = compute_ranks(&values);
        // 1.0 appears twice → average rank (1+2)/2 = 1.5
        assert!((ranks[0] - 4.0).abs() < 1e-10); // 3.0 → rank 4
        assert!((ranks[1] - 1.5).abs() < 1e-10); // 1.0 → rank 1.5
        assert!((ranks[2] - 3.0).abs() < 1e-10); // 2.0 → rank 3
        assert!((ranks[3] - 1.5).abs() < 1e-10); // 1.0 → rank 1.5
    }
}
