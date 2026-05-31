//! Weighted all-shortest-paths from a single source (`ALGO-SP-037`).
//!
//! Counterpart of `igraph_get_all_shortest_paths_dijkstra()` from
//! `references/igraph/src/paths/dijkstra.c:797-1228`.
//!
//! Returns every shortest path from a source vertex to all reachable
//! targets, using Dijkstra with multi-parent tracking.

use std::collections::BinaryHeap;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

use super::dijkstra::{DijkstraMode, incident_for_mode, validate_weights};

const EPSILON: f64 = 1e-10;

#[derive(PartialEq)]
struct Frontier(f64, VertexId);

impl Eq for Frontier {}

impl Ord for Frontier {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.0.total_cmp(&self.0).then(other.1.cmp(&self.1))
    }
}

impl PartialOrd for Frontier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Result of [`get_all_shortest_paths_dijkstra`].
#[derive(Debug, Clone)]
pub struct AllShortestPathsDijkstra {
    /// All shortest paths grouped by target vertex. `paths[v]` is the
    /// list of all shortest paths from the source to vertex `v`. If `v`
    /// is unreachable, `paths[v]` is empty. Each path includes both
    /// endpoints.
    pub paths: Vec<Vec<Vec<VertexId>>>,
    /// Edge paths grouped by target vertex. `edge_paths[v]` is the list
    /// of edge sequences along each shortest path from source to `v`.
    pub edge_paths: Vec<Vec<Vec<EdgeId>>>,
    /// Number of shortest paths to each vertex.
    pub nrgeo: Vec<u64>,
}

/// Find all shortest paths from `source` to every reachable vertex using
/// Dijkstra's algorithm with non-negative edge weights.
///
/// When `weights` is `None`, all edges are treated as having weight 1
/// (unweighted BFS-like traversal via Dijkstra).
///
/// For directed graphs, use [`get_all_shortest_paths_dijkstra_with_mode`]
/// to control direction.
///
/// # Errors
///
/// - [`IgraphError::VertexOutOfRange`] if `source >= vcount()`.
/// - [`IgraphError::InvalidArgument`] if weights contain negative, NaN,
///   or non-finite values, or if `weights.len() != ecount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_all_shortest_paths_dijkstra};
///
/// // Diamond: 0-1, 0-2, 1-3, 2-3. Weights: 1, 1, 1, 1.
/// // Two shortest paths from 0 to 3: 0→1→3 and 0→2→3, both cost 2.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let r = get_all_shortest_paths_dijkstra(&g, 0, &[1.0, 1.0, 1.0, 1.0]).unwrap();
/// assert_eq!(r.paths[3].len(), 2);
/// assert_eq!(r.nrgeo[3], 2);
/// ```
pub fn get_all_shortest_paths_dijkstra(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
) -> IgraphResult<AllShortestPathsDijkstra> {
    get_all_shortest_paths_dijkstra_with_mode(graph, source, weights, DijkstraMode::Out)
}

/// Find all shortest paths from `source` with direction control.
///
/// For undirected graphs, `mode` is ignored.
///
/// # Errors
///
/// Same as [`get_all_shortest_paths_dijkstra`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, DijkstraMode, get_all_shortest_paths_dijkstra_with_mode};
///
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let r = get_all_shortest_paths_dijkstra_with_mode(
///     &g, 0, &[1.0, 1.0, 1.0, 1.0], DijkstraMode::Out,
/// ).unwrap();
/// assert_eq!(r.paths[3].len(), 2);
/// ```
pub fn get_all_shortest_paths_dijkstra_with_mode(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<AllShortestPathsDijkstra> {
    let n = graph.vcount();
    if source >= n {
        return Err(IgraphError::VertexOutOfRange { id: source, n });
    }
    validate_weights(graph, weights)?;

    let n_us = n as usize;

    // Distance from source to each vertex (-1.0 means not yet reached).
    let mut dist = vec![-1.0_f64; n_us];
    // Parents: for each vertex, list of (parent_vertex, edge_used).
    let mut parents: Vec<Vec<(VertexId, EdgeId)>> = vec![Vec::new(); n_us];
    // Order in which vertices are settled.
    let mut order: Vec<VertexId> = Vec::with_capacity(n_us);

    let mut heap: BinaryHeap<Frontier> = BinaryHeap::new();
    dist[source as usize] = 0.0;
    heap.push(Frontier(0.0, source));

    let mut settled = vec![false; n_us];

    while let Some(Frontier(d, v)) = heap.pop() {
        let v_us = v as usize;
        if settled[v_us] {
            continue;
        }
        settled[v_us] = true;
        order.push(v);

        for eid in incident_for_mode(graph, v, mode)? {
            let w = weights[eid as usize];
            if !w.is_finite() {
                continue;
            }
            let other = graph.edge_other(eid, v)?;
            let other_us = other as usize;
            if settled[other_us] {
                continue;
            }
            let altdist = d + w;
            let curdist = dist[other_us];

            if curdist < 0.0 {
                // First time reaching this vertex.
                dist[other_us] = altdist;
                parents[other_us].push((v, eid));
                heap.push(Frontier(altdist, other));
            } else if (altdist - curdist).abs() < EPSILON && w > 0.0 {
                // Equal-cost alternative path (skip zero-weight edges to
                // avoid infinite loops in undirected graphs).
                parents[other_us].push((v, eid));
            } else if altdist < curdist - EPSILON {
                // Strictly shorter path found.
                dist[other_us] = altdist;
                parents[other_us].clear();
                parents[other_us].push((v, eid));
                heap.push(Frontier(altdist, other));
            }
        }
    }

    // Build nrgeo from parent tree in settlement order.
    let mut nrgeo = vec![0_u64; n_us];
    nrgeo[source as usize] = 1;
    for &v in order.iter().skip(1) {
        let v_us = v as usize;
        for &(pv, _) in &parents[v_us] {
            nrgeo[v_us] = nrgeo[v_us].saturating_add(nrgeo[pv as usize]);
        }
    }

    // Reconstruct all paths for every vertex in settlement order.
    let mut vertex_paths: Vec<Vec<Vec<VertexId>>> = vec![Vec::new(); n_us];
    let mut edge_seqs: Vec<Vec<Vec<EdgeId>>> = vec![Vec::new(); n_us];

    vertex_paths[source as usize] = vec![vec![source]];
    edge_seqs[source as usize] = vec![vec![]];

    for &v in order.iter().skip(1) {
        let v_us = v as usize;
        let parent_list: Vec<(VertexId, EdgeId)> = parents[v_us].clone();

        for &(pv, pe) in &parent_list {
            let pv_us = pv as usize;
            let prev_vertices: Vec<Vec<VertexId>> = vertex_paths[pv_us].clone();
            let prev_edges: Vec<Vec<EdgeId>> = edge_seqs[pv_us].clone();

            for mut vpath in prev_vertices {
                vpath.push(v);
                vertex_paths[v_us].push(vpath);
            }
            for mut epath in prev_edges {
                epath.push(pe);
                edge_seqs[v_us].push(epath);
            }
        }
    }

    Ok(AllShortestPathsDijkstra {
        paths: vertex_paths,
        edge_paths: edge_seqs,
        nrgeo,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diamond() -> Graph {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).expect("e01");
        g.add_edge(0, 2).expect("e02");
        g.add_edge(1, 3).expect("e13");
        g.add_edge(2, 3).expect("e23");
        g
    }

    #[test]
    fn diamond_equal_weights() {
        let g = diamond();
        let w = vec![1.0; 4];
        let r = get_all_shortest_paths_dijkstra(&g, 0, &w).expect("ok");

        assert_eq!(r.paths[0], vec![vec![0]]);
        assert_eq!(r.nrgeo[0], 1);
        assert_eq!(r.paths[1].len(), 1);
        assert_eq!(r.paths[2].len(), 1);
        assert_eq!(r.paths[3].len(), 2);
        assert_eq!(r.nrgeo[3], 2);

        let mut sorted = r.paths[3].clone();
        sorted.sort();
        assert_eq!(sorted, vec![vec![0, 1, 3], vec![0, 2, 3]]);
    }

    #[test]
    fn diamond_asymmetric_weights() {
        let g = diamond();
        // 0→1 cost 1, 0→2 cost 10, 1→3 cost 1, 2→3 cost 1
        // Only one shortest path to 3: 0→1→3 cost 2.
        let w = vec![1.0, 10.0, 1.0, 1.0];
        let r = get_all_shortest_paths_dijkstra(&g, 0, &w).expect("ok");
        assert_eq!(r.paths[3].len(), 1);
        assert_eq!(r.paths[3][0], vec![0, 1, 3]);
        assert_eq!(r.nrgeo[3], 1);
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let r = get_all_shortest_paths_dijkstra(&g, 0, &[]).expect("ok");
        assert_eq!(r.paths[0], vec![vec![0]]);
        assert_eq!(r.nrgeo[0], 1);
    }

    #[test]
    fn disconnected_graph() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).expect("e");
        // Vertex 2 unreachable.
        let r = get_all_shortest_paths_dijkstra(&g, 0, &[1.0]).expect("ok");
        assert!(r.paths[2].is_empty());
        assert_eq!(r.nrgeo[2], 0);
    }

    #[test]
    fn directed_out_mode() {
        let mut g = Graph::new(3, true).expect("g");
        g.add_edge(0, 1).expect("e01");
        g.add_edge(1, 2).expect("e12");
        g.add_edge(2, 0).expect("e20");
        let w = vec![1.0; 3];
        let r =
            get_all_shortest_paths_dijkstra_with_mode(&g, 0, &w, DijkstraMode::Out).expect("ok");
        assert_eq!(r.paths[2].len(), 1);
        assert_eq!(r.paths[2][0], vec![0, 1, 2]);
    }

    #[test]
    fn directed_in_mode() {
        let mut g = Graph::new(3, true).expect("g");
        g.add_edge(0, 1).expect("e01");
        g.add_edge(1, 2).expect("e12");
        g.add_edge(2, 0).expect("e20");
        let w = vec![1.0; 3];
        let r = get_all_shortest_paths_dijkstra_with_mode(&g, 2, &w, DijkstraMode::In).expect("ok");
        assert_eq!(r.paths[0].len(), 1);
        assert_eq!(r.paths[0][0], vec![2, 1, 0]);
    }

    #[test]
    fn edge_paths_match_vertex_paths() {
        let g = diamond();
        let w = vec![1.0; 4];
        let r = get_all_shortest_paths_dijkstra(&g, 0, &w).expect("ok");

        for v in 0..4u32 {
            let v_us = v as usize;
            assert_eq!(r.paths[v_us].len(), r.edge_paths[v_us].len());
            for (vp, ep) in r.paths[v_us].iter().zip(r.edge_paths[v_us].iter()) {
                if vp.len() > 1 {
                    assert_eq!(ep.len(), vp.len() - 1);
                } else {
                    assert!(ep.is_empty());
                }
            }
        }
    }

    #[test]
    fn negative_weight_error() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).expect("e");
        let r = get_all_shortest_paths_dijkstra(&g, 0, &[-1.0]);
        assert!(r.is_err());
    }

    #[test]
    fn nan_weight_error() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).expect("e");
        let r = get_all_shortest_paths_dijkstra(&g, 0, &[f64::NAN]);
        assert!(r.is_err());
    }

    #[test]
    fn source_out_of_range_error() {
        let g = Graph::with_vertices(3);
        let r = get_all_shortest_paths_dijkstra(&g, 5, &[]);
        assert!(r.is_err());
    }

    #[test]
    fn ring_multiple_shortest_paths() {
        // Ring: 0-1-2-3-4-0, all weight 1.
        // From 0 to 2: two paths of length 2: 0→1→2 and 0→4→3→... wait no.
        // Ring 0-1-2-3-4-0: from 0 to 2, either 0→1→2 (cost 2) or
        // 0→4→3→2 (cost 3). Only one shortest path. But from 0 to vertex
        // on the opposite side of even ring: let's use ring of 6.
        let mut g = Graph::with_vertices(6);
        for i in 0..6u32 {
            g.add_edge(i, (i + 1) % 6).expect("edge");
        }
        let w = vec![1.0; 6];
        // From 0 to 3: 0→1→2→3 (cost 3) and 0→5→4→3 (cost 3).
        let r = get_all_shortest_paths_dijkstra(&g, 0, &w).expect("ok");
        assert_eq!(r.paths[3].len(), 2);
        assert_eq!(r.nrgeo[3], 2);
    }

    #[test]
    fn grid_multiple_paths() {
        // 2x3 grid: 0-1-2 / 3-4-5, connections 0-3, 1-4, 2-5
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).expect("e"); // 0
        g.add_edge(1, 2).expect("e"); // 1
        g.add_edge(3, 4).expect("e"); // 2
        g.add_edge(4, 5).expect("e"); // 3
        g.add_edge(0, 3).expect("e"); // 4
        g.add_edge(1, 4).expect("e"); // 5
        g.add_edge(2, 5).expect("e"); // 6
        let w = vec![1.0; 7];
        // From 0 to 5: all shortest have length 3.
        // 0→1→2→5, 0→1→4→5, 0→3→4→5 — 3 paths.
        let r = get_all_shortest_paths_dijkstra(&g, 0, &w).expect("ok");
        assert_eq!(r.paths[5].len(), 3);
        assert_eq!(r.nrgeo[5], 3);
    }

    #[test]
    fn zero_weight_does_not_create_duplicate_parents() {
        // Zero-weight edges: ensure we don't add duplicate parents.
        // 0→1 (w=0), 1→2 (w=1).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).expect("e01");
        g.add_edge(1, 2).expect("e12");
        let w = vec![0.0, 1.0];
        let r = get_all_shortest_paths_dijkstra(&g, 0, &w).expect("ok");
        assert_eq!(r.paths[1].len(), 1);
        assert_eq!(r.paths[2].len(), 1);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
#[allow(clippy::cast_possible_truncation)]
mod proptests {
    use super::*;
    use crate::algorithms::paths::all_shortest_paths::{
        AllShortestPathsMode, get_all_shortest_paths_with_mode,
    };
    use proptest::prelude::*;

    fn arb_small_graph() -> impl Strategy<Value = (Graph, Vec<f64>)> {
        (3..12u32)
            .prop_flat_map(|n| {
                let max_edges = n * (n - 1) / 2;
                let n_edges = 0..max_edges.min(20);
                (Just(n), n_edges)
            })
            .prop_flat_map(|(n, m)| {
                let edges = proptest::collection::vec((0..n, 0..n), m as usize);
                (Just(n), edges)
            })
            .prop_flat_map(|(n, edges)| {
                let n_e = edges.len();
                let weights = proptest::collection::vec(1.0..10.0_f64, n_e);
                (Just(n), Just(edges), weights)
            })
            .prop_map(|(n, edges, weights)| {
                let mut g = Graph::with_vertices(n);
                let mut actual_weights = Vec::new();
                for (i, &(u, v)) in edges.iter().enumerate() {
                    if u != v && g.add_edge(u, v).is_ok() {
                        actual_weights.push(weights[i]);
                    }
                }
                (g, actual_weights)
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn nrgeo_matches_path_count((g, w) in arb_small_graph()) {
            if w.len() != g.ecount() { return Ok(()); }
            let r = get_all_shortest_paths_dijkstra(&g, 0, &w)?;
            for v in 0..g.vcount() as usize {
                prop_assert_eq!(
                    r.nrgeo[v] as usize,
                    r.paths[v].len(),
                    "nrgeo mismatch at vertex {}",
                    v
                );
            }
        }

        #[test]
        fn unit_weights_match_unweighted((g, _) in arb_small_graph()) {
            let unit_w = vec![1.0; g.ecount()];
            let weighted = get_all_shortest_paths_dijkstra(&g, 0, &unit_w)?;
            let unweighted = get_all_shortest_paths_with_mode(
                &g, 0, AllShortestPathsMode::All,
            )?;
            for v in 0..g.vcount() as usize {
                prop_assert_eq!(
                    weighted.nrgeo[v],
                    unweighted.nrgeo[v],
                    "nrgeo mismatch at vertex {}",
                    v
                );
            }
        }

        #[test]
        fn paths_are_valid_walks((g, w) in arb_small_graph()) {
            if w.len() != g.ecount() { return Ok(()); }
            let r = get_all_shortest_paths_dijkstra(&g, 0, &w)?;
            for v in 0..g.vcount() {
                let v_us = v as usize;
                for (pi, vpath) in r.paths[v_us].iter().enumerate() {
                    prop_assert!(!vpath.is_empty());
                    prop_assert_eq!(vpath[0], 0, "path {} to {} doesn't start at source", pi, v);
                    prop_assert_eq!(*vpath.last().unwrap(), v, "path {} doesn't end at {}", pi, v);

                    let epath = &r.edge_paths[v_us][pi];
                    prop_assert_eq!(epath.len(), vpath.len().saturating_sub(1));

                    for (i, &eid) in epath.iter().enumerate() {
                        let (eu, ev) = g.edge(eid).unwrap();
                        let pv = vpath[i];
                        let pv_next = vpath[i + 1];
                        let connects = (eu == pv && ev == pv_next)
                            || (eu == pv_next && ev == pv);
                        prop_assert!(
                            connects,
                            "edge {} ({},{}) doesn't connect path vertices {} and {}",
                            eid, eu, ev, pv, pv_next
                        );
                    }
                }
            }
        }
    }
}
