//! Vertex-level neighbor degree statistics (ALGO-TR-089).
//!
//! Aggregates over the degree of each vertex's neighborhood:
//!
//! - **Neighbor max sum** `Σ_v max_{u∈N(v)} d(u)`
//! - **Neighbor min sum** `Σ_v min_{u∈N(v)} d(u)`
//! - **Neighbor range sum** `Σ_v (max - min)` of neighbor degrees
//! - **Neighbor variance sum** `Σ_v Var(d(N(v)))` — total neighbor-degree variance

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the sum of maximum neighbor degrees.
///
/// `Σ_{v: d(v)>0} max_{u ∈ N(v)} d(u)`
///
/// For each non-isolated vertex, finds the highest degree among its
/// neighbors and sums these maxima. Returns 0 for edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_neighbor_max_sum};
///
/// // Star S_5: center max=1, 4 leaves max=4 → 1 + 4·4 = 17
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert_eq!(degree_neighbor_max_sum(&g).unwrap(), 17);
/// ```
pub fn degree_neighbor_max_sum(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    let mut result = 0_u64;

    for v in 0..n {
        let v_id = v as u32;
        let neighbors = graph.neighbors(v_id)?;
        let mut max_d = None;
        for &u in &neighbors {
            let d = graph.degree(u)?;
            max_d = Some(match max_d {
                None => d,
                Some(prev) if d > prev => d,
                Some(prev) => prev,
            });
        }
        if let Some(m) = max_d {
            result += m as u64;
        }
    }

    Ok(result)
}

/// Compute the sum of minimum neighbor degrees.
///
/// `Σ_{v: d(v)>0} min_{u ∈ N(v)} d(u)`
///
/// For each non-isolated vertex, finds the lowest degree among its
/// neighbors and sums these minima. Returns 0 for edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_neighbor_min_sum};
///
/// // Star S_5: center min=1, 4 leaves min=4 → 1 + 4·4 = 17
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert_eq!(degree_neighbor_min_sum(&g).unwrap(), 17);
/// ```
pub fn degree_neighbor_min_sum(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    let mut result = 0_u64;

    for v in 0..n {
        let v_id = v as u32;
        let neighbors = graph.neighbors(v_id)?;
        let mut min_d = None;
        for &u in &neighbors {
            let d = graph.degree(u)?;
            min_d = Some(match min_d {
                None => d,
                Some(prev) if d < prev => d,
                Some(prev) => prev,
            });
        }
        if let Some(m) = min_d {
            result += m as u64;
        }
    }

    Ok(result)
}

/// Compute the sum of neighbor degree ranges.
///
/// `Σ_{v: d(v)>0} (max_{u ∈ N(v)} d(u) - min_{u ∈ N(v)} d(u))`
///
/// For each non-isolated vertex, computes the range (max - min) of
/// neighbor degrees and sums them. Returns 0 for edgeless or regular
/// graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_neighbor_range_sum};
///
/// // K_3: each vertex sees [2,2] → range 0 → sum 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(degree_neighbor_range_sum(&g).unwrap(), 0);
/// ```
pub fn degree_neighbor_range_sum(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    let mut result = 0_u64;

    for v in 0..n {
        let v_id = v as u32;
        let neighbors = graph.neighbors(v_id)?;
        let mut max_d: Option<usize> = None;
        let mut min_d: Option<usize> = None;
        for &u in &neighbors {
            let d = graph.degree(u)?;
            max_d = Some(match max_d {
                None => d,
                Some(prev) if d > prev => d,
                Some(prev) => prev,
            });
            min_d = Some(match min_d {
                None => d,
                Some(prev) if d < prev => d,
                Some(prev) => prev,
            });
        }
        if let (Some(mx), Some(mn)) = (max_d, min_d) {
            result += (mx - mn) as u64;
        }
    }

    Ok(result)
}

/// Compute the sum of neighbor degree variances.
///
/// `Σ_{v: d(v)>0} Var(d(N(v)))`
///
/// For each non-isolated vertex, computes the population variance of
/// its neighbors' degrees and sums them. Returns 0.0 for edgeless or
/// regular graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_neighbor_variance_sum};
///
/// // K_3: each vertex sees [2,2] → Var=0 → sum 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_neighbor_variance_sum(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_neighbor_variance_sum(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let mut result = 0.0_f64;

    for v in 0..n {
        let v_id = v as u32;
        let neighbors = graph.neighbors(v_id)?;
        if neighbors.is_empty() {
            continue;
        }
        let k = neighbors.len() as f64;
        let mut sum = 0.0_f64;
        let mut sum_sq = 0.0_f64;
        for &u in &neighbors {
            let d = graph.degree(u)? as f64;
            sum += d;
            sum_sq += d * d;
        }
        let mean = sum / k;
        let var = sum_sq / k - mean * mean;
        result += var;
    }

    Ok(result)
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

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- degree_neighbor_max_sum ---

    #[test]
    fn nmax_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(degree_neighbor_max_sum(&g).unwrap(), 0);
    }

    #[test]
    fn nmax_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(degree_neighbor_max_sum(&g).unwrap(), 0);
    }

    #[test]
    fn nmax_single_edge() {
        // Both see neighbor of degree 1 → 1+1 = 2
        assert_eq!(degree_neighbor_max_sum(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn nmax_k3() {
        // Each vertex sees [2,2] → max=2, 3·2 = 6
        assert_eq!(degree_neighbor_max_sum(&k3()).unwrap(), 6);
    }

    #[test]
    fn nmax_k4() {
        // Each vertex sees [3,3,3] → max=3, 4·3 = 12
        assert_eq!(degree_neighbor_max_sum(&k4()).unwrap(), 12);
    }

    #[test]
    fn nmax_star5() {
        // Center(d=4) neighbors=[1,1,1,1] max=1
        // 4 leaves: neighbor=[4] max=4
        // 1 + 4·4 = 17
        assert_eq!(degree_neighbor_max_sum(&star5()).unwrap(), 17);
    }

    #[test]
    fn nmax_path3() {
        // v0(d=1) N=[v1(d=2)] max=2
        // v1(d=2) N=[v0(1),v2(1)] max=1
        // v2(d=1) N=[v1(d=2)] max=2
        // 2 + 1 + 2 = 5
        assert_eq!(degree_neighbor_max_sum(&path3()).unwrap(), 5);
    }

    #[test]
    fn nmax_paw() {
        // v0(d=2) N=[v1(2),v2(3)] max=3
        // v1(d=2) N=[v0(2),v2(3)] max=3
        // v2(d=3) N=[v0(2),v1(2),v3(1)] max=2
        // v3(d=1) N=[v2(3)] max=3
        // 3+3+2+3 = 11
        assert_eq!(degree_neighbor_max_sum(&paw()).unwrap(), 11);
    }

    // --- degree_neighbor_min_sum ---

    #[test]
    fn nmin_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(degree_neighbor_min_sum(&g).unwrap(), 0);
    }

    #[test]
    fn nmin_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(degree_neighbor_min_sum(&g).unwrap(), 0);
    }

    #[test]
    fn nmin_single_edge() {
        assert_eq!(degree_neighbor_min_sum(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn nmin_k3() {
        // Each sees [2,2] → min=2, 3·2=6
        assert_eq!(degree_neighbor_min_sum(&k3()).unwrap(), 6);
    }

    #[test]
    fn nmin_star5() {
        // Center: min=1, leaves: min=4 → 1 + 4·4 = 17
        assert_eq!(degree_neighbor_min_sum(&star5()).unwrap(), 17);
    }

    #[test]
    fn nmin_path3() {
        // v0:min=2, v1:min=1, v2:min=2 → 5
        assert_eq!(degree_neighbor_min_sum(&path3()).unwrap(), 5);
    }

    #[test]
    fn nmin_paw() {
        // v0:min=2, v1:min=2, v2:min=1, v3:min=3
        // 2+2+1+3 = 8
        assert_eq!(degree_neighbor_min_sum(&paw()).unwrap(), 8);
    }

    // --- degree_neighbor_range_sum ---

    #[test]
    fn nrange_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(degree_neighbor_range_sum(&g).unwrap(), 0);
    }

    #[test]
    fn nrange_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(degree_neighbor_range_sum(&g).unwrap(), 0);
    }

    #[test]
    fn nrange_regular() {
        // Regular: all neighbors same degree → range 0
        assert_eq!(degree_neighbor_range_sum(&k3()).unwrap(), 0);
        assert_eq!(degree_neighbor_range_sum(&k4()).unwrap(), 0);
        assert_eq!(degree_neighbor_range_sum(&cycle4()).unwrap(), 0);
    }

    #[test]
    fn nrange_single_edge() {
        // Each has 1 neighbor → range 0
        assert_eq!(degree_neighbor_range_sum(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn nrange_star5() {
        // Center: all neighbors d=1 → range=0
        // Leaves: 1 neighbor → range=0
        assert_eq!(degree_neighbor_range_sum(&star5()).unwrap(), 0);
    }

    #[test]
    fn nrange_path3() {
        // v0: [2] range=0; v1: [1,1] range=0; v2: [2] range=0
        assert_eq!(degree_neighbor_range_sum(&path3()).unwrap(), 0);
    }

    #[test]
    fn nrange_paw() {
        // v0: N=[2,3] range=1; v1: N=[2,3] range=1
        // v2: N=[2,2,1] range=1; v3: N=[3] range=0
        // 1+1+1+0 = 3
        assert_eq!(degree_neighbor_range_sum(&paw()).unwrap(), 3);
    }

    // --- degree_neighbor_variance_sum ---

    #[test]
    fn nvar_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_neighbor_variance_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn nvar_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_neighbor_variance_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn nvar_regular() {
        // All neighbors have same degree → variance 0
        assert!(degree_neighbor_variance_sum(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_neighbor_variance_sum(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_neighbor_variance_sum(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn nvar_single_edge() {
        // 1 neighbor each → var=0
        assert!(degree_neighbor_variance_sum(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn nvar_star5() {
        // Center: [1,1,1,1] var=0; leaves: [4] var=0
        assert!(degree_neighbor_variance_sum(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn nvar_path3() {
        // v0:[2] var=0; v1:[1,1] var=0; v2:[2] var=0
        assert!(degree_neighbor_variance_sum(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn nvar_paw() {
        // v0: N=[2,3] mean=2.5, var=((2-2.5)²+(3-2.5)²)/2 = 0.25
        // v1: N=[2,3] same → 0.25
        // v2: N=[2,2,1] mean=5/3, var=((2-5/3)²+(2-5/3)²+(1-5/3)²)/3
        //     = ((1/3)²+(1/3)²+(-2/3)²)/3 = (1/9+1/9+4/9)/3 = (6/9)/3 = 2/9
        // v3: N=[3] var=0
        let expected = 0.25 + 0.25 + 2.0 / 9.0;
        assert!((degree_neighbor_variance_sum(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn max_ge_min() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_neighbor_max_sum(g).unwrap() >= degree_neighbor_min_sum(g).unwrap());
        }
    }

    #[test]
    fn range_equals_max_minus_min() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let mx = degree_neighbor_max_sum(g).unwrap();
            let mn = degree_neighbor_min_sum(g).unwrap();
            let rng = degree_neighbor_range_sum(g).unwrap();
            assert!(rng <= mx - mn);
        }
    }

    #[test]
    fn variance_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_neighbor_variance_sum(g).unwrap() >= -1e-10);
        }
    }
}
