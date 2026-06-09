//! Graph irregularity indices (ALGO-TR-048).
//!
//! - **Albertson index** `ALB(G) = Σ_{(u,v)∈E} |d_u − d_v|`
//!   The sum of absolute degree differences over all edges. Introduced by
//!   Albertson (1997). Zero iff the graph is regular.
//! - **Sigma index** `σ(G) = Σ_{(u,v)∈E} (d_u − d_v)²`
//!   The Gutman irregularity index. Sums the squared degree differences.
//!   Always ≥ ALB(G)² / m. Zero iff regular.
//! - **Total irregularity** `irr_t(G) = ½ Σ_{u,v∈V} |d_u − d_v|`
//!   The sum over all vertex pairs, not just edges. Introduced by
//!   Abdo, Brandt & Dimitrov (2014). Zero iff regular.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the Albertson index (edge irregularity).
///
/// `ALB(G) = Σ_{(u,v)∈E} |d_u − d_v|`
///
/// Self-loops contribute 0 (both endpoints have the same vertex).
/// For regular graphs the result is 0.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, albertson_index};
///
/// // Star S_4 (center degree 4, leaves degree 1): |4-1| × 4 = 12
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert!((albertson_index(&g).unwrap() - 12.0).abs() < 1e-10);
/// ```
pub fn albertson_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let mut alb = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        alb += (du - dv).abs();
    }

    Ok(alb)
}

/// Compute the sigma index (Gutman irregularity).
///
/// `σ(G) = Σ_{(u,v)∈E} (d_u − d_v)²`
///
/// Self-loops contribute 0.  For regular graphs the result is 0.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, sigma_index};
///
/// // Path 0-1-2: degrees [1,2,1]
/// // edge(0,1): (1-2)²=1, edge(1,2): (2-1)²=1
/// // σ = 2
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((sigma_index(&g).unwrap() - 2.0).abs() < 1e-10);
/// ```
pub fn sigma_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let mut sigma = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let diff = du - dv;
        sigma += diff * diff;
    }

    Ok(sigma)
}

/// Compute the total irregularity.
///
/// `irr_t(G) = ½ Σ_{u≠v} |d_u − d_v|`
///
/// Sums over *all* unordered vertex pairs, not just edges. For
/// regular graphs the result is 0.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, total_irregularity};
///
/// // Path 0-1-2: degrees [1,2,1]
/// // pairs: (0,1)→|1-2|=1, (0,2)→|1-1|=0, (1,2)→|2-1|=1
/// // irr_t = ½(1+0+1) = 1
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((total_irregularity(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn total_irregularity(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n < 2 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n as usize);
    for v in 0..n {
        degrees.push(graph.degree(v)? as f64);
    }

    let mut total = 0.0_f64;
    for i in 0..degrees.len() {
        for j in (i + 1)..degrees.len() {
            total += (degrees[i] - degrees[j]).abs();
        }
    }

    Ok(total / 2.0)
}

/// Compute the variance of the degree sequence.
///
/// `Var(G) = (1/n) Σ_{v∈V} (d_v − d̄)²`
///
/// Where `d̄ = 2m/n` is the mean degree. Zero iff regular.
/// This is a normalised irregularity measure.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_variance};
///
/// // K_3: all degrees 2 → Var = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((degree_variance(&g).unwrap()).abs() < 1e-10);
/// ```
pub fn degree_variance(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let nf = f64::from(n);
    let m = graph.ecount() as f64;
    let mean_deg = 2.0 * m / nf;

    let mut var = 0.0_f64;
    for v in 0..n {
        let d = graph.degree(v)? as f64;
        let diff = d - mean_deg;
        var += diff * diff;
    }

    Ok(var / nf)
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
        // Triangle 0-1-2 plus pendant 2-3
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn diamond() -> Graph {
        // K4 minus one edge
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3)], false, Some(4)).unwrap()
    }

    // --- albertson_index ---

    #[test]
    fn alb_empty() {
        let g = Graph::with_vertices(0);
        assert!((albertson_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn alb_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((albertson_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn alb_no_edges() {
        let g = Graph::with_vertices(3);
        assert!((albertson_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn alb_single_edge() {
        // both degree 1 → |1-1| = 0
        assert!((albertson_index(&single_edge()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn alb_path3() {
        // degrees [1,2,1]: edges (0,1)→|1-2|=1, (1,2)→|2-1|=1
        assert!((albertson_index(&path3()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn alb_path4() {
        // degrees [1,2,2,1]: (0,1)→1, (1,2)→0, (2,3)→1 → ALB=2
        assert!((albertson_index(&path4()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn alb_k3() {
        // regular → 0
        assert!((albertson_index(&k3()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn alb_k4() {
        assert!((albertson_index(&k4()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn alb_cycle4() {
        assert!((albertson_index(&cycle4()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn alb_cycle5() {
        assert!((albertson_index(&cycle5()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn alb_star5() {
        // center=4, leaf=1: |4-1|=3, 4 edges → ALB=12
        assert!((albertson_index(&star5()).unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn alb_paw() {
        // degrees [2,2,3,1]
        // (0,1): |2-2|=0, (0,2): |2-3|=1, (1,2): |2-3|=1, (2,3): |3-1|=2
        // ALB = 0+1+1+2 = 4
        assert!((albertson_index(&paw()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn alb_diamond() {
        // degrees [3,3,2,2]
        // (0,1):0, (0,2):1, (0,3):1, (1,2):1, (1,3):1 → ALB=4
        assert!((albertson_index(&diamond()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn alb_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(albertson_index(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn alb_zero_iff_regular() {
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            assert!((albertson_index(g).unwrap()).abs() < 1e-10);
        }
        for g in &[path3(), star5(), paw(), diamond()] {
            assert!(albertson_index(g).unwrap() > 1e-10);
        }
    }

    // --- sigma_index ---

    #[test]
    fn sig_empty() {
        let g = Graph::with_vertices(0);
        assert!((sigma_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn sig_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((sigma_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn sig_single_edge() {
        assert!((sigma_index(&single_edge()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn sig_path3() {
        // (0,1):(1-2)²=1, (1,2):(2-1)²=1 → σ=2
        assert!((sigma_index(&path3()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn sig_path4() {
        // (0,1):(1-2)²=1, (1,2):0, (2,3):(2-1)²=1 → σ=2
        assert!((sigma_index(&path4()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn sig_k3() {
        assert!((sigma_index(&k3()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn sig_k4() {
        assert!((sigma_index(&k4()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn sig_cycle4() {
        assert!((sigma_index(&cycle4()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn sig_star5() {
        // (4-1)²=9, 4 edges → σ=36
        assert!((sigma_index(&star5()).unwrap() - 36.0).abs() < 1e-10);
    }

    #[test]
    fn sig_paw() {
        // (0,1):0, (0,2):(2-3)²=1, (1,2):1, (2,3):(3-1)²=4
        // σ = 0+1+1+4 = 6
        assert!((sigma_index(&paw()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn sig_diamond() {
        // (0,1):0, (0,2):(3-2)²=1, (0,3):1, (1,2):1, (1,3):1 → σ=4
        assert!((sigma_index(&diamond()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn sig_zero_iff_regular() {
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            assert!((sigma_index(g).unwrap()).abs() < 1e-10);
        }
        for g in &[path3(), star5(), paw()] {
            assert!(sigma_index(g).unwrap() > 1e-10);
        }
    }

    #[test]
    fn sig_geq_alb_squared_over_m() {
        // By Cauchy-Schwarz: σ ≥ ALB² / m
        for g in &[
            single_edge(),
            path3(),
            path4(),
            k3(),
            star5(),
            paw(),
            diamond(),
        ] {
            let sig = sigma_index(g).unwrap();
            let alb = albertson_index(g).unwrap();
            let m = g.ecount() as f64;
            if m > 0.0 {
                assert!(sig >= alb * alb / m - 1e-8);
            }
        }
    }

    // --- total_irregularity ---

    #[test]
    fn irrt_empty() {
        let g = Graph::with_vertices(0);
        assert!((total_irregularity(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn irrt_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((total_irregularity(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn irrt_single_edge() {
        // degrees [1,1] → 0
        assert!((total_irregularity(&single_edge()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn irrt_path3() {
        // degrees [1,2,1]
        // pairs: (0,1)→1, (0,2)→0, (1,2)→1
        // irr_t = (1+0+1)/2 = 1
        assert!((total_irregularity(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn irrt_path4() {
        // degrees [1,2,2,1]
        // pairs: (0,1)→1,(0,2)→1,(0,3)→0,(1,2)→0,(1,3)→1,(2,3)→1
        // irr_t = (1+1+0+0+1+1)/2 = 2
        assert!((total_irregularity(&path4()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn irrt_k3() {
        assert!((total_irregularity(&k3()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn irrt_k4() {
        assert!((total_irregularity(&k4()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn irrt_cycle4() {
        assert!((total_irregularity(&cycle4()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn irrt_star5() {
        // degrees [4,1,1,1,1]
        // (0,1):3, (0,2):3, (0,3):3, (0,4):3, (1,2):0,(1,3):0,(1,4):0,(2,3):0,(2,4):0,(3,4):0
        // irr_t = (4·3)/2 = 6
        assert!((total_irregularity(&star5()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn irrt_paw() {
        // degrees [2,2,3,1]
        // (0,1):0,(0,2):1,(0,3):1,(1,2):1,(1,3):1,(2,3):2
        // irr_t = (0+1+1+1+1+2)/2 = 3
        assert!((total_irregularity(&paw()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn irrt_diamond() {
        // degrees [3,3,2,2]
        // (0,1):0,(0,2):1,(0,3):1,(1,2):1,(1,3):1,(2,3):0
        // irr_t = (0+1+1+1+1+0)/2 = 2
        assert!((total_irregularity(&diamond()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn irrt_zero_iff_regular() {
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            assert!((total_irregularity(g).unwrap()).abs() < 1e-10);
        }
        for g in &[path3(), star5(), paw()] {
            assert!(total_irregularity(g).unwrap() > 1e-10);
        }
    }

    #[test]
    fn irrt_geq_alb() {
        // ALB counts only edge pairs; irr_t counts all pairs → irr_t ≥ ALB/2
        // Actually irr_t = ½ Σ_{all pairs} and ALB = Σ_{edges}, so irr_t ≥ ½ ALB
        for g in &[
            single_edge(),
            path3(),
            path4(),
            k3(),
            star5(),
            paw(),
            diamond(),
        ] {
            let irrt = total_irregularity(g).unwrap();
            let alb = albertson_index(g).unwrap();
            assert!(irrt >= alb / 2.0 - 1e-8);
        }
    }

    #[test]
    fn irrt_with_isolated() {
        // 0-1 plus isolated 2: degrees [1,1,0]
        // (0,1):0, (0,2):1, (1,2):1
        // irr_t = (0+1+1)/2 = 1
        let g = Graph::from_edges(&[(0, 1)], false, Some(3)).unwrap();
        assert!((total_irregularity(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    // --- degree_variance ---

    #[test]
    fn dv_empty() {
        let g = Graph::with_vertices(0);
        assert!((degree_variance(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn dv_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((degree_variance(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn dv_no_edges() {
        let g = Graph::with_vertices(3);
        assert!((degree_variance(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn dv_single_edge() {
        // degrees [1,1], mean=1, var=0
        assert!((degree_variance(&single_edge()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn dv_k3() {
        assert!((degree_variance(&k3()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn dv_k4() {
        assert!((degree_variance(&k4()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn dv_cycle4() {
        assert!((degree_variance(&cycle4()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn dv_path3() {
        // degrees [1,2,1], mean=4/3
        // var = ((1-4/3)²+(2-4/3)²+(1-4/3)²)/3
        //     = (1/9 + 4/9 + 1/9)/3 = (6/9)/3 = 2/9
        assert!((degree_variance(&path3()).unwrap() - 2.0 / 9.0).abs() < 1e-10);
    }

    #[test]
    fn dv_star5() {
        // degrees [4,1,1,1,1], mean=8/5=1.6
        // var = ((4-1.6)²+4·(1-1.6)²)/5 = (5.76+4·0.36)/5 = (5.76+1.44)/5 = 7.2/5 = 1.44
        assert!((degree_variance(&star5()).unwrap() - 1.44).abs() < 1e-10);
    }

    #[test]
    fn dv_paw() {
        // degrees [2,2,3,1], mean=8/4=2
        // var = ((0)²+(0)²+(1)²+(-1)²)/4 = 2/4 = 0.5
        assert!((degree_variance(&paw()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn dv_zero_iff_regular() {
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            assert!((degree_variance(g).unwrap()).abs() < 1e-10);
        }
        for g in &[path3(), star5(), paw()] {
            assert!(degree_variance(g).unwrap() > 1e-10);
        }
    }

    #[test]
    fn dv_with_isolated() {
        // 0-1 plus isolated 2: degrees [1,1,0], mean=2/3
        // var = ((1-2/3)²+(1-2/3)²+(0-2/3)²)/3
        //     = (1/9 + 1/9 + 4/9)/3 = (6/9)/3 = 2/9
        let g = Graph::from_edges(&[(0, 1)], false, Some(3)).unwrap();
        assert!((degree_variance(&g).unwrap() - 2.0 / 9.0).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn all_nonneg_for_connected() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(albertson_index(g).unwrap() >= -1e-10);
            assert!(sigma_index(g).unwrap() >= -1e-10);
            assert!(total_irregularity(g).unwrap() >= -1e-10);
            assert!(degree_variance(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn all_zero_for_regular() {
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            assert!((albertson_index(g).unwrap()).abs() < 1e-10);
            assert!((sigma_index(g).unwrap()).abs() < 1e-10);
            assert!((total_irregularity(g).unwrap()).abs() < 1e-10);
            assert!((degree_variance(g).unwrap()).abs() < 1e-10);
        }
    }

    #[test]
    fn sigma_geq_albertson() {
        // σ(G) ≥ ALB(G) for edges where |d_u-d_v|≥1
        // (because x² ≥ |x| for |x|≥1, and |x|²≥0 always)
        // Actually σ = Σ(d_u-d_v)² can be < ALB = Σ|d_u-d_v| only if diffs are <1 (impossible for integers)
        // For integer degrees: (d_u-d_v)² ≥ |d_u-d_v| when |d_u-d_v|≥1, and =0 when equal.
        // So σ ≥ ALB for integer degrees.
        for g in &[
            single_edge(),
            path3(),
            k3(),
            k4(),
            star5(),
            paw(),
            diamond(),
        ] {
            let sig = sigma_index(g).unwrap();
            let alb = albertson_index(g).unwrap();
            assert!(sig >= alb - 1e-8);
        }
    }
}
