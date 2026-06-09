//! Forgotten topological index and reduced second Zagreb (ALGO-TR-049).
//!
//! - **Forgotten topological index** `F(G) = Σ_{(u,v)∈E} (d_u² + d_v²)`
//!   Also written as `Σ_{v∈V} d_v³`. Introduced by Furtula & Gutman
//!   (2015), rediscovered from a 1972 paper. Sums the cubes of degrees,
//!   or equivalently sums squared degrees over edge endpoints.
//! - **Reduced second Zagreb index** `RM₂(G) = Σ_{(u,v)∈E} (d_u-1)(d_v-1)`
//!   Counts "reduced" degree products. Useful in QSPR for modelling
//!   molecular properties. For a tree on n vertices, `RM₂ = M₁ - (n-1)`
//!   where `M₁` is the first Zagreb index.
//! - **Modified first Zagreb index** `^mM₁(G) = Σ_{v∈V} 1/d_v²`
//!   The sum of reciprocal squared degrees (excludes isolated vertices).
//!   Used in QSPR studies of graph invariants.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the forgotten topological index.
///
/// `F(G) = Σ_{(u,v)∈E} (d_u² + d_v²)`
///
/// Equivalently, `F(G) = Σ_{v∈V} d_v³` (each vertex's cube-degree
/// summed). Self-loops are skipped in the edge formulation.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, forgotten_index};
///
/// // K_3: all degrees 2 → F = 3·(4+4) = 24
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((forgotten_index(&g).unwrap() - 24.0).abs() < 1e-10);
/// ```
pub fn forgotten_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let mut f = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        f += du * du + dv * dv;
    }

    Ok(f)
}

/// Compute the reduced second Zagreb index.
///
/// `RM₂(G) = Σ_{(u,v)∈E} (d_u − 1)(d_v − 1)`
///
/// Self-loops are skipped. Pendant edges (where one endpoint has
/// degree 1) contribute 0.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reduced_second_zagreb};
///
/// // K_3: all degrees 2 → each edge: (2-1)(2-1)=1, 3 edges → RM₂=3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((reduced_second_zagreb(&g).unwrap() - 3.0).abs() < 1e-10);
/// ```
pub fn reduced_second_zagreb(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let mut rm2 = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        rm2 += (du - 1.0) * (dv - 1.0);
    }

    Ok(rm2)
}

/// Compute the modified first Zagreb index.
///
/// `^mM₁(G) = Σ_{v∈V, d_v>0} 1/d_v²`
///
/// Sums the reciprocal of squared degree over all non-isolated vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, modified_first_zagreb};
///
/// // K_3: all degrees 2 → 3 · (1/4) = 3/4
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((modified_first_zagreb(&g).unwrap() - 0.75).abs() < 1e-10);
/// ```
pub fn modified_first_zagreb(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let mut mm1 = 0.0_f64;

    for v in 0..n {
        let d = graph.degree(v)? as f64;
        if d > 0.0 {
            mm1 += 1.0 / (d * d);
        }
    }

    Ok(mm1)
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

    // --- forgotten_index ---

    #[test]
    fn fi_empty() {
        let g = Graph::with_vertices(0);
        assert!((forgotten_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn fi_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((forgotten_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn fi_no_edges() {
        let g = Graph::with_vertices(3);
        assert!((forgotten_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn fi_single_edge() {
        // d_u=d_v=1: 1²+1² = 2
        assert!((forgotten_index(&single_edge()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn fi_path3() {
        // degrees [1,2,1]
        // (0,1): 1+4=5, (1,2): 4+1=5 → F=10
        assert!((forgotten_index(&path3()).unwrap() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn fi_path4() {
        // degrees [1,2,2,1]
        // (0,1): 1+4=5, (1,2): 4+4=8, (2,3): 4+1=5 → F=18
        assert!((forgotten_index(&path4()).unwrap() - 18.0).abs() < 1e-10);
    }

    #[test]
    fn fi_k3() {
        // all degrees 2: 3·(4+4) = 24
        assert!((forgotten_index(&k3()).unwrap() - 24.0).abs() < 1e-10);
    }

    #[test]
    fn fi_k4() {
        // all degrees 3: 6·(9+9) = 108
        assert!((forgotten_index(&k4()).unwrap() - 108.0).abs() < 1e-10);
    }

    #[test]
    fn fi_cycle4() {
        // all degrees 2: 4·8 = 32
        assert!((forgotten_index(&cycle4()).unwrap() - 32.0).abs() < 1e-10);
    }

    #[test]
    fn fi_cycle5() {
        // all degrees 2: 5·8 = 40
        assert!((forgotten_index(&cycle5()).unwrap() - 40.0).abs() < 1e-10);
    }

    #[test]
    fn fi_star5() {
        // center deg 4, leaf deg 1: 4·(16+1) = 68
        assert!((forgotten_index(&star5()).unwrap() - 68.0).abs() < 1e-10);
    }

    #[test]
    fn fi_paw() {
        // degrees [2,2,3,1]
        // (0,1): 4+4=8, (0,2): 4+9=13, (1,2): 4+9=13, (2,3): 9+1=10
        // F = 8+13+13+10 = 44
        assert!((forgotten_index(&paw()).unwrap() - 44.0).abs() < 1e-10);
    }

    #[test]
    fn fi_diamond() {
        // degrees [3,3,2,2]
        // (0,1): 9+9=18, (0,2): 9+4=13, (0,3): 9+4=13, (1,2): 9+4=13, (1,3): 9+4=13
        // F = 18+13+13+13+13 = 70
        assert!((forgotten_index(&diamond()).unwrap() - 70.0).abs() < 1e-10);
    }

    #[test]
    fn fi_equals_sum_cubes() {
        // F(G) = Σ d_v³ (vertex sum form)
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let f_edge = forgotten_index(g).unwrap();
            let mut f_vertex = 0.0_f64;
            for v in 0..g.vcount() {
                let d = g.degree(v).unwrap() as f64;
                f_vertex += d * d * d;
            }
            assert!((f_edge - f_vertex).abs() < 1e-8);
        }
    }

    #[test]
    fn fi_regular_formula() {
        // For r-regular: F = m·2r² = 2m·r²
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = 2.0 * m * r * r;
            assert!((forgotten_index(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    // --- reduced_second_zagreb ---

    #[test]
    fn rm2_empty() {
        let g = Graph::with_vertices(0);
        assert!((reduced_second_zagreb(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn rm2_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((reduced_second_zagreb(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn rm2_no_edges() {
        let g = Graph::with_vertices(3);
        assert!((reduced_second_zagreb(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn rm2_single_edge() {
        // (1-1)(1-1) = 0
        assert!((reduced_second_zagreb(&single_edge()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn rm2_path3() {
        // (0,1): (1-1)(2-1)=0, (1,2): (2-1)(1-1)=0 → RM₂=0
        assert!((reduced_second_zagreb(&path3()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn rm2_path4() {
        // (0,1): (0)(1)=0, (1,2): (1)(1)=1, (2,3): (1)(0)=0 → RM₂=1
        assert!((reduced_second_zagreb(&path4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rm2_k3() {
        // each: (2-1)(2-1)=1, 3 edges → 3
        assert!((reduced_second_zagreb(&k3()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn rm2_k4() {
        // each: (3-1)(3-1)=4, 6 edges → 24
        assert!((reduced_second_zagreb(&k4()).unwrap() - 24.0).abs() < 1e-10);
    }

    #[test]
    fn rm2_cycle4() {
        // each: (1)(1)=1, 4 edges → 4
        assert!((reduced_second_zagreb(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn rm2_cycle5() {
        // each: 1, 5 edges → 5
        assert!((reduced_second_zagreb(&cycle5()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn rm2_star5() {
        // center deg 4, leaf deg 1: (4-1)(1-1)=0, 4 edges → 0
        assert!((reduced_second_zagreb(&star5()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn rm2_paw() {
        // degrees [2,2,3,1]
        // (0,1): (1)(1)=1, (0,2): (1)(2)=2, (1,2): (1)(2)=2, (2,3): (2)(0)=0
        // RM₂ = 1+2+2+0 = 5
        assert!((reduced_second_zagreb(&paw()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn rm2_diamond() {
        // degrees [3,3,2,2]
        // (0,1): (2)(2)=4, (0,2): (2)(1)=2, (0,3): (2)(1)=2, (1,2): (2)(1)=2, (1,3): (2)(1)=2
        // RM₂ = 4+2+2+2+2 = 12
        assert!((reduced_second_zagreb(&diamond()).unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn rm2_nonneg_for_connected() {
        for g in &[
            single_edge(),
            path3(),
            k3(),
            k4(),
            star5(),
            paw(),
            diamond(),
        ] {
            assert!(reduced_second_zagreb(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn rm2_regular_formula() {
        // r-regular: RM₂ = m·(r-1)²
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = m * (r - 1.0) * (r - 1.0);
            assert!((reduced_second_zagreb(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    // --- modified_first_zagreb ---

    #[test]
    fn mm1_empty() {
        let g = Graph::with_vertices(0);
        assert!((modified_first_zagreb(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn mm1_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((modified_first_zagreb(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn mm1_no_edges() {
        let g = Graph::with_vertices(3);
        assert!((modified_first_zagreb(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn mm1_single_edge() {
        // 2 · 1/1² = 2
        assert!((modified_first_zagreb(&single_edge()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn mm1_path3() {
        // degrees [1,2,1]: 1/1 + 1/4 + 1/1 = 2.25
        assert!((modified_first_zagreb(&path3()).unwrap() - 2.25).abs() < 1e-10);
    }

    #[test]
    fn mm1_k3() {
        // 3 · 1/4 = 0.75
        assert!((modified_first_zagreb(&k3()).unwrap() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn mm1_k4() {
        // 4 · 1/9 = 4/9
        assert!((modified_first_zagreb(&k4()).unwrap() - 4.0 / 9.0).abs() < 1e-10);
    }

    #[test]
    fn mm1_cycle4() {
        // 4 · 1/4 = 1
        assert!((modified_first_zagreb(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mm1_cycle5() {
        // 5 · 1/4 = 1.25
        assert!((modified_first_zagreb(&cycle5()).unwrap() - 1.25).abs() < 1e-10);
    }

    #[test]
    fn mm1_star5() {
        // 1/16 + 4·(1/1) = 1/16 + 4 = 65/16
        let expected = 1.0 / 16.0 + 4.0;
        assert!((modified_first_zagreb(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mm1_paw() {
        // degrees [2,2,3,1]: 1/4 + 1/4 + 1/9 + 1/1 = 0.5 + 1/9 + 1 = 1.5 + 1/9
        let expected = 0.25 + 0.25 + 1.0 / 9.0 + 1.0;
        assert!((modified_first_zagreb(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mm1_with_isolated() {
        // 0-1 plus isolated 2: degrees [1,1,0], skip vertex 2
        // 1/1 + 1/1 = 2
        let g = Graph::from_edges(&[(0, 1)], false, Some(3)).unwrap();
        assert!((modified_first_zagreb(&g).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn mm1_regular_formula() {
        // r-regular: n · 1/r² = n/r²
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let n = f64::from(g.vcount());
            let r = g.degree(0).unwrap() as f64;
            let expected = n / (r * r);
            assert!((modified_first_zagreb(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn all_positive_for_connected() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            assert!(forgotten_index(g).unwrap() > 0.0);
            assert!(modified_first_zagreb(g).unwrap() > 0.0);
        }
    }

    #[test]
    fn fi_geq_2m() {
        // F(G) = Σ(d_u²+d_v²) ≥ 2m since d_u≥1, d_v≥1
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            let f = forgotten_index(g).unwrap();
            assert!(f >= 2.0 * g.ecount() as f64 - 1e-10);
        }
    }

    #[test]
    fn fi_relationship_to_m1_sigma() {
        // F(G) = M₁(G) + σ(G) where M₁ = Σ(d_u+d_v) = first Zagreb, σ = sigma index
        // Because (d_u²+d_v²) = (d_u+d_v)² - 2d_u·d_v, and
        // (d_u-d_v)² = d_u²+d_v² - 2d_u·d_v, actually:
        // F = Σ(d_u²+d_v²), σ = Σ(d_u-d_v)², M₁ = Σ(d_u+d_v)
        // Note: F + 2·M₂ = Σ(d_u²+d_v²+2d_u·d_v) = Σ(d_u+d_v)² not M₁
        // F = M₁ + σ is NOT correct in general.
        // But we can verify: F ≥ σ (since d_u²+d_v² ≥ (d_u-d_v)²)
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let f = forgotten_index(g).unwrap();
            let mut sigma = 0.0_f64;
            for (u, v) in g.edges() {
                if u == v {
                    continue;
                }
                let du = g.degree(u).unwrap() as f64;
                let dv = g.degree(v).unwrap() as f64;
                sigma += (du - dv) * (du - dv);
            }
            assert!(f >= sigma - 1e-8);
        }
    }
}
