//! ALGO-PR-027 / PR-027b — BFS-based k-hop neighbourhoods.
//!
//! - [`neighborhood_size`] / [`neighborhood_size_with_mode`] (PR-027):
//!   for every vertex `v` return *how many* vertices `w` satisfy
//!   `mindist <= dist(v, w) <= order`.
//! - [`neighborhood`] / [`neighborhood_with_mode`] (PR-027b): for every
//!   vertex `v` return *the list of* such vertices, in BFS visitation
//!   order (matching `igraph_neighborhood()` from
//!   `references/igraph/src/properties/neighborhood.c:208-303`).
//!
//! `dist` is unweighted graph distance; `order < 0` is treated as
//! infinity; `mode` is ignored on undirected graphs.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Direction mode for `neighborhood_size_with_mode` on directed graphs.
/// Ignored on undirected graphs — every mode reduces to [`NeighborhoodMode::All`].
///
/// Counterpart of `igraph_neimode_t` (`include/igraph_constants.h`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighborhoodMode {
    /// Follow outgoing edges only (`IGRAPH_OUT`). For each source `v`,
    /// counts vertices reachable by following out-edges.
    Out,
    /// Follow incoming edges only (`IGRAPH_IN`). For each source `v`,
    /// counts vertices that can reach `v` by following out-edges (i.e.
    /// reachable from `v` along reversed edges).
    In,
    /// Ignore direction — treat every edge as bidirectional
    /// (`IGRAPH_ALL`).
    All,
}

/// k-hop neighbourhood size for every vertex (`mode = All`, `mindist = 0`).
///
/// For each vertex `v` returns the number of vertices within `order`
/// hops (inclusive), counting `v` itself. Negative `order` means
/// infinity (every reachable vertex is counted).
///
/// Counterpart of `igraph_neighborhood_size(graph, _, igraph_vss_all(),
/// order, IGRAPH_ALL, /*mindist=*/0)`.
///
/// # Errors
/// - [`IgraphError::InvalidArgument`] if `order >= 0` but `order` cannot
///   be represented as a non-negative integer (always satisfied for
///   `i32 >= 0`, so this can only fail via the with-mode variant when
///   `mindist > order`).
///
/// # Examples
/// ```
/// use rust_igraph::{Graph, neighborhood_size};
///
/// // Path P5: 0-1-2-3-4
/// let mut g = Graph::with_vertices(5);
/// for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 4)] {
///     g.add_edge(u, v).unwrap();
/// }
/// // Order 1: self + immediate neighbours.
/// assert_eq!(neighborhood_size(&g, 1).unwrap(), vec![2, 3, 3, 3, 2]);
/// // Order 2: self + 2-hop ball.
/// assert_eq!(neighborhood_size(&g, 2).unwrap(), vec![3, 4, 5, 4, 3]);
/// ```
pub fn neighborhood_size(graph: &Graph, order: i32) -> IgraphResult<Vec<u32>> {
    neighborhood_size_with_mode(graph, order, NeighborhoodMode::All, 0)
}

/// Full mode-aware k-hop neighbourhood size with `mindist` filter.
///
/// For each source vertex `v` returns the number of vertices `w` such
/// that `mindist <= dist(v, w) <= order` (or `dist(v, w) >= mindist`
/// when `order < 0`, treating order as infinity). Direction follows
/// `mode` on directed graphs and is ignored on undirected graphs.
///
/// `mindist = 0` includes `v` itself; `mindist = 1` excludes `v` but
/// counts immediate neighbours; `mindist = k` excludes vertices reached
/// in fewer than `k` hops.
///
/// Counterpart of `igraph_neighborhood_size(graph, _, igraph_vss_all(),
/// order, mode, mindist)`.
///
/// # Errors
/// - [`IgraphError::InvalidArgument`] if `mindist < 0`.
/// - [`IgraphError::InvalidArgument`] if `order >= 0` and `mindist > order`.
///
/// # Examples
/// ```
/// use rust_igraph::{Graph, neighborhood_size_with_mode, NeighborhoodMode};
///
/// // Directed star: 0->1, 0->2, 0->3.
/// let mut g = Graph::new(4, true).unwrap();
/// for v in [1, 2, 3] { g.add_edge(0, v).unwrap(); }
///
/// // Out: 0 reaches all; leaves only see themselves.
/// assert_eq!(
///     neighborhood_size_with_mode(&g, -1, NeighborhoodMode::Out, 0).unwrap(),
///     vec![4, 1, 1, 1]
/// );
/// // In: leaves can reach 0 via reversed edges (in-mode walks against arc).
/// assert_eq!(
///     neighborhood_size_with_mode(&g, -1, NeighborhoodMode::In, 0).unwrap(),
///     vec![1, 2, 2, 2]
/// );
/// // mindist=1 excludes the vertex itself.
/// assert_eq!(
///     neighborhood_size_with_mode(&g, 1, NeighborhoodMode::All, 1).unwrap(),
///     vec![3, 1, 1, 1]
/// );
/// ```
pub fn neighborhood_size_with_mode(
    graph: &Graph,
    order: i32,
    mode: NeighborhoodMode,
    mindist: i32,
) -> IgraphResult<Vec<u32>> {
    if mindist < 0 {
        return Err(IgraphError::InvalidArgument(format!(
            "minimum distance must not be negative, got {mindist}"
        )));
    }
    if order >= 0 && mindist > order {
        return Err(IgraphError::InvalidArgument(format!(
            "minimum distance must not exceed neighbourhood order ({order}), got {mindist}"
        )));
    }

    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }
    let n_us = n as usize;

    // C uses `order = no_of_nodes` when negative — effectively infinite
    // because BFS depth is bounded by n-1. We model the same way using
    // i64 to avoid sign-mixing on the comparisons inside the loop.
    let inf_order = order < 0;
    let effective_order: i64 = if inf_order {
        i64::from(n)
    } else {
        i64::from(order)
    };
    let mindist_i64 = i64::from(mindist);

    let directed = graph.is_directed();
    // `added[v] = src + 1` marks "v has been seen by source `src`",
    // matching the C reference (avoids per-source array allocation).
    let mut added: Vec<u32> = vec![0; n_us];
    let mut queue: VecDeque<(VertexId, i64)> = VecDeque::new();
    let mut result: Vec<u32> = vec![0; n_us];

    for src in 0..n {
        let marker = src + 1;
        added[src as usize] = marker;
        let mut size: u32 = u32::from(mindist_i64 == 0);
        queue.clear();
        if effective_order > 0 {
            queue.push_back((src, 0));
        }

        while let Some((actnode, actdist)) = queue.pop_front() {
            let neis = neighbours_for(graph, actnode, mode, directed)?;
            if actdist < effective_order - 1 {
                for nei in neis {
                    if added[nei as usize] != marker {
                        added[nei as usize] = marker;
                        queue.push_back((nei, actdist + 1));
                        if actdist + 1 >= mindist_i64 {
                            size = size
                                .checked_add(1)
                                .ok_or(IgraphError::Internal("neighborhood size overflowed u32"))?;
                        }
                    }
                }
            } else {
                // At the frontier: count but don't enqueue.
                for nei in neis {
                    if added[nei as usize] != marker {
                        added[nei as usize] = marker;
                        if actdist + 1 >= mindist_i64 {
                            size = size
                                .checked_add(1)
                                .ok_or(IgraphError::Internal("neighborhood size overflowed u32"))?;
                        }
                    }
                }
            }
        }

        result[src as usize] = size;
    }

    Ok(result)
}

/// k-hop neighbourhood vertex list for every vertex (`mode = All`, `mindist = 0`).
///
/// For each vertex `v` returns the list of vertices `w` within `order`
/// hops (inclusive), in BFS visitation order with `v` first. Negative
/// `order` means infinity.
///
/// Counterpart of `igraph_neighborhood(graph, _, igraph_vss_all(),
/// order, IGRAPH_ALL, /*mindist=*/0)`.
///
/// # Errors
/// Same as [`neighborhood_with_mode`].
///
/// # Examples
/// ```
/// use rust_igraph::{Graph, neighborhood};
///
/// // Path P5: 0-1-2-3-4
/// let mut g = Graph::with_vertices(5);
/// for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 4)] {
///     g.add_edge(u, v).unwrap();
/// }
/// // Order 1: each vertex's 1-hop ball.
/// let n1 = neighborhood(&g, 1).unwrap();
/// assert_eq!(n1[0], vec![0, 1]);
/// assert_eq!(n1[2], vec![2, 1, 3]);
/// ```
pub fn neighborhood(graph: &Graph, order: i32) -> IgraphResult<Vec<Vec<u32>>> {
    neighborhood_with_mode(graph, order, NeighborhoodMode::All, 0)
}

/// Full mode-aware k-hop neighbourhood vertex list with `mindist` filter.
///
/// For each source vertex `v` returns the list of vertices `w` such that
/// `mindist <= dist(v, w) <= order` (or `dist(v, w) >= mindist` when
/// `order < 0`). The list is in BFS visitation order; when `mindist = 0`
/// the source is the first element.
///
/// Direction follows `mode` on directed graphs; on undirected graphs
/// every mode reduces to [`NeighborhoodMode::All`].
///
/// Counterpart of `igraph_neighborhood(graph, _, igraph_vss_all(),
/// order, mode, mindist)`.
///
/// # Errors
/// - [`IgraphError::InvalidArgument`] if `mindist < 0`.
/// - [`IgraphError::InvalidArgument`] if `order >= 0` and `mindist > order`.
///
/// # Examples
/// ```
/// use rust_igraph::{Graph, neighborhood_with_mode, NeighborhoodMode};
///
/// // Directed star: 0->1, 0->2, 0->3.
/// let mut g = Graph::new(4, true).unwrap();
/// for v in [1, 2, 3] { g.add_edge(0, v).unwrap(); }
///
/// // Out from 0: hub reaches all (BFS order: self, then 1, 2, 3).
/// let nbh = neighborhood_with_mode(&g, -1, NeighborhoodMode::Out, 0).unwrap();
/// assert_eq!(nbh[0], vec![0, 1, 2, 3]);
/// // Leaves stay alone.
/// assert_eq!(nbh[1], vec![1]);
///
/// // mindist=1 strips the source.
/// let nbh1 = neighborhood_with_mode(&g, 1, NeighborhoodMode::All, 1).unwrap();
/// assert_eq!(nbh1[0].len(), 3);  // 1, 2, 3 (some order)
/// assert!(!nbh1[0].contains(&0));
/// ```
pub fn neighborhood_with_mode(
    graph: &Graph,
    order: i32,
    mode: NeighborhoodMode,
    mindist: i32,
) -> IgraphResult<Vec<Vec<u32>>> {
    if mindist < 0 {
        return Err(IgraphError::InvalidArgument(format!(
            "minimum distance must not be negative, got {mindist}"
        )));
    }
    if order >= 0 && mindist > order {
        return Err(IgraphError::InvalidArgument(format!(
            "minimum distance must not exceed neighbourhood order ({order}), got {mindist}"
        )));
    }

    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }
    let n_us = n as usize;

    let inf_order = order < 0;
    let effective_order: i64 = if inf_order {
        i64::from(n)
    } else {
        i64::from(order)
    };
    let mindist_i64 = i64::from(mindist);

    let directed = graph.is_directed();
    let mut added: Vec<u32> = vec![0; n_us];
    let mut queue: VecDeque<(VertexId, i64)> = VecDeque::new();
    let mut result: Vec<Vec<u32>> = Vec::with_capacity(n_us);

    for src in 0..n {
        let marker = src + 1;
        added[src as usize] = marker;
        let mut tmp: Vec<u32> = Vec::new();
        if mindist_i64 == 0 {
            tmp.push(src);
        }
        queue.clear();
        if effective_order > 0 {
            queue.push_back((src, 0));
        }

        while let Some((actnode, actdist)) = queue.pop_front() {
            let neis = neighbours_for(graph, actnode, mode, directed)?;
            if actdist < effective_order - 1 {
                for nei in neis {
                    if added[nei as usize] != marker {
                        added[nei as usize] = marker;
                        queue.push_back((nei, actdist + 1));
                        if actdist + 1 >= mindist_i64 {
                            tmp.push(nei);
                        }
                    }
                }
            } else {
                for nei in neis {
                    if added[nei as usize] != marker {
                        added[nei as usize] = marker;
                        if actdist + 1 >= mindist_i64 {
                            tmp.push(nei);
                        }
                    }
                }
            }
        }

        result.push(tmp);
    }

    Ok(result)
}

/// Direction-aware neighbour list. Undirected graphs use
/// `Graph::neighbors` regardless of `mode` (matches C semantics).
fn neighbours_for(
    graph: &Graph,
    v: VertexId,
    mode: NeighborhoodMode,
    directed: bool,
) -> IgraphResult<Vec<VertexId>> {
    if !directed {
        return graph.neighbors(v);
    }
    match mode {
        NeighborhoodMode::Out => graph.out_neighbors_vec(v),
        NeighborhoodMode::In => graph.in_neighbors_vec(v),
        NeighborhoodMode::All => {
            let mut out = graph.out_neighbors_vec(v)?;
            out.extend(graph.in_neighbors_vec(v)?);
            Ok(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Graph;

    // ---- C reference fixture: directed n=6 with loops and multi-edges ----
    // Built from references/igraph/tests/unit/igraph_neighborhood_size.c
    // edges: 0->1, 0->2, 1->1 (self-loop), 1->3, 2->0, 2->3, 3->4, 3->4
    fn c_ref_graph() -> Graph {
        let mut g = Graph::new(6, true).unwrap();
        for (u, v) in [
            (0, 1),
            (0, 2),
            (1, 1),
            (1, 3),
            (2, 0),
            (2, 3),
            (3, 4),
            (3, 4),
        ] {
            g.add_edge(u, v).unwrap();
        }
        g
    }

    #[test]
    fn empty_graph_returns_empty_vector() {
        let g = Graph::with_vertices(0);
        assert!(neighborhood_size(&g, 1).unwrap().is_empty());
    }

    #[test]
    fn singleton_order_0_is_one() {
        let g = Graph::with_vertices(1);
        assert_eq!(neighborhood_size(&g, 0).unwrap(), vec![1]);
        assert_eq!(neighborhood_size(&g, 5).unwrap(), vec![1]);
    }

    #[test]
    fn no_edges_only_self_at_any_order() {
        let g = Graph::with_vertices(4);
        assert_eq!(neighborhood_size(&g, 0).unwrap(), vec![1, 1, 1, 1]);
        assert_eq!(neighborhood_size(&g, 5).unwrap(), vec![1, 1, 1, 1]);
    }

    #[test]
    fn ring_p5_matches_python_reference_order_1() {
        // python-igraph testStructural: Ring(10, circular=False) order=1
        // → [2,3,3,3,3,3,3,3,3,2]. Smaller P5 equivalent: [2,3,3,3,2].
        let mut g = Graph::with_vertices(5);
        for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 4)] {
            g.add_edge(u, v).unwrap();
        }
        assert_eq!(neighborhood_size(&g, 1).unwrap(), vec![2, 3, 3, 3, 2]);
    }

    #[test]
    fn ring_p10_matches_python_order_1() {
        let mut g = Graph::with_vertices(10);
        for i in 0..9 {
            g.add_edge(i, i + 1).unwrap();
        }
        assert_eq!(
            neighborhood_size(&g, 1).unwrap(),
            vec![2, 3, 3, 3, 3, 3, 3, 3, 3, 2]
        );
    }

    #[test]
    fn ring_p10_matches_python_order_3() {
        let mut g = Graph::with_vertices(10);
        for i in 0..9 {
            g.add_edge(i, i + 1).unwrap();
        }
        assert_eq!(
            neighborhood_size(&g, 3).unwrap(),
            vec![4, 5, 6, 7, 7, 7, 7, 6, 5, 4]
        );
    }

    #[test]
    fn ring_p10_order_3_mindist_2_matches_python() {
        let mut g = Graph::with_vertices(10);
        for i in 0..9 {
            g.add_edge(i, i + 1).unwrap();
        }
        assert_eq!(
            neighborhood_size_with_mode(&g, 3, NeighborhoodMode::All, 2).unwrap(),
            vec![2, 2, 3, 4, 4, 4, 4, 3, 2, 2]
        );
    }

    #[test]
    fn c_ref_order_0_is_self_only() {
        let g = c_ref_graph();
        // C .out: ( 1 1 1 1 1 1 )
        assert_eq!(neighborhood_size(&g, 0).unwrap(), vec![1, 1, 1, 1, 1, 1]);
    }

    #[test]
    fn c_ref_order_1_all_mode() {
        let g = c_ref_graph();
        // C .out: ( 3 3 3 4 2 1 )
        assert_eq!(neighborhood_size(&g, 1).unwrap(), vec![3, 3, 3, 4, 2, 1]);
    }

    #[test]
    fn c_ref_order_1_in_mode() {
        let g = c_ref_graph();
        // C .out: ( 2 2 2 3 2 1 )
        assert_eq!(
            neighborhood_size_with_mode(&g, 1, NeighborhoodMode::In, 0).unwrap(),
            vec![2, 2, 2, 3, 2, 1]
        );
    }

    #[test]
    fn c_ref_order_10_all_mode_saturates() {
        let g = c_ref_graph();
        // C .out: ( 5 5 5 5 5 1 ) — vertex 5 is isolated.
        assert_eq!(neighborhood_size(&g, 10).unwrap(), vec![5, 5, 5, 5, 5, 1]);
    }

    #[test]
    fn c_ref_order_2_mindist_2_out_mode() {
        let g = c_ref_graph();
        // C .out: ( 1 1 2 0 0 0 )
        assert_eq!(
            neighborhood_size_with_mode(&g, 2, NeighborhoodMode::Out, 2).unwrap(),
            vec![1, 1, 2, 0, 0, 0]
        );
    }

    #[test]
    fn c_ref_order_4_mindist_4_all_mode_all_zero() {
        let g = c_ref_graph();
        // Diameter is 3, so mindist=4 yields all zeros.
        assert_eq!(
            neighborhood_size_with_mode(&g, 4, NeighborhoodMode::All, 4).unwrap(),
            vec![0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn c_ref_infinite_order_out_mode() {
        let g = c_ref_graph();
        // C .out: ( 5 3 5 2 1 1 )
        assert_eq!(
            neighborhood_size_with_mode(&g, -1, NeighborhoodMode::Out, 0).unwrap(),
            vec![5, 3, 5, 2, 1, 1]
        );
    }

    #[test]
    fn c_ref_infinite_order_mindist_2_out_mode() {
        let g = c_ref_graph();
        // C .out: ( 2 1 2 0 0 0 )
        assert_eq!(
            neighborhood_size_with_mode(&g, -1, NeighborhoodMode::Out, 2).unwrap(),
            vec![2, 1, 2, 0, 0, 0]
        );
    }

    #[test]
    fn c_ref_infinite_order_mindist_2_in_mode() {
        let g = c_ref_graph();
        // C .out: ( 0 1 0 1 3 0 )
        assert_eq!(
            neighborhood_size_with_mode(&g, -1, NeighborhoodMode::In, 2).unwrap(),
            vec![0, 1, 0, 1, 3, 0]
        );
    }

    #[test]
    fn negative_mindist_errors() {
        let g = Graph::with_vertices(3);
        match neighborhood_size_with_mode(&g, 2, NeighborhoodMode::All, -1) {
            Err(IgraphError::InvalidArgument(msg)) => assert!(msg.contains("negative")),
            other => panic!("expected InvalidArgument for negative mindist, got {other:?}"),
        }
    }

    #[test]
    fn mindist_exceeding_finite_order_errors() {
        let g = Graph::with_vertices(3);
        match neighborhood_size_with_mode(&g, 2, NeighborhoodMode::All, 3) {
            Err(IgraphError::InvalidArgument(msg)) => assert!(msg.contains("exceed")),
            other => panic!("expected InvalidArgument for mindist > order, got {other:?}"),
        }
    }

    #[test]
    fn infinite_order_allows_any_mindist() {
        // mindist > vcount is fine when order is infinite.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // mindist=10: nobody is at distance >= 10 → all zeros.
        assert_eq!(
            neighborhood_size_with_mode(&g, -1, NeighborhoodMode::All, 10).unwrap(),
            vec![0, 0, 0]
        );
    }

    #[test]
    fn k4_complete_undirected_order_1() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        // Every vertex sees self + 3 neighbours.
        assert_eq!(neighborhood_size(&g, 1).unwrap(), vec![4, 4, 4, 4]);
    }

    #[test]
    fn directed_star_out_in_modes() {
        // 0 -> 1, 0 -> 2, 0 -> 3
        let mut g = Graph::new(4, true).unwrap();
        for v in [1, 2, 3] {
            g.add_edge(0, v).unwrap();
        }
        // Out: hub reaches all, leaves stay alone.
        assert_eq!(
            neighborhood_size_with_mode(&g, -1, NeighborhoodMode::Out, 0).unwrap(),
            vec![4, 1, 1, 1]
        );
        // In: leaves reach hub by reversed walk.
        assert_eq!(
            neighborhood_size_with_mode(&g, -1, NeighborhoodMode::In, 0).unwrap(),
            vec![1, 2, 2, 2]
        );
        // All: everything connected.
        assert_eq!(
            neighborhood_size_with_mode(&g, -1, NeighborhoodMode::All, 0).unwrap(),
            vec![4, 4, 4, 4]
        );
    }

    #[test]
    fn self_loop_does_not_inflate_count() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // Self-loop on 0: order 1 still {0, 1} → size 2.
        assert_eq!(neighborhood_size(&g, 1).unwrap(), vec![2, 3, 2]);
    }

    #[test]
    fn multi_edge_does_not_double_count() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(neighborhood_size(&g, 1).unwrap(), vec![2, 3, 2]);
    }

    #[test]
    fn mindist_equals_order_counts_frontier_only() {
        // P5: 0-1-2-3-4. order=2, mindist=2 → only vertices at distance 2.
        let mut g = Graph::with_vertices(5);
        for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 4)] {
            g.add_edge(u, v).unwrap();
        }
        // d=2 ball for each vertex: 0→{2}, 1→{3}, 2→{0,4}, 3→{1}, 4→{2}.
        assert_eq!(
            neighborhood_size_with_mode(&g, 2, NeighborhoodMode::All, 2).unwrap(),
            vec![1, 1, 2, 1, 1]
        );
    }

    // ---- neighborhood (vertex lists) tests ----
    //
    // BFS visitation order depends on adjacency-list iteration, so it
    // differs subtly between Rust / C / Python. The semantically
    // meaningful claim is set equality, so all neighborhood tests sort
    // both sides before comparing.

    fn sorted(mut v: Vec<u32>) -> Vec<u32> {
        v.sort_unstable();
        v
    }

    fn sorted_all(nbh: Vec<Vec<u32>>) -> Vec<Vec<u32>> {
        nbh.into_iter().map(sorted).collect()
    }

    #[test]
    fn neighborhood_empty_graph_returns_empty() {
        let g = Graph::with_vertices(0);
        assert!(neighborhood(&g, 1).unwrap().is_empty());
    }

    #[test]
    fn neighborhood_singleton_order_0_is_self_only() {
        let g = Graph::with_vertices(1);
        assert_eq!(neighborhood(&g, 0).unwrap(), vec![vec![0]]);
        assert_eq!(neighborhood(&g, 5).unwrap(), vec![vec![0]]);
    }

    #[test]
    fn neighborhood_no_edges_returns_singletons() {
        let g = Graph::with_vertices(3);
        let n0 = neighborhood(&g, 0).unwrap();
        assert_eq!(n0, vec![vec![0], vec![1], vec![2]]);
        let n5 = neighborhood(&g, 5).unwrap();
        assert_eq!(n5, vec![vec![0], vec![1], vec![2]]);
    }

    #[test]
    fn neighborhood_p5_order_1_set_eq() {
        // P5: 0-1-2-3-4
        let mut g = Graph::with_vertices(5);
        for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 4)] {
            g.add_edge(u, v).unwrap();
        }
        let got = sorted_all(neighborhood(&g, 1).unwrap());
        let exp: Vec<Vec<u32>> = vec![
            vec![0, 1],
            vec![0, 1, 2],
            vec![1, 2, 3],
            vec![2, 3, 4],
            vec![3, 4],
        ];
        assert_eq!(got, exp);
    }

    #[test]
    fn neighborhood_p5_order_2_set_eq() {
        let mut g = Graph::with_vertices(5);
        for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 4)] {
            g.add_edge(u, v).unwrap();
        }
        let got = sorted_all(neighborhood(&g, 2).unwrap());
        let exp: Vec<Vec<u32>> = vec![
            vec![0, 1, 2],
            vec![0, 1, 2, 3],
            vec![0, 1, 2, 3, 4],
            vec![1, 2, 3, 4],
            vec![2, 3, 4],
        ];
        assert_eq!(got, exp);
    }

    #[test]
    fn neighborhood_c_ref_order_0_is_self_only() {
        // C .out: 0:(0) 1:(1) 2:(2) 3:(3) 4:(4) 5:(5)
        let g = c_ref_graph();
        let got = neighborhood(&g, 0).unwrap();
        let exp: Vec<Vec<u32>> = (0..6).map(|i| vec![i]).collect();
        assert_eq!(got, exp);
    }

    #[test]
    fn neighborhood_c_ref_order_1_all_set_eq() {
        // C .out: 0:(0 1 2) 1:(1 0 3) 2:(2 0 3) 3:(3 1 2 4) 4:(4 3) 5:(5)
        let g = c_ref_graph();
        let got = sorted_all(neighborhood(&g, 1).unwrap());
        let exp: Vec<Vec<u32>> = vec![
            vec![0, 1, 2],
            vec![0, 1, 3],
            vec![0, 2, 3],
            vec![1, 2, 3, 4],
            vec![3, 4],
            vec![5],
        ];
        assert_eq!(got, exp);
    }

    #[test]
    fn neighborhood_c_ref_order_1_in_set_eq() {
        // C .out: 0:(0 2) 1:(1 0) 2:(2 0) 3:(3 1 2) 4:(4 3) 5:(5)
        let g = c_ref_graph();
        let got = sorted_all(neighborhood_with_mode(&g, 1, NeighborhoodMode::In, 0).unwrap());
        let exp: Vec<Vec<u32>> = vec![
            vec![0, 2],
            vec![0, 1],
            vec![0, 2],
            vec![1, 2, 3],
            vec![3, 4],
            vec![5],
        ];
        assert_eq!(got, exp);
    }

    #[test]
    fn neighborhood_c_ref_order_10_all_saturates_set_eq() {
        // C .out: 0:(0 1 2 3 4) 1:(0 1 2 3 4) 2:(0 1 2 3 4) 3:(0 1 2 3 4)
        //         4:(0 1 2 3 4) 5:(5)
        let g = c_ref_graph();
        let got = sorted_all(neighborhood(&g, 10).unwrap());
        let big: Vec<u32> = (0..5).collect();
        let exp: Vec<Vec<u32>> = vec![
            big.clone(),
            big.clone(),
            big.clone(),
            big.clone(),
            big,
            vec![5],
        ];
        assert_eq!(got, exp);
    }

    #[test]
    fn neighborhood_c_ref_order_2_mindist_2_out_set_eq() {
        // C .out: 0:(3) 1:(4) 2:(1 4) 3:() 4:() 5:()
        let g = c_ref_graph();
        let got = sorted_all(neighborhood_with_mode(&g, 2, NeighborhoodMode::Out, 2).unwrap());
        let exp: Vec<Vec<u32>> = vec![vec![3], vec![4], vec![1, 4], vec![], vec![], vec![]];
        assert_eq!(got, exp);
    }

    #[test]
    fn neighborhood_c_ref_order_4_mindist_4_all_empty() {
        // Diameter < 4 ⇒ all empty.
        let g = c_ref_graph();
        let got = neighborhood_with_mode(&g, 4, NeighborhoodMode::All, 4).unwrap();
        let exp: Vec<Vec<u32>> = vec![vec![]; 6];
        assert_eq!(got, exp);
    }

    #[test]
    fn neighborhood_c_ref_infinite_order_out_set_eq() {
        // C .out: 0:(0 1 2 3 4) 1:(1 3 4) 2:(0 1 2 3 4) 3:(3 4) 4:(4) 5:(5)
        let g = c_ref_graph();
        let got = sorted_all(neighborhood_with_mode(&g, -1, NeighborhoodMode::Out, 0).unwrap());
        let exp: Vec<Vec<u32>> = vec![
            vec![0, 1, 2, 3, 4],
            vec![1, 3, 4],
            vec![0, 1, 2, 3, 4],
            vec![3, 4],
            vec![4],
            vec![5],
        ];
        assert_eq!(got, exp);
    }

    #[test]
    fn neighborhood_c_ref_infinite_mindist_2_out_set_eq() {
        // C .out: 0:(3 4) 1:(4) 2:(1 4) 3:() 4:() 5:()
        let g = c_ref_graph();
        let got = sorted_all(neighborhood_with_mode(&g, -1, NeighborhoodMode::Out, 2).unwrap());
        let exp: Vec<Vec<u32>> = vec![vec![3, 4], vec![4], vec![1, 4], vec![], vec![], vec![]];
        assert_eq!(got, exp);
    }

    #[test]
    fn neighborhood_c_ref_infinite_mindist_2_in_set_eq() {
        // C .out: 0:() 1:(2) 2:() 3:(0) 4:(0 1 2) 5:()
        let g = c_ref_graph();
        let got = sorted_all(neighborhood_with_mode(&g, -1, NeighborhoodMode::In, 2).unwrap());
        let exp: Vec<Vec<u32>> = vec![vec![], vec![2], vec![], vec![0], vec![0, 1, 2], vec![]];
        assert_eq!(got, exp);
    }

    #[test]
    fn neighborhood_negative_mindist_errors() {
        let g = Graph::with_vertices(3);
        match neighborhood_with_mode(&g, 2, NeighborhoodMode::All, -1) {
            Err(IgraphError::InvalidArgument(msg)) => assert!(msg.contains("negative")),
            other => panic!("expected InvalidArgument for negative mindist, got {other:?}"),
        }
    }

    #[test]
    fn neighborhood_mindist_exceeds_order_errors() {
        let g = Graph::with_vertices(3);
        match neighborhood_with_mode(&g, 2, NeighborhoodMode::All, 3) {
            Err(IgraphError::InvalidArgument(msg)) => assert!(msg.contains("exceed")),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn neighborhood_lengths_match_neighborhood_size() {
        // PR-027b list lengths should equal PR-027 sizes for any
        // (order, mode, mindist) triple — the two are tightly coupled.
        let g = c_ref_graph();
        for order in [0_i32, 1, 2, 3, 10, -1] {
            for &mode in &[
                NeighborhoodMode::Out,
                NeighborhoodMode::In,
                NeighborhoodMode::All,
            ] {
                for mindist in [0_i32, 1, 2] {
                    if order >= 0 && mindist > order {
                        continue;
                    }
                    let sizes = neighborhood_size_with_mode(&g, order, mode, mindist).unwrap();
                    let lists = neighborhood_with_mode(&g, order, mode, mindist).unwrap();
                    let list_lens: Vec<u32> = lists
                        .iter()
                        .map(|v| u32::try_from(v.len()).unwrap())
                        .collect();
                    assert_eq!(
                        sizes, list_lens,
                        "size/list-len mismatch at order={order} mode={mode:?} mindist={mindist}",
                    );
                }
            }
        }
    }

    #[test]
    fn neighborhood_self_loop_not_double_visited() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let got = sorted_all(neighborhood(&g, 1).unwrap());
        assert_eq!(got, vec![vec![0, 1], vec![0, 1, 2], vec![1, 2]]);
    }

    #[test]
    fn neighborhood_multi_edge_not_double_visited() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let got = sorted_all(neighborhood(&g, 1).unwrap());
        assert_eq!(got, vec![vec![0, 1], vec![0, 1, 2], vec![1, 2]]);
    }

    #[test]
    fn neighborhood_mindist_1_excludes_self() {
        // P5 order 1 mindist 1 → only neighbours, not self.
        let mut g = Graph::with_vertices(5);
        for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 4)] {
            g.add_edge(u, v).unwrap();
        }
        let got = sorted_all(neighborhood_with_mode(&g, 1, NeighborhoodMode::All, 1).unwrap());
        assert_eq!(
            got,
            vec![vec![1], vec![0, 2], vec![1, 3], vec![2, 4], vec![3]]
        );
        for list in &got {
            for (i, neighbors) in got.iter().enumerate() {
                let i_u32 = u32::try_from(i).unwrap();
                if std::ptr::eq(list, neighbors) {
                    assert!(!list.contains(&i_u32), "mindist=1 should exclude self");
                }
            }
        }
    }
}
