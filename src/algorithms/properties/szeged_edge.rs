//! Edge-Szeged and Graovac-Ghorbani indices (ALGO-TR-059).
//!
//! Szeged-like indices using edge and vertex proximity counts.
//!
//! - **Edge-Szeged index** `Sz_e(G) = Σ_{(u,v)∈E} m_u(e) · m_v(e)`
//!   where `m_u(e)` = number of edges closer to u than to v.
//!   Introduced by Gutman & Ashrafi (2008).
//! - **Edge-PI index** `PI_e(G) = Σ_{(u,v)∈E} [m_u(e) + m_v(e)]`
//!   Edge version of the Padmakar-Ivan index.
//! - **Graovac-Ghorbani index** `GG(G) = Σ_{(u,v)∈E} ln(n_u·n_v) / ln(n_u+n_v)`
//!   where `n_u(e)` = vertices closer to u than v for edge (u,v).
//!   Introduced by Graovac & Ghorbani (2010). Undefined (skipped)
//!   when `n_u + n_v ≤ 1`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

fn all_pairs_bfs(graph: &Graph, n: usize) -> Vec<Vec<u32>> {
    let mut dist = vec![vec![u32::MAX; n]; n];
    for s in 0..n {
        dist[s][s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);
        while let Some(u) = queue.pop_front() {
            let d_u = dist[s][u];
            if let Ok(nbs) = graph.neighbors(u as u32) {
                for nb in nbs {
                    let idx = nb as usize;
                    if dist[s][idx] == u32::MAX {
                        dist[s][idx] = d_u + 1;
                        queue.push_back(idx);
                    }
                }
            }
        }
    }
    dist
}

/// Compute the edge-Szeged index.
///
/// `Sz_e(G) = Σ_{(u,v)∈E} m_u(e) · m_v(e)`
///
/// where `m_u(e)` is the number of edges whose midpoint is closer to u
/// than to v. An edge (a,b) is closer to u when
/// `dist(u,a) + dist(u,b) < dist(v,a) + dist(v,b)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_szeged_index};
///
/// // Path 0-1-2: 2 edges
/// // edge (0,1): m_0=0, m_1=1 (edge (1,2) closer to 1) → 0
/// // edge (1,2): m_1=1 (edge (0,1) closer to 1), m_2=0 → 0
/// // Sz_e = 0
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(edge_szeged_index(&g).unwrap(), 0);
/// ```
pub fn edge_szeged_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let dist = all_pairs_bfs(graph, n);
    let edges: Vec<(u32, u32)> = graph.edges().collect();
    let m = edges.len();
    let mut sz_e = 0_u64;

    for &(u, v) in &edges {
        let ui = u as usize;
        let vi = v as usize;
        let mut mu = 0_u64;
        let mut mv = 0_u64;

        for i in 0..m {
            let (a, b) = edges[i];
            let ai = a as usize;
            let bi = b as usize;

            if dist[ui][ai] == u32::MAX
                || dist[ui][bi] == u32::MAX
                || dist[vi][ai] == u32::MAX
                || dist[vi][bi] == u32::MAX
            {
                continue;
            }

            let du = u64::from(dist[ui][ai]) + u64::from(dist[ui][bi]);
            let dv = u64::from(dist[vi][ai]) + u64::from(dist[vi][bi]);

            match du.cmp(&dv) {
                std::cmp::Ordering::Less => mu += 1,
                std::cmp::Ordering::Greater => mv += 1,
                std::cmp::Ordering::Equal => {}
            }
        }

        sz_e = sz_e.saturating_add(mu.saturating_mul(mv));
    }

    Ok(sz_e)
}

/// Compute the edge-PI index.
///
/// `PI_e(G) = Σ_{(u,v)∈E} [m_u(e) + m_v(e)]`
///
/// Edge version of the Padmakar-Ivan index. Uses the same proximity
/// definition as the edge-Szeged index.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_pi_index};
///
/// // Path 0-1-2:
/// // edge (0,1): m_0=0, m_1=1 → sum=1
/// // edge (1,2): m_1=1, m_2=0 → sum=1
/// // PI_e = 2
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(edge_pi_index(&g).unwrap(), 2);
/// ```
pub fn edge_pi_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let dist = all_pairs_bfs(graph, n);
    let edges: Vec<(u32, u32)> = graph.edges().collect();
    let m = edges.len();
    let mut pi_e = 0_u64;

    for &(u, v) in &edges {
        let ui = u as usize;
        let vi = v as usize;
        let mut mu = 0_u64;
        let mut mv = 0_u64;

        for i in 0..m {
            let (a, b) = edges[i];
            let ai = a as usize;
            let bi = b as usize;

            if dist[ui][ai] == u32::MAX
                || dist[ui][bi] == u32::MAX
                || dist[vi][ai] == u32::MAX
                || dist[vi][bi] == u32::MAX
            {
                continue;
            }

            let du = u64::from(dist[ui][ai]) + u64::from(dist[ui][bi]);
            let dv = u64::from(dist[vi][ai]) + u64::from(dist[vi][bi]);

            match du.cmp(&dv) {
                std::cmp::Ordering::Less => mu += 1,
                std::cmp::Ordering::Greater => mv += 1,
                std::cmp::Ordering::Equal => {}
            }
        }

        pi_e = pi_e.saturating_add(mu + mv);
    }

    Ok(pi_e)
}

/// Compute the Graovac-Ghorbani index.
///
/// `GG(G) = Σ_{(u,v)∈E} ln(n_u · n_v) / ln(n_u + n_v)`
///
/// where `n_u(e)` = number of vertices closer to u than to v.
/// Terms where `n_u + n_v ≤ 1` or either count is zero are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, graovac_ghorbani_index};
///
/// // K_3: all edges have n_u=1, n_v=1 → ln(1)/ln(2) = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((graovac_ghorbani_index(&g).unwrap()).abs() < 1e-10);
/// ```
pub fn graovac_ghorbani_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let dist = all_pairs_bfs(graph, n);
    let mut gg = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let ui = u as usize;
        let vi = v as usize;
        let mut nu = 0_u64;
        let mut nv = 0_u64;

        for w in 0..n {
            if dist[ui][w] == u32::MAX || dist[vi][w] == u32::MAX {
                continue;
            }
            match dist[ui][w].cmp(&dist[vi][w]) {
                std::cmp::Ordering::Less => nu += 1,
                std::cmp::Ordering::Greater => nv += 1,
                std::cmp::Ordering::Equal => {}
            }
        }

        if nu > 0 && nv > 0 {
            let sum = nu + nv;
            if sum > 1 {
                let prod = (nu as f64) * (nv as f64);
                gg += prod.ln() / (sum as f64).ln();
            }
        }
    }

    Ok(gg)
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

    // --- edge_szeged_index ---

    #[test]
    fn esz_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(edge_szeged_index(&g).unwrap(), 0);
    }

    #[test]
    fn esz_single_edge() {
        // Only 1 edge, no other edges to count → m_u=0, m_v=0 → 0
        assert_eq!(edge_szeged_index(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn esz_path3() {
        // edge (0,1): other edge (1,2) → du=0+1=1, dv=1+0=1 → tie, neither counted
        // Actually: dist(0,1)+dist(0,2)=0+2=2 vs dist(1,1)+dist(1,2)=0+1=1
        // Wait, m_u counts edges closer to u. For edge e=(0,1), checking edge f=(1,2):
        // dist(0,1)+dist(0,2) = 1+2 = 3 (dist from 0 to endpoints of f)
        // dist(1,1)+dist(1,2) = 0+1 = 1 (dist from 1 to endpoints of f)
        // 1 < 3 → f closer to 1 → m_1 += 1
        // So m_0=0, m_1=1 → 0·1 = 0
        //
        // edge (1,2): checking edge f=(0,1):
        // dist(1,0)+dist(1,1) = 1+0 = 1
        // dist(2,0)+dist(2,1) = 2+1 = 3
        // 1 < 3 → f closer to 1 → m_1 += 1
        // So m_1=1, m_2=0 → 1·0 = 0
        // Sz_e = 0
        assert_eq!(edge_szeged_index(&path3()).unwrap(), 0);
    }

    #[test]
    fn esz_path4() {
        // edges: (0,1), (1,2), (2,3)
        // edge (0,1): check (1,2): d0=1+2=3, d1=0+1=1 → m1; check (2,3): d0=2+3=5, d1=1+2=3 → m1
        // m_0=0, m_1=2 → 0
        // edge (1,2): check (0,1): d1=1+0=1, d2=2+1=3 → m1; check (2,3): d1=1+2=3, d2=0+1=1 → m2
        // m_1=1, m_2=1 → 1
        // edge (2,3): check (0,1): d2=2+1=3, d3=3+2=5 → m2; check (1,2): d2=0+1=1, d3=1+2=3 → m2
        // m_2=2, m_3=0 → 0
        // Sz_e = 0 + 1 + 0 = 1
        assert_eq!(edge_szeged_index(&path4()).unwrap(), 1);
    }

    #[test]
    fn esz_k3() {
        // All distances are 1. For any edge (u,v), other edge (a,b):
        // du_e = dist(u,a)+dist(u,b), dv_e = dist(v,a)+dist(v,b)
        // In K3, all non-self distances are 1. For edge (0,1), check edge (0,2):
        // d0 = 0+1 = 1, d1 = 1+1 = 2 → closer to 0 → m_0++
        // Check edge (1,2): d0 = 1+1 = 2, d1 = 0+1 = 1 → closer to 1 → m_1++
        // m_0=1, m_1=1 → 1. By symmetry each edge contributes 1.
        // Sz_e = 3
        assert_eq!(edge_szeged_index(&k3()).unwrap(), 3);
    }

    #[test]
    fn esz_k4() {
        // For K4, for any edge (u,v), consider another edge (a,b):
        // Case 1: u=a or u=b → du includes dist(u,u)=0, so du < dv (tie-break to u side)
        // Actually let's compute precisely.
        // For edge (0,1), check edge (0,2): d0=0+1=1, d1=1+1=2 → m0++
        // check edge (0,3): d0=0+1=1, d1=1+1=2 → m0++
        // check edge (1,2): d0=1+1=2, d1=0+1=1 → m1++
        // check edge (1,3): d0=1+1=2, d1=0+1=1 → m1++
        // check edge (2,3): d0=1+1=2, d1=1+1=2 → tie
        // m_0=2, m_1=2 → 4. By symmetry, 6 edges × 4 = 24
        assert_eq!(edge_szeged_index(&k4()).unwrap(), 24);
    }

    #[test]
    fn esz_cycle4() {
        // C4: edges (0,1),(1,2),(2,3),(3,0), distances: adjacent=1, opposite=2
        // For edge (0,1), check (1,2): d0=1+2=3, d1=0+1=1 → m1
        // check (2,3): d0=2+1=3, d1=1+2=3 → tie
        // check (3,0): d0=0+1=1, d1=1+2=3 → m0
        // m_0=1, m_1=1 → 1. By symmetry, 4 edges × 1 = 4
        assert_eq!(edge_szeged_index(&cycle4()).unwrap(), 4);
    }

    #[test]
    fn esz_star5() {
        // Star: center=0, leaves 1-4. All edges (0,k).
        // For edge (0,1), check edge (0,2): d0=0+1=1, d1=2+1=3 → m0
        // check (0,3): same → m0. check (0,4): same → m0.
        // m_0=3, m_1=0 → 0. By symmetry all 4 edges give 0.
        assert_eq!(edge_szeged_index(&star5()).unwrap(), 0);
    }

    #[test]
    fn esz_computes_ok() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let _ = edge_szeged_index(g).unwrap();
        }
    }

    // --- edge_pi_index ---

    #[test]
    fn epi_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(edge_pi_index(&g).unwrap(), 0);
    }

    #[test]
    fn epi_single_edge() {
        assert_eq!(edge_pi_index(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn epi_path3() {
        // edge (0,1): m0=0, m1=1 → 1; edge (1,2): m1=1, m2=0 → 1
        // PI_e = 2
        assert_eq!(edge_pi_index(&path3()).unwrap(), 2);
    }

    #[test]
    fn epi_path4() {
        // edge (0,1): 0+2=2; edge (1,2): 1+1=2; edge (2,3): 2+0=2
        // PI_e = 6
        assert_eq!(edge_pi_index(&path4()).unwrap(), 6);
    }

    #[test]
    fn epi_k3() {
        // each edge: 1+1=2, 3 edges → 6
        assert_eq!(edge_pi_index(&k3()).unwrap(), 6);
    }

    #[test]
    fn epi_k4() {
        // each edge: 2+2=4, 6 edges → 24
        assert_eq!(edge_pi_index(&k4()).unwrap(), 24);
    }

    #[test]
    fn epi_cycle4() {
        // each edge: 1+1=2, 4 edges → 8
        assert_eq!(edge_pi_index(&cycle4()).unwrap(), 8);
    }

    #[test]
    fn epi_star5() {
        // each edge: 3+0=3, 4 edges → 12
        assert_eq!(edge_pi_index(&star5()).unwrap(), 12);
    }

    #[test]
    fn epi_computes_ok() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let _ = edge_pi_index(g).unwrap();
        }
    }

    // --- graovac_ghorbani_index ---

    #[test]
    fn gg_empty() {
        let g = Graph::with_vertices(0);
        assert!((graovac_ghorbani_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn gg_single_edge() {
        // edge (0,1): n_0=1(vertex 0), n_1=1(vertex 1) → ln(1)/ln(2) = 0
        assert!((graovac_ghorbani_index(&single_edge()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn gg_path3() {
        // edge (0,1): v0→n0(d=0<1), v1→n1(d=1>0), v2→n1(d=2>1)
        // n_0=1, n_1=2 → ln(2)/ln(3)
        // edge (1,2): v0→n1(d=1<2), v1→n1(d=0<1), v2→n2(d=2>0→wait: d(1,2)=1,d(2,2)=0)
        // v0: d(1,0)=1 < d(2,0)=2 → n_1; v1: d(1,1)=0 < d(2,1)=1 → n_1; v2: d(1,2)=1 > d(2,2)=0 → n_2
        // n_1=2, n_2=1 → ln(2)/ln(3)
        // GG = 2·ln(2)/ln(3)
        let expected = 2.0 * 2.0_f64.ln() / 3.0_f64.ln();
        assert!((graovac_ghorbani_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn gg_k3() {
        // For edge (0,1): n_0=1(v0), n_1=1(v1), vertex 2 equidistant → not counted
        // ln(1·1)/ln(1+1) = ln(1)/ln(2) = 0. GG = 0
        assert!((graovac_ghorbani_index(&k3()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn gg_path4() {
        // edge (0,1): n_0=1, n_1=3 → ln(3)/ln(4)
        // edge (1,2): n_1=2, n_2=2 → ln(4)/ln(4) = 1
        // edge (2,3): n_2=3, n_3=1 → ln(3)/ln(4)
        // GG = 2·ln(3)/ln(4) + 1
        let expected = 2.0 * 3.0_f64.ln() / 4.0_f64.ln() + 1.0;
        assert!((graovac_ghorbani_index(&path4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn gg_cycle4() {
        // C4: for edge (0,1): v0 closer to 0, v3 closer to 0 (dist 1 vs 2),
        // v1 closer to 1, v2 closer to 1 (dist 1 vs 2)
        // Wait: v0=0 itself, dist(0,0)=0 < dist(1,0)=1 → n_0++
        // v3: dist(0,3)=1, dist(1,3)=2 → n_0++
        // v1: dist(0,1)=1, dist(1,1)=0 → n_1++
        // v2: dist(0,2)=2, dist(1,2)=1 → n_1++
        // n_0=2, n_1=2 → ln(4)/ln(4) = 1
        // By symmetry all 4 edges give 1 → GG = 4
        assert!((graovac_ghorbani_index(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn gg_k4() {
        // For edge (0,1) in K4: all distances are 1 between non-identical vertices
        // v0: dist(0,0)=0 < dist(1,0)=1 → n_0++
        // v1: dist(0,1)=1 > dist(1,1)=0 → n_1++
        // v2: dist(0,2)=1 = dist(1,2)=1 → tie
        // v3: dist(0,3)=1 = dist(1,3)=1 → tie
        // n_0=1, n_1=1 → ln(1)/ln(2) = 0
        // GG = 0
        assert!((graovac_ghorbani_index(&k4()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn gg_star5() {
        // edge (0,1): v0 closer to 0 (dist 0 vs 1), v1 closer to 1 (dist 1 vs 0)
        // v2: dist(0,2)=1, dist(1,2)=2 → n_0++
        // v3: dist(0,3)=1, dist(1,3)=2 → n_0++
        // v4: dist(0,4)=1, dist(1,4)=2 → n_0++
        // n_0=4, n_1=1 → ln(4)/ln(5)
        // By symmetry, 4 edges × ln(4)/ln(5)
        let expected = 4.0 * 4.0_f64.ln() / 5.0_f64.ln();
        assert!((graovac_ghorbani_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn gg_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(graovac_ghorbani_index(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn gg_cycle5() {
        // C5: for edge (0,1):
        // v0: d(0,0)=0 < d(1,0)=1 → n_0
        // v1: d(0,1)=1 > d(1,1)=0 → n_1
        // v4: d(0,4)=1, d(1,4)=2 → n_0
        // v2: d(0,2)=2, d(1,2)=1 → n_1
        // v3: d(0,3)=2, d(1,3)=2 → tie
        // n_0=2, n_1=2 → ln(4)/ln(4) = 1
        // By symmetry, 5 edges → GG = 5
        assert!((graovac_ghorbani_index(&cycle5()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn gg_paw() {
        // paw: edges (0,1),(0,2),(1,2),(2,3), degrees [2,2,3,1]
        // edge (0,1): v0→n0, v1→n1, v2: d(0,2)=1=d(1,2)=1→tie, v3: d(0,3)=2=d(1,3)=2→tie
        // n_0=1, n_1=1 → ln(1)/ln(2) = 0
        // edge (0,2): v0→n0, v2→n2, v1: d(0,1)=1=d(2,1)=1→tie, v3: d(0,3)=2,d(2,3)=1→n2
        // n_0=1, n_2=2 → ln(2)/ln(3)
        // edge (1,2): v1→n1, v2→n2, v0: d(1,0)=1=d(2,0)=1→tie, v3: d(1,3)=2,d(2,3)=1→n2
        // n_1=1, n_2=2 → ln(2)/ln(3)
        // edge (2,3): v2→n2, v3→n3, v0: d(2,0)=1,d(3,0)=2→n2, v1: d(2,1)=1,d(3,1)=2→n2
        // n_2=3, n_3=1 → ln(3)/ln(4)
        let expected = 2.0 * 2.0_f64.ln() / 3.0_f64.ln() + 3.0_f64.ln() / 4.0_f64.ln();
        assert!((graovac_ghorbani_index(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn esz_trees_compute_ok() {
        for g in &[single_edge(), path3(), path4(), star5()] {
            let _ = edge_szeged_index(g).unwrap();
        }
    }
}
