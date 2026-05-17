//! Dijkstra single-source shortest distances (ALGO-SP-001).
//!
//! Counterpart of `igraph_distances_dijkstra()` from
//! `references/igraph/src/paths/dijkstra.c`. Phase-1 minimal slice:
//! single source, `IGRAPH_OUT` mode (i.e. on directed graphs we follow
//! out-edges; on undirected graphs both directions are walked
//! symmetrically because `Graph::neighbors` already merges the two
//! adjacency views). `IN` and `ALL` modes plus paths / parents / cutoff
//! variants ship later (SP-001b/c).
//!
//! All edge weights must be non-negative and finite; otherwise we
//! return [`IgraphError::InvalidArgument`].

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Min-heap entry. `Ord` is reversed so that `BinaryHeap` (a max-heap)
/// pops the smallest distance first. NaN distances are forbidden by the
/// public API contract — `total_cmp` is therefore safe.
#[derive(Copy, Clone)]
struct Frontier(f64, VertexId);

impl PartialEq for Frontier {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
impl Eq for Frontier {}
impl Ord for Frontier {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering: smaller distance is "greater" for the heap
        // so it gets popped first.
        other.0.total_cmp(&self.0).then(other.1.cmp(&self.1))
    }
}
impl PartialOrd for Frontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Single-source Dijkstra distances.
///
/// Returns `Vec<Option<f64>>` of length `vcount`: `result[v] = Some(d)`
/// if there is a path from `source` to `v` with weighted distance `d`,
/// `None` if `v` is unreachable. The source's own distance is
/// `Some(0.0)` whenever `source` is a valid vertex.
///
/// `weights[e]` is the weight of edge `e`; `weights.len()` must equal
/// `graph.ecount()`. All weights must be `>= 0` and finite (no NaN, no
/// infinity).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dijkstra_distances};
///
/// // Triangle 0-1-2 with weights 1, 4, 2 → dist(0→2) = 1+2 via 0-1-2
/// // (3.0) is shorter than the direct 0-2 edge weight 4.0.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();   // edge 0, weight 1
/// g.add_edge(0, 2).unwrap();   // edge 1, weight 4
/// g.add_edge(1, 2).unwrap();   // edge 2, weight 2
/// let d = dijkstra_distances(&g, 0, &[1.0, 4.0, 2.0]).unwrap();
/// assert_eq!(d, vec![Some(0.0), Some(1.0), Some(3.0)]);
/// ```
pub fn dijkstra_distances(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
) -> IgraphResult<Vec<Option<f64>>> {
    let n = graph.vcount();
    if source >= n {
        return Err(IgraphError::VertexOutOfRange { id: source, n });
    }
    let m = graph.ecount();
    if weights.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "weights vector size ({}) differs from edge count ({})",
            weights.len(),
            m
        )));
    }
    for (e, &w) in weights.iter().enumerate() {
        if w.is_nan() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is NaN"
            )));
        }
        if w < 0.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is negative ({w}); Dijkstra requires non-negative weights"
            )));
        }
        if !w.is_finite() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is not finite ({w})"
            )));
        }
    }

    let n_us = n as usize;
    let mut dist = vec![f64::INFINITY; n_us];
    let mut visited = vec![false; n_us];
    dist[source as usize] = 0.0;

    let mut heap: BinaryHeap<Frontier> = BinaryHeap::new();
    heap.push(Frontier(0.0, source));

    while let Some(Frontier(d, v)) = heap.pop() {
        let v_us = v as usize;
        if visited[v_us] {
            continue;
        }
        visited[v_us] = true;

        // Walk incident edges; for undirected, `incident` returns all;
        // for directed, only out-edges. (Phase-1 = OUT mode.)
        for eid in graph.incident(v)? {
            let w = weights[eid as usize];
            let other = graph.edge_other(eid as EdgeId, v)?;
            let nd = d + w;
            if nd < dist[other as usize] {
                dist[other as usize] = nd;
                heap.push(Frontier(nd, other));
            }
        }
    }

    Ok(dist
        .into_iter()
        .map(|d| if d.is_infinite() { None } else { Some(d) })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_source_out_of_range() {
        let g = Graph::with_vertices(0);
        assert!(dijkstra_distances(&g, 0, &[]).is_err());
    }

    #[test]
    fn singleton_distance_zero() {
        let g = Graph::with_vertices(1);
        assert_eq!(dijkstra_distances(&g, 0, &[]).unwrap(), vec![Some(0.0)]);
    }

    #[test]
    fn unreachable_yields_none() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let d = dijkstra_distances(&g, 0, &[1.0]).unwrap();
        assert_eq!(d, vec![Some(0.0), Some(1.0), None]);
    }

    #[test]
    fn shortcut_via_smaller_weights() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = dijkstra_distances(&g, 0, &[1.0, 4.0, 2.0]).unwrap();
        assert_eq!(d, vec![Some(0.0), Some(1.0), Some(3.0)]);
    }

    #[test]
    fn directed_respects_edge_direction() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 1).unwrap();
        let d = dijkstra_distances(&g, 0, &[1.0, 1.0]).unwrap();
        // 0 reaches 1 via direct edge; 2 has no incoming path from 0.
        assert_eq!(d, vec![Some(0.0), Some(1.0), None]);
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(dijkstra_distances(&g, 0, &[]).is_err());
        assert!(dijkstra_distances(&g, 0, &[1.0, 2.0]).is_err());
    }

    #[test]
    fn negative_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(dijkstra_distances(&g, 0, &[-1.0]).is_err());
    }

    #[test]
    fn nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(dijkstra_distances(&g, 0, &[f64::NAN]).is_err());
    }

    #[test]
    fn infinite_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(dijkstra_distances(&g, 0, &[f64::INFINITY]).is_err());
    }

    #[test]
    fn zero_weights_match_bfs_distance() {
        // All edges weight 1.0 reduces Dijkstra to BFS distances.
        let mut g = Graph::with_vertices(5);
        for u in 0..4u32 {
            g.add_edge(u, u + 1).unwrap();
        }
        let w = vec![1.0; 4];
        let d = dijkstra_distances(&g, 0, &w).unwrap();
        assert_eq!(
            d,
            vec![Some(0.0), Some(1.0), Some(2.0), Some(3.0), Some(4.0)]
        );
    }

    #[test]
    fn parallel_edges_pick_minimum_weight() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let d = dijkstra_distances(&g, 0, &[5.0, 1.5]).unwrap();
        assert_eq!(d, vec![Some(0.0), Some(1.5)]);
    }

    #[test]
    fn star_graph_distances() {
        // Star: vertex 0 is centre, vertices 1..=4 attached with weight i.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        let w = vec![1.0, 2.5, 0.5, 7.0];
        let d = dijkstra_distances(&g, 0, &w).unwrap();
        assert_eq!(
            d,
            vec![Some(0.0), Some(1.0), Some(2.5), Some(0.5), Some(7.0)]
        );
    }
}
