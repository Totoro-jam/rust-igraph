//! Vertex-edge degree indices (ALGO-TR-068).
//!
//! These indices use the **ve-degree** of a vertex v with respect to
//! the edges incident to it: `d_{ve}(v) = Σ_{e∋v} d_{ev}(e)` where
//! `d_{ev}(e)` for edge e=(u,w) equals `d(u)+d(w)-2`.
//!
//! Equivalently, `d_{ve}(v) = Σ_{u∈N(v)} [d(u)+d(v)-2]`
//!             `= d(v)·[d(v)-2] + Σ_{u∈N(v)} d(u)`
//!             `= d(v)² - 2·d(v) + S(v)`
//! where `S(v) = Σ_{u∈N(v)} d(u)` is the neighbor degree sum.
//!
//! - **First ve-degree Zagreb alpha** `M₁^{αve}(G) = Σ_v d_{ve}(v)²`
//! - **First ve-degree Zagreb beta**  `M₁^{βve}(G) = Σ_{(u,v)∈E} [d_{ve}(u)+d_{ve}(v)]`
//! - **Second ve-degree Zagreb**      `M₂^{ve}(G) = Σ_{(u,v)∈E} d_{ve}(u)·d_{ve}(v)`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

fn ve_degrees(graph: &Graph) -> IgraphResult<Vec<u64>> {
    let n = graph.vcount() as usize;
    let mut dve = vec![0_u64; n];

    for v in 0..n {
        let dv = graph.degree(v as u32)? as u64;
        let nbs = graph.neighbors(v as u32)?;
        let mut s_v = 0_u64;
        for nb in nbs {
            let du = graph.degree(nb)? as u64;
            s_v = s_v.saturating_add(du);
        }
        dve[v] = dv
            .saturating_mul(dv)
            .saturating_add(s_v)
            .saturating_sub(2 * dv);
    }

    Ok(dve)
}

/// Compute the first ve-degree Zagreb alpha index.
///
/// `M₁^{αve}(G) = Σ_v d_{ve}(v)²`
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_ve_degree_zagreb_alpha};
///
/// // K_3: d=[2,2,2], S(v)=4, d_ve(v)=4+4-4=4 for all
/// // M₁^αve = 3×16 = 48
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(first_ve_degree_zagreb_alpha(&g).unwrap(), 48);
/// ```
pub fn first_ve_degree_zagreb_alpha(graph: &Graph) -> IgraphResult<u64> {
    let dve = ve_degrees(graph)?;
    let mut m1a = 0_u64;

    for &d in &dve {
        m1a = m1a.saturating_add(d.saturating_mul(d));
    }

    Ok(m1a)
}

/// Compute the first ve-degree Zagreb beta index.
///
/// `M₁^{βve}(G) = Σ_{(u,v)∈E} [d_{ve}(u) + d_{ve}(v)]`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_ve_degree_zagreb_beta};
///
/// // K_3: d_ve=[4,4,4], 3 edges × (4+4) = 24
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(first_ve_degree_zagreb_beta(&g).unwrap(), 24);
/// ```
pub fn first_ve_degree_zagreb_beta(graph: &Graph) -> IgraphResult<u64> {
    let dve = ve_degrees(graph)?;
    let mut m1b = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = dve[u as usize];
        let dv = dve[v as usize];
        m1b = m1b.saturating_add(du.saturating_add(dv));
    }

    Ok(m1b)
}

/// Compute the second ve-degree Zagreb index.
///
/// `M₂^{ve}(G) = Σ_{(u,v)∈E} d_{ve}(u) · d_{ve}(v)`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_ve_degree_zagreb};
///
/// // K_3: d_ve=[4,4,4], 3 edges × 16 = 48
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(second_ve_degree_zagreb(&g).unwrap(), 48);
/// ```
pub fn second_ve_degree_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let dve = ve_degrees(graph)?;
    let mut m2 = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = dve[u as usize];
        let dv = dve[v as usize];
        m2 = m2.saturating_add(du.saturating_mul(dv));
    }

    Ok(m2)
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

    fn dve(g: &Graph) -> Vec<u64> {
        ve_degrees(g).unwrap()
    }

    // --- ve_degrees ---

    #[test]
    fn dve_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(dve(&g), Vec::<u64>::new());
    }

    #[test]
    fn dve_isolated() {
        let g = Graph::with_vertices(3);
        assert_eq!(dve(&g), vec![0, 0, 0]);
    }

    #[test]
    fn dve_single_edge() {
        // d=[1,1], S(0)=1, S(1)=1
        // d_ve(0)=1+1-2=0, d_ve(1)=0
        assert_eq!(dve(&single_edge()), vec![0, 0]);
    }

    #[test]
    fn dve_path3() {
        // d=[1,2,1], S(0)=2, S(1)=1+1=2, S(2)=2
        // d_ve(0)=1+2-2=1, d_ve(1)=4+2-4=2, d_ve(2)=1+2-2=1
        assert_eq!(dve(&path3()), vec![1, 2, 1]);
    }

    #[test]
    fn dve_path4() {
        // d=[1,2,2,1], S(0)=2, S(1)=1+2=3, S(2)=2+1=3, S(3)=2
        // d_ve(0)=1+2-2=1, d_ve(1)=4+3-4=3, d_ve(2)=4+3-4=3, d_ve(3)=1+2-2=1
        assert_eq!(dve(&path4()), vec![1, 3, 3, 1]);
    }

    #[test]
    fn dve_k3() {
        // d=[2,2,2], S(v)=4, d_ve(v)=4+4-4=4 for all
        assert_eq!(dve(&k3()), vec![4, 4, 4]);
    }

    #[test]
    fn dve_k4() {
        // d=[3,3,3,3], S(v)=9, d_ve(v)=9+9-6=12 for all
        assert_eq!(dve(&k4()), vec![12, 12, 12, 12]);
    }

    #[test]
    fn dve_cycle4() {
        // d=[2,2,2,2], S(v)=4, d_ve(v)=4+4-4=4 for all
        assert_eq!(dve(&cycle4()), vec![4, 4, 4, 4]);
    }

    #[test]
    fn dve_cycle5() {
        // d=[2,2,2,2,2], S(v)=4, d_ve(v)=4+4-4=4 for all
        assert_eq!(dve(&cycle5()), vec![4, 4, 4, 4, 4]);
    }

    #[test]
    fn dve_star5() {
        // d=[4,1,1,1,1], S(0)=4, S(leaf)=4
        // d_ve(0)=16+4-8=12, d_ve(leaf)=1+4-2=3
        assert_eq!(dve(&star5()), vec![12, 3, 3, 3, 3]);
    }

    #[test]
    fn dve_paw() {
        // d=[2,2,3,1], S(0)=2+3=5, S(1)=2+3=5, S(2)=2+2+1=5, S(3)=3
        // d_ve(0)=4+5-4=5, d_ve(1)=4+5-4=5, d_ve(2)=9+5-6=8, d_ve(3)=1+3-2=2
        assert_eq!(dve(&paw()), vec![5, 5, 8, 2]);
    }

    // --- first_ve_degree_zagreb_alpha ---

    #[test]
    fn m1a_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(first_ve_degree_zagreb_alpha(&g).unwrap(), 0);
    }

    #[test]
    fn m1a_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(first_ve_degree_zagreb_alpha(&g).unwrap(), 0);
    }

    #[test]
    fn m1a_single_edge() {
        assert_eq!(first_ve_degree_zagreb_alpha(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn m1a_path3() {
        // d_ve=[1,2,1], M₁α = 1+4+1 = 6
        assert_eq!(first_ve_degree_zagreb_alpha(&path3()).unwrap(), 6);
    }

    #[test]
    fn m1a_path4() {
        // d_ve=[1,3,3,1], M₁α = 1+9+9+1 = 20
        assert_eq!(first_ve_degree_zagreb_alpha(&path4()).unwrap(), 20);
    }

    #[test]
    fn m1a_k3() {
        // d_ve=[4,4,4], M₁α = 48
        assert_eq!(first_ve_degree_zagreb_alpha(&k3()).unwrap(), 48);
    }

    #[test]
    fn m1a_k4() {
        // d_ve=[12,12,12,12], M₁α = 4×144 = 576
        assert_eq!(first_ve_degree_zagreb_alpha(&k4()).unwrap(), 576);
    }

    #[test]
    fn m1a_cycle4() {
        // d_ve=[4,4,4,4], M₁α = 4×16 = 64
        assert_eq!(first_ve_degree_zagreb_alpha(&cycle4()).unwrap(), 64);
    }

    #[test]
    fn m1a_star5() {
        // d_ve=[12,3,3,3,3], M₁α = 144+9+9+9+9 = 180
        assert_eq!(first_ve_degree_zagreb_alpha(&star5()).unwrap(), 180);
    }

    #[test]
    fn m1a_paw() {
        // d_ve=[5,5,8,2], M₁α = 25+25+64+4 = 118
        assert_eq!(first_ve_degree_zagreb_alpha(&paw()).unwrap(), 118);
    }

    // --- first_ve_degree_zagreb_beta ---

    #[test]
    fn m1b_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(first_ve_degree_zagreb_beta(&g).unwrap(), 0);
    }

    #[test]
    fn m1b_single_edge() {
        assert_eq!(first_ve_degree_zagreb_beta(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn m1b_path3() {
        // d_ve=[1,2,1], edges: (0,1):3, (1,2):3 → 6
        assert_eq!(first_ve_degree_zagreb_beta(&path3()).unwrap(), 6);
    }

    #[test]
    fn m1b_path4() {
        // d_ve=[1,3,3,1], edges: (0,1):4, (1,2):6, (2,3):4 → 14
        assert_eq!(first_ve_degree_zagreb_beta(&path4()).unwrap(), 14);
    }

    #[test]
    fn m1b_k3() {
        // d_ve=[4,4,4], 3 edges × 8 = 24
        assert_eq!(first_ve_degree_zagreb_beta(&k3()).unwrap(), 24);
    }

    #[test]
    fn m1b_k4() {
        // d_ve=[12,12,12,12], 6 edges × 24 = 144
        assert_eq!(first_ve_degree_zagreb_beta(&k4()).unwrap(), 144);
    }

    #[test]
    fn m1b_cycle4() {
        // d_ve=[4,4,4,4], 4 edges × 8 = 32
        assert_eq!(first_ve_degree_zagreb_beta(&cycle4()).unwrap(), 32);
    }

    #[test]
    fn m1b_star5() {
        // d_ve=[12,3,3,3,3], edges (0,leaf): 12+3=15, 4 edges → 60
        assert_eq!(first_ve_degree_zagreb_beta(&star5()).unwrap(), 60);
    }

    #[test]
    fn m1b_paw() {
        // d_ve=[5,5,8,2]
        // (0,1):10, (0,2):13, (1,2):13, (2,3):10 → 46
        assert_eq!(first_ve_degree_zagreb_beta(&paw()).unwrap(), 46);
    }

    // --- second_ve_degree_zagreb ---

    #[test]
    fn m2_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(second_ve_degree_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn m2_single_edge() {
        assert_eq!(second_ve_degree_zagreb(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn m2_path3() {
        // d_ve=[1,2,1], edges: (0,1):2, (1,2):2 → 4
        assert_eq!(second_ve_degree_zagreb(&path3()).unwrap(), 4);
    }

    #[test]
    fn m2_path4() {
        // d_ve=[1,3,3,1], edges: (0,1):3, (1,2):9, (2,3):3 → 15
        assert_eq!(second_ve_degree_zagreb(&path4()).unwrap(), 15);
    }

    #[test]
    fn m2_k3() {
        // d_ve=[4,4,4], 3 edges × 16 = 48
        assert_eq!(second_ve_degree_zagreb(&k3()).unwrap(), 48);
    }

    #[test]
    fn m2_k4() {
        // d_ve=[12,12,12,12], 6 × 144 = 864
        assert_eq!(second_ve_degree_zagreb(&k4()).unwrap(), 864);
    }

    #[test]
    fn m2_cycle4() {
        // d_ve=[4,4,4,4], 4 × 16 = 64
        assert_eq!(second_ve_degree_zagreb(&cycle4()).unwrap(), 64);
    }

    #[test]
    fn m2_star5() {
        // d_ve=[12,3,3,3,3], edges: 4 × (12×3) = 144
        assert_eq!(second_ve_degree_zagreb(&star5()).unwrap(), 144);
    }

    #[test]
    fn m2_paw() {
        // d_ve=[5,5,8,2]
        // (0,1):25, (0,2):40, (1,2):40, (2,3):16 → 121
        assert_eq!(second_ve_degree_zagreb(&paw()).unwrap(), 121);
    }

    // --- cross-consistency ---

    #[test]
    fn m1b_regular_formula() {
        // r-regular: d_ve = r²-2r+r² = 2r²-2r = 2r(r-1)
        // M₁β = m × 2 × d_ve = m × 4r(r-1)
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as u64;
            let r = g.degree(0).unwrap() as u64;
            let dve_val = 2 * r * (r - 1);
            let expected = m * 2 * dve_val;
            assert_eq!(first_ve_degree_zagreb_beta(g).unwrap(), expected);
        }
    }

    #[test]
    fn m2_regular_formula() {
        // r-regular: M₂ve = m × d_ve²
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as u64;
            let r = g.degree(0).unwrap() as u64;
            let dve_val = 2 * r * (r - 1);
            let expected = m * dve_val * dve_val;
            assert_eq!(second_ve_degree_zagreb(g).unwrap(), expected);
        }
    }

    #[test]
    fn all_positive_for_nontrivial() {
        for g in &[path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(first_ve_degree_zagreb_alpha(g).unwrap() > 0);
            assert!(first_ve_degree_zagreb_beta(g).unwrap() > 0);
            assert!(second_ve_degree_zagreb(g).unwrap() > 0);
        }
    }

    #[test]
    fn single_edge_all_zero() {
        // K_2: both vertices have d_ve=0 → all indices are 0
        let g = single_edge();
        assert_eq!(first_ve_degree_zagreb_alpha(&g).unwrap(), 0);
        assert_eq!(first_ve_degree_zagreb_beta(&g).unwrap(), 0);
        assert_eq!(second_ve_degree_zagreb(&g).unwrap(), 0);
    }
}
