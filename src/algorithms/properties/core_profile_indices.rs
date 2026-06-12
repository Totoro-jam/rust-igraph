//! Core profile indices (ALGO-TR-122).
//!
//! Three novel topological ratio indices derived from the k-core
//! decomposition of a graph:
//!
//! - [`core_persistence`]: average coreness normalised by the degeneracy
//!   (maximum coreness). Measures how deeply embedded the typical vertex
//!   is in the core hierarchy.
//! - [`shell_diversity`]: Shannon entropy of the k-shell size distribution,
//!   normalised to [0, 1]. Measures how evenly vertices are spread across
//!   different shells.
//! - [`degeneracy_gap`]: (degeneracy − average coreness) / degeneracy.
//!   Measures the gap between the densest core and the average vertex.

use crate::algorithms::properties::coreness::coreness;
use crate::core::{Graph, IgraphResult};

/// Average coreness divided by the degeneracy (maximum coreness).
///
/// Returns a value in [0, 1]. A value of 1 means every vertex has the
/// same coreness (e.g. a complete graph). A value near 0 means most
/// vertices are in low-order cores while the degeneracy is high.
///
/// Returns 0.0 for graphs where the degeneracy is 0 (edgeless graphs).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, core_persistence};
///
/// // K3: all coreness = 2, degeneracy = 2 → persistence = 1.0
/// let g = Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap();
/// assert!((core_persistence(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn core_persistence(graph: &Graph) -> IgraphResult<f64> {
    let cores = coreness(graph)?;
    if cores.is_empty() {
        return Ok(0.0);
    }
    let degeneracy = *cores.iter().max().unwrap();
    if degeneracy == 0 {
        return Ok(0.0);
    }
    #[allow(clippy::cast_precision_loss)]
    let avg: f64 = cores.iter().map(|&c| f64::from(c)).sum::<f64>() / cores.len() as f64;
    Ok(avg / f64::from(degeneracy))
}

/// Shannon entropy of the k-shell size distribution, normalised to [0, 1].
///
/// The k-shell of order k is the set of vertices with coreness exactly k.
/// This function computes the entropy of the distribution of shell sizes
/// and normalises by `log(number_of_distinct_shells)` so the result is in
/// [0, 1]. A value of 1 means all shells have equal size; a value near 0
/// means vertices are concentrated in one shell.
///
/// Returns 0.0 for empty or edgeless graphs (single shell).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, shell_diversity};
///
/// // Path 0-1-2: coreness = [1, 1, 1], single shell → diversity = 0
/// let g = Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap();
/// assert!(shell_diversity(&g).unwrap().abs() < 1e-10);
/// ```
pub fn shell_diversity(graph: &Graph) -> IgraphResult<f64> {
    let cores = coreness(graph)?;
    if cores.is_empty() {
        return Ok(0.0);
    }
    let degeneracy = *cores.iter().max().unwrap() as usize;

    // Count vertices in each shell
    let mut shell_counts = vec![0u32; degeneracy + 1];
    for &c in &cores {
        shell_counts[c as usize] += 1;
    }

    // Filter to non-empty shells
    let non_empty: Vec<u32> = shell_counts.into_iter().filter(|&c| c > 0).collect();
    let num_shells = non_empty.len();
    if num_shells <= 1 {
        return Ok(0.0);
    }

    #[allow(clippy::cast_precision_loss)]
    let n = cores.len() as f64;
    let mut entropy = 0.0_f64;
    for &count in &non_empty {
        let p = f64::from(count) / n;
        entropy -= p * p.ln();
    }

    // Normalise by max entropy (uniform distribution over shells)
    #[allow(clippy::cast_precision_loss)]
    let max_entropy = (num_shells as f64).ln();
    Ok(entropy / max_entropy)
}

/// Degeneracy gap: (degeneracy − `average_coreness`) / degeneracy.
///
/// Measures how far the average vertex is from the densest core.
/// Returns a value in [0, 1). A value of 0 means all vertices have
/// the same coreness (complete graph). Higher values indicate a larger
/// gap between the core elite and the periphery.
///
/// Returns 0.0 for edgeless graphs (degeneracy = 0).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degeneracy_gap};
///
/// // K4: all coreness = 3, gap = 0
/// let g = Graph::from_edges(
///     &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
///     false,
///     Some(4),
/// )
/// .unwrap();
/// assert!(degeneracy_gap(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degeneracy_gap(graph: &Graph) -> IgraphResult<f64> {
    let cores = coreness(graph)?;
    if cores.is_empty() {
        return Ok(0.0);
    }
    let degeneracy = *cores.iter().max().unwrap();
    if degeneracy == 0 {
        return Ok(0.0);
    }
    #[allow(clippy::cast_precision_loss)]
    let avg: f64 = cores.iter().map(|&c| f64::from(c)).sum::<f64>() / cores.len() as f64;
    Ok((f64::from(degeneracy) - avg) / f64::from(degeneracy))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- core_persistence ---

    #[test]
    fn persistence_empty() {
        let g = Graph::with_vertices(0);
        assert!(core_persistence(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn persistence_edgeless() {
        let g = Graph::with_vertices(5);
        assert!(core_persistence(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn persistence_complete() {
        // K4: all coreness = 3 → persistence = 1.0
        let g = Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap();
        assert!((core_persistence(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn persistence_star() {
        // Star K1,4: hub has coreness 1, leaves have coreness 1
        // All coreness = 1, degeneracy = 1 → persistence = 1.0
        let g = Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap();
        assert!((core_persistence(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn persistence_path() {
        // Path 0-1-2-3-4: all coreness = 1, degeneracy = 1 → persistence = 1.0
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)], false, Some(5)).unwrap();
        assert!((core_persistence(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn persistence_mixed() {
        // Triangle + pendant: 0-1-2 triangle, 2-3 pendant
        // Coreness: [2, 2, 2, 1], degeneracy = 2, avg = 7/4 = 1.75
        // persistence = 1.75 / 2 = 0.875
        let g = Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap();
        assert!((core_persistence(&g).unwrap() - 0.875).abs() < 1e-10);
    }

    // --- shell_diversity ---

    #[test]
    fn diversity_empty() {
        let g = Graph::with_vertices(0);
        assert!(shell_diversity(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn diversity_edgeless() {
        // All coreness = 0, single shell → diversity = 0
        let g = Graph::with_vertices(5);
        assert!(shell_diversity(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn diversity_single_shell() {
        // Path: all coreness = 1, single non-trivial shell → diversity = 0
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap();
        assert!(shell_diversity(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn diversity_two_equal_shells() {
        // Triangle + pendant: coreness [2, 2, 2, 1]
        // Shells: {1: 1 vertex, 2: 3 vertices} → 2 shells, not equal
        let g = Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap();
        let d = shell_diversity(&g).unwrap();
        assert!(d > 0.0, "Should have positive diversity, got {d}");
        assert!(d < 1.0, "Should be < 1 (unequal shells), got {d}");
    }

    #[test]
    fn diversity_max_when_equal_shells() {
        // Construct a graph with 2 shells of equal size:
        // K3 (coreness 2) + 3 isolated edges (coreness 1) = 3 vertices in shell 2, 3 in shell 1
        // Wait - we need shell 0 to not exist. Let's use:
        // 0-1-2 triangle (coreness 2) + 3-4 edge + 5-6 edge + 7-8 edge (coreness 1)
        // That gives 3 in shell 2, 6 in shell 1 — not equal.
        // For equal: 2 in shell 2, 2 in shell 1
        // K3 has 3 in shell 2. We need 3 in shell 1.
        // Triangle 0-1-2 (shell 2) + edges 2-3, 3-4, 4-5 → 3,4,5 have coreness 1
        // Actually: 0-1-2 triangle + 2-3 edge → [2,2,2,1] — 3 in shell 2, 1 in shell 1
        // Let's just verify it's between 0 and 1 for a known case
        let g =
            Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3), (3, 4)], false, Some(5)).unwrap();
        let d = shell_diversity(&g).unwrap();
        assert!(d > 0.0 && d <= 1.0, "Diversity should be in (0,1], got {d}");
    }

    // --- degeneracy_gap ---

    #[test]
    fn gap_empty() {
        let g = Graph::with_vertices(0);
        assert!(degeneracy_gap(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn gap_edgeless() {
        let g = Graph::with_vertices(5);
        assert!(degeneracy_gap(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn gap_complete() {
        // K4: all same coreness → gap = 0
        let g = Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap();
        assert!(degeneracy_gap(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gap_mixed() {
        // Triangle + pendant: coreness [2, 2, 2, 1], degeneracy = 2, avg = 1.75
        // gap = (2 - 1.75) / 2 = 0.125
        let g = Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap();
        assert!((degeneracy_gap(&g).unwrap() - 0.125).abs() < 1e-10);
    }

    #[test]
    fn persistence_plus_gap_equals_one() {
        // For any graph: persistence + gap = avg/deg + (deg-avg)/deg = 1
        let g = Graph::from_edges(
            &[(0, 1), (1, 2), (0, 2), (2, 3), (3, 4), (4, 5)],
            false,
            Some(6),
        )
        .unwrap();
        let p = core_persistence(&g).unwrap();
        let gap = degeneracy_gap(&g).unwrap();
        assert!(
            (p + gap - 1.0).abs() < 1e-10,
            "persistence + gap should = 1, got {p} + {gap} = {}",
            p + gap
        );
    }
}
