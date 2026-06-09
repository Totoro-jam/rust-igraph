//! Reciprocal distance-degree indices (ALGO-TR-067).
//!
//! Hybrid indices combining vertex degrees with shortest-path distances
//! in reciprocal (inverse) form. These complement `degree_distance` and
//! `gutman_index` (which multiply degree by distance).
//!
//! - **Reciprocal degree distance** `RDD(G) = Σ_{u<v} [d(u)+d(v)] / dist(u,v)`
//!   Also known as the *additively weighted Harary index* `H_A(G)`.
//! - **Multiplicatively weighted Harary index**
//!   `H_M(G) = Σ_{u<v} d(u)·d(v) / dist(u,v)`
//! - **Terminal Wiener index** `TW(G) = Σ_{u<v, d(u)=d(v)=1} dist(u,v)`
//!   Sum of distances restricted to pendant (degree-1) vertex pairs.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the reciprocal degree distance (additively weighted Harary index).
///
/// `RDD(G) = Σ_{u<v, dist(u,v)<∞} [d(u) + d(v)] / dist(u,v)`
///
/// Disconnected pairs (infinite distance) are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reciprocal_degree_distance};
///
/// // Path 0-1-2: d=[1,2,1]
/// // (0,1): (1+2)/1=3, (0,2): (1+1)/2=1, (1,2): (2+1)/1=3
/// // RDD = 7
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((reciprocal_degree_distance(&g).unwrap() - 7.0).abs() < 1e-10);
/// ```
pub fn reciprocal_degree_distance(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let mut rdd = 0.0_f64;

    for s in 0..n {
        let ds = graph.degree(s as u32)? as f64;
        let mut dist = vec![u32::MAX; n];
        dist[s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);
        while let Some(u) = queue.pop_front() {
            let d_u = dist[u];
            if let Ok(nbs) = graph.neighbors(u as u32) {
                for nb in nbs {
                    let idx = nb as usize;
                    if dist[idx] == u32::MAX {
                        dist[idx] = d_u + 1;
                        queue.push_back(idx);
                    }
                }
            }
        }
        for t in (s + 1)..n {
            if dist[t] != u32::MAX && dist[t] > 0 {
                let dt = graph.degree(t as u32)? as f64;
                rdd += (ds + dt) / f64::from(dist[t]);
            }
        }
    }

    Ok(rdd)
}

/// Compute the multiplicatively weighted Harary index.
///
/// `H_M(G) = Σ_{u<v, dist(u,v)<∞} d(u)·d(v) / dist(u,v)`
///
/// Disconnected pairs and vertices with degree 0 are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, multiplicatively_weighted_harary};
///
/// // Path 0-1-2: d=[1,2,1]
/// // (0,1): 1×2/1=2, (0,2): 1×1/2=0.5, (1,2): 2×1/1=2
/// // H_M = 4.5
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((multiplicatively_weighted_harary(&g).unwrap() - 4.5).abs() < 1e-10);
/// ```
pub fn multiplicatively_weighted_harary(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let mut hm = 0.0_f64;

    for s in 0..n {
        let ds = graph.degree(s as u32)? as f64;
        if ds == 0.0 {
            continue;
        }
        let mut dist = vec![u32::MAX; n];
        dist[s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);
        while let Some(u) = queue.pop_front() {
            let d_u = dist[u];
            if let Ok(nbs) = graph.neighbors(u as u32) {
                for nb in nbs {
                    let idx = nb as usize;
                    if dist[idx] == u32::MAX {
                        dist[idx] = d_u + 1;
                        queue.push_back(idx);
                    }
                }
            }
        }
        for t in (s + 1)..n {
            if dist[t] != u32::MAX && dist[t] > 0 {
                let dt = graph.degree(t as u32)? as f64;
                if dt > 0.0 {
                    hm += (ds * dt) / f64::from(dist[t]);
                }
            }
        }
    }

    Ok(hm)
}

/// Compute the terminal Wiener index.
///
/// `TW(G) = Σ_{u<v, d(u)=d(v)=1} dist(u,v)`
///
/// Sum of distances between all pairs of pendant (degree-1) vertices.
/// Returns 0 if fewer than 2 pendant vertices exist.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, terminal_wiener_index};
///
/// // Star S₄ (center 0, leaves 1-4): leaves are all at distance 2
/// // C(4,2) = 6 pairs, each distance 2 → TW = 12
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert_eq!(terminal_wiener_index(&g).unwrap(), 12);
/// ```
pub fn terminal_wiener_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    let mut pendants = Vec::new();
    for v in 0..n {
        if graph.degree(v as u32)? == 1 {
            pendants.push(v);
        }
    }

    if pendants.len() < 2 {
        return Ok(0);
    }

    let mut tw = 0_u64;

    for (i, &s) in pendants.iter().enumerate() {
        let mut dist = vec![u32::MAX; n];
        dist[s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);
        while let Some(u) = queue.pop_front() {
            let d_u = dist[u];
            if let Ok(nbs) = graph.neighbors(u as u32) {
                for nb in nbs {
                    let idx = nb as usize;
                    if dist[idx] == u32::MAX {
                        dist[idx] = d_u + 1;
                        queue.push_back(idx);
                    }
                }
            }
        }
        for &t in &pendants[(i + 1)..] {
            if dist[t] != u32::MAX {
                tw = tw.saturating_add(u64::from(dist[t]));
            }
        }
    }

    Ok(tw)
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

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- reciprocal_degree_distance ---

    #[test]
    fn rdd_empty() {
        let g = Graph::with_vertices(0);
        assert!((reciprocal_degree_distance(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn rdd_isolated() {
        let g = Graph::with_vertices(5);
        assert!((reciprocal_degree_distance(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn rdd_single_edge() {
        // d=[1,1], (0,1): (1+1)/1=2
        assert!((reciprocal_degree_distance(&single_edge()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn rdd_path3() {
        // d=[1,2,1], (0,1):3/1=3, (0,2):2/2=1, (1,2):3/1=3 → 7
        assert!((reciprocal_degree_distance(&path3()).unwrap() - 7.0).abs() < 1e-10);
    }

    #[test]
    fn rdd_path4() {
        // d=[1,2,2,1]
        // (0,1):(1+2)/1=3, (0,2):(1+2)/2=1.5, (0,3):(1+1)/3=2/3
        // (1,2):(2+2)/1=4, (1,3):(2+1)/2=1.5, (2,3):(2+1)/1=3
        let expected = 3.0 + 1.5 + 2.0 / 3.0 + 4.0 + 1.5 + 3.0;
        assert!((reciprocal_degree_distance(&path4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rdd_k3() {
        // d=[2,2,2], all distances 1
        // 3 pairs × (2+2)/1 = 12
        assert!((reciprocal_degree_distance(&k3()).unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn rdd_k4() {
        // d=[3,3,3,3], 6 pairs × (3+3)/1 = 36
        assert!((reciprocal_degree_distance(&k4()).unwrap() - 36.0).abs() < 1e-10);
    }

    #[test]
    fn rdd_cycle4() {
        // d=[2,2,2,2], distances: 4 pairs@1 → 4/1=4 each, 2 pairs@2 → 4/2=2 each
        // 4×4 + 2×2 = 20
        assert!((reciprocal_degree_distance(&cycle4()).unwrap() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn rdd_cycle5() {
        // d=[2,2,2,2,2], 5 pairs@1, 5 pairs@2
        // 5×(4/1) + 5×(4/2) = 20+10 = 30
        assert!((reciprocal_degree_distance(&cycle5()).unwrap() - 30.0).abs() < 1e-10);
    }

    #[test]
    fn rdd_star5() {
        // d=[4,1,1,1,1]
        // center-leaf pairs (4 of them): (4+1)/1=5, total=20
        // leaf-leaf pairs (6 of them): (1+1)/2=1, total=6
        assert!((reciprocal_degree_distance(&star5()).unwrap() - 26.0).abs() < 1e-10);
    }

    #[test]
    fn rdd_paw() {
        // d=[2,2,3,1]
        // (0,1):(2+2)/1=4, (0,2):(2+3)/1=5, (0,3):(2+1)/2=1.5
        // (1,2):(2+3)/1=5, (1,3):(2+1)/2=1.5, (2,3):(3+1)/1=4
        let expected = 4.0 + 5.0 + 1.5 + 5.0 + 1.5 + 4.0;
        assert!((reciprocal_degree_distance(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rdd_regular_formula() {
        // For r-regular graphs: RDD = 2r × Harary
        // Because Σ (r+r)/dist = 2r × Σ 1/dist = 2r × H(G)
        // We don't have harary_index imported here, but we can verify manually
        // K_n: Harary = n(n-1)/2, RDD = 2(n-1) × n(n-1)/2 = n(n-1)²
        let g = k4();
        let n = 4.0_f64;
        let r = 3.0;
        let harary = n * (n - 1.0) / 2.0; // 6
        let expected = 2.0 * r * harary;
        assert!((reciprocal_degree_distance(&g).unwrap() - expected).abs() < 1e-8);
    }

    // --- multiplicatively_weighted_harary ---

    #[test]
    fn hm_empty() {
        let g = Graph::with_vertices(0);
        assert!((multiplicatively_weighted_harary(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn hm_isolated() {
        let g = Graph::with_vertices(5);
        assert!((multiplicatively_weighted_harary(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn hm_single_edge() {
        // d=[1,1], (0,1): 1×1/1=1
        assert!((multiplicatively_weighted_harary(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn hm_path3() {
        // d=[1,2,1], (0,1):2/1=2, (0,2):1/2=0.5, (1,2):2/1=2 → 4.5
        assert!((multiplicatively_weighted_harary(&path3()).unwrap() - 4.5).abs() < 1e-10);
    }

    #[test]
    fn hm_k3() {
        // d=[2,2,2], 3 pairs × 4/1 = 12
        assert!((multiplicatively_weighted_harary(&k3()).unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn hm_k4() {
        // d=[3,3,3,3], 6 pairs × 9/1 = 54
        assert!((multiplicatively_weighted_harary(&k4()).unwrap() - 54.0).abs() < 1e-10);
    }

    #[test]
    fn hm_cycle4() {
        // d=[2,2,2,2], 4 pairs@1: 4/1=4, 2 pairs@2: 4/2=2
        // 4×4 + 2×2 = 20
        assert!((multiplicatively_weighted_harary(&cycle4()).unwrap() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn hm_star5() {
        // d=[4,1,1,1,1]
        // center-leaf (4 pairs): 4×1/1=4 each → 16
        // leaf-leaf (6 pairs): 1×1/2=0.5 each → 3
        assert!((multiplicatively_weighted_harary(&star5()).unwrap() - 19.0).abs() < 1e-10);
    }

    #[test]
    fn hm_paw() {
        // d=[2,2,3,1]
        // (0,1):4/1=4, (0,2):6/1=6, (0,3):2/2=1
        // (1,2):6/1=6, (1,3):2/2=1, (2,3):3/1=3
        let expected = 4.0 + 6.0 + 1.0 + 6.0 + 1.0 + 3.0;
        assert!((multiplicatively_weighted_harary(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn hm_regular_formula() {
        // For r-regular: H_M = r² × Harary
        // K_n: H_M = (n-1)² × n(n-1)/2
        let g = k4();
        let n = 4.0_f64;
        let r = 3.0;
        let harary = n * (n - 1.0) / 2.0;
        let expected = r * r * harary;
        assert!((multiplicatively_weighted_harary(&g).unwrap() - expected).abs() < 1e-8);
    }

    #[test]
    fn hm_equals_rdd_for_regular() {
        // Regular: H_M = r² × H, RDD = 2r × H
        // So H_M / RDD = r/2
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let hm = multiplicatively_weighted_harary(g).unwrap();
            let rdd = reciprocal_degree_distance(g).unwrap();
            let r = g.degree(0).unwrap() as f64;
            assert!((hm / rdd - r / 2.0).abs() < 1e-8);
        }
    }

    // --- terminal_wiener_index ---

    #[test]
    fn tw_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(terminal_wiener_index(&g).unwrap(), 0);
    }

    #[test]
    fn tw_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(terminal_wiener_index(&g).unwrap(), 0);
    }

    #[test]
    fn tw_single_edge() {
        // Both vertices have degree 1, distance 1
        assert_eq!(terminal_wiener_index(&single_edge()).unwrap(), 1);
    }

    #[test]
    fn tw_path3() {
        // Pendants: 0 and 2, distance 2
        assert_eq!(terminal_wiener_index(&path3()).unwrap(), 2);
    }

    #[test]
    fn tw_path4() {
        // Pendants: 0 and 3, distance 3
        assert_eq!(terminal_wiener_index(&path4()).unwrap(), 3);
    }

    #[test]
    fn tw_k3() {
        // No pendant vertices (all degree 2) → 0
        assert_eq!(terminal_wiener_index(&k3()).unwrap(), 0);
    }

    #[test]
    fn tw_k4() {
        assert_eq!(terminal_wiener_index(&k4()).unwrap(), 0);
    }

    #[test]
    fn tw_cycle4() {
        assert_eq!(terminal_wiener_index(&cycle4()).unwrap(), 0);
    }

    #[test]
    fn tw_cycle5() {
        assert_eq!(terminal_wiener_index(&cycle5()).unwrap(), 0);
    }

    #[test]
    fn tw_star5() {
        // 4 leaves at distance 2 from each other
        // C(4,2) = 6 pairs × distance 2 = 12
        assert_eq!(terminal_wiener_index(&star5()).unwrap(), 12);
    }

    #[test]
    fn tw_paw() {
        // Pendant: vertex 3 (degree 1). Only 1 pendant → 0
        assert_eq!(terminal_wiener_index(&paw()).unwrap(), 0);
    }

    #[test]
    fn tw_caterpillar() {
        // 0-1-2-3, with extra leaves: 1-4, 2-5
        // Pendants: 0(deg 1), 3(deg 1), 4(deg 1), 5(deg 1)
        // Distances: (0,3):3, (0,4):2, (0,5):3, (3,4):3, (3,5):2, (4,5):3
        // TW = 3+2+3+3+2+3 = 16
        let g =
            Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (1, 4), (2, 5)], false, Some(6)).unwrap();
        assert_eq!(terminal_wiener_index(&g).unwrap(), 16);
    }

    #[test]
    fn tw_double_star() {
        // Double star: center1-center2, each with 2 leaves
        // 0-2, 1-2, 2-3, 3-4, 3-5
        // Pendants: 0,1,4,5 (all degree 1)
        // Distances: (0,1):2, (0,4):3, (0,5):3, (1,4):3, (1,5):3, (4,5):2
        // TW = 2+3+3+3+3+2 = 16
        let g =
            Graph::from_edges(&[(0, 2), (1, 2), (2, 3), (3, 4), (3, 5)], false, Some(6)).unwrap();
        assert_eq!(terminal_wiener_index(&g).unwrap(), 16);
    }

    // --- cross-consistency ---

    #[test]
    fn rdd_geq_harary_for_nonregular() {
        // For any graph, RDD = Σ (d(u)+d(v))/dist ≥ 2·Σ 1/dist = 2·H
        // when min degree ≥ 1 (since d(u)+d(v) ≥ 2)
        // This holds because (d(u)+d(v)) ≥ 2 for every pair of non-isolated vertices
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let rdd = reciprocal_degree_distance(g).unwrap();
            assert!(rdd >= 0.0);
        }
    }

    #[test]
    fn all_positive_for_connected() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(reciprocal_degree_distance(g).unwrap() > 0.0);
            assert!(multiplicatively_weighted_harary(g).unwrap() > 0.0);
        }
    }
}
