//! Path-based ratio indices (ALGO-TR-101).
//!
//! Path and distance-based structural ratios:
//!
//! - **Diameter-radius ratio** — diameter / radius
//! - **Average path fraction** — average shortest-path length / diameter
//! - **Efficiency ratio** — global efficiency / max possible efficiency
//! - **Compactness** — 1 - average distance / diameter

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the diameter-radius ratio.
///
/// `diameter / radius` for connected graphs. A value of 1 means the
/// graph is self-centered (diameter equals radius). Returns 0.0 for
/// disconnected or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, diameter_radius_ratio};
///
/// // K_3: diameter=1, radius=1 → 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((diameter_radius_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn diameter_radius_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let dist = all_pairs_bfs(graph)?;

    let mut max_ecc = 0_u32;
    let mut min_ecc = u32::MAX;

    for v in 0..n {
        let mut ecc = 0_u32;
        for u in 0..n {
            if u == v {
                continue;
            }
            let d = dist[v * n + u];
            if d == u32::MAX {
                return Ok(0.0);
            }
            if d > ecc {
                ecc = d;
            }
        }
        if ecc > max_ecc {
            max_ecc = ecc;
        }
        if ecc < min_ecc {
            min_ecc = ecc;
        }
    }

    if min_ecc == 0 {
        return Ok(0.0);
    }

    Ok(f64::from(max_ecc) / f64::from(min_ecc))
}

/// Compute the average path fraction.
///
/// `avg_dist / diameter` — how close the average shortest-path length
/// is to the diameter. Values close to 1 indicate most paths are near
/// the diameter; values close to 0 indicate most paths are short.
/// Returns 0.0 for disconnected or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, avg_path_fraction};
///
/// // K_3: avg_dist=1, diameter=1 → 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((avg_path_fraction(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn avg_path_fraction(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let dist = all_pairs_bfs(graph)?;

    let mut sum = 0_u64;
    let mut diameter = 0_u32;
    let mut pair_count = 0_u64;

    for v in 0..n {
        for u in (v + 1)..n {
            let d = dist[v * n + u];
            if d == u32::MAX {
                return Ok(0.0);
            }
            sum += u64::from(d);
            pair_count += 1;
            if d > diameter {
                diameter = d;
            }
        }
    }

    if diameter == 0 || pair_count == 0 {
        return Ok(0.0);
    }

    let avg_dist = sum as f64 / pair_count as f64;
    Ok(avg_dist / f64::from(diameter))
}

/// Compute the efficiency ratio.
///
/// `global_efficiency / max_efficiency` where `global_efficiency` is the
/// average inverse shortest-path length over all pairs, and
/// `max_efficiency = 1` (the efficiency of a complete graph).
/// Equivalent to just the global efficiency for simple graphs.
/// Returns 0.0 for graphs with fewer than 2 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, efficiency_ratio};
///
/// // K_3: all pairs at distance 1, eff = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((efficiency_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn efficiency_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let dist = all_pairs_bfs(graph)?;

    let mut sum_inv = 0.0_f64;
    let mut pair_count = 0_u64;

    for v in 0..n {
        for u in (v + 1)..n {
            let d = dist[v * n + u];
            if d != u32::MAX && d > 0 {
                sum_inv += 1.0 / f64::from(d);
            }
            pair_count += 1;
        }
    }

    if pair_count == 0 {
        return Ok(0.0);
    }

    Ok(sum_inv / pair_count as f64)
}

/// Compute the compactness of the graph.
///
/// `1 - avg_dist / diameter` — how compact the graph is relative to
/// its diameter. A value of 0 means the average distance equals the
/// diameter; close to 1 means most pairs are much closer than the
/// diameter. Returns 0.0 for disconnected or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, graph_compactness};
///
/// // K_3: avg=1, diam=1 → 1-1/1 = 0.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(graph_compactness(&g).unwrap().abs() < 1e-10);
/// ```
pub fn graph_compactness(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let dist = all_pairs_bfs(graph)?;

    let mut sum = 0_u64;
    let mut diameter = 0_u32;
    let mut pair_count = 0_u64;

    for v in 0..n {
        for u in (v + 1)..n {
            let d = dist[v * n + u];
            if d == u32::MAX {
                return Ok(0.0);
            }
            sum += u64::from(d);
            pair_count += 1;
            if d > diameter {
                diameter = d;
            }
        }
    }

    if diameter == 0 || pair_count == 0 {
        return Ok(0.0);
    }

    let avg_dist = sum as f64 / pair_count as f64;
    Ok(1.0 - avg_dist / f64::from(diameter))
}

fn all_pairs_bfs(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount() as usize;
    let mut dist = vec![u32::MAX; n * n];

    for s in 0..n {
        dist[s * n + s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);
        while let Some(v) = queue.pop_front() {
            let d = dist[s * n + v];
            let neighbors = graph.neighbors(v as u32)?;
            for &u in &neighbors {
                let ui = u as usize;
                if dist[s * n + ui] == u32::MAX {
                    dist[s * n + ui] = d + 1;
                    queue.push_back(ui);
                }
            }
        }
    }

    Ok(dist)
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

    // --- diameter_radius_ratio ---

    #[test]
    fn drr_empty() {
        let g = Graph::with_vertices(0);
        assert!(diameter_radius_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn drr_single() {
        let g = Graph::with_vertices(1);
        assert!(diameter_radius_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn drr_single_edge() {
        // diam=1, rad=1 → 1.0
        assert!((diameter_radius_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn drr_k3() {
        // diam=1, rad=1 → 1.0
        assert!((diameter_radius_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn drr_k4() {
        // diam=1, rad=1 → 1.0
        assert!((diameter_radius_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn drr_cycle4() {
        // diam=2, rad=2 → 1.0
        assert!((diameter_radius_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn drr_path3() {
        // ecc: [2,1,2], diam=2, rad=1 → 2.0
        assert!((diameter_radius_ratio(&path3()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn drr_star5() {
        // ecc: center=1, leaves=2, diam=2, rad=1 → 2.0
        assert!((diameter_radius_ratio(&star5()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn drr_paw() {
        // Distances from each vertex:
        // v0: [0,1,1,2] ecc=2
        // v1: [1,0,1,2] ecc=2
        // v2: [1,1,0,1] ecc=1
        // v3: [2,2,1,0] ecc=2
        // diam=2, rad=1 → 2.0
        assert!((diameter_radius_ratio(&paw()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn drr_disconnected() {
        // Disconnected → 0.0
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!(diameter_radius_ratio(&g).unwrap().abs() < 1e-10);
    }

    // --- avg_path_fraction ---

    #[test]
    fn apf_empty() {
        let g = Graph::with_vertices(0);
        assert!(avg_path_fraction(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn apf_single() {
        let g = Graph::with_vertices(1);
        assert!(avg_path_fraction(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn apf_k3() {
        // avg=1, diam=1 → 1.0
        assert!((avg_path_fraction(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn apf_k4() {
        // avg=1, diam=1 → 1.0
        assert!((avg_path_fraction(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn apf_path3() {
        // pairs: (0,1)=1, (0,2)=2, (1,2)=1 → sum=4, avg=4/3
        // diam=2 → 4/3/2 = 2/3
        assert!((avg_path_fraction(&path3()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn apf_cycle4() {
        // pairs: (0,1)=1,(0,2)=2,(0,3)=1,(1,2)=1,(1,3)=2,(2,3)=1
        // sum=8, avg=8/6=4/3, diam=2 → (4/3)/2 = 2/3
        assert!((avg_path_fraction(&cycle4()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn apf_disconnected() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!(avg_path_fraction(&g).unwrap().abs() < 1e-10);
    }

    // --- efficiency_ratio ---

    #[test]
    fn er_empty() {
        let g = Graph::with_vertices(0);
        assert!(efficiency_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn er_single() {
        let g = Graph::with_vertices(1);
        assert!(efficiency_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn er_k3() {
        // All pairs distance 1, inv = 1 each, avg=3/3=1.0
        assert!((efficiency_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn er_k4() {
        // All pairs distance 1, inv = 1 each, avg=6/6=1.0
        assert!((efficiency_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn er_path3() {
        // pairs: (0,1)=1→1, (0,2)=2→0.5, (1,2)=1→1 → sum=2.5/3
        assert!((efficiency_ratio(&path3()).unwrap() - 2.5 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn er_cycle4() {
        // pairs: (0,1)=1→1, (0,2)=2→0.5, (0,3)=1→1, (1,2)=1→1, (1,3)=2→0.5, (2,3)=1→1
        // sum_inv = 5, pairs=6, eff=5/6
        assert!((efficiency_ratio(&cycle4()).unwrap() - 5.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn er_isolated() {
        let g = Graph::with_vertices(3);
        // All pairs infinite → 0 inverse → eff=0
        assert!(efficiency_ratio(&g).unwrap().abs() < 1e-10);
    }

    // --- graph_compactness ---

    #[test]
    fn gc_empty() {
        let g = Graph::with_vertices(0);
        assert!(graph_compactness(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gc_single() {
        let g = Graph::with_vertices(1);
        assert!(graph_compactness(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gc_k3() {
        // avg=1, diam=1 → 1-1/1 = 0
        assert!(graph_compactness(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gc_k4() {
        // avg=1, diam=1 → 0
        assert!(graph_compactness(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gc_path3() {
        // avg=4/3, diam=2 → 1 - (4/3)/2 = 1 - 2/3 = 1/3
        assert!((graph_compactness(&path3()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn gc_cycle4() {
        // avg=4/3, diam=2 → 1 - 2/3 = 1/3
        assert!((graph_compactness(&cycle4()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn gc_disconnected() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!(graph_compactness(&g).unwrap().abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn drr_ge1() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = diameter_radius_ratio(g).unwrap();
            assert!(r >= 1.0 - 1e-10 || r.abs() < 1e-10);
        }
    }

    #[test]
    fn apf_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = avg_path_fraction(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn er_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = efficiency_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn gc_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = graph_compactness(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn complete_graphs_self_centered() {
        // Complete graphs: diameter = radius → ratio = 1
        assert!((diameter_radius_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((diameter_radius_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn complete_graphs_full_efficiency() {
        assert!((efficiency_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((efficiency_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }
}
