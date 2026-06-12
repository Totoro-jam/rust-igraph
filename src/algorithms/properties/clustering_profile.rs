//! Clustering profile indices (ALGO-TR-124).
//!
//! Three novel topological ratio indices derived from the distribution of
//! local clustering coefficients across vertices:
//!
//! - [`clustering_variance`]: variance of local clustering coefficients
//!   (measures heterogeneity of local triangle density).
//! - [`clustering_entropy`]: Shannon entropy of the binned clustering
//!   coefficient distribution, normalised to \[0, 1\].
//! - [`clustering_bimodality`]: Sarle's bimodality coefficient of the
//!   clustering coefficient distribution (values > 5/9 suggest bimodality).

use crate::algorithms::properties::triangles::transitivity_local_undirected;
use crate::core::{Graph, IgraphResult};

/// Variance of local clustering coefficients.
///
/// Computes the population variance of the local clustering coefficients
/// across all vertices with degree ≥ 2 (vertices with degree < 2 have
/// undefined clustering coefficient and are excluded).
///
/// Returns 0.0 if fewer than 2 vertices have defined clustering coefficients.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, clustering_variance};
///
/// // K4: all local CC = 1.0 → variance = 0
/// let g = Graph::from_edges(
///     &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
///     false,
///     Some(4),
/// )
/// .unwrap();
/// assert!(clustering_variance(&g).unwrap().abs() < 1e-10);
/// ```
pub fn clustering_variance(graph: &Graph) -> IgraphResult<f64> {
    let ccs = defined_clustering_coefficients(graph)?;
    if ccs.len() < 2 {
        return Ok(0.0);
    }
    #[allow(clippy::cast_precision_loss)]
    let n = ccs.len() as f64;
    let mean = ccs.iter().sum::<f64>() / n;
    let var = ccs.iter().map(|&c| (c - mean).powi(2)).sum::<f64>() / n;
    Ok(var)
}

/// Normalised Shannon entropy of the clustering coefficient distribution.
///
/// Bins the local clustering coefficients into 10 equal-width bins on
/// \[0, 1\] and computes the Shannon entropy of the resulting histogram,
/// normalised by `ln(num_non_empty_bins)` so the result is in \[0, 1\].
///
/// Returns 0.0 if fewer than 2 vertices have defined clustering coefficients
/// or if all values fall in a single bin.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, clustering_entropy};
///
/// // K4: all CC = 1.0, single bin → entropy = 0
/// let g = Graph::from_edges(
///     &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
///     false,
///     Some(4),
/// )
/// .unwrap();
/// assert!(clustering_entropy(&g).unwrap().abs() < 1e-10);
/// ```
pub fn clustering_entropy(graph: &Graph) -> IgraphResult<f64> {
    const NUM_BINS: usize = 10;

    let ccs = defined_clustering_coefficients(graph)?;
    if ccs.len() < 2 {
        return Ok(0.0);
    }

    // Bin into 10 bins: [0,0.1), [0.1,0.2), ..., [0.9,1.0]
    let mut bins = [0u32; NUM_BINS];
    for &c in &ccs {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let idx = if c >= 1.0 {
            NUM_BINS - 1
        } else {
            #[allow(clippy::cast_precision_loss)]
            let scaled = c * (NUM_BINS as f64);
            scaled as usize
        };
        bins[idx] += 1;
    }

    let non_empty: Vec<u32> = bins.iter().copied().filter(|&b| b > 0).collect();
    let num_non_empty = non_empty.len();
    if num_non_empty <= 1 {
        return Ok(0.0);
    }

    #[allow(clippy::cast_precision_loss)]
    let total = ccs.len() as f64;
    let mut entropy = 0.0_f64;
    for &count in &non_empty {
        let p = f64::from(count) / total;
        entropy -= p * p.ln();
    }

    #[allow(clippy::cast_precision_loss)]
    let max_entropy = (num_non_empty as f64).ln();
    Ok(entropy / max_entropy)
}

/// Sarle's bimodality coefficient of the clustering coefficient distribution.
///
/// Defined as `(skewness² + 1) / kurtosis` where kurtosis is the excess
/// kurtosis + 3 (i.e. the raw kurtosis). Values > 5/9 ≈ 0.556 suggest
/// a bimodal or uniform distribution; values near 1/3 suggest unimodal.
///
/// Returns 0.0 if fewer than 4 vertices have defined clustering coefficients.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, clustering_bimodality};
///
/// // K4: all CC = 1.0, zero variance → bimodality not meaningful, returns 0
/// let g = Graph::from_edges(
///     &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
///     false,
///     Some(4),
/// )
/// .unwrap();
/// assert!(clustering_bimodality(&g).unwrap().abs() < 1e-10);
/// ```
pub fn clustering_bimodality(graph: &Graph) -> IgraphResult<f64> {
    let ccs = defined_clustering_coefficients(graph)?;
    if ccs.len() < 4 {
        return Ok(0.0);
    }
    #[allow(clippy::cast_precision_loss)]
    let n = ccs.len() as f64;
    let mean = ccs.iter().sum::<f64>() / n;
    let m2 = ccs.iter().map(|&c| (c - mean).powi(2)).sum::<f64>() / n;
    if m2 < 1e-15 {
        return Ok(0.0);
    }
    let m3 = ccs.iter().map(|&c| (c - mean).powi(3)).sum::<f64>() / n;
    let m4 = ccs.iter().map(|&c| (c - mean).powi(4)).sum::<f64>() / n;

    let skewness = m3 / m2.powf(1.5);
    let kurtosis = m4 / (m2 * m2); // raw kurtosis (not excess)

    if kurtosis < 1e-15 {
        return Ok(0.0);
    }

    Ok((skewness * skewness + 1.0) / kurtosis)
}

/// Extract defined (non-None) local clustering coefficients.
fn defined_clustering_coefficients(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let local_cc = transitivity_local_undirected(graph)?;
    Ok(local_cc.into_iter().flatten().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- clustering_variance ---

    #[test]
    fn variance_empty() {
        let g = Graph::with_vertices(0);
        assert!(clustering_variance(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn variance_edgeless() {
        let g = Graph::with_vertices(5);
        assert!(clustering_variance(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn variance_complete() {
        // K4: all CC = 1.0 → variance = 0
        let g = Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap();
        assert!(clustering_variance(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn variance_mixed() {
        // Triangle + pendant: vertex 2 has CC=1/3, vertices 0,1 have CC=1.0
        // (vertex 3 has degree 1, excluded)
        // mean = (1 + 1 + 1/3) / 3 = 7/9
        // var = ((1-7/9)^2 + (1-7/9)^2 + (1/3-7/9)^2) / 3
        let g = Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap();
        let v = clustering_variance(&g).unwrap();
        assert!(
            v > 0.0,
            "Mixed graph should have positive variance, got {v}"
        );
    }

    // --- clustering_entropy ---

    #[test]
    fn entropy_empty() {
        let g = Graph::with_vertices(0);
        assert!(clustering_entropy(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn entropy_complete() {
        // K4: all CC = 1.0, single bin → entropy = 0
        let g = Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap();
        assert!(clustering_entropy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn entropy_mixed() {
        // Graph with varied CC values → positive entropy
        let g =
            Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3), (3, 4)], false, Some(5)).unwrap();
        let h = clustering_entropy(&g).unwrap();
        assert!(h >= 0.0, "Entropy should be >= 0, got {h}");
        assert!(h <= 1.0, "Entropy should be <= 1, got {h}");
    }

    // --- clustering_bimodality ---

    #[test]
    fn bimodality_empty() {
        let g = Graph::with_vertices(0);
        assert!(clustering_bimodality(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn bimodality_complete() {
        // K4: all CC = 1.0, zero variance → 0
        let g = Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap();
        assert!(clustering_bimodality(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn bimodality_positive() {
        // Larger graph with varied CC
        let g = Graph::from_edges(
            &[
                (0, 1),
                (1, 2),
                (0, 2),
                (2, 3),
                (3, 4),
                (4, 5),
                (3, 5),
                (5, 6),
                (6, 7),
                (5, 7),
            ],
            false,
            Some(8),
        )
        .unwrap();
        let b = clustering_bimodality(&g).unwrap();
        assert!(b > 0.0, "Should have positive bimodality, got {b}");
    }
}
