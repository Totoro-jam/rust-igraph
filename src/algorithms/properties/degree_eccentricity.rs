//! Degree-eccentricity indices (ALGO-TR-058).
//!
//! Topological indices combining vertex degree with eccentricity.
//!
//! - **Lanzhou index** `Lz(G) = Σ_{v∈V} d(v)² · ε(v)`
//!   Introduced by Vukičević et al. (2018). Product of squared degree
//!   and eccentricity summed over all vertices.
//! - **Degree-eccentricity index** `DE(G) = Σ_{v∈V} d(v) · ε(v)`
//!   Product of degree and eccentricity summed over all vertices.
//! - **Eccentric-distance sum** `ξ^d(G) = Σ_{v∈V} ε(v) · D(v)`
//!   where `D(v) = Σ_{u∈V} dist(v,u)` is the distance sum (status) of v.
//!   Introduced by Gupta et al. (2002).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

fn bfs_ecc_and_dist_sum(graph: &Graph) -> IgraphResult<(Vec<u32>, Vec<u64>)> {
    let n = graph.vcount() as usize;
    let mut ecc = vec![0_u32; n];
    let mut dist_sum = vec![0_u64; n];

    for s in 0..n {
        let mut dist = vec![u32::MAX; n];
        dist[s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s as u32);
        while let Some(u) = queue.pop_front() {
            let d_u = dist[u as usize];
            for nb in graph.neighbors(u)? {
                let idx = nb as usize;
                if dist[idx] == u32::MAX {
                    dist[idx] = d_u + 1;
                    queue.push_back(nb);
                }
            }
        }
        let mut max_d = 0_u32;
        let mut sum = 0_u64;
        for &d in &dist {
            if d != u32::MAX {
                if d > max_d {
                    max_d = d;
                }
                sum += u64::from(d);
            }
        }
        ecc[s] = max_d;
        dist_sum[s] = sum;
    }

    Ok((ecc, dist_sum))
}

/// Compute the Lanzhou index.
///
/// `Lz(G) = Σ_{v∈V} d(v)² · ε(v)`
///
/// Returns 0 for empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, lanzhou_index};
///
/// // Path 0-1-2: degrees [1,2,1], eccentricities [2,1,2]
/// // Lz = 1·2 + 4·1 + 1·2 = 8
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(lanzhou_index(&g).unwrap(), 8);
/// ```
pub fn lanzhou_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0);
    }

    let (ecc, _) = bfs_ecc_and_dist_sum(graph)?;
    let mut lz = 0_u64;

    for v in 0..n {
        let d = graph.degree(v)? as u64;
        let e = u64::from(ecc[v as usize]);
        lz = lz.saturating_add(d.saturating_mul(d).saturating_mul(e));
    }

    Ok(lz)
}

/// Compute the degree-eccentricity index.
///
/// `DE(G) = Σ_{v∈V} d(v) · ε(v)`
///
/// Returns 0 for empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_eccentricity_index};
///
/// // Path 0-1-2: degrees [1,2,1], eccentricities [2,1,2]
/// // DE = 1·2 + 2·1 + 1·2 = 6
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(degree_eccentricity_index(&g).unwrap(), 6);
/// ```
pub fn degree_eccentricity_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0);
    }

    let (ecc, _) = bfs_ecc_and_dist_sum(graph)?;
    let mut de = 0_u64;

    for v in 0..n {
        let d = graph.degree(v)? as u64;
        let e = u64::from(ecc[v as usize]);
        de = de.saturating_add(d.saturating_mul(e));
    }

    Ok(de)
}

/// Compute the eccentric-distance sum.
///
/// `ξ^d(G) = Σ_{v∈V} ε(v) · D(v)`
///
/// where `D(v) = Σ_{u} dist(v,u)` is the distance sum of v.
/// Returns 0 for empty graphs. Disconnected components contribute
/// only within their own component.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, eccentric_distance_sum};
///
/// // Path 0-1-2: ecc [2,1,2], D [3,2,3]
/// // ξ^d = 2·3 + 1·2 + 2·3 = 14
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(eccentric_distance_sum(&g).unwrap(), 14);
/// ```
pub fn eccentric_distance_sum(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0);
    }

    let (ecc, dist_sum) = bfs_ecc_and_dist_sum(graph)?;
    let mut eds = 0_u64;

    for v in 0..n as usize {
        let e = u64::from(ecc[v]);
        eds = eds.saturating_add(e.saturating_mul(dist_sum[v]));
    }

    Ok(eds)
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

    // --- bfs helper ---

    #[test]
    fn bfs_path3() {
        let (ecc, ds) = bfs_ecc_and_dist_sum(&path3()).unwrap();
        assert_eq!(ecc, vec![2, 1, 2]);
        assert_eq!(ds, vec![3, 2, 3]);
    }

    #[test]
    fn bfs_k3() {
        let (ecc, ds) = bfs_ecc_and_dist_sum(&k3()).unwrap();
        assert_eq!(ecc, vec![1, 1, 1]);
        assert_eq!(ds, vec![2, 2, 2]);
    }

    #[test]
    fn bfs_star5() {
        let (ecc, ds) = bfs_ecc_and_dist_sum(&star5()).unwrap();
        assert_eq!(ecc[0], 1);
        for i in 1..5 {
            assert_eq!(ecc[i], 2);
        }
        assert_eq!(ds[0], 4);
        for i in 1..5 {
            assert_eq!(ds[i], 7);
        }
    }

    // --- lanzhou_index ---

    #[test]
    fn lz_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(lanzhou_index(&g).unwrap(), 0);
    }

    #[test]
    fn lz_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(lanzhou_index(&g).unwrap(), 0);
    }

    #[test]
    fn lz_single_edge() {
        // degrees [1,1], ecc [1,1]: 1·1 + 1·1 = 2
        assert_eq!(lanzhou_index(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn lz_path3() {
        // d²·ε: 1·2 + 4·1 + 1·2 = 8
        assert_eq!(lanzhou_index(&path3()).unwrap(), 8);
    }

    #[test]
    fn lz_path4() {
        // degrees [1,2,2,1], ecc [3,2,2,3]
        // 1·3 + 4·2 + 4·2 + 1·3 = 3+8+8+3 = 22
        assert_eq!(lanzhou_index(&path4()).unwrap(), 22);
    }

    #[test]
    fn lz_k3() {
        // d²·ε: 3·(4·1) = 12
        assert_eq!(lanzhou_index(&k3()).unwrap(), 12);
    }

    #[test]
    fn lz_k4() {
        // d²·ε: 4·(9·1) = 36
        assert_eq!(lanzhou_index(&k4()).unwrap(), 36);
    }

    #[test]
    fn lz_cycle4() {
        // degrees all 2, ecc all 2: 4·(4·2) = 32
        assert_eq!(lanzhou_index(&cycle4()).unwrap(), 32);
    }

    #[test]
    fn lz_cycle5() {
        // degrees all 2, ecc all 2: 5·(4·2) = 40
        assert_eq!(lanzhou_index(&cycle5()).unwrap(), 40);
    }

    #[test]
    fn lz_star5() {
        // center: d=4, ecc=1: 16·1 = 16
        // leaves: d=1, ecc=2: 4·(1·2) = 8
        assert_eq!(lanzhou_index(&star5()).unwrap(), 24);
    }

    #[test]
    fn lz_paw() {
        // degrees [2,2,3,1], ecc [2,2,1,2]
        // 4·2 + 4·2 + 9·1 + 1·2 = 8+8+9+2 = 27
        assert_eq!(lanzhou_index(&paw()).unwrap(), 27);
    }

    // --- degree_eccentricity_index ---

    #[test]
    fn de_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(degree_eccentricity_index(&g).unwrap(), 0);
    }

    #[test]
    fn de_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(degree_eccentricity_index(&g).unwrap(), 0);
    }

    #[test]
    fn de_single_edge() {
        // d·ε: 1·1 + 1·1 = 2
        assert_eq!(degree_eccentricity_index(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn de_path3() {
        // 1·2 + 2·1 + 1·2 = 6
        assert_eq!(degree_eccentricity_index(&path3()).unwrap(), 6);
    }

    #[test]
    fn de_path4() {
        // degrees [1,2,2,1], ecc [3,2,2,3]
        // 1·3 + 2·2 + 2·2 + 1·3 = 3+4+4+3 = 14
        assert_eq!(degree_eccentricity_index(&path4()).unwrap(), 14);
    }

    #[test]
    fn de_k3() {
        // 3·(2·1) = 6
        assert_eq!(degree_eccentricity_index(&k3()).unwrap(), 6);
    }

    #[test]
    fn de_k4() {
        // 4·(3·1) = 12
        assert_eq!(degree_eccentricity_index(&k4()).unwrap(), 12);
    }

    #[test]
    fn de_cycle4() {
        // 4·(2·2) = 16
        assert_eq!(degree_eccentricity_index(&cycle4()).unwrap(), 16);
    }

    #[test]
    fn de_cycle5() {
        // 5·(2·2) = 20
        assert_eq!(degree_eccentricity_index(&cycle5()).unwrap(), 20);
    }

    #[test]
    fn de_star5() {
        // center: 4·1 = 4, leaves: 4·(1·2) = 8
        assert_eq!(degree_eccentricity_index(&star5()).unwrap(), 12);
    }

    #[test]
    fn de_paw() {
        // [2,2,3,1], ecc [2,2,1,2]
        // 2·2 + 2·2 + 3·1 + 1·2 = 4+4+3+2 = 13
        assert_eq!(degree_eccentricity_index(&paw()).unwrap(), 13);
    }

    #[test]
    fn de_leq_lz() {
        // DE = Σd·ε ≤ Σd²·ε = Lz when d ≥ 1
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(degree_eccentricity_index(g).unwrap() <= lanzhou_index(g).unwrap());
        }
    }

    // --- eccentric_distance_sum ---

    #[test]
    fn eds_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(eccentric_distance_sum(&g).unwrap(), 0);
    }

    #[test]
    fn eds_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(eccentric_distance_sum(&g).unwrap(), 0);
    }

    #[test]
    fn eds_single_edge() {
        // ecc [1,1], D [1,1]: 1·1 + 1·1 = 2
        assert_eq!(eccentric_distance_sum(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn eds_path3() {
        // ecc [2,1,2], D [3,2,3]: 2·3 + 1·2 + 2·3 = 14
        assert_eq!(eccentric_distance_sum(&path3()).unwrap(), 14);
    }

    #[test]
    fn eds_path4() {
        // ecc [3,2,2,3], D [6,4,4,6]: 3·6 + 2·4 + 2·4 + 3·6 = 18+8+8+18 = 52
        assert_eq!(eccentric_distance_sum(&path4()).unwrap(), 52);
    }

    #[test]
    fn eds_k3() {
        // ecc [1,1,1], D [2,2,2]: 3·(1·2) = 6
        assert_eq!(eccentric_distance_sum(&k3()).unwrap(), 6);
    }

    #[test]
    fn eds_k4() {
        // ecc [1,1,1,1], D [3,3,3,3]: 4·(1·3) = 12
        assert_eq!(eccentric_distance_sum(&k4()).unwrap(), 12);
    }

    #[test]
    fn eds_cycle4() {
        // ecc [2,2,2,2], D [4,4,4,4]: 4·(2·4) = 32
        assert_eq!(eccentric_distance_sum(&cycle4()).unwrap(), 32);
    }

    #[test]
    fn eds_cycle5() {
        // ecc [2,2,2,2,2], D [6,6,6,6,6]: 5·(2·6) = 60
        assert_eq!(eccentric_distance_sum(&cycle5()).unwrap(), 60);
    }

    #[test]
    fn eds_star5() {
        // ecc [1,2,2,2,2], D [4,7,7,7,7]
        // 1·4 + 4·(2·7) = 4+56 = 60
        assert_eq!(eccentric_distance_sum(&star5()).unwrap(), 60);
    }

    #[test]
    fn eds_paw() {
        // ecc [2,2,1,2], D:
        // D(0): 0+1+1+2=4, D(1): 1+0+1+2=4, D(2): 1+1+0+1=3, D(3): 2+2+1+0=5
        // ξ^d = 2·4 + 2·4 + 1·3 + 2·5 = 8+8+3+10 = 29
        assert_eq!(eccentric_distance_sum(&paw()).unwrap(), 29);
    }

    // --- cross-consistency ---

    #[test]
    fn all_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(lanzhou_index(g).unwrap() > 0);
            assert!(degree_eccentricity_index(g).unwrap() > 0);
            assert!(eccentric_distance_sum(g).unwrap() > 0);
        }
    }

    #[test]
    fn lz_regular_formula() {
        // r-regular, ε-uniform: Lz = n · r² · ε
        for g in &[k3(), k4()] {
            let n = u64::from(g.vcount());
            let r = g.degree(0).unwrap() as u64;
            let (ecc, _) = bfs_ecc_and_dist_sum(g).unwrap();
            let e = u64::from(ecc[0]);
            assert_eq!(lanzhou_index(g).unwrap(), n * r * r * e);
        }
    }

    #[test]
    fn de_regular_formula() {
        // r-regular, ε-uniform: DE = n · r · ε
        for g in &[k3(), k4()] {
            let n = u64::from(g.vcount());
            let r = g.degree(0).unwrap() as u64;
            let (ecc, _) = bfs_ecc_and_dist_sum(g).unwrap();
            let e = u64::from(ecc[0]);
            assert_eq!(degree_eccentricity_index(g).unwrap(), n * r * e);
        }
    }
}
