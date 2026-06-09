//! Neighborhood-degree based Zagreb indices (ALGO-TR-062).
//!
//! These indices use `S(v) = Σ_{u∈N(v)} d(u)`, the sum of degrees
//! of all neighbors of vertex v (the "connection number" or
//! "neighborhood degree sum").
//!
//! - **First neighborhood Zagreb index** `NM₁(G) = Σ_v S(v)²`
//!   Introduced by Mondal et al. (2019). Vertex-level squared
//!   neighborhood sums.
//! - **Second neighborhood Zagreb index** `NM₂(G) = Σ_{(u,v)∈E} S(u)·S(v)`
//!   Edge-level product of neighborhood sums.
//! - **Neighborhood forgotten index** `NF(G) = Σ_v S(v)³`
//!   Cube of neighborhood degree sums (analog of the forgotten
//!   index but using S(v) instead of d(v)).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

fn neighbor_degree_sums(graph: &Graph) -> IgraphResult<Vec<u64>> {
    let n = graph.vcount();
    let mut s = vec![0_u64; n as usize];

    for v in 0..n {
        let nbs = graph.neighbors(v)?;
        let mut sum = 0_u64;
        for nb in nbs {
            sum = sum.saturating_add(graph.degree(nb)? as u64);
        }
        s[v as usize] = sum;
    }

    Ok(s)
}

/// Compute the first neighborhood Zagreb index.
///
/// `NM₁(G) = Σ_v S(v)²` where `S(v) = Σ_{u∈N(v)} d(u)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_neighborhood_zagreb};
///
/// // K_3: each vertex has 2 neighbors of degree 2, S(v)=4
/// // NM₁ = 3 × 16 = 48
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(first_neighborhood_zagreb(&g).unwrap(), 48);
/// ```
pub fn first_neighborhood_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let s = neighbor_degree_sums(graph)?;
    let mut nm1 = 0_u64;

    for &sv in &s {
        nm1 = nm1.saturating_add(sv.saturating_mul(sv));
    }

    Ok(nm1)
}

/// Compute the second neighborhood Zagreb index.
///
/// `NM₂(G) = Σ_{(u,v)∈E} S(u) · S(v)`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_neighborhood_zagreb};
///
/// // K_3: S(v)=4 for all v, 3 edges → 3 × 16 = 48
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(second_neighborhood_zagreb(&g).unwrap(), 48);
/// ```
pub fn second_neighborhood_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let s = neighbor_degree_sums(graph)?;
    let mut nm2 = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let su = s[u as usize];
        let sv = s[v as usize];
        nm2 = nm2.saturating_add(su.saturating_mul(sv));
    }

    Ok(nm2)
}

/// Compute the neighborhood forgotten index.
///
/// `NF(G) = Σ_v S(v)³` where `S(v) = Σ_{u∈N(v)} d(u)`.
///
/// Vertex-level cubes of neighbor degree sums.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, neighborhood_forgotten_index};
///
/// // K_3: S(v)=4 for all v, NF = 3 × 64 = 192
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(neighborhood_forgotten_index(&g).unwrap(), 192);
/// ```
pub fn neighborhood_forgotten_index(graph: &Graph) -> IgraphResult<u64> {
    let s = neighbor_degree_sums(graph)?;
    let mut nf = 0_u64;

    for &sv in &s {
        nf = nf.saturating_add(sv.saturating_mul(sv).saturating_mul(sv));
    }

    Ok(nf)
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

    fn diamond() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3)], false, Some(4)).unwrap()
    }

    // Helper: compute S(v) for manual verification
    fn compute_s(g: &Graph) -> Vec<u64> {
        neighbor_degree_sums(g).unwrap()
    }

    // --- neighbor_degree_sums ---

    #[test]
    fn nds_single_edge() {
        // v0: N={1}, d(1)=1 → S=1; v1: same
        assert_eq!(compute_s(&single_edge()), vec![1, 1]);
    }

    #[test]
    fn nds_path3() {
        // v0: N={1}, d(1)=2 → S=2; v1: N={0,2}, d(0)+d(2)=1+1=2 → S=2; v2: N={1}, S=2
        assert_eq!(compute_s(&path3()), vec![2, 2, 2]);
    }

    #[test]
    fn nds_k3() {
        // Each vertex: 2 neighbors of degree 2 → S=4
        assert_eq!(compute_s(&k3()), vec![4, 4, 4]);
    }

    #[test]
    fn nds_k4() {
        // Each vertex: 3 neighbors of degree 3 → S=9
        assert_eq!(compute_s(&k4()), vec![9, 9, 9, 9]);
    }

    #[test]
    fn nds_star5() {
        // center (d=4): 4 neighbors of d=1 → S=4
        // leaf (d=1): 1 neighbor of d=4 → S=4
        assert_eq!(compute_s(&star5()), vec![4, 4, 4, 4, 4]);
    }

    #[test]
    fn nds_cycle4() {
        // Each vertex: 2 neighbors of degree 2 → S=4
        assert_eq!(compute_s(&cycle4()), vec![4, 4, 4, 4]);
    }

    #[test]
    fn nds_paw() {
        // degrees [2,2,3,1]
        // v0: N={1,2}, d(1)+d(2)=2+3=5
        // v1: N={0,2}, d(0)+d(2)=2+3=5
        // v2: N={0,1,3}, d(0)+d(1)+d(3)=2+2+1=5
        // v3: N={2}, d(2)=3
        assert_eq!(compute_s(&paw()), vec![5, 5, 5, 3]);
    }

    // --- first_neighborhood_zagreb ---

    #[test]
    fn nm1_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(first_neighborhood_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn nm1_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(first_neighborhood_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn nm1_single_edge() {
        // S = [1,1], NM1 = 1+1 = 2
        assert_eq!(first_neighborhood_zagreb(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn nm1_path3() {
        // S = [2,2,2], NM1 = 4+4+4 = 12
        assert_eq!(first_neighborhood_zagreb(&path3()).unwrap(), 12);
    }

    #[test]
    fn nm1_path4() {
        // degrees [1,2,2,1]
        // S(0)=d(1)=2, S(1)=d(0)+d(2)=1+2=3, S(2)=d(1)+d(3)=2+1=3, S(3)=d(2)=2
        // NM1 = 4+9+9+4 = 26
        assert_eq!(first_neighborhood_zagreb(&path4()).unwrap(), 26);
    }

    #[test]
    fn nm1_k3() {
        // S = [4,4,4], NM1 = 48
        assert_eq!(first_neighborhood_zagreb(&k3()).unwrap(), 48);
    }

    #[test]
    fn nm1_k4() {
        // S = [9,9,9,9], NM1 = 4×81 = 324
        assert_eq!(first_neighborhood_zagreb(&k4()).unwrap(), 324);
    }

    #[test]
    fn nm1_cycle4() {
        // S = [4,4,4,4], NM1 = 4×16 = 64
        assert_eq!(first_neighborhood_zagreb(&cycle4()).unwrap(), 64);
    }

    #[test]
    fn nm1_cycle5() {
        // S = [4,4,4,4,4], NM1 = 5×16 = 80
        assert_eq!(first_neighborhood_zagreb(&cycle5()).unwrap(), 80);
    }

    #[test]
    fn nm1_star5() {
        // S = [4,4,4,4,4], NM1 = 5×16 = 80
        assert_eq!(first_neighborhood_zagreb(&star5()).unwrap(), 80);
    }

    #[test]
    fn nm1_paw() {
        // S = [5,5,5,3], NM1 = 25+25+25+9 = 84
        assert_eq!(first_neighborhood_zagreb(&paw()).unwrap(), 84);
    }

    #[test]
    fn nm1_diamond() {
        // degrees [3,3,2,2]
        // S(0)=d(1)+d(2)+d(3)=3+2+2=7
        // S(1)=d(0)+d(2)+d(3)=3+2+2=7
        // S(2)=d(0)+d(1)=3+3=6
        // S(3)=d(0)+d(1)=3+3=6
        // NM1 = 49+49+36+36 = 170
        assert_eq!(first_neighborhood_zagreb(&diamond()).unwrap(), 170);
    }

    #[test]
    fn nm1_regular_formula() {
        // r-regular: S(v) = r·r = r² for all v, NM1 = n·r⁴
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let n = u64::from(g.vcount());
            let r = g.degree(0).unwrap() as u64;
            assert_eq!(first_neighborhood_zagreb(g).unwrap(), n * r * r * r * r);
        }
    }

    // --- second_neighborhood_zagreb ---

    #[test]
    fn nm2_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(second_neighborhood_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn nm2_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(second_neighborhood_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn nm2_single_edge() {
        // S=[1,1], edge: 1×1=1
        assert_eq!(second_neighborhood_zagreb(&single_edge()).unwrap(), 1);
    }

    #[test]
    fn nm2_path3() {
        // S=[2,2,2], edges: (0,1):4, (1,2):4 → NM2=8
        assert_eq!(second_neighborhood_zagreb(&path3()).unwrap(), 8);
    }

    #[test]
    fn nm2_path4() {
        // S=[2,3,3,2], edges: (0,1):6, (1,2):9, (2,3):6 → NM2=21
        assert_eq!(second_neighborhood_zagreb(&path4()).unwrap(), 21);
    }

    #[test]
    fn nm2_k3() {
        // S=[4,4,4], 3 edges × 16 = 48
        assert_eq!(second_neighborhood_zagreb(&k3()).unwrap(), 48);
    }

    #[test]
    fn nm2_k4() {
        // S=[9,9,9,9], 6 edges × 81 = 486
        assert_eq!(second_neighborhood_zagreb(&k4()).unwrap(), 486);
    }

    #[test]
    fn nm2_cycle4() {
        // S=[4,4,4,4], 4 edges × 16 = 64
        assert_eq!(second_neighborhood_zagreb(&cycle4()).unwrap(), 64);
    }

    #[test]
    fn nm2_cycle5() {
        // S=[4,4,4,4,4], 5 × 16 = 80
        assert_eq!(second_neighborhood_zagreb(&cycle5()).unwrap(), 80);
    }

    #[test]
    fn nm2_star5() {
        // S=[4,4,4,4,4], 4 edges × 16 = 64
        assert_eq!(second_neighborhood_zagreb(&star5()).unwrap(), 64);
    }

    #[test]
    fn nm2_paw() {
        // S=[5,5,5,3]
        // (0,1):25, (0,2):25, (1,2):25, (2,3):15
        // NM2 = 90
        assert_eq!(second_neighborhood_zagreb(&paw()).unwrap(), 90);
    }

    #[test]
    fn nm2_diamond() {
        // S=[7,7,6,6]
        // (0,1):49, (0,2):42, (0,3):42, (1,2):42, (1,3):42
        // NM2 = 49+42+42+42+42 = 217
        assert_eq!(second_neighborhood_zagreb(&diamond()).unwrap(), 217);
    }

    #[test]
    fn nm2_regular_formula() {
        // r-regular: NM2 = m·r⁴
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as u64;
            let r = g.degree(0).unwrap() as u64;
            assert_eq!(second_neighborhood_zagreb(g).unwrap(), m * r * r * r * r);
        }
    }

    // --- neighborhood_forgotten_index ---

    #[test]
    fn nf_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(neighborhood_forgotten_index(&g).unwrap(), 0);
    }

    #[test]
    fn nf_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(neighborhood_forgotten_index(&g).unwrap(), 0);
    }

    #[test]
    fn nf_single_edge() {
        // S=[1,1], NF = 1+1 = 2
        assert_eq!(neighborhood_forgotten_index(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn nf_path3() {
        // S=[2,2,2], NF = 3×8 = 24
        assert_eq!(neighborhood_forgotten_index(&path3()).unwrap(), 24);
    }

    #[test]
    fn nf_path4() {
        // S=[2,3,3,2], NF = 8+27+27+8 = 70
        assert_eq!(neighborhood_forgotten_index(&path4()).unwrap(), 70);
    }

    #[test]
    fn nf_k3() {
        // S=[4,4,4], NF = 3×64 = 192
        assert_eq!(neighborhood_forgotten_index(&k3()).unwrap(), 192);
    }

    #[test]
    fn nf_k4() {
        // S=[9,9,9,9], NF = 4×729 = 2916
        assert_eq!(neighborhood_forgotten_index(&k4()).unwrap(), 2916);
    }

    #[test]
    fn nf_cycle4() {
        // S=[4,4,4,4], NF = 4×64 = 256
        assert_eq!(neighborhood_forgotten_index(&cycle4()).unwrap(), 256);
    }

    #[test]
    fn nf_cycle5() {
        // S=[4,4,4,4,4], NF = 5×64 = 320
        assert_eq!(neighborhood_forgotten_index(&cycle5()).unwrap(), 320);
    }

    #[test]
    fn nf_star5() {
        // S=[4,4,4,4,4], NF = 5×64 = 320
        assert_eq!(neighborhood_forgotten_index(&star5()).unwrap(), 320);
    }

    #[test]
    fn nf_paw() {
        // S=[5,5,5,3], NF = 125+125+125+27 = 402
        assert_eq!(neighborhood_forgotten_index(&paw()).unwrap(), 402);
    }

    #[test]
    fn nf_diamond() {
        // S=[7,7,6,6], NF = 343+343+216+216 = 1118
        assert_eq!(neighborhood_forgotten_index(&diamond()).unwrap(), 1118);
    }

    #[test]
    fn nf_regular_formula() {
        // r-regular: NF = n·r⁶
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let n = u64::from(g.vcount());
            let r = g.degree(0).unwrap() as u64;
            let r6 = r * r * r * r * r * r;
            assert_eq!(neighborhood_forgotten_index(g).unwrap(), n * r6);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn nm1_nm2_coincide_for_regular() {
        // For r-regular graphs: NM1/n = NM2/m since both equal r⁴·(n or m)
        // Actually NM1 = n·r⁴ and NM2 = m·r⁴ and m=nr/2,
        // so NM2 = NM1·r/2
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let nm1 = first_neighborhood_zagreb(g).unwrap();
            let nm2 = second_neighborhood_zagreb(g).unwrap();
            let r = g.degree(0).unwrap() as u64;
            assert_eq!(nm2 * 2, nm1 * r);
        }
    }

    #[test]
    fn all_positive_for_nonempty_edges() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(first_neighborhood_zagreb(g).unwrap() > 0);
            assert!(second_neighborhood_zagreb(g).unwrap() > 0);
            assert!(neighborhood_forgotten_index(g).unwrap() > 0);
        }
    }
}
