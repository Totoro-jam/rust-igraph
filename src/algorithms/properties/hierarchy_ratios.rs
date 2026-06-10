//! Hierarchy-based ratio indices (ALGO-TR-120).
//!
//! Measures of hierarchical structure in graphs:
//!
//! - **Degree hierarchy** — Gini coefficient of the degree sequence,
//!   measuring inequality in vertex importance
//! - **Layer ratio** — fraction of vertices reachable at each BFS layer
//!   from the highest-degree vertex, normalized by an ideal hierarchy
//! - **Dominance ratio** — fraction of vertex pairs where one dominates
//!   the other in the neighborhood inclusion order

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

/// Compute the degree hierarchy (Gini coefficient of degrees).
///
/// The Gini coefficient measures inequality in the degree distribution.
/// Values near 0 indicate all vertices have similar degree (regular graph);
/// values near 1 indicate extreme inequality (star-like). Returns 0.0
/// for trivial or edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_hierarchy};
///
/// // K_4: all degrees equal → Gini = 0.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!(degree_hierarchy(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_hierarchy(graph: &Graph) -> IgraphResult<f64> {
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

    degrees.sort_unstable();

    // Gini coefficient: (2 * sum_i(i * x_i)) / (n * sum_x) - (n + 1) / n
    let mut weighted_sum = 0_u64;
    for (i, &d) in degrees.iter().enumerate() {
        weighted_sum += (i as u64 + 1) * d as u64;
    }

    let gini = (2.0 * weighted_sum as f64) / (n as f64 * sum as f64) - (n as f64 + 1.0) / n as f64;
    Ok(gini.clamp(0.0, 1.0))
}

/// Compute the layer ratio.
///
/// BFS from the highest-degree vertex; measures how concentrated the
/// graph is around a hub. Returns the ratio of the actual average layer
/// depth to the maximum possible depth (n-1, for a path). Values near 0
/// indicate a flat structure (star-like, all vertices near the hub);
/// values near 1 indicate a deep, chain-like structure. Returns 0.0 for
/// trivial or disconnected graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layer_ratio};
///
/// // Star graph: all leaves at layer 1, avg_depth = 1, max = n-1 = 4
/// // ratio = 1/4 = 0.25
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(0,4)], false, Some(5)
/// ).unwrap();
/// let r = layer_ratio(&g).unwrap();
/// assert!(r > 0.2 && r < 0.3);
/// ```
pub fn layer_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    // Find highest-degree vertex
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

    // BFS from hub
    let mut dist = vec![u32::MAX; n];
    dist[hub] = 0;
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(hub);
    let mut visit_count = 1_usize;
    let mut depth_sum = 0_u64;

    while let Some(v) = queue.pop_front() {
        let nbrs = graph.neighbors(v as u32)?;
        for &u in &nbrs {
            let ui = u as usize;
            if dist[ui] == u32::MAX {
                dist[ui] = dist[v] + 1;
                depth_sum += dist[ui] as u64;
                visit_count += 1;
                queue.push_back(ui);
            }
        }
    }

    if visit_count < n {
        return Ok(0.0);
    }

    let avg_depth = depth_sum as f64 / (n - 1) as f64;
    let max_depth = (n - 1) as f64;

    Ok(avg_depth / max_depth)
}

/// Compute the dominance ratio.
///
/// The neighborhood inclusion order: vertex u dominates v if N(v) ⊆ N(u)∪{u}.
/// The dominance ratio is the fraction of directed pairs (u,v) where u
/// dominates v. Values near 0 indicate no dominance relationships
/// (random-like); values near 1 indicate a strongly hierarchical structure.
/// Returns 0.0 for trivial or edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dominance_ratio};
///
/// // Star K_{1,3}: center dominates leaves, and each leaf dominates other leaves
/// // (N(leaf_j)={center} ⊆ N(leaf_i)∪{leaf_i}), total 9/12 = 0.75
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3)], false, Some(4)).unwrap();
/// let r = dominance_ratio(&g).unwrap();
/// assert!((r - 0.75).abs() < 1e-10);
/// ```
pub fn dominance_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    // Build neighbor sets (including the vertex itself for the dominator)
    let mut nbr_sets: Vec<Vec<bool>> = Vec::with_capacity(n);
    for v in 0..n {
        let mut set = vec![false; n];
        let nbrs = graph.neighbors(v as u32)?;
        for &u in &nbrs {
            set[u as usize] = true;
        }
        set[v] = true; // N(v) ∪ {v}
        nbr_sets.push(set);
    }

    let mut dominance_count = 0_u64;
    let directed_pairs = (n * (n - 1)) as u64;

    for u in 0..n {
        for v in 0..n {
            if u == v {
                continue;
            }
            // Check if u dominates v: N(v)\{v} ⊆ N(u)∪{u}
            let mut dominates = true;
            let nbrs_v = graph.neighbors(v as u32)?;
            for &w in &nbrs_v {
                if !nbr_sets[u][w as usize] {
                    dominates = false;
                    break;
                }
            }
            if dominates {
                dominance_count += 1;
            }
        }
    }

    Ok(dominance_count as f64 / directed_pairs as f64)
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

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- degree_hierarchy ---

    #[test]
    fn dh_empty() {
        assert!(degree_hierarchy(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dh_single() {
        assert!(degree_hierarchy(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dh_k3() {
        // Regular → Gini = 0
        assert!(degree_hierarchy(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dh_k4() {
        assert!(degree_hierarchy(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dh_cycle4() {
        assert!(degree_hierarchy(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dh_star5() {
        // Degrees: 4,1,1,1,1 → non-zero Gini
        let r = degree_hierarchy(&star5()).unwrap();
        assert!(r > 0.1);
    }

    #[test]
    fn dh_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = degree_hierarchy(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- layer_ratio ---

    #[test]
    fn lr_empty() {
        assert!(layer_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn lr_single() {
        assert!(layer_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn lr_star5() {
        // avg depth = 1, max = 4, ratio = 0.25
        let r = layer_ratio(&star5()).unwrap();
        assert!((r - 0.25).abs() < 1e-10);
    }

    #[test]
    fn lr_path4() {
        // Hub is an endpoint (deg 1 at 0 or 3, but center has deg 2)
        // Actually hub is vertex with max degree = vertex 1 or 2 (deg 2)
        // BFS from 1: depths 1,0,1,2 → sum=4, avg=4/3, max=3, ratio=4/9
        let r = layer_ratio(&path4()).unwrap();
        assert!(r > 0.3 && r < 0.6);
    }

    #[test]
    fn lr_k4() {
        // All at distance 1 from hub, avg=1, max=3, ratio=1/3
        let r = layer_ratio(&k4()).unwrap();
        assert!((r - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn lr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = layer_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- dominance_ratio ---

    #[test]
    fn dr_empty() {
        assert!(dominance_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dr_single() {
        assert!(dominance_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dr_single_edge() {
        // N(0)={1}, N(1)={0}. N(0)∪{0}={0,1}, N(1)∪{1}={0,1}
        // 0 dominates 1: N(1)\{1}={0} ⊆ {0,1} ✓
        // 1 dominates 0: N(0)\{0}={1} ⊆ {0,1} ✓
        // 2/2 = 1.0
        assert!((dominance_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn dr_k3() {
        // All vertices have same neighborhood structure → all dominate all
        // Each pair: N(v)\{v}={other two} ⊆ N(u)∪{u}={all three} ✓
        // 6/6 = 1.0
        assert!((dominance_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn dr_k4() {
        assert!((dominance_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn dr_star5() {
        // Center (0): N(0)={1,2,3,4}, N(0)∪{0}={0,1,2,3,4}
        // Leaf (i): N(i)={0}, N(i)∪{i}={0,i}
        // 0 dominates leaf_i: N(leaf_i)\{leaf_i}={0} ⊆ {0,1,2,3,4} ✓ → 4 pairs
        // leaf_i dominates 0: N(0)\{0}={1,2,3,4} ⊆ {0,i}? No → 0 pairs
        // leaf_i dominates leaf_j: N(leaf_j)\{leaf_j}={0} ⊆ {0,i}? Yes! → 12 pairs
        // Total: 4 + 12 = 16 out of 5*4 = 20
        let r = dominance_ratio(&star5()).unwrap();
        assert!((r - 16.0 / 20.0).abs() < 1e-10);
    }

    #[test]
    fn dr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = dominance_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn regular_zero_hierarchy() {
        assert!(degree_hierarchy(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_hierarchy(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_hierarchy(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn complete_full_dominance() {
        assert!((dominance_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((dominance_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn star_hierarchy_measures() {
        // Star should have high Gini and low layer ratio
        let dh = degree_hierarchy(&star5()).unwrap();
        let lr = layer_ratio(&star5()).unwrap();
        assert!(dh > 0.3);
        assert!(lr < 0.5);
    }
}
