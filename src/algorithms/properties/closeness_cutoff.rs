//! Range-limited closeness centrality (ALGO-PR-047).
//!
//! Counterpart of `igraph_closeness_cutoff()` from
//! `references/igraph/src/centrality/closeness.c:324`.
//!
//! Computes closeness centrality considering only shortest paths whose
//! length is at most the given cutoff. Vertices beyond the cutoff are
//! treated as unreachable.

use crate::core::{Graph, IgraphResult, VertexId};
use std::collections::VecDeque;

/// Result of [`closeness_cutoff`].
#[derive(Debug, Clone)]
pub struct ClosenessCutoffResult {
    /// Per-vertex closeness score. `None` for vertices with no reachable
    /// neighbours within the cutoff distance.
    pub scores: Vec<Option<f64>>,
    /// Per-vertex count of vertices reachable within the cutoff (excluding self).
    pub reachable_count: Vec<u32>,
    /// Whether all vertices are reachable from every source within the cutoff.
    pub all_reachable: bool,
}

/// Range-limited closeness centrality.
///
/// Computes closeness centrality for all vertices, but only considering
/// paths of length at most `cutoff`. Shorter cutoffs make the computation
/// faster (early BFS termination) while capturing local structure.
///
/// The closeness of vertex `v` is `reachable_count / sum_of_distances`
/// when `normalized = true`, or `1.0 / sum_of_distances` when
/// `normalized = false`. Vertices with no reachable neighbours within
/// the cutoff get `None`.
///
/// # Parameters
///
/// * `graph` — the input graph.
/// * `cutoff` — maximum path length to consider. Paths longer than
///   this are ignored.
/// * `normalized` — if `true`, returns `reach / sum_dist`; if `false`,
///   returns `1.0 / sum_dist`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, closeness_cutoff};
///
/// // Path: 0—1—2—3—4
/// let mut g = Graph::with_vertices(5);
/// for i in 0..4 { g.add_edge(i, i + 1).unwrap(); }
/// let r = closeness_cutoff(&g, 2, true).unwrap();
/// // Vertex 0: reaches 1 (dist 1) and 2 (dist 2) within cutoff 2.
/// // closeness = 2 / (1+2) = 2/3
/// let expected = 2.0 / 3.0;
/// assert!((r.scores[0].unwrap() - expected).abs() < 1e-12);
/// assert_eq!(r.reachable_count[0], 2);
/// ```
pub fn closeness_cutoff(
    graph: &Graph,
    cutoff: u32,
    normalized: bool,
) -> IgraphResult<ClosenessCutoffResult> {
    let n = graph.vcount();
    let n_us = n as usize;
    let mut scores = Vec::with_capacity(n_us);
    let mut reachable_count = Vec::with_capacity(n_us);
    let mut all_reachable = true;

    for v in 0..n {
        let (sum_dist, reach) = bfs_cutoff(graph, v, cutoff)?;

        reachable_count.push(reach);

        if reach == 0 || sum_dist == 0 {
            scores.push(None);
        } else if normalized {
            #[allow(clippy::cast_precision_loss)]
            let val = f64::from(reach) / f64::from(sum_dist);
            scores.push(Some(val));
        } else {
            #[allow(clippy::cast_precision_loss)]
            let val = 1.0 / f64::from(sum_dist);
            scores.push(Some(val));
        }

        if reach < n.saturating_sub(1) {
            all_reachable = false;
        }
    }

    Ok(ClosenessCutoffResult {
        scores,
        reachable_count,
        all_reachable,
    })
}

/// BFS from `source` limited to `cutoff` hops.
/// Returns `(sum_of_distances, reachable_count)` excluding self.
fn bfs_cutoff(graph: &Graph, source: VertexId, cutoff: u32) -> IgraphResult<(u32, u32)> {
    let n = graph.vcount() as usize;
    let mut visited = vec![false; n];
    visited[source as usize] = true;

    let mut queue: VecDeque<(VertexId, u32)> = VecDeque::new();
    queue.push_back((source, 0));

    let mut sum_dist: u32 = 0;
    let mut reach: u32 = 0;

    while let Some((node, dist)) = queue.pop_front() {
        if dist > cutoff {
            continue;
        }

        if node != source {
            sum_dist = sum_dist.saturating_add(dist);
            reach = reach.saturating_add(1);
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

    Ok((sum_dist, reach))
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::core::Graph;

    #[test]
    fn empty_graph() {
        let g = Graph::new(0, false).unwrap();
        let r = closeness_cutoff(&g, 1, true).unwrap();
        assert!(r.scores.is_empty());
        assert!(r.all_reachable);
    }

    #[test]
    fn single_vertex() {
        let g = Graph::new(1, false).unwrap();
        let r = closeness_cutoff(&g, 1, true).unwrap();
        assert_eq!(r.scores[0], None);
        assert_eq!(r.reachable_count[0], 0);
        assert!(r.all_reachable);
    }

    #[test]
    fn disconnected() {
        let g = Graph::new(3, false).unwrap();
        let r = closeness_cutoff(&g, 5, true).unwrap();
        for s in &r.scores {
            assert_eq!(*s, None);
        }
        assert!(!r.all_reachable);
    }

    #[test]
    fn complete_graph_cutoff_1() {
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)])
            .unwrap();
        let r = closeness_cutoff(&g, 1, true).unwrap();
        // All vertices have distance 1 to all others.
        // reach=3, sum=3 → closeness = 3/3 = 1.0
        for s in &r.scores {
            assert_eq!(*s, Some(1.0));
        }
        assert!(r.all_reachable);
    }

    #[test]
    fn path_cutoff_1() {
        // 0—1—2—3
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let r = closeness_cutoff(&g, 1, true).unwrap();
        // Vertex 0: reaches only 1 → score = 1/1 = 1.0
        assert_eq!(r.scores[0], Some(1.0));
        assert_eq!(r.reachable_count[0], 1);
        // Vertex 1: reaches 0 and 2 → reach=2, sum=2 → 2/2=1.0
        assert_eq!(r.scores[1], Some(1.0));
        assert_eq!(r.reachable_count[1], 2);
        assert!(!r.all_reachable);
    }

    #[test]
    fn path_cutoff_2() {
        // 0—1—2—3—4
        let mut g = Graph::new(5, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3), (3, 4)]).unwrap();
        let r = closeness_cutoff(&g, 2, true).unwrap();
        // Vertex 0: reaches 1 (d=1), 2 (d=2) → reach=2, sum=3 → 2/3
        let expected = 2.0 / 3.0;
        assert!((r.scores[0].unwrap() - expected).abs() < 1e-12);
        assert_eq!(r.reachable_count[0], 2);
        assert!(!r.all_reachable);
    }

    #[test]
    fn large_cutoff_equals_closeness() {
        // With cutoff larger than diameter, results should equal regular closeness.
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let r = closeness_cutoff(&g, 100, true).unwrap();
        assert!(r.all_reachable);
        // Vertex 0: reaches 3, sum = 1+2+3=6 → 3/6 = 0.5
        assert!((r.scores[0].unwrap() - 0.5).abs() < 1e-12);
        // Vertex 1: reaches 3, sum = 1+1+2=4 → 3/4 = 0.75
        assert!((r.scores[1].unwrap() - 0.75).abs() < 1e-12);
    }

    #[test]
    fn normalized_false() {
        // 0—1—2
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let r = closeness_cutoff(&g, 10, false).unwrap();
        // Vertex 0: sum = 1+2=3 → 1/3
        let expected = 1.0 / 3.0;
        assert!((r.scores[0].unwrap() - expected).abs() < 1e-12);
        // Vertex 1: sum = 1+1=2 → 1/2
        assert!((r.scores[1].unwrap() - 0.5).abs() < 1e-12);
    }

    #[test]
    fn cutoff_zero() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let r = closeness_cutoff(&g, 0, true).unwrap();
        // With cutoff 0, no vertex can reach any other.
        for s in &r.scores {
            assert_eq!(*s, None);
        }
    }

    #[test]
    fn directed_graph() {
        // 0→1→2
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let r = closeness_cutoff(&g, 10, true).unwrap();
        // Vertex 0: reaches 1 (d=1), 2 (d=2) → reach=2, sum=3 → 2/3
        let expected = 2.0 / 3.0;
        assert!((r.scores[0].unwrap() - expected).abs() < 1e-12);
        // Vertex 2: can't reach anyone in directed mode
        assert_eq!(r.scores[2], None);
    }

    #[test]
    fn star_cutoff_1() {
        // Star: 0 connected to 1,2,3,4
        let mut g = Graph::new(5, false).unwrap();
        g.add_edges(vec![(0, 1), (0, 2), (0, 3), (0, 4)]).unwrap();
        let r = closeness_cutoff(&g, 1, true).unwrap();
        // Centre reaches all 4 at distance 1: reach=4, sum=4 → 1.0
        assert_eq!(r.scores[0], Some(1.0));
        assert_eq!(r.reachable_count[0], 4);
        // Leaves reach only centre at distance 1: reach=1, sum=1 → 1.0
        assert_eq!(r.scores[1], Some(1.0));
        assert_eq!(r.reachable_count[1], 1);
        assert!(!r.all_reachable);
    }
}
