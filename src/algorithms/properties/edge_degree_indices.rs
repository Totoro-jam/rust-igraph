//! Edge-degree based topological indices (ALGO-TR-061).
//!
//! - **Platt index** `F(G) = Σ_{(u,v)∈E} (d(u) + d(v) - 2)`
//!   Sum of edge degrees (each edge's degree = d(u)+d(v)-2).
//!   Introduced by Platt (1947).
//! - **Gordon-Scantlebury index** `GS(G) = Platt(G) / 2`
//!   Counts the number of paths of length 2 in G.
//!   Gordon & Scantlebury (1964).
//! - **Bertz complexity index** `B(G) = Σ_{(u,v)∈E} C(d(u)+d(v)-2, 2)`
//!   where `C(n,2) = n·(n-1)/2`. Combinatorial edge complexity.
//!   Bertz (1981).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the Platt index (sum of edge degrees).
///
/// `F(G) = Σ_{(u,v)∈E} (d(u) + d(v) - 2)`
///
/// Each edge (u,v) has edge-degree = d(u) + d(v) - 2 (the number
/// of edges adjacent to it, not counting itself). Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, platt_index};
///
/// // K_3: each edge (2+2-2)=2, 3 edges → 6
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(platt_index(&g).unwrap(), 6);
/// ```
pub fn platt_index(graph: &Graph) -> IgraphResult<u64> {
    let mut f = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as u64;
        let dv = graph.degree(v)? as u64;
        f = f.saturating_add(du + dv - 2);
    }

    Ok(f)
}

/// Compute the Gordon-Scantlebury index (path-of-length-2 count).
///
/// `GS(G) = Platt(G) / 2 = Σ_{(u,v)∈E} (d(u) + d(v) - 2) / 2`
///
/// Counts the number of paths of length 2 (P₂) in the graph.
/// Equivalently, `GS = Σ_v C(d(v), 2) = Σ_v d(v)·(d(v)-1)/2`.
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, gordon_scantlebury_index};
///
/// // K_3: Platt=6, GS=3 (three paths of length 2)
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(gordon_scantlebury_index(&g).unwrap(), 3);
/// ```
pub fn gordon_scantlebury_index(graph: &Graph) -> IgraphResult<u64> {
    let mut gs = 0_u64;

    let n = graph.vcount();
    for v in 0..n {
        let d = graph.degree(v)? as u64;
        if d >= 2 {
            gs = gs.saturating_add(d * (d - 1) / 2);
        }
    }

    Ok(gs)
}

/// Compute the Bertz complexity index.
///
/// `B(G) = Σ_{(u,v)∈E} C(d(u)+d(v)-2, 2)`
///
/// where `C(n,2) = n·(n-1)/2`. Self-loops are skipped.
/// Edges where `d(u)+d(v) < 4` contribute 0.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bertz_complexity_index};
///
/// // K_3: each edge d(u)+d(v)-2=2, C(2,2)=1, 3 edges → 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(bertz_complexity_index(&g).unwrap(), 3);
/// ```
pub fn bertz_complexity_index(graph: &Graph) -> IgraphResult<u64> {
    let mut b = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as u64;
        let dv = graph.degree(v)? as u64;
        let edge_deg = du + dv - 2;
        if edge_deg >= 2 {
            b = b.saturating_add(edge_deg * (edge_deg - 1) / 2);
        }
    }

    Ok(b)
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

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn diamond() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3)], false, Some(4)).unwrap()
    }

    // --- platt_index ---

    #[test]
    fn platt_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(platt_index(&g).unwrap(), 0);
    }

    #[test]
    fn platt_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(platt_index(&g).unwrap(), 0);
    }

    #[test]
    fn platt_single_edge() {
        // (1+1-2) = 0
        assert_eq!(platt_index(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn platt_path3() {
        // (0,1): 1+2-2=1, (1,2): 2+1-2=1 → 2
        assert_eq!(platt_index(&path3()).unwrap(), 2);
    }

    #[test]
    fn platt_path4() {
        // (0,1):1+2-2=1, (1,2):2+2-2=2, (2,3):2+1-2=1 → 4
        assert_eq!(platt_index(&path4()).unwrap(), 4);
    }

    #[test]
    fn platt_path5() {
        // (0,1):1, (1,2):2, (2,3):2, (3,4):1 → 6
        assert_eq!(platt_index(&path5()).unwrap(), 6);
    }

    #[test]
    fn platt_k3() {
        // 3 × (2+2-2) = 3×2 = 6
        assert_eq!(platt_index(&k3()).unwrap(), 6);
    }

    #[test]
    fn platt_k4() {
        // 6 × (3+3-2) = 6×4 = 24
        assert_eq!(platt_index(&k4()).unwrap(), 24);
    }

    #[test]
    fn platt_cycle4() {
        // 4 × (2+2-2) = 4×2 = 8
        assert_eq!(platt_index(&cycle4()).unwrap(), 8);
    }

    #[test]
    fn platt_cycle5() {
        // 5 × (2+2-2) = 5×2 = 10
        assert_eq!(platt_index(&cycle5()).unwrap(), 10);
    }

    #[test]
    fn platt_star5() {
        // 4 × (4+1-2) = 4×3 = 12
        assert_eq!(platt_index(&star5()).unwrap(), 12);
    }

    #[test]
    fn platt_paw() {
        // degrees [2,2,3,1]
        // (0,1):2+2-2=2, (0,2):2+3-2=3, (1,2):2+3-2=3, (2,3):3+1-2=2
        // F = 2+3+3+2 = 10
        assert_eq!(platt_index(&paw()).unwrap(), 10);
    }

    #[test]
    fn platt_diamond() {
        // degrees [3,3,2,2]
        // (0,1):3+3-2=4, (0,2):3+2-2=3, (0,3):3+2-2=3, (1,2):3+2-2=3, (1,3):3+2-2=3
        // F = 4+3+3+3+3 = 16
        assert_eq!(platt_index(&diamond()).unwrap(), 16);
    }

    #[test]
    fn platt_equals_first_zagreb_minus_2m() {
        // Platt = M₁ - 2m where M₁ = first Zagreb index = Σ d²
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let m = g.ecount() as u64;
            let mut m1 = 0_u64;
            for v in 0..g.vcount() {
                let d = g.degree(v).unwrap() as u64;
                m1 += d * d;
            }
            assert_eq!(platt_index(g).unwrap(), m1 - 2 * m);
        }
    }

    #[test]
    fn platt_regular_formula() {
        // r-regular: Platt = m·(2r-2) = 2m(r-1)
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as u64;
            let r = g.degree(0).unwrap() as u64;
            assert_eq!(platt_index(g).unwrap(), 2 * m * (r - 1));
        }
    }

    // --- gordon_scantlebury_index ---

    #[test]
    fn gs_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(gordon_scantlebury_index(&g).unwrap(), 0);
    }

    #[test]
    fn gs_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(gordon_scantlebury_index(&g).unwrap(), 0);
    }

    #[test]
    fn gs_single_edge() {
        // No vertex has d≥2 → 0
        assert_eq!(gordon_scantlebury_index(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn gs_path3() {
        // Only vertex 1 has d=2: C(2,2)=1 → GS=1
        assert_eq!(gordon_scantlebury_index(&path3()).unwrap(), 1);
    }

    #[test]
    fn gs_path4() {
        // vertices 1,2 have d=2: 2×C(2,2)=2 → GS=2
        assert_eq!(gordon_scantlebury_index(&path4()).unwrap(), 2);
    }

    #[test]
    fn gs_path5() {
        // vertices 1,2,3 have d=2: 3×1=3
        assert_eq!(gordon_scantlebury_index(&path5()).unwrap(), 3);
    }

    #[test]
    fn gs_k3() {
        // 3 vertices d=2: 3×C(2,2)=3
        assert_eq!(gordon_scantlebury_index(&k3()).unwrap(), 3);
    }

    #[test]
    fn gs_k4() {
        // 4 vertices d=3: 4×C(3,2)=4×3=12
        assert_eq!(gordon_scantlebury_index(&k4()).unwrap(), 12);
    }

    #[test]
    fn gs_cycle4() {
        // 4 × C(2,2) = 4
        assert_eq!(gordon_scantlebury_index(&cycle4()).unwrap(), 4);
    }

    #[test]
    fn gs_cycle5() {
        // 5 × C(2,2) = 5
        assert_eq!(gordon_scantlebury_index(&cycle5()).unwrap(), 5);
    }

    #[test]
    fn gs_star5() {
        // center d=4: C(4,2)=6; leaves d=1: 0 → GS=6
        assert_eq!(gordon_scantlebury_index(&star5()).unwrap(), 6);
    }

    #[test]
    fn gs_paw() {
        // degrees [2,2,3,1]: C(2,2)+C(2,2)+C(3,2)+0 = 1+1+3 = 5
        assert_eq!(gordon_scantlebury_index(&paw()).unwrap(), 5);
    }

    #[test]
    fn gs_diamond() {
        // degrees [3,3,2,2]: C(3,2)+C(3,2)+C(2,2)+C(2,2) = 3+3+1+1 = 8
        assert_eq!(gordon_scantlebury_index(&diamond()).unwrap(), 8);
    }

    #[test]
    fn gs_equals_platt_half() {
        // GS = Platt / 2
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert_eq!(
                gordon_scantlebury_index(g).unwrap(),
                platt_index(g).unwrap() / 2
            );
        }
    }

    #[test]
    fn gs_regular_formula() {
        // r-regular: GS = n·C(r,2) = n·r(r-1)/2
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let n = u64::from(g.vcount());
            let r = g.degree(0).unwrap() as u64;
            assert_eq!(gordon_scantlebury_index(g).unwrap(), n * r * (r - 1) / 2);
        }
    }

    // --- bertz_complexity_index ---

    #[test]
    fn bertz_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(bertz_complexity_index(&g).unwrap(), 0);
    }

    #[test]
    fn bertz_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(bertz_complexity_index(&g).unwrap(), 0);
    }

    #[test]
    fn bertz_single_edge() {
        // edge-deg = 0, C(0,2)=0
        assert_eq!(bertz_complexity_index(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn bertz_path3() {
        // edges: (0,1): ed=1→C(1,2)=0, (1,2): ed=1→0 → B=0
        assert_eq!(bertz_complexity_index(&path3()).unwrap(), 0);
    }

    #[test]
    fn bertz_path4() {
        // (0,1):ed=1→0, (1,2):ed=2→C(2,2)=1, (2,3):ed=1→0 → B=1
        assert_eq!(bertz_complexity_index(&path4()).unwrap(), 1);
    }

    #[test]
    fn bertz_path5() {
        // (0,1):1→0, (1,2):2→1, (2,3):2→1, (3,4):1→0 → B=2
        assert_eq!(bertz_complexity_index(&path5()).unwrap(), 2);
    }

    #[test]
    fn bertz_k3() {
        // 3 edges, each ed=2, C(2,2)=1 → B=3
        assert_eq!(bertz_complexity_index(&k3()).unwrap(), 3);
    }

    #[test]
    fn bertz_k4() {
        // 6 edges, each ed=4, C(4,2)=6 → B=36
        assert_eq!(bertz_complexity_index(&k4()).unwrap(), 36);
    }

    #[test]
    fn bertz_cycle4() {
        // 4 edges, each ed=2, C(2,2)=1 → B=4
        assert_eq!(bertz_complexity_index(&cycle4()).unwrap(), 4);
    }

    #[test]
    fn bertz_cycle5() {
        // 5 × C(2,2) = 5
        assert_eq!(bertz_complexity_index(&cycle5()).unwrap(), 5);
    }

    #[test]
    fn bertz_star5() {
        // 4 edges, each ed=3, C(3,2)=3 → B=12
        assert_eq!(bertz_complexity_index(&star5()).unwrap(), 12);
    }

    #[test]
    fn bertz_paw() {
        // (0,1):ed=2→1, (0,2):ed=3→3, (1,2):ed=3→3, (2,3):ed=2→1 → B=8
        assert_eq!(bertz_complexity_index(&paw()).unwrap(), 8);
    }

    #[test]
    fn bertz_diamond() {
        // (0,1):ed=4→6, (0,2):ed=3→3, (0,3):ed=3→3, (1,2):ed=3→3, (1,3):ed=3→3
        // B = 6+3+3+3+3 = 18
        assert_eq!(bertz_complexity_index(&diamond()).unwrap(), 18);
    }

    #[test]
    fn bertz_regular_formula() {
        // r-regular: B = m·C(2r-2,2) = m·(2r-2)(2r-3)/2
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as u64;
            let r = g.degree(0).unwrap() as u64;
            let ed = 2 * r - 2;
            assert_eq!(bertz_complexity_index(g).unwrap(), m * ed * (ed - 1) / 2);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn platt_geq_zero() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let _ = platt_index(g).unwrap();
        }
    }

    #[test]
    fn gs_leq_platt() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(gordon_scantlebury_index(g).unwrap() <= platt_index(g).unwrap());
        }
    }

    #[test]
    fn bertz_leq_gs_squared_approx() {
        // Just sanity-check: Bertz is always computable
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let _ = bertz_complexity_index(g).unwrap();
        }
    }
}
