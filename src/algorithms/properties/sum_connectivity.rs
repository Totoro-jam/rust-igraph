//! Sum-connectivity, inverse sum indeg, and symmetric division deg (ALGO-TR-047).
//!
//! - **Sum-connectivity index** `SCI(G) = Σ_{(u,v)∈E} 1/√(d_u + d_v)`
//!   Introduced by Zhou & Trinajstić (2009). Related to the Randić
//!   index but uses degree sums instead of products.
//! - **Inverse sum indeg index** `ISI(G) = Σ_{(u,v)∈E} (d_u·d_v)/(d_u+d_v)`
//!   The inverse of the sum of reciprocals of endpoint degrees, a
//!   descriptor with strong predictive power for total surface area.
//! - **Symmetric division deg index** `SDD(G) = Σ_{(u,v)∈E} (d_u²+d_v²)/(d_u·d_v)`
//!   Equivalently `d_u/d_v + d_v/d_u` per edge. Introduced by
//!   Vukičević & Gašperov (2010).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the sum-connectivity index.
///
/// `SCI(G) = Σ_{(u,v)∈E} 1/√(d_u + d_v)`
///
/// Self-loops are skipped. Edges where both endpoints have degree 0
/// (impossible in practice) are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, sum_connectivity_index};
///
/// // Path 0-1-2: degrees [1,2,1]
/// // edge(0,1): 1/√3, edge(1,2): 1/√3
/// // SCI = 2/√3
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((sum_connectivity_index(&g).unwrap() - 2.0/3.0_f64.sqrt()).abs() < 1e-10);
/// ```
pub fn sum_connectivity_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let mut sci = 0.0_f64;

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
        sci += 1.0 / sum_d.sqrt();
    }

    Ok(sci)
}

/// Compute the inverse sum indeg index.
///
/// `ISI(G) = Σ_{(u,v)∈E} (d_u · d_v) / (d_u + d_v)`
///
/// This equals `½ · Σ 1/(1/d_u + 1/d_v)`, the harmonic mean of
/// endpoint degrees summed over edges, divided by 2.
///
/// Self-loops are skipped. Edges with `d_u + d_v = 0` are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, inverse_sum_indeg_index};
///
/// // K_3: all degrees 2
/// // each edge: (2·2)/(2+2) = 1, 3 edges → ISI = 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((inverse_sum_indeg_index(&g).unwrap() - 3.0).abs() < 1e-10);
/// ```
pub fn inverse_sum_indeg_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let mut isi = 0.0_f64;

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
        isi += du * dv / sum_d;
    }

    Ok(isi)
}

/// Compute the symmetric division deg index.
///
/// `SDD(G) = Σ_{(u,v)∈E} (d_u² + d_v²) / (d_u · d_v)`
///
/// Equivalently, each edge contributes `d_u/d_v + d_v/d_u`.
/// For regular graphs every edge contributes 2, so `SDD = 2m`.
///
/// Self-loops are skipped. Edges with `d_u · d_v = 0` are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, symmetric_division_deg_index};
///
/// // K_3: all degrees 2 → each edge contributes 2, SDD = 6
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((symmetric_division_deg_index(&g).unwrap() - 6.0).abs() < 1e-10);
/// ```
pub fn symmetric_division_deg_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let mut sdd = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let prod = du * dv;
        if prod <= 0.0 {
            continue;
        }
        sdd += (du * du + dv * dv) / prod;
    }

    Ok(sdd)
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

    // --- sum_connectivity_index ---

    #[test]
    fn sci_empty() {
        let g = Graph::with_vertices(0);
        assert!((sum_connectivity_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn sci_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((sum_connectivity_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn sci_no_edges() {
        let g = Graph::with_vertices(3);
        assert!((sum_connectivity_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn sci_single_edge() {
        // d_u=d_v=1: 1/√2
        let expected = 1.0 / 2.0_f64.sqrt();
        assert!((sum_connectivity_index(&single_edge()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn sci_path3() {
        // (0,1): 1/√3, (1,2): 1/√3 → 2/√3
        let expected = 2.0 / 3.0_f64.sqrt();
        assert!((sum_connectivity_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn sci_path4() {
        // (0,1): 1/√3, (1,2): 1/√4=1/2, (2,3): 1/√3
        let expected = 2.0 / 3.0_f64.sqrt() + 0.5;
        assert!((sum_connectivity_index(&path4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn sci_k3() {
        // all degrees 2: each 1/√4=1/2, 3 edges → 3/2
        assert!((sum_connectivity_index(&k3()).unwrap() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn sci_k4() {
        // all degrees 3: each 1/√6, 6 edges → 6/√6 = √6
        let expected = 6.0 / 6.0_f64.sqrt();
        assert!((sum_connectivity_index(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn sci_cycle4() {
        // all degrees 2: each 1/2, 4 edges → 2
        assert!((sum_connectivity_index(&cycle4()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn sci_cycle5() {
        // all degrees 2: each 1/2, 5 edges → 5/2
        assert!((sum_connectivity_index(&cycle5()).unwrap() - 2.5).abs() < 1e-10);
    }

    #[test]
    fn sci_star5() {
        // center=4, leaf=1: 1/√5, 4 edges → 4/√5
        let expected = 4.0 / 5.0_f64.sqrt();
        assert!((sum_connectivity_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn sci_regular_formula() {
        // r-regular: SCI = m/√(2r)
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = m / (2.0 * r).sqrt();
            assert!((sum_connectivity_index(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    #[test]
    fn sci_positive() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            assert!(sum_connectivity_index(g).unwrap() > 0.0);
        }
    }

    // --- inverse_sum_indeg_index ---

    #[test]
    fn isi_empty() {
        let g = Graph::with_vertices(0);
        assert!((inverse_sum_indeg_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn isi_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((inverse_sum_indeg_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn isi_no_edges() {
        let g = Graph::with_vertices(3);
        assert!((inverse_sum_indeg_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn isi_single_edge() {
        // (1·1)/(1+1) = 1/2
        assert!((inverse_sum_indeg_index(&single_edge()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn isi_path3() {
        // (0,1): (1·2)/3 = 2/3, (1,2): same → 4/3
        let expected = 4.0 / 3.0;
        assert!((inverse_sum_indeg_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn isi_path4() {
        // (0,1): (1·2)/3=2/3, (1,2): (2·2)/4=1, (2,3): (2·1)/3=2/3
        // ISI = 2/3 + 1 + 2/3 = 7/3
        let expected = 7.0 / 3.0;
        assert!((inverse_sum_indeg_index(&path4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn isi_k3() {
        // all: (2·2)/4=1, 3 edges → 3
        assert!((inverse_sum_indeg_index(&k3()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn isi_k4() {
        // all: (3·3)/6=3/2, 6 edges → 9
        assert!((inverse_sum_indeg_index(&k4()).unwrap() - 9.0).abs() < 1e-10);
    }

    #[test]
    fn isi_cycle4() {
        // all: (2·2)/4=1, 4 edges → 4
        assert!((inverse_sum_indeg_index(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn isi_cycle5() {
        // all: (2·2)/4=1, 5 edges → 5
        assert!((inverse_sum_indeg_index(&cycle5()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn isi_star5() {
        // (4·1)/5=4/5, 4 edges → 16/5
        let expected = 16.0 / 5.0;
        assert!((inverse_sum_indeg_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn isi_regular_formula() {
        // r-regular: ISI = m · r²/(2r) = m·r/2
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = m * r / 2.0;
            assert!((inverse_sum_indeg_index(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    #[test]
    fn isi_positive() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            assert!(inverse_sum_indeg_index(g).unwrap() > 0.0);
        }
    }

    #[test]
    fn isi_diamond() {
        // K4 minus (2,3): edges 0-1,0-2,0-3,1-2,1-3
        // deg=[3,3,2,2]
        // (0,1): 9/6=3/2, (0,2): 6/5, (0,3): 6/5
        // (1,2): 6/5, (1,3): 6/5
        // ISI = 3/2 + 4·(6/5) = 3/2 + 24/5 = 15/10+48/10 = 63/10
        let g =
            Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3)], false, Some(4)).unwrap();
        let expected = 63.0 / 10.0;
        assert!((inverse_sum_indeg_index(&g).unwrap() - expected).abs() < 1e-10);
    }

    // --- symmetric_division_deg_index ---

    #[test]
    fn sdd_empty() {
        let g = Graph::with_vertices(0);
        assert!((symmetric_division_deg_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn sdd_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((symmetric_division_deg_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn sdd_no_edges() {
        let g = Graph::with_vertices(3);
        assert!((symmetric_division_deg_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn sdd_single_edge() {
        // (1²+1²)/(1·1) = 2
        assert!((symmetric_division_deg_index(&single_edge()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn sdd_path3() {
        // (0,1): (1+4)/(1·2) = 5/2
        // (1,2): (4+1)/(2·1) = 5/2
        // SDD = 5
        assert!((symmetric_division_deg_index(&path3()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn sdd_path4() {
        // (0,1): (1+4)/2=5/2, (1,2): (4+4)/4=2, (2,3): (4+1)/2=5/2
        // SDD = 5/2+2+5/2 = 7
        assert!((symmetric_division_deg_index(&path4()).unwrap() - 7.0).abs() < 1e-10);
    }

    #[test]
    fn sdd_k3() {
        // all: (4+4)/4=2, 3 edges → 6
        assert!((symmetric_division_deg_index(&k3()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn sdd_k4() {
        // all: (9+9)/9=2, 6 edges → 12
        assert!((symmetric_division_deg_index(&k4()).unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn sdd_cycle4() {
        // all: 2, 4 edges → 8
        assert!((symmetric_division_deg_index(&cycle4()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn sdd_cycle5() {
        // all: 2, 5 edges → 10
        assert!((symmetric_division_deg_index(&cycle5()).unwrap() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn sdd_star5() {
        // center=4, leaf=1: (16+1)/(4·1)=17/4, 4 edges → 17
        assert!((symmetric_division_deg_index(&star5()).unwrap() - 17.0).abs() < 1e-10);
    }

    #[test]
    fn sdd_regular_equals_2m() {
        // r-regular: each edge contributes r/r + r/r = 2, so SDD = 2m
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let expected = 2.0 * g.ecount() as f64;
            assert!((symmetric_division_deg_index(g).unwrap() - expected).abs() < 1e-10);
        }
    }

    #[test]
    fn sdd_geq_2m() {
        // SDD(G) >= 2m by AM-GM: d_u/d_v + d_v/d_u >= 2
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            let sdd = symmetric_division_deg_index(g).unwrap();
            assert!(sdd >= 2.0 * g.ecount() as f64 - 1e-10);
        }
    }

    #[test]
    fn sdd_diamond() {
        // K4 minus (2,3): edges 0-1,0-2,0-3,1-2,1-3
        // deg=[3,3,2,2]
        // (0,1): (9+9)/9=2
        // (0,2): (9+4)/6=13/6
        // (0,3): 13/6
        // (1,2): 13/6
        // (1,3): 13/6
        // SDD = 2 + 4·(13/6) = 2 + 52/6 = 2 + 26/3 = 32/3
        let g =
            Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3)], false, Some(4)).unwrap();
        let expected = 32.0 / 3.0;
        assert!((symmetric_division_deg_index(&g).unwrap() - expected).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn all_positive_for_connected() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            assert!(sum_connectivity_index(g).unwrap() > 0.0);
            assert!(inverse_sum_indeg_index(g).unwrap() > 0.0);
            assert!(symmetric_division_deg_index(g).unwrap() > 0.0);
        }
    }

    #[test]
    fn with_isolated_vertex() {
        let g = Graph::from_edges(&[(0, 1)], false, Some(3)).unwrap();
        // single edge with isolated: d_u=d_v=1
        let sci = sum_connectivity_index(&g).unwrap();
        assert!((sci - 1.0 / 2.0_f64.sqrt()).abs() < 1e-10);

        let isi = inverse_sum_indeg_index(&g).unwrap();
        assert!((isi - 0.5).abs() < 1e-10);

        let sdd = symmetric_division_deg_index(&g).unwrap();
        assert!((sdd - 2.0).abs() < 1e-10);
    }

    #[test]
    fn isi_leq_second_zagreb_over_2() {
        // ISI(G) = Σ d_u·d_v/(d_u+d_v) <= Σ d_u·d_v / 2 = M₂/2
        // since d_u+d_v >= 2 for connected endpoints
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            let isi = inverse_sum_indeg_index(g).unwrap();
            let mut m2 = 0.0_f64;
            for (u, v) in g.edges() {
                if u == v {
                    continue;
                }
                let du = g.degree(u).unwrap() as f64;
                let dv = g.degree(v).unwrap() as f64;
                m2 += du * dv;
            }
            assert!(isi <= m2 / 2.0 + 1e-10);
        }
    }
}
