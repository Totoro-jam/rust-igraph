//! Bipartivity ratio indices (ALGO-TR-109).
//!
//! Measures capturing how close a graph is to being bipartite:
//!
//! - **Bipartivity index** — fraction of vertices in a max-cut
//!   two-coloring (BFS greedy)
//! - **Frustration ratio** — fraction of edges that violate a
//!   two-coloring (bipartite edges / total edges)
//! - **Odd cycle density** — fraction of triangles (odd cycles of
//!   length 3) relative to max possible
//! - **Even-odd walk ratio** — ratio of even-length to odd-length
//!   walks of length 2 (via degree sums)

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the bipartivity index.
///
/// Uses a BFS greedy 2-coloring to partition vertices into two sets,
/// then returns `max(|A|, |B|) / n` — the fraction of vertices in
/// the larger partition. For a perfectly bipartite graph this equals
/// the larger side fraction; for complete graphs it approaches 0.5.
/// Returns 0.0 for empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bipartivity_index};
///
/// // Path 0-1-2 is bipartite with partition {0,2} vs {1} → 2/3
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((bipartivity_index(&g).unwrap() - 2.0/3.0).abs() < 1e-10);
/// ```
pub fn bipartivity_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut color: Vec<i8> = vec![-1; n];
    let mut side_a = 0_u64;
    let mut side_b = 0_u64;

    for start in 0..n {
        if color[start] != -1 {
            continue;
        }
        color[start] = 0;
        side_a += 1;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        while let Some(v) = queue.pop_front() {
            let nbrs = graph.neighbors(v as u32)?;
            let next_color = 1 - color[v];
            for &u in &nbrs {
                let ui = u as usize;
                if color[ui] == -1 {
                    color[ui] = next_color;
                    if next_color == 0 {
                        side_a += 1;
                    } else {
                        side_b += 1;
                    }
                    queue.push_back(ui);
                }
            }
        }
    }

    let larger = side_a.max(side_b);
    Ok(larger as f64 / n as f64)
}

/// Compute the frustration ratio.
///
/// Fraction of edges that are "frustrated" (connect same-color vertices)
/// under a BFS greedy 2-coloring. For bipartite graphs this is 0.0;
/// for complete graphs on n≥3 vertices it is positive. Returns 0.0
/// for graphs with no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, frustration_ratio};
///
/// // K_3: BFS colors 0→A, 1→B, 2→B; edge (1,2) frustrated → 1/3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((frustration_ratio(&g).unwrap() - 1.0/3.0).abs() < 1e-10);
/// ```
pub fn frustration_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let mut color: Vec<i8> = vec![-1; n];

    for start in 0..n {
        if color[start] != -1 {
            continue;
        }
        color[start] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        while let Some(v) = queue.pop_front() {
            let nbrs = graph.neighbors(v as u32)?;
            let next_color = 1 - color[v];
            for &u in &nbrs {
                let ui = u as usize;
                if color[ui] == -1 {
                    color[ui] = next_color;
                    queue.push_back(ui);
                }
            }
        }
    }

    let mut frustrated = 0_u64;
    for v in 0..n {
        let nbrs = graph.neighbors(v as u32)?;
        for &u in &nbrs {
            let ui = u as usize;
            if ui > v && color[v] == color[ui] {
                frustrated += 1;
            }
        }
    }

    Ok(frustrated as f64 / m as f64)
}

/// Compute the odd cycle density.
///
/// Fraction of closed triplets (triangles) relative to possible triplets.
/// This equals the global clustering coefficient, but interpreted here as
/// a measure of odd-cycle (length-3) density — a non-zero value proves
/// the graph is not bipartite. Returns 0.0 for triangle-free or trivial
/// graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, odd_cycle_density};
///
/// // K_4: C=1.0 (all triplets closed)
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((odd_cycle_density(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn odd_cycle_density(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let mut total_triplets = 0_u64;
    let mut closed_triplets = 0_u64;

    for v in 0..n {
        let neighbors = graph.neighbors(v as u32)?;
        let d = neighbors.len();
        if d < 2 {
            continue;
        }
        let possible = (d * (d - 1)) / 2;
        total_triplets += possible as u64;

        for i in 0..d {
            for j in (i + 1)..d {
                if graph.has_edge(neighbors[i], neighbors[j]) {
                    closed_triplets += 1;
                }
            }
        }
    }

    if total_triplets == 0 {
        return Ok(0.0);
    }

    Ok(closed_triplets as f64 / total_triplets as f64)
}

/// Compute the even-odd walk ratio.
///
/// Ratio of even-length walks of length 2 to odd-length walks of
/// length 1 (edges). A walk of length 2 from u to w passes through
/// some intermediate v — the total count is `Σ_v d(v)²` (counting
/// ordered pairs of neighbors). The odd-length count is `2m` (each
/// edge contributes 2 directed walks of length 1).
///
/// Returns `(Σ d²) / (2m)` — for bipartite graphs this relates to
/// the ratio of even vs odd spectral moments. Returns 0.0 for graphs
/// with no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, even_odd_walk_ratio};
///
/// // Path 0-1-2: degrees [1,2,1], Σd²=1+4+1=6, 2m=4 → 6/4=1.5
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((even_odd_walk_ratio(&g).unwrap() - 1.5).abs() < 1e-10);
/// ```
pub fn even_odd_walk_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let mut sum_deg_sq = 0_u64;
    for v in 0..n {
        let d = graph.degree(v as u32)? as u64;
        sum_deg_sq += d * d;
    }

    let two_m = 2 * m as u64;
    Ok(sum_deg_sq as f64 / two_m as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> Graph {
        Graph::with_vertices(0)
    }

    fn single() -> Graph {
        Graph::with_vertices(1)
    }

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

    fn bipartite_k23() -> Graph {
        // K_{2,3}: vertices {0,1} connected to {2,3,4}
        Graph::from_edges(
            &[(0, 2), (0, 3), (0, 4), (1, 2), (1, 3), (1, 4)],
            false,
            Some(5),
        )
        .unwrap()
    }

    // --- bipartivity_index ---

    #[test]
    fn bi_empty() {
        assert!(bipartivity_index(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn bi_single() {
        assert!((bipartivity_index(&single()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn bi_single_edge() {
        // {0} vs {1} → max(1,1)/2 = 0.5
        assert!((bipartivity_index(&single_edge()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn bi_path3() {
        // Bipartite: {0,2} vs {1} → max(2,1)/3 = 2/3
        assert!((bipartivity_index(&path3()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn bi_cycle4() {
        // Bipartite: {0,2} vs {1,3} → max(2,2)/4 = 0.5
        assert!((bipartivity_index(&cycle4()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn bi_bipartite_k23() {
        // {0,1} vs {2,3,4} → max(3,2)/5 = 3/5
        assert!((bipartivity_index(&bipartite_k23()).unwrap() - 3.0 / 5.0).abs() < 1e-10);
    }

    #[test]
    fn bi_in_range() {
        for g in &[
            empty(),
            single(),
            single_edge(),
            path3(),
            k3(),
            k4(),
            cycle4(),
            star5(),
            paw(),
        ] {
            let r = bipartivity_index(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- frustration_ratio ---

    #[test]
    fn fr_empty() {
        assert!(frustration_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fr_single_edge() {
        // Bipartite → 0 frustrated
        assert!(frustration_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fr_path3() {
        // Bipartite → 0 frustrated
        assert!(frustration_ratio(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fr_cycle4() {
        // Bipartite → 0 frustrated
        assert!(frustration_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fr_bipartite_k23() {
        assert!(frustration_ratio(&bipartite_k23()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fr_k3() {
        // BFS: 0→A, 1→B, 2→B; edge(1,2) frustrated → 1/3
        assert!((frustration_ratio(&k3()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn fr_cycle5() {
        // Odd cycle: BFS 0→A,1→B,2→A,3→B,4→A; edge(4,0) same color → 1/5
        assert!((frustration_ratio(&cycle5()).unwrap() - 1.0 / 5.0).abs() < 1e-10);
    }

    #[test]
    fn fr_in_01() {
        for g in &[
            single_edge(),
            path3(),
            k3(),
            k4(),
            cycle4(),
            cycle5(),
            star5(),
            paw(),
        ] {
            let r = frustration_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- odd_cycle_density ---

    #[test]
    fn ocd_empty() {
        assert!(odd_cycle_density(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ocd_single() {
        assert!(odd_cycle_density(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ocd_path3() {
        // No triangles
        assert!(odd_cycle_density(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ocd_cycle4() {
        // No triangles
        assert!(odd_cycle_density(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ocd_bipartite_k23() {
        // Bipartite → no odd cycles of length 3
        assert!(odd_cycle_density(&bipartite_k23()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ocd_k3() {
        // C = 1.0
        assert!((odd_cycle_density(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ocd_k4() {
        // C = 1.0
        assert!((odd_cycle_density(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ocd_star5() {
        // No triangles
        assert!(odd_cycle_density(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ocd_paw() {
        // Paw: degrees [2,2,3,1]. Triplets at v=0: C(2,2)=1, v=1: C(2,2)=1, v=2: C(3,2)=3
        // Total = 5. Closed: (0,1,2) triangle → each endpoint contributes 1 closed → 3 closed
        // But wait — closed_triplets counts per-vertex contribution:
        // v=0: nbrs={1,2}, edge(1,2)? yes → 1
        // v=1: nbrs={0,2}, edge(0,2)? yes → 1
        // v=2: nbrs={0,1,3}, pairs: (0,1)→yes, (0,3)→no, (1,3)→no → 1
        // Total closed = 3, total triplets = 1+1+3 = 5
        // Clustering = 3/5 = 0.6
        let r = odd_cycle_density(&paw()).unwrap();
        assert!((r - 3.0 / 5.0).abs() < 1e-10);
    }

    #[test]
    fn ocd_in_01() {
        for g in &[path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = odd_cycle_density(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- even_odd_walk_ratio ---

    #[test]
    fn eowr_empty() {
        assert!(even_odd_walk_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn eowr_single() {
        assert!(even_odd_walk_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn eowr_single_edge() {
        // degrees [1,1], Σd²=2, 2m=2 → 1.0
        assert!((even_odd_walk_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn eowr_path3() {
        // degrees [1,2,1], Σd²=6, 2m=4 → 1.5
        assert!((even_odd_walk_ratio(&path3()).unwrap() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn eowr_k3() {
        // degrees [2,2,2], Σd²=12, 2m=6 → 2.0
        assert!((even_odd_walk_ratio(&k3()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn eowr_k4() {
        // degrees [3,3,3,3], Σd²=36, 2m=12 → 3.0
        assert!((even_odd_walk_ratio(&k4()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn eowr_cycle4() {
        // degrees [2,2,2,2], Σd²=16, 2m=8 → 2.0
        assert!((even_odd_walk_ratio(&cycle4()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn eowr_star5() {
        // degrees [4,1,1,1,1], Σd²=16+4=20, 2m=8 → 2.5
        assert!((even_odd_walk_ratio(&star5()).unwrap() - 2.5).abs() < 1e-10);
    }

    #[test]
    fn eowr_positive() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(even_odd_walk_ratio(g).unwrap() > 0.0);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn bipartite_zero_frustration() {
        assert!(frustration_ratio(&path3()).unwrap().abs() < 1e-10);
        assert!(frustration_ratio(&cycle4()).unwrap().abs() < 1e-10);
        assert!(frustration_ratio(&star5()).unwrap().abs() < 1e-10);
        assert!(frustration_ratio(&bipartite_k23()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn bipartite_zero_odd_cycles() {
        assert!(odd_cycle_density(&path3()).unwrap().abs() < 1e-10);
        assert!(odd_cycle_density(&cycle4()).unwrap().abs() < 1e-10);
        assert!(odd_cycle_density(&star5()).unwrap().abs() < 1e-10);
        assert!(odd_cycle_density(&bipartite_k23()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn regular_graph_walk_ratio_equals_degree() {
        // For k-regular graph: Σd²=n·k², 2m=n·k → ratio = k
        assert!((even_odd_walk_ratio(&k3()).unwrap() - 2.0).abs() < 1e-10);
        assert!((even_odd_walk_ratio(&k4()).unwrap() - 3.0).abs() < 1e-10);
        assert!((even_odd_walk_ratio(&cycle4()).unwrap() - 2.0).abs() < 1e-10);
    }
}
