//! Atom-bond connectivity variants (ALGO-TR-055).
//!
//! Extended versions of the atom-bond connectivity index.
//!
//! - **Fourth ABC index** `ABC₄(G) = Σ_{(u,v)∈E} √((ε(u)+ε(v)-2) / (ε(u)·ε(v)))`
//!   where `ε(v)` is the eccentricity of v. Introduced by Ghorbani & Hosseinzadeh (2010).
//! - **Fifth geometric-arithmetic index** `GA₅(G) = Σ_{(u,v)∈E} 2√(ε(u)·ε(v)) / (ε(u)+ε(v))`
//!   Uses eccentricities instead of degrees. Introduced by Graovac et al. (2011).
//! - **Degree-sum index** `DS(G) = Σ_{(u,v)∈E} √(d(u)+d(v))`
//!   Simple edge-weight using the square root of the degree sum.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

fn eccentricities(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount() as usize;
    let mut ecc = vec![0_u32; n];

    for s in 0..n {
        let mut dist = vec![u32::MAX; n];
        dist[s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s as u32);
        while let Some(u) = queue.pop_front() {
            let d_u = dist[u as usize];
            for nb in graph.neighbors(u)? {
                let idx = nb as usize;
                if dist[idx] == u32::MAX {
                    dist[idx] = d_u + 1;
                    queue.push_back(nb);
                }
            }
        }
        let mut max_d = 0_u32;
        for &d in &dist {
            if d != u32::MAX && d > max_d {
                max_d = d;
            }
        }
        ecc[s] = max_d;
    }

    Ok(ecc)
}

/// Compute the fourth atom-bond connectivity index.
///
/// `ABC₄(G) = Σ_{(u,v)∈E} √((ε(u)+ε(v)-2) / (ε(u)·ε(v)))`
///
/// where `ε(v)` is the eccentricity of vertex v. Self-loops and
/// edges where `ε(u)·ε(v) = 0` are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, fourth_abc_index};
///
/// // Path 0-1-2: eccentricities [2,1,2]
/// // (0,1): √((2+1-2)/(2·1)) = √(1/2) = 1/√2
/// // (1,2): √((1+2-2)/(1·2)) = √(1/2) = 1/√2
/// // ABC₄ = 2/√2 = √2
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((fourth_abc_index(&g).unwrap() - std::f64::consts::SQRT_2).abs() < 1e-10);
/// ```
pub fn fourth_abc_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let ecc = eccentricities(graph)?;
    let mut abc4 = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let eu = f64::from(ecc[u as usize]);
        let ev = f64::from(ecc[v as usize]);
        let prod = eu * ev;
        if prod <= 0.0 {
            continue;
        }
        let numer = eu + ev - 2.0;
        if numer < 0.0 {
            continue;
        }
        abc4 += (numer / prod).sqrt();
    }

    Ok(abc4)
}

/// Compute the fifth geometric-arithmetic index.
///
/// `GA₅(G) = Σ_{(u,v)∈E} 2√(ε(u)·ε(v)) / (ε(u)+ε(v))`
///
/// where `ε(v)` is the eccentricity. Self-loops and edges where
/// `ε(u)+ε(v) = 0` are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, fifth_ga_index};
///
/// // K_3: all eccentricities = 1
/// // Each edge: 2√(1·1)/(1+1) = 1, 3 edges → GA₅ = 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((fifth_ga_index(&g).unwrap() - 3.0).abs() < 1e-10);
/// ```
pub fn fifth_ga_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let ecc = eccentricities(graph)?;
    let mut ga5 = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let eu = f64::from(ecc[u as usize]);
        let ev = f64::from(ecc[v as usize]);
        let denom = eu + ev;
        if denom <= 0.0 {
            continue;
        }
        ga5 += 2.0 * (eu * ev).sqrt() / denom;
    }

    Ok(ga5)
}

/// Compute the degree-sum index.
///
/// `DS(G) = Σ_{(u,v)∈E} √(d(u) + d(v))`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_sum_index};
///
/// // K_3: each edge √(2+2) = 2, 3 edges → DS = 6
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((degree_sum_index(&g).unwrap() - 6.0).abs() < 1e-10);
/// ```
pub fn degree_sum_index(graph: &Graph) -> IgraphResult<f64> {
    let mut ds = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        ds += (du + dv).sqrt();
    }

    Ok(ds)
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

    // --- eccentricities helper ---

    #[test]
    fn ecc_path3() {
        let ecc = eccentricities(&path3()).unwrap();
        assert_eq!(ecc, vec![2, 1, 2]);
    }

    #[test]
    fn ecc_k3() {
        let ecc = eccentricities(&k3()).unwrap();
        assert_eq!(ecc, vec![1, 1, 1]);
    }

    #[test]
    fn ecc_k4() {
        let ecc = eccentricities(&k4()).unwrap();
        assert_eq!(ecc, vec![1, 1, 1, 1]);
    }

    #[test]
    fn ecc_cycle4() {
        let ecc = eccentricities(&cycle4()).unwrap();
        assert_eq!(ecc, vec![2, 2, 2, 2]);
    }

    #[test]
    fn ecc_star5() {
        let ecc = eccentricities(&star5()).unwrap();
        assert_eq!(ecc[0], 1);
        for i in 1..5 {
            assert_eq!(ecc[i], 2);
        }
    }

    // --- fourth_abc_index ---

    #[test]
    fn abc4_empty() {
        let g = Graph::with_vertices(0);
        assert!((fourth_abc_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn abc4_single_edge() {
        // eccentricities [1,1], (1+1-2)/(1·1) = 0 → √0 = 0
        assert!((fourth_abc_index(&single_edge()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn abc4_path3() {
        // ecc [2,1,2]
        // (0,1): √((2+1-2)/(2·1)) = √(1/2)
        // (1,2): √((1+2-2)/(1·2)) = √(1/2)
        // ABC₄ = 2·√(1/2) = √2
        let expected = std::f64::consts::SQRT_2;
        assert!((fourth_abc_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn abc4_k3() {
        // ecc [1,1,1]
        // each edge: √((1+1-2)/(1·1)) = 0
        assert!((fourth_abc_index(&k3()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn abc4_k4() {
        // ecc [1,1,1,1], same as K_3
        assert!((fourth_abc_index(&k4()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn abc4_cycle4() {
        // ecc [2,2,2,2]
        // each edge: √((2+2-2)/(2·2)) = √(2/4) = √(1/2)
        // 4 edges → 4·√(1/2) = 4/√2 = 2√2
        let expected = 2.0 * std::f64::consts::SQRT_2;
        assert!((fourth_abc_index(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn abc4_cycle5() {
        // ecc [2,2,2,2,2]
        // each edge: √((2+2-2)/(2·2)) = √(1/2)
        // 5 edges → 5/√2
        let expected = 5.0 / std::f64::consts::SQRT_2;
        assert!((fourth_abc_index(&cycle5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn abc4_star5() {
        // ecc [1,2,2,2,2]
        // (0,leaf): √((1+2-2)/(1·2)) = √(1/2)
        // 4 edges → 4/√2 = 2√2
        let expected = 2.0 * std::f64::consts::SQRT_2;
        assert!((fourth_abc_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn abc4_paw() {
        // ecc: v0→max(1,1,2)=2, v1→max(1,1,2)=2, v2→max(1,1,1)=1, v3→max(2,2,1)=2
        let ecc = eccentricities(&paw()).unwrap();
        assert_eq!(ecc, vec![2, 2, 1, 2]);
        // (0,1): √((2+2-2)/(2·2)) = √(1/2)
        // (0,2): √((2+1-2)/(2·1)) = √(1/2)
        // (1,2): √((2+1-2)/(2·1)) = √(1/2)
        // (2,3): √((1+2-2)/(1·2)) = √(1/2)
        // ABC₄ = 4/√2 = 2√2
        let expected = 2.0 * std::f64::consts::SQRT_2;
        assert!((fourth_abc_index(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- fifth_ga_index ---

    #[test]
    fn ga5_empty() {
        let g = Graph::with_vertices(0);
        assert!((fifth_ga_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn ga5_single_edge() {
        // ecc [1,1]: 2√1/(1+1) = 1
        assert!((fifth_ga_index(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ga5_path3() {
        // ecc [2,1,2]
        // (0,1): 2√(2·1)/3 = 2√2/3
        // (1,2): 2√(1·2)/3 = 2√2/3
        // GA₅ = 4√2/3
        let expected = 4.0 * std::f64::consts::SQRT_2 / 3.0;
        assert!((fifth_ga_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ga5_k3() {
        // ecc [1,1,1]: each edge 2·1/2 = 1, 3 edges → 3
        assert!((fifth_ga_index(&k3()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn ga5_k4() {
        // ecc [1,1,1,1]: each edge 1, 6 edges → 6
        assert!((fifth_ga_index(&k4()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn ga5_cycle4() {
        // ecc [2,2,2,2]: each edge 2·2/4 = 1, 4 edges → 4
        assert!((fifth_ga_index(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn ga5_cycle5() {
        // ecc [2,2,2,2,2]: each edge 1, 5 edges → 5
        assert!((fifth_ga_index(&cycle5()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn ga5_star5() {
        // ecc [1,2,2,2,2]
        // (0,leaf): 2√(1·2)/(1+2) = 2√2/3
        // 4 edges → 8√2/3
        let expected = 8.0 * std::f64::consts::SQRT_2 / 3.0;
        assert!((fifth_ga_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ga5_equal_ecc_gives_m() {
        // When all ecc equal: each edge contributes 1 → GA₅ = m
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            assert!((fifth_ga_index(g).unwrap() - m).abs() < 1e-10);
        }
    }

    // --- degree_sum_index ---

    #[test]
    fn ds_empty() {
        let g = Graph::with_vertices(0);
        assert!((degree_sum_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn ds_single_edge() {
        // √(1+1) = √2
        let expected = std::f64::consts::SQRT_2;
        assert!((degree_sum_index(&single_edge()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ds_path3() {
        // (0,1): √(1+2)=√3, (1,2): √(2+1)=√3
        // DS = 2√3
        let expected = 2.0 * 3.0_f64.sqrt();
        assert!((degree_sum_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ds_k3() {
        // each edge √(2+2)=2, 3 edges → 6
        assert!((degree_sum_index(&k3()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn ds_k4() {
        // each edge √(3+3)=√6, 6 edges → 6√6
        let expected = 6.0 * 6.0_f64.sqrt();
        assert!((degree_sum_index(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ds_cycle4() {
        // each edge √4=2, 4 edges → 8
        assert!((degree_sum_index(&cycle4()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn ds_star5() {
        // each edge √(4+1)=√5, 4 edges → 4√5
        let expected = 4.0 * 5.0_f64.sqrt();
        assert!((degree_sum_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ds_paw() {
        // degrees [2,2,3,1]
        // (0,1):√4=2, (0,2):√5, (1,2):√5, (2,3):√4=2
        // DS = 4 + 2√5
        let expected = 4.0 + 2.0 * 5.0_f64.sqrt();
        assert!((degree_sum_index(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ds_regular_formula() {
        // r-regular: DS = m·√(2r)
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = m * (2.0 * r).sqrt();
            assert!((degree_sum_index(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn all_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(fourth_abc_index(g).unwrap() >= -1e-10);
            assert!(fifth_ga_index(g).unwrap() >= -1e-10);
            assert!(degree_sum_index(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn ga5_leq_m() {
        // GA₅ ≤ m (AM-GM inequality: 2√(ab)/(a+b) ≤ 1)
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let m = g.ecount() as f64;
            assert!(fifth_ga_index(g).unwrap() <= m + 1e-10);
        }
    }
}
