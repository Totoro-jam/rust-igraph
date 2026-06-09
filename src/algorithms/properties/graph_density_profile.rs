//! Graph density profile indices (ALGO-TR-094).
//!
//! Density-like measures that quantify the graph's connectivity
//! from different perspectives:
//!
//! - **Triangle density** — fraction of possible triangles that exist
//! - **Square density** — fraction of possible 4-cycles that exist
//! - **Edge connectivity ratio** — 2m / (n·(n-1)) for undirected
//! - **Degree density** — (Σd²/Σd - 1) / (n - 1) normalized second moment

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the triangle density of the graph.
///
/// `T_d = 6·triangles / (n·(n-1)·(n-2))` for n ≥ 3
///
/// Fraction of all possible vertex triples that form a triangle.
/// Returns 0.0 for graphs with fewer than 3 vertices. Related to
/// transitivity but normalized by the total number of triples, not
/// just connected triples (paths of length 2).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, triangle_density};
///
/// // K_3: 1 triangle, n=3, denom=6 → 6·1/6 = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((triangle_density(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn triangle_density(graph: &Graph) -> IgraphResult<f64> {
    let n = u64::from(graph.vcount());
    if n < 3 {
        return Ok(0.0);
    }

    let mut tri_count = 0_u64;
    let nv = n as usize;
    for v in 0..nv {
        let vid = v as u32;
        let neighbors = graph.neighbors(vid)?;
        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                let a = neighbors[i];
                let b = neighbors[j];
                if a > vid && b > vid && graph.has_edge(a, b) {
                    tri_count += 1;
                }
            }
        }
    }

    let denom = n * (n - 1) * (n - 2);
    Ok(6.0 * tri_count as f64 / denom as f64)
}

/// Compute the square (4-cycle) density of the graph.
///
/// Number of distinct 4-cycles divided by `C(n, 4)` for n ≥ 4.
///
/// A 4-cycle is an induced cycle on 4 vertices (no chord). Counted
/// via non-adjacent pairs sharing common neighbors: each pair of
/// common neighbors of a non-edge (u,v) forms a 4-cycle u-w1-v-w2-u
/// provided w1-w2 are not adjacent. Returns 0.0 for n < 4.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, square_density};
///
/// // C_4: exactly 1 chordless 4-cycle, C(4,4)=1 → 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,0)], false, Some(4)).unwrap();
/// assert!((square_density(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn square_density(graph: &Graph) -> IgraphResult<f64> {
    let n = u64::from(graph.vcount());
    if n < 4 {
        return Ok(0.0);
    }

    let nv = n as usize;
    let mut count = 0_u64;

    for u in 0..nv {
        let uid = u as u32;
        for v in (u + 1)..nv {
            let vid = v as u32;
            if graph.has_edge(uid, vid) {
                continue;
            }
            let nu = graph.neighbors(uid)?;
            let nv_list = graph.neighbors(vid)?;

            let mut common = Vec::new();
            for &w in &nu {
                if w != vid {
                    for &x in &nv_list {
                        if x == w && x != uid {
                            common.push(w);
                            break;
                        }
                    }
                }
            }

            for i in 0..common.len() {
                for j in (i + 1)..common.len() {
                    if !graph.has_edge(common[i], common[j]) {
                        count += 1;
                    }
                }
            }
        }
    }

    let denom = n * (n - 1) * (n - 2) * (n - 3) / 24;
    if denom == 0 {
        return Ok(0.0);
    }

    // Each chordless 4-cycle is found twice (once per diagonal pair)
    Ok(count as f64 / (2.0 * denom as f64))
}

/// Compute the edge connectivity ratio.
///
/// `r = 2m / (n·(n-1))` for undirected, `m / (n·(n-1))` for directed
///
/// Identical to `density()` for simple graphs but computed directly
/// from edge and vertex counts without checking simplicity. Returns
/// 0.0 for graphs with fewer than 2 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_connectivity_ratio};
///
/// // K_3: 3 edges, n=3 → 2·3/(3·2) = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_connectivity_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn edge_connectivity_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = u64::from(graph.vcount());
    if n < 2 {
        return Ok(0.0);
    }

    let m = graph.ecount() as f64;
    let denom = n * (n - 1);

    if graph.is_directed() {
        Ok(m / denom as f64)
    } else {
        Ok(2.0 * m / denom as f64)
    }
}

/// Compute the degree density (normalized second moment of degree).
///
/// `DD = (⟨d²⟩/⟨d⟩ - 1) / (n - 1)`
///
/// where `⟨d²⟩` and `⟨d⟩` are the mean of d² and d. For a random
/// Erdős–Rényi graph, this equals the edge probability p. For
/// regular graphs, equals (degree) / (n-1). Returns 0.0 for graphs
/// with fewer than 2 vertices or zero total degree.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_density};
///
/// // K_3: d=2 for all, ⟨d²⟩=4, ⟨d⟩=2 → (4/2-1)/(3-1) = 1/2 = 0.5
/// // But density(K_3) = 1.0, so this is different
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((degree_density(&g).unwrap() - 0.5).abs() < 1e-10);
/// ```
pub fn degree_density(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let mut sum_d = 0.0_f64;
    let mut sum_d2 = 0.0_f64;

    for v in 0..n {
        let d = graph.degree(v as u32)? as f64;
        sum_d += d;
        sum_d2 += d * d;
    }

    if sum_d < 1e-15 {
        return Ok(0.0);
    }

    let mean_d = sum_d / n as f64;
    let mean_d2 = sum_d2 / n as f64;

    Ok((mean_d2 / mean_d - 1.0) / (n as f64 - 1.0))
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

    // --- triangle_density ---

    #[test]
    fn tri_empty() {
        let g = Graph::with_vertices(0);
        assert!(triangle_density(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tri_two() {
        let g = Graph::with_vertices(2);
        assert!(triangle_density(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tri_k3() {
        // 1 triangle, 6·1/(3·2·1) = 1.0
        assert!((triangle_density(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tri_k4() {
        // 4 triangles, 6·4/(4·3·2) = 24/24 = 1.0
        assert!((triangle_density(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tri_path3() {
        // 0 triangles
        assert!(triangle_density(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tri_cycle4() {
        // 0 triangles on C_4
        assert!(triangle_density(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tri_star5() {
        // 0 triangles
        assert!(triangle_density(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tri_paw() {
        // 1 triangle (0,1,2), n=4
        // 6·1/(4·3·2) = 6/24 = 0.25
        assert!((triangle_density(&paw()).unwrap() - 0.25).abs() < 1e-10);
    }

    // --- square_density ---

    #[test]
    fn sq_empty() {
        let g = Graph::with_vertices(0);
        assert!(square_density(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sq_three() {
        assert!(square_density(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sq_cycle4() {
        // C_4 has exactly 1 distinct 4-cycle
        // C(4,4) = 1, so density = 1/1 = 1.0
        assert!((square_density(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn sq_k4() {
        // K_4: every 4-cycle has a chord → 0 chordless 4-cycles
        assert!(square_density(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sq_star5() {
        // No 4-cycles in a star
        assert!(square_density(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sq_path3() {
        assert!(square_density(&path3()).unwrap().abs() < 1e-10);
    }

    // --- edge_connectivity_ratio ---

    #[test]
    fn ecr_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_connectivity_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ecr_single() {
        let g = Graph::with_vertices(1);
        assert!(edge_connectivity_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ecr_k3() {
        // 2·3/(3·2) = 1.0
        assert!((edge_connectivity_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ecr_k4() {
        // 2·6/(4·3) = 1.0
        assert!((edge_connectivity_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ecr_cycle4() {
        // 2·4/(4·3) = 8/12 = 2/3
        assert!((edge_connectivity_ratio(&cycle4()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn ecr_single_edge() {
        // 2·1/(2·1) = 1.0
        assert!((edge_connectivity_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ecr_star5() {
        // 2·4/(5·4) = 8/20 = 0.4
        assert!((edge_connectivity_ratio(&star5()).unwrap() - 0.4).abs() < 1e-10);
    }

    #[test]
    fn ecr_path3() {
        // 2·2/(3·2) = 4/6 = 2/3
        assert!((edge_connectivity_ratio(&path3()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn ecr_paw() {
        // 2·4/(4·3) = 8/12 = 2/3
        assert!((edge_connectivity_ratio(&paw()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    // --- degree_density ---

    #[test]
    fn dd_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_density(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dd_single() {
        let g = Graph::with_vertices(1);
        assert!(degree_density(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dd_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_density(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dd_k3() {
        // d=2 for all: ⟨d²⟩=4, ⟨d⟩=2, (4/2-1)/(3-1) = 1/2
        assert!((degree_density(&k3()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn dd_k4() {
        // d=3 for all: ⟨d²⟩=9, ⟨d⟩=3, (9/3-1)/(4-1) = 2/3
        assert!((degree_density(&k4()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn dd_single_edge() {
        // d=1 for all: (1/1-1)/(2-1) = 0/1 = 0
        assert!(degree_density(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dd_star5() {
        // degrees [4,1,1,1,1], n=5
        // ⟨d⟩ = 8/5 = 1.6
        // ⟨d²⟩ = (16+1+1+1+1)/5 = 20/5 = 4.0
        // DD = (4.0/1.6 - 1) / (5-1) = (2.5-1)/4 = 1.5/4 = 0.375
        assert!((degree_density(&star5()).unwrap() - 0.375).abs() < 1e-10);
    }

    #[test]
    fn dd_path3() {
        // degrees [1,2,1], n=3
        // ⟨d⟩ = 4/3, ⟨d²⟩ = (1+4+1)/3 = 2
        // DD = (2/(4/3) - 1) / (3-1) = (1.5-1)/2 = 0.5/2 = 0.25
        assert!((degree_density(&path3()).unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn dd_paw() {
        // degrees [2,2,3,1], n=4
        // ⟨d⟩ = 8/4 = 2, ⟨d²⟩ = (4+4+9+1)/4 = 18/4 = 4.5
        // DD = (4.5/2 - 1) / (4-1) = (2.25-1)/3 = 1.25/3
        let expected = 1.25 / 3.0;
        assert!((degree_density(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn tri_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let t = triangle_density(g).unwrap();
            assert!(t >= -1e-10);
            assert!(t <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn ecr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let r = edge_connectivity_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn dd_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_density(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn complete_ecr_is_one() {
        assert!((edge_connectivity_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((edge_connectivity_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn complete_tri_is_one() {
        assert!((triangle_density(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((triangle_density(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }
}
