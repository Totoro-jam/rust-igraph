//! Edge density ratio indices (ALGO-TR-098).
//!
//! Normalized edge-level density measures:
//!
//! - **Self-loop ratio** — fraction of edges that are self-loops
//! - **Multi-edge ratio** — fraction of edges that are duplicated
//! - **Reciprocity ratio** — fraction of directed edges with a reciprocal
//! - **Average local clustering** — mean local clustering coefficient

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the self-loop ratio of the graph.
///
/// Fraction of edges that are self-loops (edges where both endpoints
/// are the same vertex). Returns 0.0 for graphs with no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, self_loop_ratio};
///
/// // No self-loops in a simple triangle
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(self_loop_ratio(&g).unwrap().abs() < 1e-10);
/// ```
pub fn self_loop_ratio(graph: &Graph) -> IgraphResult<f64> {
    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let mut loop_count = 0_usize;
    for (u, v) in graph.edges() {
        if u == v {
            loop_count += 1;
        }
    }

    Ok(loop_count as f64 / m as f64)
}

/// Compute the multi-edge ratio of the graph.
///
/// Fraction of edges that are duplicates (i.e., share the same pair
/// of endpoints with at least one other edge). For undirected graphs,
/// edges (u,v) and (v,u) are the same. Returns 0.0 for graphs with
/// no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, multi_edge_ratio};
///
/// // Simple triangle — no multi-edges → 0.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(multi_edge_ratio(&g).unwrap().abs() < 1e-10);
/// ```
pub fn multi_edge_ratio(graph: &Graph) -> IgraphResult<f64> {
    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let directed = graph.is_directed();
    let mut edge_set = std::collections::HashSet::new();
    let mut multi_count = 0_usize;

    for (u, v) in graph.edges() {
        let key = if directed {
            (u, v)
        } else {
            (u.min(v), u.max(v))
        };
        if !edge_set.insert(key) {
            multi_count += 1;
        }
    }

    Ok(multi_count as f64 / m as f64)
}

/// Compute the reciprocity ratio of the graph.
///
/// For directed graphs, the fraction of directed edges `(u,v)` for which
/// the reverse edge `(v,u)` also exists. For undirected graphs, returns
/// 1.0 (every edge is trivially reciprocal). Returns 0.0 for graphs
/// with no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reciprocity_ratio};
///
/// // Undirected triangle → all reciprocal → 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((reciprocity_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn reciprocity_ratio(graph: &Graph) -> IgraphResult<f64> {
    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    if !graph.is_directed() {
        return Ok(1.0);
    }

    let mut edge_set = std::collections::HashSet::new();
    for (u, v) in graph.edges() {
        edge_set.insert((u, v));
    }

    let mut reciprocal_count = 0_usize;
    for &(u, v) in &edge_set {
        if edge_set.contains(&(v, u)) {
            reciprocal_count += 1;
        }
    }

    Ok(reciprocal_count as f64 / m as f64)
}

/// Compute the average local clustering coefficient.
///
/// The mean of the local clustering coefficients over all vertices
/// with degree >= 2. The local clustering coefficient of vertex `v`
/// is the fraction of pairs of neighbors of `v` that are themselves
/// connected. Returns 0.0 for graphs where no vertex has degree >= 2.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, avg_local_clustering};
///
/// // K_3: each vertex has 2 neighbors, 1 pair, 1 edge → C(v)=1 → avg=1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((avg_local_clustering(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn avg_local_clustering(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut sum = 0.0_f64;
    let mut count = 0_usize;

    for v in 0..n {
        let vid = v as u32;
        let deg = graph.degree(vid)?;
        if deg < 2 {
            continue;
        }

        let neighbors = graph.neighbors(vid)?;
        let mut triangles = 0_usize;
        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                if graph.has_edge(neighbors[i], neighbors[j]) {
                    triangles += 1;
                }
            }
        }

        let pairs = deg * (deg - 1) / 2;
        sum += triangles as f64 / pairs as f64;
        count += 1;
    }

    if count == 0 {
        return Ok(0.0);
    }

    Ok(sum / count as f64)
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

    // --- self_loop_ratio ---

    #[test]
    fn slr_empty() {
        let g = Graph::with_vertices(0);
        assert!(self_loop_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn slr_isolated() {
        let g = Graph::with_vertices(5);
        assert!(self_loop_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn slr_single_edge() {
        assert!(self_loop_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn slr_k3() {
        assert!(self_loop_ratio(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn slr_k4() {
        assert!(self_loop_ratio(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn slr_cycle4() {
        assert!(self_loop_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn slr_star5() {
        assert!(self_loop_ratio(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn slr_with_loops() {
        // Graph with 1 self-loop and 2 normal edges → 1/3
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 0).unwrap();
        assert!((self_loop_ratio(&g).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn slr_all_loops() {
        // Graph with 2 self-loops only → 1.0
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(1, 1).unwrap();
        assert!((self_loop_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    // --- multi_edge_ratio ---

    #[test]
    fn mer_empty() {
        let g = Graph::with_vertices(0);
        assert!(multi_edge_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mer_isolated() {
        let g = Graph::with_vertices(5);
        assert!(multi_edge_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mer_single_edge() {
        assert!(multi_edge_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mer_k3() {
        assert!(multi_edge_ratio(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mer_k4() {
        assert!(multi_edge_ratio(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mer_cycle4() {
        assert!(multi_edge_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mer_with_multi() {
        // Graph: 0-1, 0-1 (dup), 1-2 → 1 multi / 3 edges = 1/3
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!((multi_edge_ratio(&g).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn mer_all_multi() {
        // 2 copies of the same edge → 1 multi / 2 edges = 0.5
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!((multi_edge_ratio(&g).unwrap() - 0.5).abs() < 1e-10);
    }

    // --- reciprocity_ratio ---

    #[test]
    fn rr_empty() {
        let g = Graph::with_vertices(0);
        assert!(reciprocity_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rr_isolated() {
        let g = Graph::with_vertices(5);
        assert!(reciprocity_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rr_undirected_always_one() {
        // Undirected → trivially reciprocal
        assert!((reciprocity_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((reciprocity_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rr_directed_full_reciprocal() {
        // Directed triangle with both directions → all reciprocal
        let g = Graph::from_edges(
            &[(0, 1), (1, 0), (1, 2), (2, 1), (0, 2), (2, 0)],
            true,
            Some(3),
        )
        .unwrap();
        assert!((reciprocity_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rr_directed_no_reciprocal() {
        // Directed cycle 0→1→2→0, no back edges
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], true, Some(3)).unwrap();
        assert!(reciprocity_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rr_directed_partial() {
        // 0→1, 1→0, 0→2 → 2 reciprocal / 3 edges = 2/3
        let g = Graph::from_edges(&[(0, 1), (1, 0), (0, 2)], true, Some(3)).unwrap();
        assert!((reciprocity_ratio(&g).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    // --- avg_local_clustering ---

    #[test]
    fn alc_empty() {
        let g = Graph::with_vertices(0);
        assert!(avg_local_clustering(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn alc_isolated() {
        let g = Graph::with_vertices(5);
        assert!(avg_local_clustering(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn alc_single_edge() {
        // Both vertices have d=1 < 2 → no eligible vertex → 0.0
        assert!(avg_local_clustering(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn alc_path3() {
        // v0: d=1 skip, v1: d=2, neighbors {0,2}, edge(0,2)? no → C=0, v2: d=1 skip
        // avg = 0/1 = 0
        assert!(avg_local_clustering(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn alc_k3() {
        // All d=2, each pair connected → C(v)=1 → avg=1.0
        assert!((avg_local_clustering(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn alc_k4() {
        // All d=3, C(3,2)=3 pairs, all connected → C(v)=1 → avg=1.0
        assert!((avg_local_clustering(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn alc_cycle4() {
        // Each vertex has d=2, 1 pair, no triangle → C(v)=0 → avg=0.0
        assert!(avg_local_clustering(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn alc_star5() {
        // Center d=4: C(4,2)=6 pairs, 0 triangles → C=0
        // Leaves d=1: skip
        // avg = 0/1 = 0
        assert!(avg_local_clustering(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn alc_paw() {
        // v0: d=2, neighbors {1,2}, edge(1,2)? yes → C=1/1=1
        // v1: d=2, neighbors {0,2}, edge(0,2)? yes → C=1/1=1
        // v2: d=3, neighbors {0,1,3}, pairs: (0,1),(0,3),(1,3)
        //   edge(0,1)? yes, edge(0,3)? no, edge(1,3)? no → C=1/3
        // v3: d=1, skip
        // avg = (1 + 1 + 1/3) / 3 = (7/3)/3 = 7/9
        assert!((avg_local_clustering(&paw()).unwrap() - 7.0 / 9.0).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn slr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = self_loop_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn mer_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = multi_edge_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn rr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = reciprocity_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn alc_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = avg_local_clustering(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn simple_graphs_no_loops_no_multi() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(self_loop_ratio(g).unwrap().abs() < 1e-10);
            assert!(multi_edge_ratio(g).unwrap().abs() < 1e-10);
        }
    }

    #[test]
    fn complete_graphs_full_clustering() {
        assert!((avg_local_clustering(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((avg_local_clustering(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }
}
