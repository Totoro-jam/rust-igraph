//! Degree-based vertex class ratios (ALGO-TR-088).
//!
//! Fractions of the vertex set that fall into degree-based classes:
//!
//! - **Leaf ratio** — fraction of vertices with degree 1 (pendants)
//! - **Isolated ratio** — fraction of vertices with degree 0
//! - **Core ratio** — fraction of vertices with degree ≥ d̄ (mean)
//! - **Tail ratio** — fraction of vertices with degree ≥ 2·d̄ (heavy tail)

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the leaf ratio (fraction of degree-1 vertices).
///
/// Returns the fraction of vertices with degree exactly 1 (pendants).
/// Returns 0.0 for empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_leaf_ratio};
///
/// // Star S_5: 4 leaves out of 5 → 0.8
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert!((degree_leaf_ratio(&g).unwrap() - 0.8).abs() < 1e-10);
/// ```
pub fn degree_leaf_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut count = 0_usize;
    for v in 0..n {
        if graph.degree(v as u32)? == 1 {
            count += 1;
        }
    }

    Ok(count as f64 / n as f64)
}

/// Compute the isolated ratio (fraction of degree-0 vertices).
///
/// Returns the fraction of vertices with degree 0.
/// Returns 0.0 for empty graphs. Returns 1.0 for edgeless
/// non-empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_isolated_ratio};
///
/// // 3 isolated + 2 connected → 3/5 = 0.6
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(3, 4).unwrap();
/// assert!((degree_isolated_ratio(&g).unwrap() - 0.6).abs() < 1e-10);
/// ```
pub fn degree_isolated_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut count = 0_usize;
    for v in 0..n {
        if graph.degree(v as u32)? == 0 {
            count += 1;
        }
    }

    Ok(count as f64 / n as f64)
}

/// Compute the core ratio (fraction of vertices with degree ≥ d̄).
///
/// Returns the fraction of vertices whose degree is at least the
/// mean degree. Returns 0.0 for empty graphs. For regular graphs
/// this is always 1.0.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_core_ratio};
///
/// // K_3: all degree 2, mean = 2 → all qualify → 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((degree_core_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn degree_core_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let degrees = collect_degrees(graph)?;
    let mean = degrees.iter().sum::<usize>() as f64 / n as f64;

    let count = degrees
        .iter()
        .filter(|&&d| d as f64 >= mean - 1e-12)
        .count();

    Ok(count as f64 / n as f64)
}

/// Compute the tail ratio (fraction of vertices with degree ≥ 2·d̄).
///
/// Returns the fraction of vertices whose degree is at least twice
/// the mean degree. A heavy-tail indicator: sparse graphs with hubs
/// have higher tail ratios. Returns 0.0 for empty or edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_tail_ratio};
///
/// // Star S_5: mean = 8/5 = 1.6, threshold = 3.2, only vertex 0 (d=4) qualifies → 1/5 = 0.2
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert!((degree_tail_ratio(&g).unwrap() - 0.2).abs() < 1e-10);
/// ```
pub fn degree_tail_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let degrees = collect_degrees(graph)?;
    let mean = degrees.iter().sum::<usize>() as f64 / n as f64;
    let threshold = 2.0 * mean;

    let count = degrees
        .iter()
        .filter(|&&d| d as f64 >= threshold - 1e-12)
        .count();

    Ok(count as f64 / n as f64)
}

fn collect_degrees(graph: &Graph) -> IgraphResult<Vec<usize>> {
    let n = graph.vcount() as usize;
    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }
    Ok(degrees)
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

    // --- degree_leaf_ratio ---

    #[test]
    fn leaf_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_leaf_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn leaf_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_leaf_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn leaf_single_edge() {
        // Both vertices have degree 1 → 1.0
        assert!((degree_leaf_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn leaf_star5() {
        // 4 leaves / 5 = 0.8
        assert!((degree_leaf_ratio(&star5()).unwrap() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn leaf_path3() {
        // [1,2,1] → 2 leaves / 3
        let expected = 2.0 / 3.0;
        assert!((degree_leaf_ratio(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn leaf_k3() {
        // All degree 2 → 0 leaves
        assert!(degree_leaf_ratio(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn leaf_paw() {
        // [2,2,3,1] → 1 leaf / 4 = 0.25
        assert!((degree_leaf_ratio(&paw()).unwrap() - 0.25).abs() < 1e-10);
    }

    // --- degree_isolated_ratio ---

    #[test]
    fn iso_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_isolated_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn iso_all_isolated() {
        let g = Graph::with_vertices(5);
        assert!((degree_isolated_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn iso_single_edge() {
        assert!(degree_isolated_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn iso_k3() {
        assert!(degree_isolated_ratio(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn iso_with_isolates() {
        // 5 vertices, only edge (0,1) → 3 isolated / 5 = 0.6
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        assert!((degree_isolated_ratio(&g).unwrap() - 0.6).abs() < 1e-10);
    }

    #[test]
    fn iso_star5() {
        assert!(degree_isolated_ratio(&star5()).unwrap().abs() < 1e-10);
    }

    // --- degree_core_ratio ---

    #[test]
    fn core_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_core_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn core_isolated() {
        // All degree 0, mean = 0 → all qualify → 1.0
        let g = Graph::with_vertices(5);
        assert!((degree_core_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn core_regular() {
        // Regular: all degrees = mean → all qualify → 1.0
        assert!((degree_core_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((degree_core_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((degree_core_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn core_single_edge() {
        // [1,1] mean=1 → both qualify → 1.0
        assert!((degree_core_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn core_star5() {
        // [4,1,1,1,1] mean=8/5=1.6, only vertex 0 (d=4) qualifies → 1/5 = 0.2
        assert!((degree_core_ratio(&star5()).unwrap() - 0.2).abs() < 1e-10);
    }

    #[test]
    fn core_path3() {
        // [1,2,1] mean=4/3≈1.33, vertex 1 (d=2) qualifies → 1/3
        let expected = 1.0 / 3.0;
        assert!((degree_core_ratio(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn core_paw() {
        // [2,2,3,1] mean=8/4=2, vertices 0,1,2 qualify (d≥2) → 3/4 = 0.75
        assert!((degree_core_ratio(&paw()).unwrap() - 0.75).abs() < 1e-10);
    }

    // --- degree_tail_ratio ---

    #[test]
    fn tail_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_tail_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tail_isolated() {
        // mean=0, threshold=0, all qualify → 1.0
        let g = Graph::with_vertices(5);
        assert!((degree_tail_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tail_regular() {
        // Regular degree r: threshold=2r, no vertex reaches 2r → 0.0
        assert!(degree_tail_ratio(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_tail_ratio(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_tail_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tail_single_edge() {
        // [1,1] mean=1, threshold=2, none qualify → 0.0
        assert!(degree_tail_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tail_star5() {
        // [4,1,1,1,1] mean=1.6, threshold=3.2, only d=4 → 1/5 = 0.2
        assert!((degree_tail_ratio(&star5()).unwrap() - 0.2).abs() < 1e-10);
    }

    #[test]
    fn tail_path3() {
        // [1,2,1] mean=4/3, threshold=8/3≈2.67, none qualify → 0.0
        assert!(degree_tail_ratio(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tail_paw() {
        // [2,2,3,1] mean=2, threshold=4, none qualify → 0.0
        assert!(degree_tail_ratio(&paw()).unwrap().abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn all_ratios_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            for f in &[
                degree_leaf_ratio as fn(&Graph) -> IgraphResult<f64>,
                degree_isolated_ratio,
                degree_core_ratio,
                degree_tail_ratio,
            ] {
                let val = f(g).unwrap();
                assert!(val >= -1e-10, "ratio below 0: {val}");
                assert!(val <= 1.0 + 1e-10, "ratio above 1: {val}");
            }
        }
    }

    #[test]
    fn tail_le_core() {
        // d ≥ 2·mean implies d ≥ mean, so tail ⊆ core
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let tail = degree_tail_ratio(g).unwrap();
            let core = degree_core_ratio(g).unwrap();
            assert!(tail <= core + 1e-10);
        }
    }

    #[test]
    fn leaf_plus_iso_le_one() {
        // leaf (d=1) and isolated (d=0) are disjoint classes
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let leaf = degree_leaf_ratio(g).unwrap();
            let iso = degree_isolated_ratio(g).unwrap();
            assert!(leaf + iso <= 1.0 + 1e-10);
        }
    }
}
