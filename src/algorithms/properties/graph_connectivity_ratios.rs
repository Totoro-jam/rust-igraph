//! Graph connectivity ratio indices (ALGO-TR-096).
//!
//! Measures of how well-connected a graph is relative to its size:
//!
//! - **Circuit rank ratio** — cyclomatic number normalized by edges
//! - **Meshedness coefficient** — circuit rank relative to max planar
//! - **Edge surplus ratio** — fraction of edges beyond a spanning tree
//! - **Connectivity index** — average degree divided by max possible degree

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the circuit rank ratio of the graph.
///
/// `CRR = (m - n + c) / m`
///
/// where c is the number of connected components. The circuit rank
/// (cyclomatic number) `m - n + c` counts the number of independent
/// cycles. Dividing by m gives the fraction of edges that are
/// "redundant" beyond a spanning forest. Returns 0.0 for graphs
/// with no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, circuit_rank_ratio};
///
/// // K_3: m=3, n=3, c=1 → circuit_rank=1, CRR=1/3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((circuit_rank_ratio(&g).unwrap() - 1.0/3.0).abs() < 1e-10);
/// ```
pub fn circuit_rank_ratio(graph: &Graph) -> IgraphResult<f64> {
    let m = graph.ecount() as i64;
    if m == 0 {
        return Ok(0.0);
    }

    let n = i64::from(graph.vcount());
    let c = count_components(graph)? as i64;
    let circuit_rank = (m - n + c).max(0);

    Ok(circuit_rank as f64 / m as f64)
}

/// Compute the meshedness coefficient of the graph.
///
/// `MC = (m - n + 1) / (2n - 5)`
///
/// For connected graphs, the circuit rank divided by the maximum
/// possible circuit rank of a planar graph on n vertices. Ranges
/// from 0 (tree) to values above 1 for dense/non-planar graphs.
/// Returns 0.0 for disconnected graphs, graphs with fewer than 3
/// vertices, or edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, meshedness_coefficient};
///
/// // K_3: m=3, n=3, MC = (3-3+1)/(2·3-5) = 1/1 = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((meshedness_coefficient(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn meshedness_coefficient(graph: &Graph) -> IgraphResult<f64> {
    let n = i64::from(graph.vcount());
    let m = graph.ecount() as i64;

    if n < 3 || m == 0 {
        return Ok(0.0);
    }

    let c = count_components(graph)? as i64;
    if c > 1 {
        return Ok(0.0);
    }

    let denom = 2 * n - 5;
    if denom <= 0 {
        return Ok(0.0);
    }

    let circuit_rank = (m - n + 1).max(0);
    Ok(circuit_rank as f64 / denom as f64)
}

/// Compute the edge surplus ratio.
///
/// `ESR = (m - n + c) / (n·(n-1)/2 - n + c)`
///
/// The fraction of the available "surplus" edge slots that are
/// actually used, where the denominator is the maximum possible
/// circuit rank (for a complete graph). Zero for forests, 1.0 for
/// complete graphs. Returns 0.0 for graphs with fewer than 3
/// vertices or when the denominator is zero.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_surplus_ratio};
///
/// // K_3: circuit_rank=1, max_circuit_rank = 3-3+1 = 1 → ESR=1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_surplus_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn edge_surplus_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = i64::from(graph.vcount());
    let m = graph.ecount() as i64;

    if n < 2 || m == 0 {
        return Ok(0.0);
    }

    let c = count_components(graph)? as i64;
    let circuit_rank = (m - n + c).max(0);

    let max_m = n * (n - 1) / 2;
    let max_circuit_rank = (max_m - n + c).max(0);

    if max_circuit_rank == 0 {
        return Ok(0.0);
    }

    Ok(circuit_rank as f64 / max_circuit_rank as f64)
}

/// Compute the connectivity index of the graph.
///
/// `CI = (2m/n) / (n - 1)`
///
/// Average degree divided by the maximum possible average degree
/// (for a simple graph). Equals the graph density for simple
/// undirected graphs. Returns 0.0 for graphs with fewer than 2
/// vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, connectivity_index};
///
/// // K_3: avg_deg=2, max_avg_deg=2, CI=1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((connectivity_index(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn connectivity_index(graph: &Graph) -> IgraphResult<f64> {
    let n = u64::from(graph.vcount());
    if n < 2 {
        return Ok(0.0);
    }

    let m = graph.ecount() as f64;
    let avg_deg = 2.0 * m / n as f64;
    let max_avg_deg = (n - 1) as f64;

    Ok(avg_deg / max_avg_deg)
}

fn count_components(graph: &Graph) -> IgraphResult<usize> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let mut visited = vec![false; n];
    let mut count = 0_usize;

    for start in 0..n {
        if visited[start] {
            continue;
        }
        count += 1;
        let mut stack = vec![start];
        visited[start] = true;
        while let Some(v) = stack.pop() {
            let neighbors = graph.neighbors(v as u32)?;
            for &u in &neighbors {
                let ui = u as usize;
                if !visited[ui] {
                    visited[ui] = true;
                    stack.push(ui);
                }
            }
        }
    }

    Ok(count)
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

    fn disconnected() -> Graph {
        // Two components: {0,1} and {2,3}
        Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap()
    }

    // --- circuit_rank_ratio ---

    #[test]
    fn crr_empty() {
        let g = Graph::with_vertices(0);
        assert!(circuit_rank_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn crr_isolated() {
        let g = Graph::with_vertices(5);
        assert!(circuit_rank_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn crr_single_edge() {
        // m=1, n=2, c=1 → cr=0, CRR=0
        assert!(circuit_rank_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn crr_path3() {
        // m=2, n=3, c=1 → cr=0, CRR=0 (tree)
        assert!(circuit_rank_ratio(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn crr_k3() {
        // m=3, n=3, c=1 → cr=1, CRR=1/3
        assert!((circuit_rank_ratio(&k3()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn crr_k4() {
        // m=6, n=4, c=1 → cr=3, CRR=3/6=0.5
        assert!((circuit_rank_ratio(&k4()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn crr_cycle4() {
        // m=4, n=4, c=1 → cr=1, CRR=1/4=0.25
        assert!((circuit_rank_ratio(&cycle4()).unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn crr_star5() {
        // m=4, n=5, c=1 → cr=0, CRR=0 (tree)
        assert!(circuit_rank_ratio(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn crr_paw() {
        // m=4, n=4, c=1 → cr=1, CRR=1/4=0.25
        assert!((circuit_rank_ratio(&paw()).unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn crr_disconnected() {
        // m=2, n=4, c=2 → cr=2-4+2=0, CRR=0 (forest)
        assert!(circuit_rank_ratio(&disconnected()).unwrap().abs() < 1e-10);
    }

    // --- meshedness_coefficient ---

    #[test]
    fn mc_empty() {
        let g = Graph::with_vertices(0);
        assert!(meshedness_coefficient(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mc_too_small() {
        let g = Graph::with_vertices(2);
        assert!(meshedness_coefficient(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mc_path3() {
        // Tree → cr=0 → MC=0
        assert!(meshedness_coefficient(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mc_k3() {
        // m=3, n=3, cr=1, denom=2·3-5=1, MC=1.0
        assert!((meshedness_coefficient(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mc_k4() {
        // m=6, n=4, cr=3, denom=2·4-5=3, MC=3/3=1.0
        assert!((meshedness_coefficient(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mc_cycle4() {
        // m=4, n=4, cr=1, denom=3, MC=1/3
        assert!((meshedness_coefficient(&cycle4()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn mc_star5() {
        // Tree → cr=0 → MC=0
        assert!(meshedness_coefficient(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mc_paw() {
        // m=4, n=4, cr=1, denom=3, MC=1/3
        assert!((meshedness_coefficient(&paw()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn mc_disconnected() {
        // c > 1 → MC=0
        assert!(meshedness_coefficient(&disconnected()).unwrap().abs() < 1e-10);
    }

    // --- edge_surplus_ratio ---

    #[test]
    fn esr_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_surplus_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn esr_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(edge_surplus_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn esr_single_edge() {
        // m=1, n=2, c=1, cr=0, max_cr=1-2+1=0 → denom=0 → 0.0
        assert!(edge_surplus_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn esr_path3() {
        // Tree → cr=0 → ESR=0
        assert!(edge_surplus_ratio(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn esr_k3() {
        // cr=1, max_m=3, max_cr=3-3+1=1, ESR=1/1=1.0
        assert!((edge_surplus_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn esr_k4() {
        // cr=3, max_m=6, max_cr=6-4+1=3, ESR=3/3=1.0
        assert!((edge_surplus_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn esr_cycle4() {
        // cr=1, max_m=6, max_cr=6-4+1=3, ESR=1/3
        assert!((edge_surplus_ratio(&cycle4()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn esr_star5() {
        // Tree → cr=0 → ESR=0
        assert!(edge_surplus_ratio(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn esr_paw() {
        // cr=1, max_m=6, max_cr=6-4+1=3, ESR=1/3
        assert!((edge_surplus_ratio(&paw()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    // --- connectivity_index ---

    #[test]
    fn ci_empty() {
        let g = Graph::with_vertices(0);
        assert!(connectivity_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ci_single() {
        let g = Graph::with_vertices(1);
        assert!(connectivity_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ci_single_edge() {
        // avg_deg=1, max=1, CI=1.0
        assert!((connectivity_index(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ci_k3() {
        // avg_deg=2, max=2, CI=1.0
        assert!((connectivity_index(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ci_k4() {
        // avg_deg=3, max=3, CI=1.0
        assert!((connectivity_index(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ci_cycle4() {
        // avg_deg=2, max=3, CI=2/3
        assert!((connectivity_index(&cycle4()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn ci_star5() {
        // avg_deg=8/5=1.6, max=4, CI=1.6/4=0.4
        assert!((connectivity_index(&star5()).unwrap() - 0.4).abs() < 1e-10);
    }

    #[test]
    fn ci_path3() {
        // avg_deg=4/3, max=2, CI=(4/3)/2=2/3
        assert!((connectivity_index(&path3()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn ci_paw() {
        // avg_deg=8/4=2, max=3, CI=2/3
        assert!((connectivity_index(&paw()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn ci_isolated() {
        let g = Graph::with_vertices(5);
        assert!(connectivity_index(&g).unwrap().abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn crr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = circuit_rank_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn esr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = edge_surplus_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn ci_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = connectivity_index(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn complete_graphs_all_one() {
        // Complete graphs: CRR=0.5(K4), ESR=1.0, CI=1.0
        assert!((edge_surplus_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((edge_surplus_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((connectivity_index(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((connectivity_index(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn trees_zero_surplus() {
        // Trees: CRR=0, ESR=0
        assert!(circuit_rank_ratio(&path3()).unwrap().abs() < 1e-10);
        assert!(circuit_rank_ratio(&star5()).unwrap().abs() < 1e-10);
        assert!(edge_surplus_ratio(&path3()).unwrap().abs() < 1e-10);
        assert!(edge_surplus_ratio(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mc_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(meshedness_coefficient(g).unwrap() >= -1e-10);
        }
    }
}
