//! Distance distribution ratio indices (ALGO-TR-112).
//!
//! Shape measures of the all-pairs shortest path length distribution:
//!
//! - **Distance skewness** — skewness of the distance distribution
//! - **Distance kurtosis** — excess kurtosis of the distance distribution
//! - **Diameter ratio** — diameter / n (normalized longest shortest path)
//! - **Mean eccentricity ratio** — mean eccentricity / diameter

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

/// Compute the distance skewness.
///
/// Skewness (third standardized moment) of the distribution of all
/// pairwise shortest path lengths. Positive skew indicates most pairs
/// are close with a long tail; negative skew indicates most pairs are
/// far apart. Returns 0.0 for disconnected or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distance_skewness};
///
/// // K_4: all distances = 1, zero variance → skewness = 0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!(distance_skewness(&g).unwrap().abs() < 1e-10);
/// ```
pub fn distance_skewness(graph: &Graph) -> IgraphResult<f64> {
    let moments = distance_moments(graph)?;
    match moments {
        None => Ok(0.0),
        Some((_, variance, skew, _)) => {
            if variance < 1e-30 {
                return Ok(0.0);
            }
            Ok(skew)
        }
    }
}

/// Compute the distance kurtosis.
///
/// Excess kurtosis (fourth standardized moment minus 3) of the
/// distribution of all pairwise shortest path lengths. Positive values
/// indicate heavy tails; negative values indicate light tails relative
/// to a normal distribution. Returns 0.0 for disconnected or trivial
/// graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distance_kurtosis};
///
/// // K_4: all distances = 1, zero variance → kurtosis = 0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!(distance_kurtosis(&g).unwrap().abs() < 1e-10);
/// ```
pub fn distance_kurtosis(graph: &Graph) -> IgraphResult<f64> {
    let moments = distance_moments(graph)?;
    match moments {
        None => Ok(0.0),
        Some((_, variance, _, kurt)) => {
            if variance < 1e-30 {
                return Ok(0.0);
            }
            Ok(kurt)
        }
    }
}

/// Compute the diameter ratio.
///
/// `diameter / (n - 1)` — the diameter normalized by the maximum
/// possible diameter (a path graph). Values near 1 indicate the graph
/// is elongated; values near 0 indicate short diameters (e.g. complete
/// graphs). Returns 0.0 for disconnected or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, diameter_ratio};
///
/// // Path 0-1-2-3: diameter=3, n=4 → 3/3 = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// assert!((diameter_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn diameter_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let diam = compute_diameter_bfs(graph)?;
    if diam == 0 {
        return Ok(0.0);
    }

    Ok(diam as f64 / (n - 1) as f64)
}

/// Compute the mean eccentricity ratio.
///
/// `mean_eccentricity / diameter` — how close the average vertex's
/// eccentricity is to the maximum (diameter). Values near 1 indicate
/// most vertices are far from the center; values near radius/diameter
/// indicate a compact center. Returns 0.0 for disconnected or trivial
/// graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, mean_eccentricity_ratio};
///
/// // K_4: all eccentricities = 1, diameter = 1 → ratio = 1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((mean_eccentricity_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn mean_eccentricity_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let eccs = compute_eccentricities(graph)?;
    match eccs {
        None => Ok(0.0),
        Some(ecc_vec) => {
            let diam = ecc_vec.iter().copied().max().unwrap_or(0);
            if diam == 0 {
                return Ok(0.0);
            }
            let mean_ecc = ecc_vec.iter().copied().sum::<u32>() as f64 / n as f64;
            Ok(mean_ecc / diam as f64)
        }
    }
}

/// Compute distance moments (mean, variance, skewness, kurtosis).
/// Returns None for disconnected or trivial graphs.
fn distance_moments(graph: &Graph) -> IgraphResult<Option<(f64, f64, f64, f64)>> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(None);
    }

    let mut sum = 0_u64;
    let mut count = 0_u64;
    let mut distances = Vec::new();

    for v in 0..n {
        let dist = bfs_distances(graph, v)?;
        for u in (v + 1)..n {
            if dist[u] == u32::MAX {
                return Ok(None);
            }
            let d = dist[u] as u64;
            sum += d;
            count += 1;
            distances.push(d as f64);
        }
    }

    if count < 2 {
        return Ok(None);
    }

    let mean = sum as f64 / count as f64;

    let mut m2 = 0.0_f64;
    let mut m3 = 0.0_f64;
    let mut m4 = 0.0_f64;
    for &d in &distances {
        let diff = d - mean;
        let d2 = diff * diff;
        m2 += d2;
        m3 += d2 * diff;
        m4 += d2 * d2;
    }
    m2 /= count as f64;
    m3 /= count as f64;
    m4 /= count as f64;

    let variance = m2;
    let skewness = if variance < 1e-30 {
        0.0
    } else {
        m3 / (variance * variance.sqrt())
    };
    let kurtosis = if variance < 1e-30 {
        0.0
    } else {
        m4 / (variance * variance) - 3.0
    };

    Ok(Some((mean, variance, skewness, kurtosis)))
}

/// BFS from a single source, returns distance array.
fn bfs_distances(graph: &Graph, source: usize) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount() as usize;
    let mut dist = vec![u32::MAX; n];
    dist[source] = 0;
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(source);
    while let Some(v) = queue.pop_front() {
        let cd = dist[v];
        let nbrs = graph.neighbors(v as u32)?;
        for &u in &nbrs {
            let ui = u as usize;
            if dist[ui] == u32::MAX {
                dist[ui] = cd + 1;
                queue.push_back(ui);
            }
        }
    }
    Ok(dist)
}

/// Compute diameter via all-pairs BFS. Returns 0 for disconnected graphs.
fn compute_diameter_bfs(graph: &Graph) -> IgraphResult<u32> {
    let n = graph.vcount() as usize;
    let mut diam = 0_u32;
    for v in 0..n {
        let dist = bfs_distances(graph, v)?;
        for u in (v + 1)..n {
            if dist[u] == u32::MAX {
                return Ok(0);
            }
            if dist[u] > diam {
                diam = dist[u];
            }
        }
    }
    Ok(diam)
}

/// Compute eccentricities. Returns None if disconnected.
fn compute_eccentricities(graph: &Graph) -> IgraphResult<Option<Vec<u32>>> {
    let n = graph.vcount() as usize;
    let mut eccs = vec![0_u32; n];
    for v in 0..n {
        let dist = bfs_distances(graph, v)?;
        let mut max_d = 0_u32;
        for u in 0..n {
            if u == v {
                continue;
            }
            if dist[u] == u32::MAX {
                return Ok(None);
            }
            if dist[u] > max_d {
                max_d = dist[u];
            }
        }
        eccs[v] = max_d;
    }
    Ok(Some(eccs))
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

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
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

    fn cycle5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)], false, Some(5)).unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn disconnected() -> Graph {
        Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap()
    }

    // --- distance_skewness ---

    #[test]
    fn ds_empty() {
        assert!(distance_skewness(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ds_single() {
        assert!(distance_skewness(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ds_single_edge() {
        // Only one pair, zero variance → 0
        assert!(distance_skewness(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ds_k3() {
        // All distances = 1, zero variance → 0
        assert!(distance_skewness(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ds_k4() {
        assert!(distance_skewness(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ds_cycle4() {
        // Distances: 1,2,1,1,2,1 → [1,1,1,1,2,2], mean=4/3
        // Symmetric around mean → skewness > 0 (more 1s than 2s)
        let s = distance_skewness(&cycle4()).unwrap();
        assert!(s > -1e-10); // non-negative for this shape
    }

    #[test]
    fn ds_disconnected() {
        assert!(distance_skewness(&disconnected()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ds_path3() {
        // Distances: (0,1)=1, (0,2)=2, (1,2)=1 → [1,1,2]
        // mean=4/3, has positive skew
        let s = distance_skewness(&path3()).unwrap();
        assert!(s.is_finite());
    }

    // --- distance_kurtosis ---

    #[test]
    fn dk_empty() {
        assert!(distance_kurtosis(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dk_single() {
        assert!(distance_kurtosis(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dk_k4() {
        // Zero variance → 0
        assert!(distance_kurtosis(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dk_disconnected() {
        assert!(distance_kurtosis(&disconnected()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dk_path4() {
        // Distances: 1,2,3,1,2,1 → mean=10/6=5/3
        let k = distance_kurtosis(&path4()).unwrap();
        assert!(k.is_finite());
    }

    #[test]
    fn dk_finite() {
        for g in &[path3(), k3(), k4(), cycle4(), cycle5(), star5()] {
            assert!(distance_kurtosis(g).unwrap().is_finite());
        }
    }

    // --- diameter_ratio ---

    #[test]
    fn dr_empty() {
        assert!(diameter_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dr_single() {
        assert!(diameter_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dr_single_edge() {
        // diameter=1, n=2 → 1/(2-1) = 1.0
        assert!((diameter_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn dr_path3() {
        // diameter=2, n=3 → 2/2 = 1.0
        assert!((diameter_ratio(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn dr_path4() {
        // diameter=3, n=4 → 3/3 = 1.0
        assert!((diameter_ratio(&path4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn dr_k3() {
        // diameter=1, n=3 → 1/2 = 0.5
        assert!((diameter_ratio(&k3()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn dr_k4() {
        // diameter=1, n=4 → 1/3
        assert!((diameter_ratio(&k4()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn dr_cycle4() {
        // diameter=2, n=4 → 2/3
        assert!((diameter_ratio(&cycle4()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn dr_star5() {
        // diameter=2, n=5 → 2/4 = 0.5
        assert!((diameter_ratio(&star5()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn dr_disconnected() {
        assert!(diameter_ratio(&disconnected()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            let r = diameter_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- mean_eccentricity_ratio ---

    #[test]
    fn mer_empty() {
        assert!(mean_eccentricity_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mer_single() {
        assert!(mean_eccentricity_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mer_k3() {
        // All eccentricities = 1, diameter = 1 → 1.0
        assert!((mean_eccentricity_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mer_k4() {
        assert!((mean_eccentricity_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mer_path3() {
        // Eccentricities: [2,1,2], diameter=2, mean=5/3, ratio=5/6
        assert!((mean_eccentricity_ratio(&path3()).unwrap() - 5.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn mer_path4() {
        // Eccentricities: [3,2,2,3], diameter=3, mean=10/4=2.5, ratio=2.5/3=5/6
        assert!((mean_eccentricity_ratio(&path4()).unwrap() - 5.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn mer_cycle4() {
        // All eccentricities = 2, diameter = 2 → 1.0
        assert!((mean_eccentricity_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mer_star5() {
        // Eccentricities: center=1, leaves=2 → mean=(1+2*4)/5=9/5=1.8
        // diameter=2, ratio=1.8/2=0.9
        assert!((mean_eccentricity_ratio(&star5()).unwrap() - 0.9).abs() < 1e-10);
    }

    #[test]
    fn mer_disconnected() {
        assert!(mean_eccentricity_ratio(&disconnected()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mer_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            let r = mean_eccentricity_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn complete_zero_skew_and_kurt() {
        assert!(distance_skewness(&k3()).unwrap().abs() < 1e-10);
        assert!(distance_skewness(&k4()).unwrap().abs() < 1e-10);
        assert!(distance_kurtosis(&k3()).unwrap().abs() < 1e-10);
        assert!(distance_kurtosis(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn path_max_diameter_ratio() {
        // Path graphs have diameter = n-1 → ratio = 1.0
        assert!((diameter_ratio(&path3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((diameter_ratio(&path4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn regular_full_ecc_ratio() {
        // Regular graphs where all eccentricities equal → ratio = 1.0
        assert!((mean_eccentricity_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((mean_eccentricity_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((mean_eccentricity_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }
}
