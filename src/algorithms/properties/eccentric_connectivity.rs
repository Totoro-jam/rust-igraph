//! Eccentric connectivity index and total eccentricity (ALGO-TR-045).
//!
//! - **Eccentric connectivity index** `ξ^c(G) = Σ_v deg(v) · ecc(v)`
//!   where `ecc(v) = max_w d(v, w)` is the eccentricity of vertex `v`.
//!   Introduced by Sharma, Goswami & Madan (1997); widely used in
//!   QSAR studies.
//! - **Total eccentricity** `ζ(G) = Σ_v ecc(v)`.
//! - **Connective eccentricity index** `ξ^ce(G) = Σ_v deg(v) / ecc(v)`.
//!
//! Disconnected vertices have `ecc = 0` within their component if
//! isolated (single vertex); otherwise eccentricity is computed
//! within the reachable set. Isolated vertices are skipped for
//! the connective eccentricity index (division by zero).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};
use std::collections::VecDeque;

/// Compute the eccentric connectivity index.
///
/// `ξ^c(G) = Σ_v deg(v) · ecc(v)`
///
/// Isolated vertices contribute 0. For disconnected graphs,
/// eccentricity is computed within the reachable set of each vertex.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, eccentric_connectivity_index};
///
/// // Path 0-1-2: deg=[1,2,1], ecc=[2,1,2]
/// // ξ^c = 1·2 + 2·1 + 1·2 = 6
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(eccentric_connectivity_index(&g).unwrap(), 6);
/// ```
pub fn eccentric_connectivity_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let ecc = compute_eccentricities(graph, n);
    let mut xi: u64 = 0;

    for v in 0..n as u32 {
        let deg = graph.degree(v)? as u64;
        xi = xi.saturating_add(deg.saturating_mul(u64::from(ecc[v as usize])));
    }

    Ok(xi)
}

/// Compute the total eccentricity.
///
/// `ζ(G) = Σ_v ecc(v)`
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, total_eccentricity};
///
/// // Path 0-1-2: ecc=[2,1,2] → ζ = 5
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(total_eccentricity(&g).unwrap(), 5);
/// ```
pub fn total_eccentricity(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let ecc = compute_eccentricities(graph, n);
    let mut total: u64 = 0;
    for &e in &ecc {
        total = total.saturating_add(u64::from(e));
    }

    Ok(total)
}

/// Compute the connective eccentricity index.
///
/// `ξ^ce(G) = Σ_v deg(v) / ecc(v)`
///
/// Vertices with eccentricity 0 (isolated) are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, connective_eccentricity_index};
///
/// // Path 0-1-2: deg=[1,2,1], ecc=[2,1,2]
/// // ξ^ce = 1/2 + 2/1 + 1/2 = 3.0
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((connective_eccentricity_index(&g).unwrap() - 3.0).abs() < 1e-10);
/// ```
pub fn connective_eccentricity_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let ecc = compute_eccentricities(graph, n);
    let mut xi = 0.0_f64;

    for v in 0..n as u32 {
        let e = ecc[v as usize];
        if e == 0 {
            continue;
        }
        let deg = graph.degree(v)? as f64;
        xi += deg / f64::from(e);
    }

    Ok(xi)
}

fn compute_eccentricities(graph: &Graph, n: usize) -> Vec<u32> {
    let adj = build_adj_list(graph, n);
    let mut ecc = vec![0_u32; n];

    for src in 0..n {
        let dist = bfs_from(&adj, n, src);
        let mut max_d = 0_u32;
        for &d in &dist {
            if d != u32::MAX && d > max_d {
                max_d = d;
            }
        }
        ecc[src] = max_d;
    }

    ecc
}

fn build_adj_list(graph: &Graph, n: usize) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); n];
    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        adj[ui].push(vi);
        if !graph.is_directed() && ui != vi {
            adj[vi].push(ui);
        }
    }
    adj
}

fn bfs_from(adj: &[Vec<usize>], n: usize, src: usize) -> Vec<u32> {
    let mut dist = vec![u32::MAX; n];
    dist[src] = 0;
    let mut queue = VecDeque::new();
    queue.push_back(src);
    while let Some(v) = queue.pop_front() {
        let d = dist[v];
        for &w in &adj[v] {
            if dist[w] == u32::MAX {
                dist[w] = d.saturating_add(1);
                queue.push_back(w);
            }
        }
    }
    dist
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

    fn path5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)], false, Some(5)).unwrap()
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

    // --- eccentric_connectivity_index ---

    #[test]
    fn eci_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(eccentric_connectivity_index(&g).unwrap(), 0);
    }

    #[test]
    fn eci_single_vertex() {
        let g = Graph::with_vertices(1);
        assert_eq!(eccentric_connectivity_index(&g).unwrap(), 0);
    }

    #[test]
    fn eci_no_edges() {
        let g = Graph::with_vertices(3);
        assert_eq!(eccentric_connectivity_index(&g).unwrap(), 0);
    }

    #[test]
    fn eci_single_edge() {
        // deg=[1,1], ecc=[1,1] → ξ = 1·1 + 1·1 = 2
        assert_eq!(eccentric_connectivity_index(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn eci_path3() {
        // deg=[1,2,1], ecc=[2,1,2] → ξ = 2 + 2 + 2 = 6
        assert_eq!(eccentric_connectivity_index(&path3()).unwrap(), 6);
    }

    #[test]
    fn eci_path4() {
        // deg=[1,2,2,1], ecc=[3,2,2,3]
        // ξ = 1·3 + 2·2 + 2·2 + 1·3 = 3+4+4+3 = 14
        assert_eq!(eccentric_connectivity_index(&path4()).unwrap(), 14);
    }

    #[test]
    fn eci_path5() {
        // deg=[1,2,2,2,1], ecc=[4,3,2,3,4]
        // ξ = 4 + 6 + 4 + 6 + 4 = 24
        assert_eq!(eccentric_connectivity_index(&path5()).unwrap(), 24);
    }

    #[test]
    fn eci_k3() {
        // deg=[2,2,2], ecc=[1,1,1] → ξ = 6
        assert_eq!(eccentric_connectivity_index(&k3()).unwrap(), 6);
    }

    #[test]
    fn eci_k4() {
        // deg=[3,3,3,3], ecc=[1,1,1,1] → ξ = 12
        assert_eq!(eccentric_connectivity_index(&k4()).unwrap(), 12);
    }

    #[test]
    fn eci_cycle4() {
        // deg=[2,2,2,2], ecc=[2,2,2,2] → ξ = 16
        assert_eq!(eccentric_connectivity_index(&cycle4()).unwrap(), 16);
    }

    #[test]
    fn eci_cycle5() {
        // deg=[2,2,2,2,2], ecc=[2,2,2,2,2] → ξ = 20
        assert_eq!(eccentric_connectivity_index(&cycle5()).unwrap(), 20);
    }

    #[test]
    fn eci_star5() {
        // deg=[4,1,1,1,1], ecc=[1,2,2,2,2]
        // ξ = 4·1 + 1·2 + 1·2 + 1·2 + 1·2 = 4+8 = 12
        assert_eq!(eccentric_connectivity_index(&star5()).unwrap(), 12);
    }

    #[test]
    fn eci_with_isolated() {
        // 0-1 plus isolated 2: deg=[1,1,0], ecc=[1,1,0]
        // ξ = 1 + 1 + 0 = 2
        let g = Graph::from_edges(&[(0, 1)], false, Some(3)).unwrap();
        assert_eq!(eccentric_connectivity_index(&g).unwrap(), 2);
    }

    #[test]
    fn eci_two_components() {
        // 0-1 and 2-3
        // deg=[1,1,1,1], ecc=[1,1,1,1]
        // ξ = 4
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert_eq!(eccentric_connectivity_index(&g).unwrap(), 4);
    }

    #[test]
    fn eci_complete_formula() {
        // K_n: all deg=n-1, all ecc=1 → ξ = n(n-1)
        for n in 2_u32..=6 {
            let edges: Vec<(u32, u32)> = (0..n)
                .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
                .collect();
            let g = Graph::from_edges(&edges, false, Some(n)).unwrap();
            assert_eq!(
                eccentric_connectivity_index(&g).unwrap(),
                u64::from(n) * u64::from(n - 1)
            );
        }
    }

    #[test]
    fn eci_diamond() {
        // K4 minus (2,3): edges 0-1,0-2,0-3,1-2,1-3
        // deg=[3,3,2,2], ecc=[1,1,2,2]
        // ξ = 3+3+4+4 = 14
        let g =
            Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3)], false, Some(4)).unwrap();
        assert_eq!(eccentric_connectivity_index(&g).unwrap(), 14);
    }

    // --- total_eccentricity ---

    #[test]
    fn te_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(total_eccentricity(&g).unwrap(), 0);
    }

    #[test]
    fn te_single_vertex() {
        let g = Graph::with_vertices(1);
        assert_eq!(total_eccentricity(&g).unwrap(), 0);
    }

    #[test]
    fn te_single_edge() {
        assert_eq!(total_eccentricity(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn te_path3() {
        // ecc=[2,1,2] → 5
        assert_eq!(total_eccentricity(&path3()).unwrap(), 5);
    }

    #[test]
    fn te_path4() {
        // ecc=[3,2,2,3] → 10
        assert_eq!(total_eccentricity(&path4()).unwrap(), 10);
    }

    #[test]
    fn te_k3() {
        // ecc=[1,1,1] → 3
        assert_eq!(total_eccentricity(&k3()).unwrap(), 3);
    }

    #[test]
    fn te_k4() {
        // ecc=[1,1,1,1] → 4
        assert_eq!(total_eccentricity(&k4()).unwrap(), 4);
    }

    #[test]
    fn te_cycle4() {
        // ecc=[2,2,2,2] → 8
        assert_eq!(total_eccentricity(&cycle4()).unwrap(), 8);
    }

    #[test]
    fn te_cycle5() {
        // ecc=[2,2,2,2,2] → 10
        assert_eq!(total_eccentricity(&cycle5()).unwrap(), 10);
    }

    #[test]
    fn te_star5() {
        // ecc=[1,2,2,2,2] → 9
        assert_eq!(total_eccentricity(&star5()).unwrap(), 9);
    }

    #[test]
    fn te_complete_formula() {
        // K_n: all ecc=1 → ζ = n
        for n in 2_u32..=6 {
            let edges: Vec<(u32, u32)> = (0..n)
                .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
                .collect();
            let g = Graph::from_edges(&edges, false, Some(n)).unwrap();
            assert_eq!(total_eccentricity(&g).unwrap(), u64::from(n));
        }
    }

    // --- connective_eccentricity_index ---

    #[test]
    fn cei_empty() {
        let g = Graph::with_vertices(0);
        assert!((connective_eccentricity_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn cei_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((connective_eccentricity_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn cei_single_edge() {
        // deg=[1,1], ecc=[1,1] → ξ^ce = 1+1 = 2
        assert!((connective_eccentricity_index(&single_edge()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn cei_path3() {
        // deg=[1,2,1], ecc=[2,1,2] → 0.5+2+0.5 = 3
        assert!((connective_eccentricity_index(&path3()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn cei_path4() {
        // deg=[1,2,2,1], ecc=[3,2,2,3]
        // 1/3 + 1 + 1 + 1/3 = 8/3
        let c = connective_eccentricity_index(&path4()).unwrap();
        assert!((c - 8.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn cei_k3() {
        // deg=[2,2,2], ecc=[1,1,1] → 2+2+2 = 6
        assert!((connective_eccentricity_index(&k3()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn cei_k4() {
        // deg=[3,3,3,3], ecc=[1,1,1,1] → 12
        assert!((connective_eccentricity_index(&k4()).unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn cei_cycle4() {
        // deg=[2,2,2,2], ecc=[2,2,2,2] → 1+1+1+1 = 4
        assert!((connective_eccentricity_index(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn cei_star5() {
        // deg=[4,1,1,1,1], ecc=[1,2,2,2,2]
        // 4/1 + 1/2 + 1/2 + 1/2 + 1/2 = 6
        assert!((connective_eccentricity_index(&star5()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn cei_with_isolated() {
        // 0-1 plus isolated 2 (ecc=0, skipped)
        // deg=[1,1,0], ecc=[1,1,0] → 1+1 = 2
        let g = Graph::from_edges(&[(0, 1)], false, Some(3)).unwrap();
        assert!((connective_eccentricity_index(&g).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn cei_complete_formula() {
        // K_n: all deg=n-1, all ecc=1 → ξ^ce = n(n-1)
        for n in 2_u32..=6 {
            let edges: Vec<(u32, u32)> = (0..n)
                .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
                .collect();
            let g = Graph::from_edges(&edges, false, Some(n)).unwrap();
            assert!(
                (connective_eccentricity_index(&g).unwrap() - f64::from(n) * f64::from(n - 1))
                    .abs()
                    < 1e-10
            );
        }
    }

    // --- cross-consistency ---

    #[test]
    fn eci_geq_total_ecc() {
        // ξ^c(G) >= ζ(G) since deg(v) >= 1 for non-isolated v
        // (for connected graphs with n >= 2)
        for g in &[
            single_edge(),
            path3(),
            path4(),
            k3(),
            k4(),
            cycle4(),
            star5(),
        ] {
            let xi = eccentric_connectivity_index(g).unwrap();
            let te = total_eccentricity(g).unwrap();
            assert!(xi >= te, "ξ^c={xi} < ζ={te}");
        }
    }

    #[test]
    fn eci_equals_2m_for_kn() {
        // For K_n: ξ^c = Σ (n-1)·1 = n(n-1) = 2m
        for n in 2_u32..=6 {
            let edges: Vec<(u32, u32)> = (0..n)
                .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
                .collect();
            let g = Graph::from_edges(&edges, false, Some(n)).unwrap();
            assert_eq!(
                eccentric_connectivity_index(&g).unwrap(),
                2 * g.ecount() as u64
            );
        }
    }

    #[test]
    fn cei_leq_eci_for_ecc_geq_1() {
        // ξ^ce(G) <= ξ^c(G) when all ecc >= 1 (deg/ecc <= deg·ecc)
        for g in &[
            single_edge(),
            path3(),
            path4(),
            k3(),
            k4(),
            cycle4(),
            star5(),
        ] {
            let xi = eccentric_connectivity_index(g).unwrap() as f64;
            let ce = connective_eccentricity_index(g).unwrap();
            assert!(ce <= xi + 1e-10, "ξ^ce={ce} > ξ^c={xi}");
        }
    }

    #[test]
    fn eci_path_formula() {
        // For P_n (n >= 2): ecc(i) = max(i, n-1-i) for i=0..n-1
        // Verify against direct formula
        for n in 2_u32..=8 {
            let edges: Vec<(u32, u32)> = (0..n - 1).map(|i| (i, i + 1)).collect();
            let g = Graph::from_edges(&edges, false, Some(n)).unwrap();

            let mut expected: u64 = 0;
            for i in 0..n {
                let ecc = std::cmp::max(i, n - 1 - i);
                let deg = if i == 0 || i == n - 1 { 1_u64 } else { 2_u64 };
                expected += deg * u64::from(ecc);
            }
            assert_eq!(eccentric_connectivity_index(&g).unwrap(), expected);
        }
    }

    #[test]
    fn te_path_formula() {
        // For P_n: total ecc = Σ max(i, n-1-i)
        for n in 2_u32..=8 {
            let edges: Vec<(u32, u32)> = (0..n - 1).map(|i| (i, i + 1)).collect();
            let g = Graph::from_edges(&edges, false, Some(n)).unwrap();

            let mut expected: u64 = 0;
            for i in 0..n {
                expected += u64::from(std::cmp::max(i, n - 1 - i));
            }
            assert_eq!(total_eccentricity(&g).unwrap(), expected);
        }
    }
}
