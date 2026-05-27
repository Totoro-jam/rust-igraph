//! All-pairs shortest path length histogram (ALGO-SP-012).
//!
//! Counterpart of `igraph_path_length_hist` from
//! `references/igraph/src/paths/histogram.c` (141 lines).
//!
//! BFS from every vertex, counting vertex pairs by shortest-path
//! distance. Directed graphs can be treated either directed
//! (`IGRAPH_OUT` mode) or undirected (`IGRAPH_ALL` mode). For
//! undirected treatment each unordered pair is counted once; for
//! directed treatment each ordered pair is counted.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphResult, VertexId};

/// Result of [`path_length_hist`].
///
/// `hist[d]` is the number of vertex pairs whose shortest-path distance
/// is `d + 1` (so `hist[0]` counts pairs at distance 1, `hist[1]` at
/// distance 2, etc.). For undirected treatment each *unordered* pair is
/// counted once; for directed each *ordered* pair.
///
/// `unconnected` is the number of (ordered or unordered, matching above)
/// pairs for which no path exists.
#[derive(Debug, Clone)]
pub struct PathLengthHistResult {
    /// `hist[d]` = count of vertex pairs at distance `d + 1`.
    pub hist: Vec<f64>,
    /// Number of vertex pairs with no connecting path.
    pub unconnected: f64,
}

/// Compute a histogram of all shortest-path lengths.
///
/// For every vertex pair `(u, v)` the function determines the
/// shortest-path length and adds it to the histogram. Self-loops and
/// multi-edges are traversed by the BFS; self-pairs (distance 0) are
/// *not* counted.
///
/// When `directed` is `false` (or the graph is undirected), both edge
/// endpoints participate symmetrically and each *unordered* pair is
/// counted once. When `directed` is `true` (on a directed graph),
/// traversal follows out-edges and each *ordered* pair is counted.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, path_length_hist};
///
/// // Undirected path 0-1-2.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let r = path_length_hist(&g, false).unwrap();
/// // Pairs at distance 1: {0,1}, {1,2} → 2
/// // Pairs at distance 2: {0,2}         → 1
/// assert_eq!(r.hist, vec![2.0, 1.0]);
/// assert_eq!(r.unconnected, 0.0);
/// ```
pub fn path_length_hist(graph: &Graph, directed: bool) -> IgraphResult<PathLengthHistResult> {
    let n = graph.vcount();
    let n_us = n as usize;

    let effective_directed = directed && graph.is_directed();

    if n == 0 {
        return Ok(PathLengthHistResult {
            hist: Vec::new(),
            unconnected: 0.0,
        });
    }

    let mut hist: Vec<f64> = Vec::new();
    let mut unconnected: f64 = 0.0;

    let mut visited: Vec<u32> = vec![0; n_us];
    let mut queue: VecDeque<(VertexId, u32)> = VecDeque::new();

    for i in 0..n {
        let marker = i
            .checked_add(1)
            .ok_or(crate::core::IgraphError::Internal("vertex marker overflow"))?;
        let mut nodes_reached: u32 = 1;

        queue.clear();
        queue.push_back((i, 0));
        visited[i as usize] = marker;

        while let Some((v, dist)) = queue.pop_front() {
            let neis = neighbors_for_mode(graph, v, effective_directed)?;
            for nb in neis {
                if visited[nb as usize] == marker {
                    continue;
                }
                visited[nb as usize] = marker;
                nodes_reached = nodes_reached
                    .checked_add(1)
                    .ok_or(crate::core::IgraphError::Internal("nodes_reached overflow"))?;

                let idx = dist as usize;
                if idx >= hist.len() {
                    hist.resize(idx + 1, 0.0);
                }
                hist[idx] += 1.0;

                queue.push_back((
                    nb,
                    dist.checked_add(1)
                        .ok_or(crate::core::IgraphError::Internal("BFS distance overflow"))?,
                ));
            }
        }

        unconnected += f64::from(n - nodes_reached);
    }

    if !effective_directed {
        for h in &mut hist {
            *h /= 2.0;
        }
        unconnected /= 2.0;
    }

    Ok(PathLengthHistResult { hist, unconnected })
}

fn neighbors_for_mode(graph: &Graph, v: VertexId, directed: bool) -> IgraphResult<Vec<VertexId>> {
    if directed {
        graph.out_neighbors_vec(v)
    } else if graph.is_directed() {
        let mut out = graph.out_neighbors_vec(v)?;
        let in_neis = graph.in_neighbors_vec(v)?;
        out.extend(in_neis);
        Ok(out)
    } else {
        graph.neighbors(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let r = path_length_hist(&g, false).expect("ok");
        assert!(r.hist.is_empty());
        assert!(approx_eq(r.unconnected, 0.0));
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let r = path_length_hist(&g, false).expect("ok");
        assert!(r.hist.is_empty());
        assert!(approx_eq(r.unconnected, 0.0));
    }

    #[test]
    fn two_isolated_vertices() {
        let g = Graph::with_vertices(2);
        let r = path_length_hist(&g, false).expect("ok");
        assert!(r.hist.is_empty());
        assert!(approx_eq(r.unconnected, 1.0));
    }

    #[test]
    fn single_edge_undirected() {
        let g = create(&[(0, 1)], 2, false).expect("ok");
        let r = path_length_hist(&g, false).expect("ok");
        assert_eq!(r.hist, vec![1.0]);
        assert!(approx_eq(r.unconnected, 0.0));
    }

    #[test]
    fn path_3_undirected() {
        // 0-1-2: pairs at dist 1 = 2 ({0,1},{1,2}); dist 2 = 1 ({0,2})
        let g = create(&[(0, 1), (1, 2)], 3, false).expect("ok");
        let r = path_length_hist(&g, false).expect("ok");
        assert_eq!(r.hist, vec![2.0, 1.0]);
        assert!(approx_eq(r.unconnected, 0.0));
    }

    #[test]
    fn triangle_undirected() {
        // 0-1, 1-2, 0-2: all pairs at distance 1
        let g = create(&[(0, 1), (1, 2), (0, 2)], 3, false).expect("ok");
        let r = path_length_hist(&g, false).expect("ok");
        assert_eq!(r.hist, vec![3.0]);
        assert!(approx_eq(r.unconnected, 0.0));
    }

    #[test]
    fn star_4_undirected() {
        // 0-1, 0-2, 0-3: dist 1 = 3 ({0,1},{0,2},{0,3}); dist 2 = 3
        // ({1,2},{1,3},{2,3})
        let g = create(&[(0, 1), (0, 2), (0, 3)], 4, false).expect("ok");
        let r = path_length_hist(&g, false).expect("ok");
        assert_eq!(r.hist, vec![3.0, 3.0]);
        assert!(approx_eq(r.unconnected, 0.0));
    }

    #[test]
    fn disconnected_components_undirected() {
        // {0-1} + {2-3}: 2 pairs connected at dist 1, 4 unconnected pairs
        let g = create(&[(0, 1), (2, 3)], 4, false).expect("ok");
        let r = path_length_hist(&g, false).expect("ok");
        assert_eq!(r.hist, vec![2.0]);
        assert!(approx_eq(r.unconnected, 4.0));
    }

    #[test]
    fn directed_path_out_mode() {
        // 0->1->2 directed
        let g = create(&[(0, 1), (1, 2)], 3, true).expect("ok");
        let r = path_length_hist(&g, true).expect("ok");
        // Ordered pairs reachable: (0,1)=1, (0,2)=2, (1,2)=1
        // hist[0] (dist 1) = 2, hist[1] (dist 2) = 1
        assert_eq!(r.hist, vec![2.0, 1.0]);
        // Unreachable ordered pairs: (1,0), (2,0), (2,1) = 3
        assert!(approx_eq(r.unconnected, 3.0));
    }

    #[test]
    fn directed_path_undirected_mode() {
        // 0->1->2 directed but treated as undirected
        let g = create(&[(0, 1), (1, 2)], 3, true).expect("ok");
        let r = path_length_hist(&g, false).expect("ok");
        // Same as undirected path 0-1-2
        assert_eq!(r.hist, vec![2.0, 1.0]);
        assert!(approx_eq(r.unconnected, 0.0));
    }

    #[test]
    fn directed_cycle() {
        // 0->1->2->0
        let g = create(&[(0, 1), (1, 2), (2, 0)], 3, true).expect("ok");
        let r = path_length_hist(&g, true).expect("ok");
        // Every ordered pair reachable:
        // (0,1)=1, (1,2)=1, (2,0)=1, (0,2)=2, (1,0)=2, (2,1)=2
        // hist[0] (dist 1) = 3, hist[1] (dist 2) = 3
        assert_eq!(r.hist, vec![3.0, 3.0]);
        assert!(approx_eq(r.unconnected, 0.0));
    }

    #[test]
    fn self_loop_ignored_in_counting() {
        let g = create(&[(0, 0), (0, 1)], 2, false).expect("ok");
        let r = path_length_hist(&g, false).expect("ok");
        assert_eq!(r.hist, vec![1.0]);
        assert!(approx_eq(r.unconnected, 0.0));
    }

    #[test]
    fn multi_edge_does_not_double_count() {
        let g = create(&[(0, 1), (0, 1), (1, 2)], 3, false).expect("ok");
        let r = path_length_hist(&g, false).expect("ok");
        assert_eq!(r.hist, vec![2.0, 1.0]);
        assert!(approx_eq(r.unconnected, 0.0));
    }

    #[test]
    fn k4_undirected() {
        // K4: all 6 pairs at distance 1
        let g = create(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)], 4, false).expect("ok");
        let r = path_length_hist(&g, false).expect("ok");
        assert_eq!(r.hist, vec![6.0]);
        assert!(approx_eq(r.unconnected, 0.0));
    }
}

#[cfg(feature = "proptest-harness")]
mod proptests {
    use super::*;
    use crate::create;
    use proptest::prelude::*;

    fn arb_graph(max_v: u32) -> impl Strategy<Value = (Graph, bool)> {
        (1..=max_v).prop_flat_map(|n| {
            let max_edges = (n as usize).saturating_mul(n.saturating_sub(1) as usize);
            let max_e = max_edges.min(20);
            (
                proptest::collection::vec((0..n, 0..n), 0..=max_e),
                Just(n),
                proptest::bool::ANY,
            )
                .prop_map(|(edges, n, directed)| {
                    let edge_tuples: Vec<(u32, u32)> = edges.into_iter().collect();
                    let g = create(&edge_tuples, n, directed).expect("arb graph creation");
                    (g, directed)
                })
        })
    }

    proptest! {
        #[test]
        fn hist_counts_are_non_negative((g, directed) in arb_graph(10)) {
            let r = path_length_hist(&g, directed).expect("ok");
            for &h in &r.hist {
                prop_assert!(h >= 0.0, "negative histogram count: {h}");
            }
            prop_assert!(r.unconnected >= 0.0);
        }

        #[test]
        fn total_pairs_equals_expected((g, directed) in arb_graph(10)) {
            let r = path_length_hist(&g, directed).expect("ok");
            let n = g.vcount() as f64;
            let total: f64 = r.hist.iter().sum::<f64>() + r.unconnected;
            let effective_directed = directed && g.is_directed();
            let expected = if effective_directed {
                n * (n - 1.0)
            } else {
                n * (n - 1.0) / 2.0
            };
            prop_assert!(
                (total - expected).abs() < 1e-9,
                "total {total} != expected {expected}"
            );
        }

        #[test]
        fn undirected_mode_on_undirected_graph_halves_directed(
            edges in proptest::collection::vec((0u32..8, 0u32..8), 0..=15)
        ) {
            let g = create(&edges, 8, false).expect("ok");
            let r_undir = path_length_hist(&g, false).expect("ok");
            let r_dir = path_length_hist(&g, true).expect("ok");
            // For undirected graph, directed=true is forced to false internally,
            // so results should be identical.
            prop_assert_eq!(r_undir.hist.len(), r_dir.hist.len());
            for (a, b) in r_undir.hist.iter().zip(r_dir.hist.iter()) {
                prop_assert!((a - b).abs() < 1e-9);
            }
            prop_assert!((r_undir.unconnected - r_dir.unconnected).abs() < 1e-9);
        }
    }
}
