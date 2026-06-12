//! Distance profile indices (ALGO-TR-125).
//!
//! Three novel topological ratio indices derived from the hop-distance
//! distribution of a graph:
//!
//! - [`hop_entropy`]: Shannon entropy of the hop-distance histogram,
//!   normalised to \[0, 1\].
//! - [`distance_gini`]: Gini coefficient of the pairwise distance
//!   distribution (measures inequality of distances).
//! - [`reach_decay`]: average fraction of vertices reachable within
//!   half the diameter (measures how quickly connectivity decays).

use crate::core::{Graph, IgraphResult};

/// Normalised Shannon entropy of the hop-distance distribution.
///
/// Computes BFS distances from all vertices, builds a histogram of
/// finite distances (excluding self-loops d=0), and returns the Shannon
/// entropy normalised by `ln(diameter)` so the result is in \[0, 1\].
///
/// Returns 0.0 for graphs with fewer than 2 finite distances or diameter ≤ 1.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, hop_entropy};
///
/// // Path 0-1-2: distances {1,1,2} → 2 classes → entropy > 0
/// let g = Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap();
/// let h = hop_entropy(&g).unwrap();
/// assert!(h > 0.0);
/// assert!(h <= 1.0);
/// ```
pub fn hop_entropy(graph: &Graph) -> IgraphResult<f64> {
    let hist = distance_histogram(graph)?;
    if hist.is_empty() {
        return Ok(0.0);
    }
    let num_bins = hist.len();
    if num_bins <= 1 {
        return Ok(0.0);
    }

    let total: u64 = hist.iter().sum();
    if total == 0 {
        return Ok(0.0);
    }

    #[allow(clippy::cast_precision_loss)]
    let total_f = total as f64;
    let mut entropy = 0.0_f64;
    let mut non_empty = 0usize;
    for &count in &hist {
        if count > 0 {
            #[allow(clippy::cast_precision_loss)]
            let p = count as f64 / total_f;
            entropy -= p * p.ln();
            non_empty += 1;
        }
    }

    if non_empty <= 1 {
        return Ok(0.0);
    }

    #[allow(clippy::cast_precision_loss)]
    let max_entropy = (non_empty as f64).ln();
    Ok(entropy / max_entropy)
}

/// Gini coefficient of the pairwise distance distribution.
///
/// Measures inequality among all finite pairwise distances. A value of 0
/// means all distances are equal (e.g. complete graph where all d=1).
/// Higher values indicate more spread in the distance distribution.
///
/// Returns 0.0 for graphs with fewer than 2 finite distances.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distance_gini};
///
/// // K3: all distances = 1 → Gini = 0
/// let g = Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap();
/// assert!(distance_gini(&g).unwrap().abs() < 1e-10);
/// ```
pub fn distance_gini(graph: &Graph) -> IgraphResult<f64> {
    let distances = all_finite_distances(graph)?;
    let n = distances.len();
    if n < 2 {
        return Ok(0.0);
    }

    let mut sorted = distances;
    sorted.sort_unstable();

    #[allow(clippy::cast_precision_loss)]
    let n_f = n as f64;
    let mean: f64 = sorted.iter().map(|&d| f64::from(d)).sum::<f64>() / n_f;
    if mean < 1e-15 {
        return Ok(0.0);
    }

    // Gini = (2 * sum_i((i+1)*x_i)) / (n * sum_i(x_i)) - (n+1)/n
    let mut weighted_sum = 0.0_f64;
    for (i, &d) in sorted.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let rank = (i + 1) as f64;
        weighted_sum += rank * f64::from(d);
    }
    let total_sum = mean * n_f;
    let gini = (2.0 * weighted_sum) / (n_f * total_sum) - (n_f + 1.0) / n_f;
    Ok(gini)
}

/// Average fraction of vertices reachable within half the diameter.
///
/// For each vertex, computes the fraction of other vertices reachable
/// within `floor(diameter / 2)` hops, then averages across all vertices.
/// Measures how quickly connectivity "fills in" relative to the graph's
/// diameter.
///
/// Returns 0.0 for disconnected or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reach_decay};
///
/// // K4: diameter=1, half=0 → no vertex reachable in 0 hops → 0
/// // Actually half_diam = floor(1/2) = 0, so reach = 0
/// let g = Graph::from_edges(
///     &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
///     false,
///     Some(4),
/// )
/// .unwrap();
/// let r = reach_decay(&g).unwrap();
/// assert!(r >= 0.0 && r <= 1.0);
/// ```
pub fn reach_decay(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n < 2 {
        return Ok(0.0);
    }

    // Find diameter
    let mut diameter: u32 = 0;
    let n_us = n as usize;
    let mut all_dists: Vec<Vec<Option<u32>>> = Vec::with_capacity(n_us);
    for v in 0..n {
        let dists = graph.distances(v)?;
        for &d in &dists {
            if let Some(dist) = d {
                if dist > diameter {
                    diameter = dist;
                }
            }
        }
        all_dists.push(dists);
    }

    if diameter == 0 {
        return Ok(0.0);
    }

    let half_diam = diameter / 2;
    if half_diam == 0 {
        return Ok(0.0);
    }

    // For each vertex, count fraction reachable within half_diam
    let mut total_fraction = 0.0_f64;
    let others = f64::from(n - 1);
    for dists in &all_dists {
        let reachable = dists
            .iter()
            .filter(|&&d| matches!(d, Some(dist) if dist > 0 && dist <= half_diam))
            .count();
        #[allow(clippy::cast_precision_loss)]
        let frac = reachable as f64 / others;
        total_fraction += frac;
    }

    Ok(total_fraction / f64::from(n))
}

/// Build histogram of finite distances (excluding d=0).
/// Returns vec where index i holds count of pairs at distance i+1.
fn distance_histogram(graph: &Graph) -> IgraphResult<Vec<u64>> {
    let n = graph.vcount();
    if n < 2 {
        return Ok(Vec::new());
    }

    let mut max_dist: u32 = 0;
    let mut pairs: Vec<u32> = Vec::new();

    for v in 0..n {
        let dists = graph.distances(v)?;
        for (u, &d) in dists.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let u_u32 = u as u32;
            if u_u32 > v {
                if let Some(dist) = d {
                    if dist > 0 {
                        pairs.push(dist);
                        if dist > max_dist {
                            max_dist = dist;
                        }
                    }
                }
            }
        }
    }

    if max_dist == 0 {
        return Ok(Vec::new());
    }

    let mut hist = vec![0u64; max_dist as usize];
    for &d in &pairs {
        hist[(d - 1) as usize] += 1;
    }
    Ok(hist)
}

/// Collect all finite pairwise distances (excluding d=0), one per unordered pair.
fn all_finite_distances(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount();
    if n < 2 {
        return Ok(Vec::new());
    }

    let mut distances = Vec::new();
    for v in 0..n {
        let dists = graph.distances(v)?;
        for (u, &d) in dists.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            if (u as u32) > v {
                if let Some(dist) = d {
                    if dist > 0 {
                        distances.push(dist);
                    }
                }
            }
        }
    }
    Ok(distances)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- hop_entropy ---

    #[test]
    fn hop_entropy_empty() {
        let g = Graph::with_vertices(0);
        assert!(hop_entropy(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn hop_entropy_edgeless() {
        let g = Graph::with_vertices(5);
        assert!(hop_entropy(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn hop_entropy_complete() {
        // K4: all distances = 1, single bin → entropy = 0
        let g = Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap();
        assert!(hop_entropy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hop_entropy_path() {
        // Path: multiple distance values → positive entropy
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)], false, Some(5)).unwrap();
        let h = hop_entropy(&g).unwrap();
        assert!(h > 0.0, "Path should have positive hop entropy, got {h}");
        assert!(h <= 1.0, "Should be <= 1, got {h}");
    }

    // --- distance_gini ---

    #[test]
    fn gini_empty() {
        let g = Graph::with_vertices(0);
        assert!(distance_gini(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn gini_complete() {
        // K4: all distances = 1 → Gini = 0
        let g = Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap();
        assert!(distance_gini(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gini_path() {
        // Path: distances vary → positive Gini
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap();
        let gini = distance_gini(&g).unwrap();
        assert!(gini > 0.0, "Path should have positive Gini, got {gini}");
        assert!(gini < 1.0, "Gini should be < 1, got {gini}");
    }

    // --- reach_decay ---

    #[test]
    fn reach_empty() {
        let g = Graph::with_vertices(0);
        assert!(reach_decay(&g).unwrap().abs() < 1e-12);
    }

    #[test]
    fn reach_path() {
        // Path 0-1-2-3-4: diameter=4, half=2
        // Each vertex can reach some within 2 hops
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)], false, Some(5)).unwrap();
        let r = reach_decay(&g).unwrap();
        assert!(r > 0.0, "Path should have positive reach, got {r}");
        assert!(r <= 1.0, "Reach should be <= 1, got {r}");
    }

    #[test]
    fn reach_star() {
        // Star K1,4: diameter=2, half=1
        // Hub reaches all 4 in 1 hop (fraction=1.0)
        // Leaves reach hub in 1 hop (fraction=1/4=0.25)
        // Average = (1.0 + 0.25*4) / 5 = 2.0/5 = 0.4
        let g = Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap();
        let r = reach_decay(&g).unwrap();
        assert!((r - 0.4).abs() < 1e-10, "Star reach should be 0.4, got {r}");
    }
}
