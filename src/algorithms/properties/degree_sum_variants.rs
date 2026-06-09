//! Degree-sum bond-additive variants (ALGO-TR-072).
//!
//! Bond-additive indices using degree-sum functions over edges:
//!
//! - **Arithmetic-geometric index** `AG(G) = Σ_{(u,v)∈E} (d(u)+d(v))/(2√(d(u)·d(v)))`
//!   Ratio of arithmetic to geometric mean of endpoint degrees.
//! - **Sigma coindex** `\bar{σ}(G) = Σ_{u<v,(u,v)∉E} (d(u)-d(v))²`
//!   Irregularity measure over complement edges.
//! - **Albertson coindex** `\bar{Alb}(G) = Σ_{u<v,(u,v)∉E} |d(u)-d(v)|`
//!   Albertson-type measure over complement edges.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the arithmetic-geometric index.
///
/// `AG(G) = Σ_{(u,v)∈E} (d(u)+d(v)) / (2√(d(u)·d(v)))`
///
/// Self-loops and edges with a degree-0 endpoint are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, arithmetic_geometric_index};
///
/// // K_3: 3 edges, d=(2,2), each: (4)/(2·2) = 1 → total = 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((arithmetic_geometric_index(&g).unwrap() - 3.0).abs() < 1e-10);
/// ```
pub fn arithmetic_geometric_index(graph: &Graph) -> IgraphResult<f64> {
    let mut ag = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let product = du * dv;
        if product > 0.0 {
            ag += (du + dv) / (2.0 * product.sqrt());
        }
    }

    Ok(ag)
}

/// Compute the sigma coindex.
///
/// `\bar{σ}(G) = Σ_{u<v, (u,v)∉E} (d(u)-d(v))²`
///
/// Uses: `\bar{σ} = (n-1)·Σd² - 2m·Σd² / n ... ` — simpler to compute
/// directly as total minus edge contribution.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, sigma_coindex};
///
/// // K_3: no non-adjacent pairs → 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(sigma_coindex(&g).unwrap(), 0);
///
/// // Path 0-1-2: non-adj (0,2), (1-1)²=0
/// let p = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(sigma_coindex(&p).unwrap(), 0);
/// ```
pub fn sigma_coindex(graph: &Graph) -> IgraphResult<u64> {
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
            let diff = degrees[u].abs_diff(degrees[v]);
            total = total.saturating_add(diff.saturating_mul(diff));
        }
    }

    let mut edge_sum = 0_u64;
    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = degrees[u as usize];
        let dv = degrees[v as usize];
        let diff = du.abs_diff(dv);
        edge_sum = edge_sum.saturating_add(diff.saturating_mul(diff));
    }

    Ok(total.saturating_sub(edge_sum))
}

/// Compute the Albertson coindex.
///
/// `\bar{Alb}(G) = Σ_{u<v, (u,v)∉E} |d(u)-d(v)|`
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, albertson_coindex};
///
/// // K_3: no non-adjacent pairs → 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(albertson_coindex(&g).unwrap(), 0);
///
/// // Star S_5: 6 leaf pairs, |1-1|=0 → 0
/// let s = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert_eq!(albertson_coindex(&s).unwrap(), 0);
/// ```
pub fn albertson_coindex(graph: &Graph) -> IgraphResult<u64> {
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
            let diff = degrees[u].abs_diff(degrees[v]);
            total = total.saturating_add(diff);
        }
    }

    let mut edge_sum = 0_u64;
    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = degrees[u as usize];
        let dv = degrees[v as usize];
        let diff = du.abs_diff(dv);
        edge_sum = edge_sum.saturating_add(diff);
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

    // --- arithmetic_geometric_index ---

    #[test]
    fn ag_empty() {
        let g = Graph::with_vertices(0);
        assert!(arithmetic_geometric_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ag_isolated() {
        let g = Graph::with_vertices(5);
        assert!(arithmetic_geometric_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ag_single_edge() {
        // (1+1)/(2√1) = 2/2 = 1
        assert!((arithmetic_geometric_index(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ag_path3() {
        // 2 edges, d=(1,2): (3)/(2√2). Total = 3/√2 ≈ 2.121
        let expected = 3.0 / 2.0_f64.sqrt();
        assert!((arithmetic_geometric_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ag_k3() {
        // 3 edges, (2+2)/(2√4)=4/4=1 each → 3
        assert!((arithmetic_geometric_index(&k3()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn ag_k4() {
        // 6 edges, (3+3)/(2√9)=6/6=1 each → 6
        assert!((arithmetic_geometric_index(&k4()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn ag_cycle4() {
        // 4 edges, d=(2,2), (4)/(2·2)=1 each → 4
        assert!((arithmetic_geometric_index(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn ag_cycle5() {
        // 5 edges, each 1 → 5
        assert!((arithmetic_geometric_index(&cycle5()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn ag_star5() {
        // 4 edges, d=(4,1): (5)/(2√4) = 5/4 each → 5
        let expected = 4.0 * 5.0 / 4.0;
        assert!((arithmetic_geometric_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ag_paw() {
        // (0,1) d=(2,2): 4/4=1
        // (0,2) d=(2,3): 5/(2√6)
        // (1,2) d=(2,3): 5/(2√6)
        // (2,3) d=(3,1): 4/(2√3)
        let expected = 1.0 + 5.0 / 6.0_f64.sqrt() + 4.0 / (2.0 * 3.0_f64.sqrt());
        assert!((arithmetic_geometric_index(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ag_regular_equals_edge_count() {
        // r-regular: AG = m × (2r)/(2r) = m
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            assert!((arithmetic_geometric_index(g).unwrap() - m).abs() < 1e-8);
        }
    }

    #[test]
    fn ag_ge_edge_count_for_simple() {
        // AM-GM: (du+dv)/(2√(du·dv)) ≥ 1, so AG ≥ m
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let m = g.ecount() as f64;
            assert!(arithmetic_geometric_index(g).unwrap() >= m - 1e-10);
        }
    }

    // --- sigma_coindex ---

    #[test]
    fn sco_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(sigma_coindex(&g).unwrap(), 0);
    }

    #[test]
    fn sco_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(sigma_coindex(&g).unwrap(), 0);
    }

    #[test]
    fn sco_single_edge() {
        assert_eq!(sigma_coindex(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn sco_k3() {
        assert_eq!(sigma_coindex(&k3()).unwrap(), 0);
    }

    #[test]
    fn sco_k4() {
        assert_eq!(sigma_coindex(&k4()).unwrap(), 0);
    }

    #[test]
    fn sco_path3() {
        // Non-adj (0,2): (1-1)²=0
        assert_eq!(sigma_coindex(&path3()).unwrap(), 0);
    }

    #[test]
    fn sco_path4() {
        // Non-adj: (0,2):(1-2)²=1, (0,3):(1-1)²=0, (1,3):(2-1)²=1 → 2
        assert_eq!(sigma_coindex(&path4()).unwrap(), 2);
    }

    #[test]
    fn sco_cycle4() {
        // Non-adj: (0,2),(1,3), d=2 each → 0
        assert_eq!(sigma_coindex(&cycle4()).unwrap(), 0);
    }

    #[test]
    fn sco_cycle5() {
        // Regular → 0
        assert_eq!(sigma_coindex(&cycle5()).unwrap(), 0);
    }

    #[test]
    fn sco_star5() {
        // Non-adj: 6 leaf pairs (1,1) → all 0
        assert_eq!(sigma_coindex(&star5()).unwrap(), 0);
    }

    #[test]
    fn sco_paw() {
        // Non-adj: (0,3):(2-1)²=1, (1,3):(2-1)²=1 → 2
        assert_eq!(sigma_coindex(&paw()).unwrap(), 2);
    }

    #[test]
    fn sco_regular_zero() {
        // Regular graphs: all differences are 0
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            assert_eq!(sigma_coindex(g).unwrap(), 0);
        }
    }

    // --- albertson_coindex ---

    #[test]
    fn aco_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(albertson_coindex(&g).unwrap(), 0);
    }

    #[test]
    fn aco_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(albertson_coindex(&g).unwrap(), 0);
    }

    #[test]
    fn aco_single_edge() {
        assert_eq!(albertson_coindex(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn aco_k3() {
        assert_eq!(albertson_coindex(&k3()).unwrap(), 0);
    }

    #[test]
    fn aco_k4() {
        assert_eq!(albertson_coindex(&k4()).unwrap(), 0);
    }

    #[test]
    fn aco_path3() {
        // (0,2): |1-1|=0
        assert_eq!(albertson_coindex(&path3()).unwrap(), 0);
    }

    #[test]
    fn aco_path4() {
        // (0,2):|1-2|=1, (0,3):|1-1|=0, (1,3):|2-1|=1 → 2
        assert_eq!(albertson_coindex(&path4()).unwrap(), 2);
    }

    #[test]
    fn aco_cycle4() {
        assert_eq!(albertson_coindex(&cycle4()).unwrap(), 0);
    }

    #[test]
    fn aco_star5() {
        // 6 leaf pairs, |1-1|=0
        assert_eq!(albertson_coindex(&star5()).unwrap(), 0);
    }

    #[test]
    fn aco_paw() {
        // (0,3):|2-1|=1, (1,3):|2-1|=1 → 2
        assert_eq!(albertson_coindex(&paw()).unwrap(), 2);
    }

    #[test]
    fn aco_regular_zero() {
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            assert_eq!(albertson_coindex(g).unwrap(), 0);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn sigma_ge_albertson_for_nontrivial() {
        // σ ≥ a (since (d-d')² ≥ |d-d'| when |d-d'|≥1, and = when |d-d'|∈{0,1})
        for g in &[path4(), paw()] {
            let s = sigma_coindex(g).unwrap();
            let a = albertson_coindex(g).unwrap();
            assert!(s >= a);
        }
    }
}
