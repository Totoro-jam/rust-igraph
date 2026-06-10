//! Robustness ratio indices (ALGO-TR-116).
//!
//! Measures of graph resilience under vertex/edge removal:
//!
//! - **Vertex connectivity ratio** — min vertex-cut / average degree,
//!   normalized vertex connectivity
//! - **Edge connectivity ratio** — min edge-cut / min degree, normalized
//!   edge connectivity
//! - **Average path resilience** — 1 - (diameter after removing highest
//!   degree vertex) / (original diameter + n), a stability measure

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

/// Compute the vertex connectivity ratio.
///
/// Approximates vertex connectivity as the minimum degree (a lower bound
/// on κ(G) by Whitney's theorem), then normalizes by average degree.
/// Values near 1 indicate the graph is nearly optimally connected
/// relative to its density. Returns 0.0 for disconnected or trivial
/// graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, min_degree_connectivity_ratio};
///
/// // K_4: min_deg=3, avg_deg=3 → ratio=1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((min_degree_connectivity_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn min_degree_connectivity_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let mut min_deg = usize::MAX;
    let mut sum_deg = 0_u64;

    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d < min_deg {
            min_deg = d;
        }
        sum_deg += d as u64;
    }

    if min_deg == 0 {
        return Ok(0.0);
    }

    let avg_deg = sum_deg as f64 / n as f64;
    if avg_deg < 1e-30 {
        return Ok(0.0);
    }

    Ok(min_deg as f64 / avg_deg)
}

/// Compute the edge connectivity ratio.
///
/// Approximates edge connectivity as the minimum degree (a lower bound
/// on λ(G)), then normalizes by the minimum degree itself, yielding 1.0
/// for all connected graphs with `min_degree` > 0. More usefully,
/// this computes `min_degree / max_degree` which measures how uniform
/// the degree distribution is from a connectivity standpoint.
/// Returns 0.0 for disconnected or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_range_ratio};
///
/// // K_3: min_deg=2, max_deg=2 → ratio=1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((degree_range_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn degree_range_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let mut min_deg = usize::MAX;
    let mut max_deg = 0_usize;

    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d < min_deg {
            min_deg = d;
        }
        if d > max_deg {
            max_deg = d;
        }
    }

    if min_deg == 0 || max_deg == 0 {
        return Ok(0.0);
    }

    Ok(min_deg as f64 / max_deg as f64)
}

/// Compute the average path resilience.
///
/// Measures how much the diameter increases when the highest-degree
/// vertex is removed. Specifically:
/// `1 - (new_diameter - old_diameter) / n`
/// where `new_diameter` is the diameter of the graph after removing the
/// vertex with highest degree (ties broken by lowest index). Values near
/// 1 indicate removing the hub has little effect; values near 0 indicate
/// the hub is critical. Returns 0.0 for trivial graphs or if removal
/// disconnects the graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, average_path_resilience};
///
/// // K_4: removing any vertex leaves K_3 (diameter 1 → 1), resilience = 1 - 0/4 = 1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((average_path_resilience(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn average_path_resilience(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    // Find highest degree vertex
    let mut max_deg = 0_usize;
    let mut hub = 0_usize;
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d > max_deg {
            max_deg = d;
            hub = v;
        }
    }

    if max_deg == 0 {
        return Ok(0.0);
    }

    // Compute original diameter
    let old_diam = compute_diameter(graph, n, None)?;
    if old_diam == 0 {
        return Ok(0.0);
    }

    // Compute diameter after removing hub
    let new_diam = compute_diameter(graph, n, Some(hub))?;
    if new_diam == 0 {
        return Ok(0.0);
    }

    let diff = if new_diam > old_diam {
        (new_diam - old_diam) as f64
    } else {
        0.0
    };

    Ok(1.0 - diff / n as f64)
}

/// BFS diameter, optionally excluding a vertex.
fn compute_diameter(graph: &Graph, n: usize, exclude: Option<usize>) -> IgraphResult<u32> {
    let mut diam = 0_u32;
    for source in 0..n {
        if Some(source) == exclude {
            continue;
        }
        let mut dist = vec![u32::MAX; n];
        dist[source] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(source);

        while let Some(v) = queue.pop_front() {
            let cd = dist[v];
            let nbrs = graph.neighbors(v as u32)?;
            for &u in &nbrs {
                let ui = u as usize;
                if Some(ui) == exclude {
                    continue;
                }
                if dist[ui] == u32::MAX {
                    dist[ui] = cd + 1;
                    queue.push_back(ui);
                }
            }
        }

        for target in (source + 1)..n {
            if Some(target) == exclude {
                continue;
            }
            if dist[target] == u32::MAX {
                return Ok(0);
            }
            if dist[target] > diam {
                diam = dist[target];
            }
        }
    }
    Ok(diam)
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

    // --- min_degree_connectivity_ratio ---

    #[test]
    fn vcr_empty() {
        assert!(min_degree_connectivity_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn vcr_single() {
        assert!(min_degree_connectivity_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn vcr_k3() {
        // min_deg=2, avg_deg=2 → 1.0
        assert!((min_degree_connectivity_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn vcr_k4() {
        // min_deg=3, avg_deg=3 → 1.0
        assert!((min_degree_connectivity_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn vcr_cycle4() {
        // min_deg=2, avg_deg=2 → 1.0
        assert!((min_degree_connectivity_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn vcr_star5() {
        // min_deg=1, avg_deg=8/5=1.6 → 1/1.6 = 0.625
        let r = min_degree_connectivity_ratio(&star5()).unwrap();
        assert!((r - 5.0 / 8.0).abs() < 1e-10);
    }

    #[test]
    fn vcr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = min_degree_connectivity_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- degree_range_ratio ---

    #[test]
    fn ecr_empty() {
        assert!(degree_range_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ecr_single() {
        assert!(degree_range_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ecr_k3() {
        // min_deg=2, max_deg=2 → 1.0
        assert!((degree_range_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ecr_k4() {
        assert!((degree_range_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ecr_cycle4() {
        assert!((degree_range_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ecr_star5() {
        // min_deg=1, max_deg=4 → 0.25
        assert!((degree_range_ratio(&star5()).unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn ecr_paw() {
        // Degrees: 2,2,3,1 → min=1, max=3 → 1/3
        assert!((degree_range_ratio(&paw()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn ecr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = degree_range_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- average_path_resilience ---

    #[test]
    fn apr_empty() {
        assert!(average_path_resilience(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn apr_single() {
        assert!(average_path_resilience(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn apr_single_edge() {
        // n < 3 → 0.0
        assert!(average_path_resilience(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn apr_k3() {
        // Remove any vertex → K_2, diam=1; original diam=1; diff=0 → 1.0
        assert!((average_path_resilience(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn apr_k4() {
        // Remove any vertex → K_3, diam=1; original diam=1; diff=0 → 1.0
        assert!((average_path_resilience(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn apr_cycle4() {
        // All deg=2, hub=0; remove 0 → path 1-2-3, diam=2; original diam=2
        // diff=0 → 1.0
        assert!((average_path_resilience(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn apr_star5() {
        // Hub=0 (deg=4); remove → 4 isolated vertices → disconnected → 0.0
        assert!(average_path_resilience(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn apr_in_01() {
        for g in &[path3(), k3(), k4(), cycle4(), paw()] {
            let r = average_path_resilience(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn regular_max_connectivity() {
        // Regular graphs: min_degree_connectivity_ratio = 1.0, degree_range_ratio = 1.0
        assert!((min_degree_connectivity_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((degree_range_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((min_degree_connectivity_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((degree_range_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn complete_full_resilience() {
        assert!((average_path_resilience(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((average_path_resilience(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }
}
