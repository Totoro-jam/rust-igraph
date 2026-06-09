//! Forgotten (F-index) coindex and related coindices (ALGO-TR-071).
//!
//! Coindices are computed over non-adjacent vertex pairs (complement edges):
//!
//! - **Forgotten coindex** `\bar{F}(G) = Σ_{u≠v, (u,v)∉E} [d(u)²+d(v)²]`
//! - **First hyper-Zagreb coindex** `\bar{HM₁}(G) = Σ_{u≠v, (u,v)∉E} [d(u)+d(v)]²`
//! - **Second hyper-Zagreb coindex** `\bar{HM₂}(G) = Σ_{u≠v, (u,v)∉E} [d(u)·d(v)]²`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the forgotten coindex (F-coindex).
///
/// `\bar{F}(G) = Σ_{u<v, (u,v)∉E} [d(u)²+d(v)²]`
///
/// Uses the identity: `\bar{F} = (n-1)·Σd² - F(G)` where F(G) is the
/// forgotten index.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, forgotten_coindex};
///
/// // K_3: no non-adjacent pairs → 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(forgotten_coindex(&g).unwrap(), 0);
///
/// // Path 0-1-2: non-adj (0,2), d=(1,1) → 1+1 = 2
/// let p = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(forgotten_coindex(&p).unwrap(), 2);
/// ```
pub fn forgotten_coindex(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0);
    }

    let mut sum_d2 = 0_u64;
    let mut f_index = 0_u64;

    for v in 0..n {
        let d = graph.degree(v as u32)? as u64;
        sum_d2 = sum_d2.saturating_add(d.saturating_mul(d));
    }

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as u64;
        let dv = graph.degree(v)? as u64;
        f_index =
            f_index.saturating_add(du.saturating_mul(du).saturating_add(dv.saturating_mul(dv)));
    }

    let n_minus_1 = (n as u64).saturating_sub(1);
    Ok(n_minus_1.saturating_mul(sum_d2).saturating_sub(f_index))
}

/// Compute the first hyper-Zagreb coindex.
///
/// `\bar{HM₁}(G) = Σ_{u<v, (u,v)∉E} [d(u)+d(v)]²`
///
/// Uses the identity: `\bar{HM₁} = 4m²·(n-1) + (n-1)·Σd² - HM₁(G)`
/// where HM₁ is the first hyper-Zagreb index (sum of (du+dv)² over edges),
/// m is edge count, and Σd² = M₁ (first Zagreb index).
///
/// Derivation: `Σ_all(du+dv)² = Σ_all(du²+2du·dv+dv²)`
///           `= (n-1)Σd² + 2·(Σd)² - 2·Σd²`
/// but simpler: just compute directly for correctness.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_hyper_zagreb_coindex};
///
/// // K_3: no non-adjacent pairs → 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(first_hyper_zagreb_coindex(&g).unwrap(), 0);
///
/// // Path 0-1-2: non-adj (0,2), (1+1)² = 4
/// let p = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(first_hyper_zagreb_coindex(&p).unwrap(), 4);
/// ```
pub fn first_hyper_zagreb_coindex(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)? as u64);
    }

    let mut total = 0_u64;
    for u in 0..n {
        for v in (u + 1)..n {
            let s = degrees[u].saturating_add(degrees[v]);
            total = total.saturating_add(s.saturating_mul(s));
        }
    }

    let mut edge_sum = 0_u64;
    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let s = degrees[u as usize].saturating_add(degrees[v as usize]);
        edge_sum = edge_sum.saturating_add(s.saturating_mul(s));
    }

    Ok(total.saturating_sub(edge_sum))
}

/// Compute the second hyper-Zagreb coindex.
///
/// `\bar{HM₂}(G) = Σ_{u<v, (u,v)∉E} [d(u)·d(v)]²`
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_hyper_zagreb_coindex};
///
/// // K_3: no non-adjacent pairs → 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(second_hyper_zagreb_coindex(&g).unwrap(), 0);
///
/// // Path 0-1-2: non-adj (0,2), (1·1)² = 1
/// let p = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(second_hyper_zagreb_coindex(&p).unwrap(), 1);
/// ```
pub fn second_hyper_zagreb_coindex(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)? as u64);
    }

    let mut total = 0_u64;
    for u in 0..n {
        for v in (u + 1)..n {
            let p = degrees[u].saturating_mul(degrees[v]);
            total = total.saturating_add(p.saturating_mul(p));
        }
    }

    let mut edge_sum = 0_u64;
    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let p = degrees[u as usize].saturating_mul(degrees[v as usize]);
        edge_sum = edge_sum.saturating_add(p.saturating_mul(p));
    }

    Ok(total.saturating_sub(edge_sum))
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

    // --- forgotten_coindex ---

    #[test]
    fn fco_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(forgotten_coindex(&g).unwrap(), 0);
    }

    #[test]
    fn fco_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(forgotten_coindex(&g).unwrap(), 0);
    }

    #[test]
    fn fco_single_edge() {
        // Only 2 vertices, both adjacent → 0
        assert_eq!(forgotten_coindex(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn fco_k3() {
        assert_eq!(forgotten_coindex(&k3()).unwrap(), 0);
    }

    #[test]
    fn fco_k4() {
        assert_eq!(forgotten_coindex(&k4()).unwrap(), 0);
    }

    #[test]
    fn fco_path3() {
        // Non-adj: (0,2), d=(1,1), 1+1=2
        assert_eq!(forgotten_coindex(&path3()).unwrap(), 2);
    }

    #[test]
    fn fco_path4() {
        // Non-adj: (0,2) d=(1,2):1+4=5, (0,3) d=(1,1):1+1=2, (1,3) d=(2,1):4+1=5
        assert_eq!(forgotten_coindex(&path4()).unwrap(), 12);
    }

    #[test]
    fn fco_cycle4() {
        // Non-adj: (0,2),(1,3), d=(2,2) each → 4+4=8 each → 16
        assert_eq!(forgotten_coindex(&cycle4()).unwrap(), 16);
    }

    #[test]
    fn fco_cycle5() {
        // Non-adj: 5 pairs (each vertex non-adj to 2 others, 5×2/2=5)
        // All d=2, each pair: 4+4=8 → 5×8=40
        assert_eq!(forgotten_coindex(&cycle5()).unwrap(), 40);
    }

    #[test]
    fn fco_star5() {
        // Non-adj: C(4,2)=6 leaf pairs, d=(1,1), 1+1=2 → 6×2=12
        assert_eq!(forgotten_coindex(&star5()).unwrap(), 12);
    }

    #[test]
    fn fco_paw() {
        // Non-adj: (0,3) d=(2,1):4+1=5, (1,3) d=(2,1):4+1=5 → 10
        assert_eq!(forgotten_coindex(&paw()).unwrap(), 10);
    }

    // --- first_hyper_zagreb_coindex ---

    #[test]
    fn hm1co_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(first_hyper_zagreb_coindex(&g).unwrap(), 0);
    }

    #[test]
    fn hm1co_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(first_hyper_zagreb_coindex(&g).unwrap(), 0);
    }

    #[test]
    fn hm1co_single_edge() {
        assert_eq!(first_hyper_zagreb_coindex(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn hm1co_k3() {
        assert_eq!(first_hyper_zagreb_coindex(&k3()).unwrap(), 0);
    }

    #[test]
    fn hm1co_k4() {
        assert_eq!(first_hyper_zagreb_coindex(&k4()).unwrap(), 0);
    }

    #[test]
    fn hm1co_path3() {
        // (0,2): (1+1)²=4
        assert_eq!(first_hyper_zagreb_coindex(&path3()).unwrap(), 4);
    }

    #[test]
    fn hm1co_path4() {
        // (0,2):(1+2)²=9, (0,3):(1+1)²=4, (1,3):(2+1)²=9 → 22
        assert_eq!(first_hyper_zagreb_coindex(&path4()).unwrap(), 22);
    }

    #[test]
    fn hm1co_cycle4() {
        // (0,2),(1,3): (2+2)²=16 each → 32
        assert_eq!(first_hyper_zagreb_coindex(&cycle4()).unwrap(), 32);
    }

    #[test]
    fn hm1co_cycle5() {
        // 5 non-adj pairs, d=2: (2+2)²=16 each → 80
        assert_eq!(first_hyper_zagreb_coindex(&cycle5()).unwrap(), 80);
    }

    #[test]
    fn hm1co_star5() {
        // 6 leaf pairs: (1+1)²=4 each → 24
        assert_eq!(first_hyper_zagreb_coindex(&star5()).unwrap(), 24);
    }

    #[test]
    fn hm1co_paw() {
        // (0,3):(2+1)²=9, (1,3):(2+1)²=9 → 18
        assert_eq!(first_hyper_zagreb_coindex(&paw()).unwrap(), 18);
    }

    // --- second_hyper_zagreb_coindex ---

    #[test]
    fn hm2co_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(second_hyper_zagreb_coindex(&g).unwrap(), 0);
    }

    #[test]
    fn hm2co_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(second_hyper_zagreb_coindex(&g).unwrap(), 0);
    }

    #[test]
    fn hm2co_single_edge() {
        assert_eq!(second_hyper_zagreb_coindex(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn hm2co_k3() {
        assert_eq!(second_hyper_zagreb_coindex(&k3()).unwrap(), 0);
    }

    #[test]
    fn hm2co_k4() {
        assert_eq!(second_hyper_zagreb_coindex(&k4()).unwrap(), 0);
    }

    #[test]
    fn hm2co_path3() {
        // (0,2): (1·1)²=1
        assert_eq!(second_hyper_zagreb_coindex(&path3()).unwrap(), 1);
    }

    #[test]
    fn hm2co_path4() {
        // (0,2):(1·2)²=4, (0,3):(1·1)²=1, (1,3):(2·1)²=4 → 9
        assert_eq!(second_hyper_zagreb_coindex(&path4()).unwrap(), 9);
    }

    #[test]
    fn hm2co_cycle4() {
        // (0,2),(1,3): (2·2)²=16 each → 32
        assert_eq!(second_hyper_zagreb_coindex(&cycle4()).unwrap(), 32);
    }

    #[test]
    fn hm2co_cycle5() {
        // 5 pairs, (2·2)²=16 each → 80
        assert_eq!(second_hyper_zagreb_coindex(&cycle5()).unwrap(), 80);
    }

    #[test]
    fn hm2co_star5() {
        // 6 leaf pairs: (1·1)²=1 each → 6
        assert_eq!(second_hyper_zagreb_coindex(&star5()).unwrap(), 6);
    }

    #[test]
    fn hm2co_paw() {
        // (0,3):(2·1)²=4, (1,3):(2·1)²=4 → 8
        assert_eq!(second_hyper_zagreb_coindex(&paw()).unwrap(), 8);
    }

    // --- cross-consistency ---

    #[test]
    fn complete_graphs_all_zero() {
        for g in &[k3(), k4()] {
            assert_eq!(forgotten_coindex(g).unwrap(), 0);
            assert_eq!(first_hyper_zagreb_coindex(g).unwrap(), 0);
            assert_eq!(second_hyper_zagreb_coindex(g).unwrap(), 0);
        }
    }

    #[test]
    fn all_positive_for_incomplete() {
        for g in &[path3(), path4(), cycle4(), star5(), paw()] {
            assert!(forgotten_coindex(g).unwrap() > 0);
            assert!(first_hyper_zagreb_coindex(g).unwrap() > 0);
            assert!(second_hyper_zagreb_coindex(g).unwrap() > 0);
        }
    }

    #[test]
    fn hm1co_ge_fco() {
        // (du+dv)² = du²+2du·dv+dv² ≥ du²+dv² when du,dv≥0 (since 2du·dv≥0)
        for g in &[path3(), path4(), cycle4(), cycle5(), star5(), paw()] {
            let fco = forgotten_coindex(g).unwrap();
            let hm1co = first_hyper_zagreb_coindex(g).unwrap();
            assert!(hm1co >= fco);
        }
    }

    #[test]
    fn fco_via_direct_sum() {
        // Verify forgotten_coindex by direct computation
        for g in &[path3(), path4(), cycle4(), star5(), paw()] {
            let n = g.vcount() as usize;
            let mut degrees = vec![0_u64; n];
            for v in 0..n {
                degrees[v] = g.degree(v as u32).unwrap() as u64;
            }

            let adj: std::collections::HashSet<(usize, usize)> = g
                .edges()
                .flat_map(|(u, v)| {
                    let a = u as usize;
                    let b = v as usize;
                    vec![(a, b), (b, a)]
                })
                .collect();

            let mut direct = 0_u64;
            for u in 0..n {
                for v in (u + 1)..n {
                    if !adj.contains(&(u, v)) {
                        direct += degrees[u] * degrees[u] + degrees[v] * degrees[v];
                    }
                }
            }

            assert_eq!(forgotten_coindex(g).unwrap(), direct);
        }
    }
}
