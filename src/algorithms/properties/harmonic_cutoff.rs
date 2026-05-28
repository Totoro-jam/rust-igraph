//! Range-limited harmonic centrality (ALGO-PR-048).
//!
//! Counterpart of `igraph_harmonic_centrality_cutoff()` from
//! `references/igraph/src/centrality/closeness.c:722`.
//!
//! Computes harmonic centrality considering only shortest paths whose
//! length is at most the given cutoff. The inverse distance to vertices
//! not reachable within the cutoff is considered to be zero.

use crate::core::{Graph, IgraphResult, VertexId};
use std::collections::VecDeque;

/// Range-limited harmonic centrality.
///
/// Computes the sum (or mean) of inverse distances to vertices reachable
/// within `cutoff` hops. Vertices beyond the cutoff contribute 0, same as
/// unreachable vertices.
///
/// When `normalized = true`, returns `(1/(n-1)) * sum(1/d)` (mean inverse
/// distance). When `normalized = false`, returns the raw `sum(1/d)`.
///
/// For graphs with `vcount < 2`, returns zeros (the normalization factor
/// `n-1` would be 0).
///
/// # Parameters
///
/// * `graph` — the input graph.
/// * `cutoff` — maximum path length to consider.
/// * `normalized` — if `true`, divides sum by `n - 1`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, harmonic_centrality_cutoff};
///
/// // Path: 0—1—2—3—4
/// let mut g = Graph::with_vertices(5);
/// for i in 0..4 { g.add_edge(i, i + 1).unwrap(); }
/// let h = harmonic_centrality_cutoff(&g, 2, true).unwrap();
/// // Vertex 0: reaches 1 (d=1) and 2 (d=2) within cutoff 2.
/// // sum_inv = 1/1 + 1/2 = 1.5, normalized by (5-1)=4 → 0.375
/// let expected = 1.5 / 4.0;
/// assert!((h[0] - expected).abs() < 1e-12);
/// ```
pub fn harmonic_centrality_cutoff(
    graph: &Graph,
    cutoff: u32,
    normalized: bool,
) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;

    if n < 2 {
        return Ok(vec![0.0; n_us]);
    }

    let denom = f64::from(n - 1);
    let mut out = Vec::with_capacity(n_us);

    for v in 0..n {
        let sum_inv = bfs_harmonic_cutoff(graph, v, cutoff)?;
        if normalized {
            out.push(sum_inv / denom);
        } else {
            out.push(sum_inv);
        }
    }

    Ok(out)
}

/// BFS from `source` limited to `cutoff` hops.
/// Returns sum of `1/distance` for all reachable vertices (excluding self).
fn bfs_harmonic_cutoff(graph: &Graph, source: VertexId, cutoff: u32) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let mut visited = vec![false; n];
    visited[source as usize] = true;

    let mut queue: VecDeque<(VertexId, u32)> = VecDeque::new();
    queue.push_back((source, 0));

    let mut sum_inv: f64 = 0.0;

    while let Some((node, dist)) = queue.pop_front() {
        if dist > cutoff {
            continue;
        }

        if node != source && dist > 0 {
            sum_inv += 1.0 / f64::from(dist);
        }

        if dist == cutoff {
            continue;
        }

        for &neighbor in &graph.neighbors(node)? {
            if !visited[neighbor as usize] {
                visited[neighbor as usize] = true;
                queue.push_back((neighbor, dist.saturating_add(1)));
            }
        }
    }

    Ok(sum_inv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Graph;

    #[test]
    fn empty_graph() {
        let g = Graph::new(0, false).unwrap();
        let h = harmonic_centrality_cutoff(&g, 1, true).unwrap();
        assert!(h.is_empty());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::new(1, false).unwrap();
        let h = harmonic_centrality_cutoff(&g, 1, true).unwrap();
        assert_eq!(h, vec![0.0]);
    }

    #[test]
    fn disconnected() {
        let g = Graph::new(3, false).unwrap();
        let h = harmonic_centrality_cutoff(&g, 5, true).unwrap();
        assert_eq!(h, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn complete_graph_cutoff_1() {
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)])
            .unwrap();
        let h = harmonic_centrality_cutoff(&g, 1, true).unwrap();
        // All vertices at distance 1 → sum_inv=3, normalized by 3 → 1.0
        for &val in &h {
            assert!((val - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn path_cutoff_1() {
        // 0—1—2—3
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let h = harmonic_centrality_cutoff(&g, 1, true).unwrap();
        // Vertex 0: reaches 1 at d=1 → sum_inv=1.0, /3 = 1/3
        let expected = 1.0 / 3.0;
        assert!((h[0] - expected).abs() < 1e-12);
        // Vertex 1: reaches 0,2 at d=1 → sum_inv=2.0, /3 = 2/3
        let expected1 = 2.0 / 3.0;
        assert!((h[1] - expected1).abs() < 1e-12);
    }

    #[test]
    fn path_cutoff_2() {
        // 0—1—2—3—4
        let mut g = Graph::new(5, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3), (3, 4)]).unwrap();
        let h = harmonic_centrality_cutoff(&g, 2, true).unwrap();
        // Vertex 0: reaches 1(d=1), 2(d=2) → sum_inv=1+0.5=1.5, /4=0.375
        let expected = 1.5 / 4.0;
        assert!((h[0] - expected).abs() < 1e-12);
    }

    #[test]
    fn large_cutoff_matches_harmonic() {
        // With cutoff > diameter, should equal regular harmonic centrality.
        // Path 0—1—2
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let h = harmonic_centrality_cutoff(&g, 100, true).unwrap();
        // Vertex 0: 1/1 + 1/2 = 1.5, /2 = 0.75
        assert!((h[0] - 0.75).abs() < 1e-12);
        // Vertex 1: 1/1 + 1/1 = 2.0, /2 = 1.0
        assert!((h[1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn normalized_false() {
        // 0—1—2
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let h = harmonic_centrality_cutoff(&g, 10, false).unwrap();
        // Vertex 0: 1/1 + 1/2 = 1.5
        assert!((h[0] - 1.5).abs() < 1e-12);
        // Vertex 1: 1/1 + 1/1 = 2.0
        assert!((h[1] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn cutoff_zero() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let h = harmonic_centrality_cutoff(&g, 0, true).unwrap();
        // No vertices reachable → all zeros
        assert_eq!(h, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn directed_graph() {
        // 0→1→2
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let h = harmonic_centrality_cutoff(&g, 10, true).unwrap();
        // Vertex 0: reaches 1(d=1), 2(d=2) → (1+0.5)/2 = 0.75
        assert!((h[0] - 0.75).abs() < 1e-12);
        // Vertex 2: can't reach anyone → 0.0
        assert!((h[2] - 0.0).abs() < 1e-12);
    }

    #[test]
    fn star_cutoff_1() {
        // Star: 0 connected to 1,2,3,4
        let mut g = Graph::new(5, false).unwrap();
        g.add_edges(vec![(0, 1), (0, 2), (0, 3), (0, 4)]).unwrap();
        let h = harmonic_centrality_cutoff(&g, 1, true).unwrap();
        // Centre: reaches 4 vertices at d=1 → sum=4, /4 = 1.0
        assert!((h[0] - 1.0).abs() < 1e-12);
        // Leaf: reaches only centre at d=1 → sum=1, /4 = 0.25
        assert!((h[1] - 0.25).abs() < 1e-12);
    }
}
