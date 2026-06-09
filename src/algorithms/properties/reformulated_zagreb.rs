//! Reformulated Zagreb indices (ALGO-TR-054).
//!
//! Edge-degree based variants of the Zagreb indices, where the
//! **edge degree** `ε(e)` of edge `e=(u,v)` is `d(u)+d(v)-2`.
//!
//! - **First reformulated Zagreb** `EM₁(G) = Σ_{e∈E} ε(e)²`
//! - **Second reformulated Zagreb** `EM₂(G) = Σ_{e~f} ε(e)·ε(f)`
//!   where `e~f` means edges e and f share a vertex.
//! - **Third Zagreb index** `M₃(G) = Σ_{(u,v)∈E} |d(u)-d(v)|`
//!   (also called the irregularity index of edges).
//!
//! Introduced by Milićević, Nikolić & Trinajstić (2004).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the first reformulated Zagreb index.
///
/// `EM₁(G) = Σ_{e∈E} ε(e)²` where `ε(e) = d(u) + d(v) - 2`.
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_reformulated_zagreb};
///
/// // K_3: each edge has ε = 2+2-2 = 2, EM₁ = 3·4 = 12
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(first_reformulated_zagreb(&g).unwrap(), 12);
/// ```
pub fn first_reformulated_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let mut em1: u64 = 0;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as u64;
        let dv = graph.degree(v)? as u64;
        let eps = du + dv - 2;
        em1 = em1.saturating_add(eps.saturating_mul(eps));
    }

    Ok(em1)
}

/// Compute the second reformulated Zagreb index.
///
/// `EM₂(G) = Σ_{e~f, e<f} ε(e) · ε(f)`
///
/// The sum is over all pairs of adjacent edges (sharing a vertex).
/// For each vertex v, every pair of edges incident to v contributes
/// one term. Self-loops are excluded.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_reformulated_zagreb};
///
/// // Path 0-1-2: edges e₁=(0,1) ε=1, e₂=(1,2) ε=1
/// // Only pair sharing vertex 1: ε·ε = 1 → EM₂ = 1
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(second_reformulated_zagreb(&g).unwrap(), 1);
/// ```
pub fn second_reformulated_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();

    if ecount == 0 {
        return Ok(0);
    }

    let edges: Vec<(u32, u32)> = graph.edges().collect();
    let mut deg = vec![0_usize; n];
    for v in 0..n as u32 {
        deg[v as usize] = graph.degree(v)?;
    }

    let mut edge_deg = Vec::with_capacity(ecount);
    for &(u, v) in &edges {
        if u == v {
            edge_deg.push(0_u64);
            continue;
        }
        edge_deg.push((deg[u as usize] + deg[v as usize] - 2) as u64);
    }

    let mut incident: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (idx, &(u, v)) in edges.iter().enumerate() {
        if u == v {
            continue;
        }
        incident[u as usize].push(idx);
        incident[v as usize].push(idx);
    }

    let mut em2: u64 = 0;
    for v in 0..n {
        let inc = &incident[v];
        let k = inc.len();
        for i in 0..k {
            for j in (i + 1)..k {
                em2 = em2.saturating_add(edge_deg[inc[i]].saturating_mul(edge_deg[inc[j]]));
            }
        }
    }

    Ok(em2)
}

/// Compute the third Zagreb index.
///
/// `M₃(G) = Σ_{(u,v)∈E} |d(u) - d(v)|`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, third_zagreb_index};
///
/// // Star S_4 (center=0): edges (0,1)..(0,4), degrees [4,1,1,1,1]
/// // Each edge: |4-1|=3, 4 edges → M₃ = 12
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert_eq!(third_zagreb_index(&g).unwrap(), 12);
/// ```
pub fn third_zagreb_index(graph: &Graph) -> IgraphResult<u64> {
    let mut m3: u64 = 0;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)?;
        let dv = graph.degree(v)?;
        m3 = m3.saturating_add(du.abs_diff(dv) as u64);
    }

    Ok(m3)
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

    // --- first_reformulated_zagreb ---

    #[test]
    fn em1_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(first_reformulated_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn em1_single_vertex() {
        let g = Graph::with_vertices(1);
        assert_eq!(first_reformulated_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn em1_single_edge() {
        // ε = 1+1-2 = 0, EM₁ = 0
        assert_eq!(first_reformulated_zagreb(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn em1_path3() {
        // edges: (0,1) ε=1+2-2=1, (1,2) ε=2+1-2=1
        // EM₁ = 1+1 = 2
        assert_eq!(first_reformulated_zagreb(&path3()).unwrap(), 2);
    }

    #[test]
    fn em1_path4() {
        // edges: (0,1) ε=1+2-2=1, (1,2) ε=2+2-2=2, (2,3) ε=2+1-2=1
        // EM₁ = 1+4+1 = 6
        assert_eq!(first_reformulated_zagreb(&path4()).unwrap(), 6);
    }

    #[test]
    fn em1_k3() {
        // each edge ε=2+2-2=2, 3 edges
        // EM₁ = 3·4 = 12
        assert_eq!(first_reformulated_zagreb(&k3()).unwrap(), 12);
    }

    #[test]
    fn em1_k4() {
        // each edge ε=3+3-2=4, 6 edges
        // EM₁ = 6·16 = 96
        assert_eq!(first_reformulated_zagreb(&k4()).unwrap(), 96);
    }

    #[test]
    fn em1_cycle4() {
        // each edge ε=2+2-2=2, 4 edges
        // EM₁ = 4·4 = 16
        assert_eq!(first_reformulated_zagreb(&cycle4()).unwrap(), 16);
    }

    #[test]
    fn em1_cycle5() {
        // each edge ε=2+2-2=2, 5 edges
        // EM₁ = 5·4 = 20
        assert_eq!(first_reformulated_zagreb(&cycle5()).unwrap(), 20);
    }

    #[test]
    fn em1_star5() {
        // each edge ε=4+1-2=3, 4 edges
        // EM₁ = 4·9 = 36
        assert_eq!(first_reformulated_zagreb(&star5()).unwrap(), 36);
    }

    #[test]
    fn em1_paw() {
        // degrees [2,2,3,1]
        // (0,1): ε=2+2-2=2, (0,2): ε=2+3-2=3, (1,2): ε=2+3-2=3, (2,3): ε=3+1-2=2
        // EM₁ = 4+9+9+4 = 26
        assert_eq!(first_reformulated_zagreb(&paw()).unwrap(), 26);
    }

    #[test]
    fn em1_regular_formula() {
        // r-regular: ε = 2r-2, EM₁ = m·(2r-2)²
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as u64;
            let r = g.degree(0).unwrap() as u64;
            let eps = 2 * r - 2;
            assert_eq!(first_reformulated_zagreb(g).unwrap(), m * eps * eps);
        }
    }

    // --- second_reformulated_zagreb ---

    #[test]
    fn em2_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(second_reformulated_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn em2_single_edge() {
        // Only 1 edge, no adjacent pair → 0
        assert_eq!(second_reformulated_zagreb(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn em2_path3() {
        // e₁=(0,1) ε=1, e₂=(1,2) ε=1, adjacent at vertex 1
        // EM₂ = 1·1 = 1
        assert_eq!(second_reformulated_zagreb(&path3()).unwrap(), 1);
    }

    #[test]
    fn em2_path4() {
        // e₁=(0,1) ε=1, e₂=(1,2) ε=2, e₃=(2,3) ε=1
        // Pairs: (e₁,e₂) at v1: 1·2=2, (e₂,e₃) at v2: 2·1=2
        // EM₂ = 4
        assert_eq!(second_reformulated_zagreb(&path4()).unwrap(), 4);
    }

    #[test]
    fn em2_k3() {
        // 3 edges, each ε=2. Each vertex has 2 incident edges.
        // At each vertex: 1 pair → 2·2=4. 3 vertices → 12.
        // But each edge pair is adjacent at exactly 1 vertex (they share exactly 1 vertex).
        // There are 3 edge pairs, each contributing 2·2=4 → EM₂ = 12
        assert_eq!(second_reformulated_zagreb(&k3()).unwrap(), 12);
    }

    #[test]
    fn em2_k4() {
        // 6 edges, each ε=4. Each vertex has degree 3 → C(3,2)=3 pairs per vertex.
        // 4 vertices → 12 adjacent-edge pairs, each 4·4=16 → EM₂ = 192
        assert_eq!(second_reformulated_zagreb(&k4()).unwrap(), 192);
    }

    #[test]
    fn em2_cycle4() {
        // 4 edges, each ε=2. Each vertex has degree 2 → C(2,2)=1 pair per vertex.
        // 4 vertices → 4 pairs, each 2·2=4 → EM₂ = 16
        assert_eq!(second_reformulated_zagreb(&cycle4()).unwrap(), 16);
    }

    #[test]
    fn em2_cycle5() {
        // 5 edges, each ε=2. 5 vertices, each C(2,2)=1 pair.
        // 5 pairs, each 2·2=4 → EM₂ = 20
        assert_eq!(second_reformulated_zagreb(&cycle5()).unwrap(), 20);
    }

    #[test]
    fn em2_star5() {
        // 4 edges, each ε=3. Center has C(4,2)=6 pairs.
        // Leaves have degree 1, no pairs. 6 pairs, each 3·3=9 → EM₂ = 54
        assert_eq!(second_reformulated_zagreb(&star5()).unwrap(), 54);
    }

    #[test]
    fn em2_paw() {
        // degrees [2,2,3,1], edges: (0,1)ε=2, (0,2)ε=3, (1,2)ε=3, (2,3)ε=2
        // v0 (deg 2): pairs (e01,e02): 2·3=6
        // v1 (deg 2): pairs (e01,e12): 2·3=6
        // v2 (deg 3): pairs (e02,e12):3·3=9, (e02,e23):3·2=6, (e12,e23):3·2=6 → 21
        // v3 (deg 1): no pairs
        // EM₂ = 6+6+21 = 33
        assert_eq!(second_reformulated_zagreb(&paw()).unwrap(), 33);
    }

    #[test]
    fn em2_regular_formula() {
        // r-regular: ε=2r-2 for all edges. Number of adjacent edge pairs = m·(r-1)
        // (each edge has r-1 neighbors at each endpoint, but each pair counted once,
        //  so total pairs = Σ_v C(deg(v),2) = n·C(r,2) = n·r(r-1)/2)
        // Each pair contributes (2r-2)² → EM₂ = n·r(r-1)/2·(2r-2)²
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let n = u64::from(g.vcount());
            let r = g.degree(0).unwrap() as u64;
            let eps = 2 * r - 2;
            let pairs = n * r * (r - 1) / 2;
            assert_eq!(second_reformulated_zagreb(g).unwrap(), pairs * eps * eps);
        }
    }

    // --- third_zagreb_index ---

    #[test]
    fn m3_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(third_zagreb_index(&g).unwrap(), 0);
    }

    #[test]
    fn m3_single_edge() {
        // |1-1| = 0
        assert_eq!(third_zagreb_index(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn m3_path3() {
        // (0,1):|1-2|=1, (1,2):|2-1|=1 → M₃ = 2
        assert_eq!(third_zagreb_index(&path3()).unwrap(), 2);
    }

    #[test]
    fn m3_path4() {
        // (0,1):|1-2|=1, (1,2):|2-2|=0, (2,3):|2-1|=1 → M₃ = 2
        assert_eq!(third_zagreb_index(&path4()).unwrap(), 2);
    }

    #[test]
    fn m3_k3() {
        // All degrees equal → M₃ = 0
        assert_eq!(third_zagreb_index(&k3()).unwrap(), 0);
    }

    #[test]
    fn m3_k4() {
        assert_eq!(third_zagreb_index(&k4()).unwrap(), 0);
    }

    #[test]
    fn m3_cycle4() {
        assert_eq!(third_zagreb_index(&cycle4()).unwrap(), 0);
    }

    #[test]
    fn m3_star5() {
        // All edges: |4-1|=3, 4 edges → M₃ = 12
        assert_eq!(third_zagreb_index(&star5()).unwrap(), 12);
    }

    #[test]
    fn m3_paw() {
        // (0,1):|2-2|=0, (0,2):|2-3|=1, (1,2):|2-3|=1, (2,3):|3-1|=2 → M₃ = 4
        assert_eq!(third_zagreb_index(&paw()).unwrap(), 4);
    }

    #[test]
    fn m3_zero_for_regular() {
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            assert_eq!(third_zagreb_index(g).unwrap(), 0);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn all_compute_ok() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            first_reformulated_zagreb(g).unwrap();
            second_reformulated_zagreb(g).unwrap();
            third_zagreb_index(g).unwrap();
        }
    }

    #[test]
    fn em1_zero_iff_matching() {
        // EM₁ = 0 iff every edge has ε = 0 iff d(u)+d(v) = 2 for all edges
        // That means all edges are between degree-1 vertices → perfect matching
        assert_eq!(first_reformulated_zagreb(&single_edge()).unwrap(), 0);
        let matching = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert_eq!(first_reformulated_zagreb(&matching).unwrap(), 0);
    }
}
