//! Augmented Zagreb index and atom-bond sum-connectivity (ALGO-TR-046).
//!
//! - **Augmented Zagreb index** `AZI(G) = Σ_{(u,v)∈E} (d_u·d_v / (d_u+d_v-2))³`
//!   Introduced by Furtula et al. (2010); a powerful predictor of
//!   physico-chemical properties. Edges with `d_u + d_v ≤ 2` (which
//!   requires isolated endpoints that shouldn't have edges) are skipped.
//! - **Atom-bond sum-connectivity** `ABS(G) = Σ_{(u,v)∈E} √((d_u+d_v-2)/(d_u+d_v))`
//!   A variant of the ABC index using degree sums instead of products.
//!   Edges with `d_u + d_v ≤ 2` are skipped (degenerate).
//! - **Geometric-arithmetic index** `GA(G) = Σ_{(u,v)∈E} 2√(d_u·d_v)/(d_u+d_v)`
//!   Ratio of geometric to arithmetic mean of endpoint degrees.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the augmented Zagreb index.
///
/// `AZI(G) = Σ_{(u,v)∈E} (d_u · d_v / (d_u + d_v - 2))³`
///
/// Self-loops and edges where `d_u + d_v ≤ 2` are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, augmented_zagreb_index};
///
/// // Path 0-1-2: degrees [1,2,1]
/// // edge(0,1): (1·2/(1+2-2))³ = 2³ = 8
/// // edge(1,2): same = 8
/// // AZI = 16
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((augmented_zagreb_index(&g).unwrap() - 16.0).abs() < 1e-10);
/// ```
pub fn augmented_zagreb_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let mut azi = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let denom = du + dv - 2.0;
        if denom <= 0.0 {
            continue;
        }
        let frac = du * dv / denom;
        azi += frac * frac * frac;
    }

    Ok(azi)
}

/// Compute the atom-bond sum-connectivity index.
///
/// `ABS(G) = Σ_{(u,v)∈E} √((d_u + d_v - 2) / (d_u + d_v))`
///
/// Self-loops and degenerate edges are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, atom_bond_sum_connectivity};
///
/// // Path 0-1-2: degrees [1,2,1]
/// // edge(0,1): √((3-2)/3) = √(1/3)
/// // edge(1,2): same
/// // ABS = 2/√3
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((atom_bond_sum_connectivity(&g).unwrap() - 2.0/3.0_f64.sqrt()).abs() < 1e-10);
/// ```
pub fn atom_bond_sum_connectivity(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let mut abs_val = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let sum_d = du + dv;
        if sum_d <= 2.0 {
            continue;
        }
        abs_val += ((sum_d - 2.0) / sum_d).sqrt();
    }

    Ok(abs_val)
}

/// Compute the geometric-arithmetic index.
///
/// `GA(G) = Σ_{(u,v)∈E} 2√(d_u · d_v) / (d_u + d_v)`
///
/// For each edge, this is the ratio of the geometric mean to the
/// arithmetic mean of the endpoint degrees.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, geometric_arithmetic_index};
///
/// // K_3: all degrees 2 → each term = 2√4/4 = 1
/// // GA = 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((geometric_arithmetic_index(&g).unwrap() - 3.0).abs() < 1e-10);
/// ```
pub fn geometric_arithmetic_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let mut ga = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let sum_d = du + dv;
        if sum_d <= 0.0 {
            continue;
        }
        ga += 2.0 * (du * dv).sqrt() / sum_d;
    }

    Ok(ga)
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

    // --- augmented_zagreb_index ---

    #[test]
    fn azi_empty() {
        let g = Graph::with_vertices(0);
        assert!((augmented_zagreb_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn azi_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((augmented_zagreb_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn azi_no_edges() {
        let g = Graph::with_vertices(3);
        assert!((augmented_zagreb_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn azi_single_edge() {
        // d_u=d_v=1, denom=0 → skipped → AZI=0
        assert!((augmented_zagreb_index(&single_edge()).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn azi_path3() {
        // (0,1): (1·2/(1+2-2))³ = (2/1)³ = 8
        // (1,2): same = 8
        // AZI = 16
        assert!((augmented_zagreb_index(&path3()).unwrap() - 16.0).abs() < 1e-10);
    }

    #[test]
    fn azi_path4() {
        // (0,1): (1·2/1)³ = 8
        // (1,2): (2·2/2)³ = 8
        // (2,3): (2·1/1)³ = 8
        // AZI = 24
        assert!((augmented_zagreb_index(&path4()).unwrap() - 24.0).abs() < 1e-10);
    }

    #[test]
    fn azi_k3() {
        // all degrees 2, each edge: (2·2/(2+2-2))³ = (4/2)³ = 8
        // AZI = 3·8 = 24
        assert!((augmented_zagreb_index(&k3()).unwrap() - 24.0).abs() < 1e-10);
    }

    #[test]
    fn azi_k4() {
        // all degrees 3, each edge: (3·3/(3+3-2))³ = (9/4)³ = 729/64
        // AZI = 6·729/64 = 4374/64 = 2187/32
        let expected = 6.0 * (9.0_f64 / 4.0).powi(3);
        assert!((augmented_zagreb_index(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn azi_cycle4() {
        // all degrees 2: same as K3 per edge but 4 edges
        // each: (4/2)³ = 8, AZI = 32
        assert!((augmented_zagreb_index(&cycle4()).unwrap() - 32.0).abs() < 1e-10);
    }

    #[test]
    fn azi_cycle5() {
        // all degrees 2, 5 edges: 5·8 = 40
        assert!((augmented_zagreb_index(&cycle5()).unwrap() - 40.0).abs() < 1e-10);
    }

    #[test]
    fn azi_star5() {
        // center deg=4, leaf deg=1
        // (4·1/(4+1-2))³ = (4/3)³ = 64/27
        // AZI = 4 · 64/27 = 256/27
        let expected = 4.0 * (4.0_f64 / 3.0).powi(3);
        assert!((augmented_zagreb_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn azi_positive_for_path() {
        for g in &[path3(), path4(), k3(), k4(), cycle4(), star5()] {
            assert!(augmented_zagreb_index(g).unwrap() > 0.0);
        }
    }

    #[test]
    fn azi_regular_formula() {
        // r-regular: AZI = m · (r²/(2r-2))³ = m · (r/2·(r/(r-1)))³
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = m * (r * r / (2.0 * r - 2.0)).powi(3);
            assert!((augmented_zagreb_index(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    // --- atom_bond_sum_connectivity ---

    #[test]
    fn abs_empty() {
        let g = Graph::with_vertices(0);
        assert!((atom_bond_sum_connectivity(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn abs_single_edge() {
        // d_u+d_v=2, skipped → 0
        assert!((atom_bond_sum_connectivity(&single_edge()).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn abs_path3() {
        // (0,1): √((3-2)/3) = √(1/3) = 1/√3
        // (1,2): same
        // ABS = 2/√3
        let expected = 2.0 / 3.0_f64.sqrt();
        assert!((atom_bond_sum_connectivity(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn abs_k3() {
        // each: √((4-2)/4) = √(1/2) = 1/√2
        // ABS = 3/√2
        let expected = 3.0 / 2.0_f64.sqrt();
        assert!((atom_bond_sum_connectivity(&k3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn abs_k4() {
        // each: √((6-2)/6) = √(2/3)
        // ABS = 6·√(2/3)
        let expected = 6.0 * (2.0_f64 / 3.0).sqrt();
        assert!((atom_bond_sum_connectivity(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn abs_cycle4() {
        // each: √(2/4) = √(1/2), 4 edges
        // ABS = 4/√2 = 2√2
        let expected = 4.0 / 2.0_f64.sqrt();
        assert!((atom_bond_sum_connectivity(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn abs_star5() {
        // center=4, leaf=1: (4+1-2)/(4+1) = 3/5
        // √(3/5), 4 edges
        let expected = 4.0 * (3.0_f64 / 5.0).sqrt();
        assert!((atom_bond_sum_connectivity(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn abs_leq_m() {
        // ABS(G) <= m since √((d_u+d_v-2)/(d_u+d_v)) < 1
        for g in &[path3(), k3(), k4(), cycle4(), star5()] {
            let abs_val = atom_bond_sum_connectivity(g).unwrap();
            assert!(abs_val < g.ecount() as f64 + 1e-10);
        }
    }

    // --- geometric_arithmetic_index ---

    #[test]
    fn ga_empty() {
        let g = Graph::with_vertices(0);
        assert!((geometric_arithmetic_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn ga_single_edge() {
        // 2√(1·1)/(1+1) = 2/2 = 1
        assert!((geometric_arithmetic_index(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ga_path3() {
        // (0,1): 2√2/3, (1,2): same → GA = 4√2/3
        let expected = 4.0 * 2.0_f64.sqrt() / 3.0;
        assert!((geometric_arithmetic_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ga_k3() {
        // all equal degrees → each term = 1. GA = 3
        assert!((geometric_arithmetic_index(&k3()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn ga_k4() {
        // all equal degrees → GA = 6
        assert!((geometric_arithmetic_index(&k4()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn ga_cycle4() {
        // all equal degrees → GA = 4
        assert!((geometric_arithmetic_index(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn ga_star5() {
        // 2√(4·1)/5 = 4/5, 4 edges → GA = 16/5
        let expected = 4.0 * 4.0 / 5.0;
        assert!((geometric_arithmetic_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ga_leq_m() {
        // GA(G) <= m by AM-GM inequality (geometric mean ≤ arithmetic mean)
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            let ga = geometric_arithmetic_index(g).unwrap();
            assert!(ga <= g.ecount() as f64 + 1e-10);
        }
    }

    #[test]
    fn ga_equals_m_for_regular() {
        // For r-regular graphs: each term = 2√(r²)/(2r) = 2r/(2r) = 1
        // GA = m
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let ga = geometric_arithmetic_index(g).unwrap();
            assert!((ga - g.ecount() as f64).abs() < 1e-10);
        }
    }

    #[test]
    fn ga_path4() {
        // (0,1): 2√(1·2)/3 = 2√2/3
        // (1,2): 2√(2·2)/4 = 2·2/4 = 1
        // (2,3): 2√(2·1)/3 = 2√2/3
        // GA = 4√2/3 + 1
        let expected = 4.0 * 2.0_f64.sqrt() / 3.0 + 1.0;
        assert!((geometric_arithmetic_index(&path4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ga_diamond() {
        // K4 minus (2,3): edges 0-1,0-2,0-3,1-2,1-3
        // deg=[3,3,2,2]
        // (0,1): 2√9/6=1, (0,2): 2√6/5, (0,3): 2√6/5
        // (1,2): 2√6/5, (1,3): 2√6/5
        // GA = 1 + 4·(2√6/5) = 1 + 8√6/5
        let g =
            Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3)], false, Some(4)).unwrap();
        let expected = 1.0 + 8.0 * 6.0_f64.sqrt() / 5.0;
        assert!((geometric_arithmetic_index(&g).unwrap() - expected).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn all_positive_for_connected() {
        for g in &[path3(), k3(), k4(), cycle4(), star5()] {
            assert!(augmented_zagreb_index(g).unwrap() > 0.0);
            assert!(atom_bond_sum_connectivity(g).unwrap() > 0.0);
            assert!(geometric_arithmetic_index(g).unwrap() > 0.0);
        }
    }

    #[test]
    fn with_isolated_vertex() {
        // 0-1 plus isolated 2
        let g = Graph::from_edges(&[(0, 1)], false, Some(3)).unwrap();
        // single edge: d_u+d_v-2 = 0, so AZI and ABS skip it
        assert!((augmented_zagreb_index(&g).unwrap() - 0.0).abs() < 1e-10);
        assert!((atom_bond_sum_connectivity(&g).unwrap() - 0.0).abs() < 1e-10);
        // GA: 2√1/2 = 1
        assert!((geometric_arithmetic_index(&g).unwrap() - 1.0).abs() < 1e-10);
    }
}
