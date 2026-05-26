//! `st_vertex_connectivity` (ALGO-FL-013) — minimum number of vertices
//! (other than `source` and `target`) whose removal disconnects
//! `source` from `target`.
//!
//! Counterpart of `igraph_st_vertex_connectivity` in
//! `references/igraph/src/flow/flow.c` (lines 1922-1940), which
//! dispatches to two internal helpers,
//! `igraph_i_st_vertex_connectivity_directed` (flow.c:1790-1849) and
//! `igraph_i_st_vertex_connectivity_undirected` (flow.c:1851-1879).
//! The undirected case forwards to the directed one after a
//! `IGRAPH_TO_DIRECTED_MUTUAL` conversion (each `(u, v)` ↔ two arcs
//! `u→v` and `v→u`).
//!
//! ## Algorithm
//!
//! The reduction is the standard vertex-splitting trick (Even,
//! *Graph Algorithms* §5.5):
//!
//! 1. Replace every vertex `v` with two vertices `v_out` and `v_in`
//!    joined by an internal arc `v_in → v_out` of capacity 1.
//! 2. Rewrite each original arc `u → v` as `u_out → v_in` with
//!    capacity 1.
//! 3. Set the internal arcs of `source` and `target` to capacity 0
//!    (the C implementation does this implicitly: it sets every arc
//!    incident on `source_in` and `target_out` to capacity 0 — same
//!    set of arcs in our layout).
//! 4. Run a unit-capacity max-flow from `source_out` (= `source` in
//!    the C indexing) to `target_in` (= `target + n`). By Menger's
//!    theorem this equals the number of internally vertex-disjoint
//!    `source → target` paths in the original graph, which is the
//!    s-t vertex connectivity.
//!
//! We mirror igraph C's layout exactly so the implementation can be
//! verified against the reference's `igraph_i_split_vertices`
//! (`references/igraph/src/flow/flow_conversion.c:61-92`): the first
//! `n` vertices of the split graph are the **output** halves
//! (`v_out = v`) and the last `n` are the **input** halves
//! (`v_in = v + n`). The internal arc is `v_in → v_out`.
//!
//! ## Direct edges between `source` and `target`
//!
//! A direct edge `source → target` cannot be cut by removing internal
//! vertices, so the behaviour when one exists is configurable via
//! [`VconnNei`]. This mirrors igraph C's `igraph_vconn_nei_t`
//! verbatim.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

use super::max_flow::max_flow_value;

/// Behaviour when `source` and `target` are directly adjacent.
///
/// Mirrors `igraph_vconn_nei_t` from
/// `references/igraph/include/igraph_flow.h`. A direct edge cannot be
/// broken by removing internal vertices, so this enum picks how the
/// library should report that case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VconnNei {
    /// Return an [`IgraphError::InvalidArgument`] when a direct edge
    /// exists. Matches igraph C `IGRAPH_VCONN_NEI_ERROR`.
    Error,
    /// Return `-1` when a direct edge exists. Matches igraph C
    /// `IGRAPH_VCONN_NEI_NEGATIVE`.
    Negative,
    /// Return `vcount()` (which is `≥ s-t vertex connectivity`
    /// trivially) when a direct edge exists. Matches igraph C
    /// `IGRAPH_VCONN_NEI_NUMBER_OF_NODES`.
    NumberOfNodes,
    /// Continue with the max-flow calculation, subtracting one path
    /// for every parallel direct arc so the result is the number of
    /// internally vertex-disjoint paths excluding the direct ones.
    /// Matches igraph C `IGRAPH_VCONN_NEI_IGNORE`.
    Ignore,
}

/// Vertex connectivity between two vertices.
///
/// Counterpart of `igraph_st_vertex_connectivity` in
/// `references/igraph/src/flow/flow.c:1922`. Returns the minimum
/// number of internal vertices whose removal disconnects `source`
/// from `target`.
///
/// # Arguments
///
/// * `graph` — input graph (directed or undirected). Undirected input
///   is treated as if every edge is two anti-parallel directed arcs
///   (`IGRAPH_TO_DIRECTED_MUTUAL`), matching igraph C.
/// * `source` — source vertex id (`0 ≤ source < vcount()`).
/// * `target` — target vertex id (`0 ≤ target < vcount()`,
///   `target != source`).
/// * `mode` — how to handle a direct `source → target` edge. See
///   [`VconnNei`].
///
/// # Returns
///
/// The s-t vertex connectivity as `i64`. Returns `0` when `source`
/// and `target` are in disjoint components. When `mode` is
/// [`VconnNei::Negative`] and a direct edge exists, returns `-1`.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] if `source == target`,
///   `vcount() < 2`, or `mode == VconnNei::Error` and a direct edge
///   `source → target` exists.
/// * [`IgraphError::VertexOutOfRange`] if `source` or `target` is
///   outside `[0, vcount())`.
/// * [`IgraphError::Internal`] if the unit-capacity max-flow value is
///   not representable as `i64` (unreachable in practice: the value
///   is bounded above by `vcount()`).
///
/// [`IgraphError::VertexOutOfRange`]: crate::core::IgraphError::VertexOutOfRange
/// [`IgraphError::InvalidArgument`]: crate::core::IgraphError::InvalidArgument
/// [`IgraphError::Internal`]: crate::core::IgraphError::Internal
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, VconnNei, st_vertex_connectivity};
///
/// // Path 0 — 1 — 2 — 3 — 4 — 5 (undirected). Any single internal
/// // vertex separates endpoints → s-t vertex connectivity = 1.
/// let mut g = Graph::new(6, false).unwrap();
/// for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)] {
///     g.add_edge(u, v).unwrap();
/// }
/// assert_eq!(
///     st_vertex_connectivity(&g, 0, 5, VconnNei::Error).unwrap(),
///     1
/// );
/// ```
pub fn st_vertex_connectivity(
    graph: &Graph,
    source: VertexId,
    target: VertexId,
    mode: VconnNei,
) -> IgraphResult<i64> {
    let n = graph.vcount();

    if n < 2 {
        return Err(IgraphError::InvalidArgument(format!(
            "st_vertex_connectivity needs at least 2 vertices, graph has {n}"
        )));
    }
    if source >= n {
        return Err(IgraphError::VertexOutOfRange { id: source, n });
    }
    if target >= n {
        return Err(IgraphError::VertexOutOfRange { id: target, n });
    }
    if source == target {
        return Err(IgraphError::InvalidArgument(format!(
            "source and target vertices are the same ({source})"
        )));
    }

    // Direct s→t edges. For undirected graphs `get_all_eids_between`
    // returns the union of both directions which matches the
    // `IGRAPH_TO_DIRECTED_MUTUAL` semantics for this check.
    let direct_eids = graph.get_all_eids_between(source, target)?;
    let no_conn = direct_eids.len();
    let has_direct = no_conn > 0;

    match mode {
        VconnNei::Error if has_direct => {
            return Err(IgraphError::InvalidArgument(format!(
                "source ({source}) and target ({target}) are directly connected"
            )));
        }
        VconnNei::Negative if has_direct => return Ok(-1),
        VconnNei::NumberOfNodes if has_direct => return Ok(i64::from(n)),
        _ => {}
    }

    // Build the split-vertex graph. Layout (mirrors igraph C
    // `igraph_i_split_vertices`):
    //   * vertices [0, n)        — output halves (`v_out == v`)
    //   * vertices [n, 2n)       — input halves (`v_in == v + n`)
    //   * arcs:
    //       - first ecount() arcs  : u_out → v_in   (for each input arc u→v)
    //       - last n arcs          : v_in → v_out   (one per original v)
    //   * undirected input is exploded into mutual arc pairs first.
    let split_n = n
        .checked_mul(2)
        .ok_or(IgraphError::Internal("split-vertex graph overflowed u32"))?;
    let mut split = Graph::new(split_n, true)?;

    let mut original_arc_count: usize = 0;
    if graph.is_directed() {
        for eid in 0..graph.ecount() {
            let eid_u32 =
                u32::try_from(eid).map_err(|_| IgraphError::Internal("edge id overflow u32"))?;
            let (u, v) = graph.edge(eid_u32)?;
            split.add_edge(u, v + n)?;
            original_arc_count += 1;
        }
    } else {
        // Mutual conversion: each undirected edge contributes both
        // `u→v` and `v→u` directed arcs (matches
        // `IGRAPH_TO_DIRECTED_MUTUAL`).
        for eid in 0..graph.ecount() {
            let eid_u32 =
                u32::try_from(eid).map_err(|_| IgraphError::Internal("edge id overflow u32"))?;
            let (u, v) = graph.edge(eid_u32)?;
            split.add_edge(u, v + n)?;
            split.add_edge(v, u + n)?;
            original_arc_count += 2;
        }
    }
    for v in 0..n {
        split.add_edge(v + n, v)?;
    }

    // Unit caps everywhere, then zero out arcs incident on
    // `source_in` (= source + n) and `target_out` (= target). These
    // are exactly the arcs that "open up" the source's input-side or
    // the target's output-side and have no role in any genuine
    // vertex-disjoint path between source and target.
    let split_ecount = split.ecount();
    let mut caps = vec![1.0f64; split_ecount];
    let source_in = source + n;
    let target_out = target;
    for eid in split.incident(source_in)? {
        caps[eid as usize] = 0.0;
    }
    for eid in split.incident(target_out)? {
        caps[eid as usize] = 0.0;
    }

    // For IGNORE mode the direct arcs source_out → target_in are kept
    // in the split graph (they're part of `original_arc_count`) and
    // contribute `no_conn` unit-flow units we need to subtract.
    let _ = original_arc_count; // documented above, no further use

    let flow = max_flow_value(&split, source, target + n, Some(&caps))?;

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let raw = {
        if !flow.is_finite() || flow < 0.0 || flow > i64::MAX as f64 {
            return Err(IgraphError::Internal(
                "unit-capacity max-flow value is not representable as i64",
            ));
        }
        flow.round() as i64
    };

    let no_conn_i64 = i64::try_from(no_conn).map_err(|_| {
        IgraphError::Internal("number of parallel direct edges does not fit in i64")
    })?;

    Ok(raw - no_conn_i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::IgraphError;

    // --- Error cases (mirrors C: igraph_st_vertex_connectivity.c lines 70-83) ---

    #[test]
    fn rejects_source_equals_target() {
        let mut g = Graph::new(2, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        let err = st_vertex_connectivity(&g, 0, 0, VconnNei::Error).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rejects_empty_graph() {
        let g = Graph::new(0, false).expect("graph");
        let err = st_vertex_connectivity(&g, 0, 0, VconnNei::Error).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rejects_single_vertex_graph() {
        let g = Graph::new(1, false).expect("graph");
        let err = st_vertex_connectivity(&g, 0, 0, VconnNei::Error).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rejects_out_of_range_source() {
        let g = Graph::new(3, false).expect("graph");
        let err = st_vertex_connectivity(&g, 5, 0, VconnNei::Error).unwrap_err();
        match err {
            IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!(id, 5);
                assert_eq!(n, 3);
            }
            other => panic!("expected VertexOutOfRange, got {other:?}"),
        }
    }

    #[test]
    fn rejects_out_of_range_target() {
        let g = Graph::new(3, false).expect("graph");
        let err = st_vertex_connectivity(&g, 0, 5, VconnNei::Error).unwrap_err();
        match err {
            IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!(id, 5);
                assert_eq!(n, 3);
            }
            other => panic!("expected VertexOutOfRange, got {other:?}"),
        }
    }

    // --- C unit-test fixtures (igraph_st_vertex_connectivity.c lines 32-66) ---

    #[test]
    fn two_unconnected_vertices_error_mode_returns_zero() {
        let g = Graph::new(2, false).expect("graph");
        let v = st_vertex_connectivity(&g, 0, 1, VconnNei::Error).expect("vc");
        assert_eq!(v, 0);
    }

    #[test]
    fn two_connected_vertices_negative_mode_returns_negative_one() {
        let mut g = Graph::new(2, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        let v = st_vertex_connectivity(&g, 0, 1, VconnNei::Negative).expect("vc");
        assert_eq!(v, -1);
    }

    #[test]
    fn two_connected_vertices_number_of_nodes_mode_returns_n() {
        let mut g = Graph::new(2, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        let v = st_vertex_connectivity(&g, 0, 1, VconnNei::NumberOfNodes).expect("vc");
        assert_eq!(v, 2);
    }

    #[test]
    fn three_parallel_undirected_edges_ignore_mode_returns_zero() {
        let mut g = Graph::new(2, false).expect("graph");
        for _ in 0..3 {
            g.add_edge(0, 1).expect("edge");
        }
        let v = st_vertex_connectivity(&g, 0, 1, VconnNei::Ignore).expect("vc");
        assert_eq!(v, 0);
    }

    #[test]
    fn mixed_parallel_undirected_edges_ignore_mode_returns_zero() {
        // Same fixture as in `igraph_st_vertex_connectivity.c` line 49
        // (despite the C printout claiming "directed", the igraph_small
        // construction is IGRAPH_UNDIRECTED — see the source).
        let mut g = Graph::new(2, false).expect("graph");
        for _ in 0..3 {
            g.add_edge(0, 1).expect("edge");
        }
        for _ in 0..2 {
            g.add_edge(1, 0).expect("edge");
        }
        let v = st_vertex_connectivity(&g, 0, 1, VconnNei::Ignore).expect("vc");
        assert_eq!(v, 0);
    }

    #[test]
    fn line_graph_6v_error_mode_returns_one() {
        // 0 — 1 — 2 — 3 — 4 — 5 undirected. Any internal vertex separates
        // endpoints. C fixture line 53-54.
        let mut g = Graph::new(6, false).expect("graph");
        for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 5)] {
            g.add_edge(u, v).expect("edge");
        }
        let v = st_vertex_connectivity(&g, 0, 5, VconnNei::Error).expect("vc");
        assert_eq!(v, 1);
    }

    #[test]
    fn full_graph_6v_undirected_ignore_mode_returns_four() {
        // K_6 undirected. Removing all 4 vertices other than s and t
        // disconnects them; can't go lower. C fixture line 57-58.
        let mut g = Graph::new(6, false).expect("graph");
        for i in 0u32..6 {
            for j in (i + 1)..6 {
                g.add_edge(i, j).expect("edge");
            }
        }
        let v = st_vertex_connectivity(&g, 0, 1, VconnNei::Ignore).expect("vc");
        assert_eq!(v, 4);
    }

    #[test]
    fn full_graph_6v_directed_ignore_mode_returns_four() {
        // K_6 with both directions for every pair. C fixture line 60-62.
        let mut g = Graph::new(6, true).expect("graph");
        for i in 0u32..6 {
            for j in 0u32..6 {
                if i != j {
                    g.add_edge(i, j).expect("edge");
                }
            }
        }
        let v = st_vertex_connectivity(&g, 0, 1, VconnNei::Ignore).expect("vc");
        assert_eq!(v, 4);
    }

    #[test]
    fn three_vertex_bottleneck_error_mode_returns_one() {
        // Vertices 0,1,2. Edges (0,1)×2 and (1,2)×4 undirected.
        // Vertex 1 is the only path through. C fixture line 64-66.
        let mut g = Graph::new(3, false).expect("graph");
        for _ in 0..2 {
            g.add_edge(0, 1).expect("edge");
        }
        for _ in 0..4 {
            g.add_edge(1, 2).expect("edge");
        }
        let v = st_vertex_connectivity(&g, 0, 2, VconnNei::Error).expect("vc");
        assert_eq!(v, 1);
    }

    // --- Additional sanity tests ---

    #[test]
    fn isolated_endpoints_have_zero_connectivity() {
        let g = Graph::new(4, true).expect("graph");
        let v = st_vertex_connectivity(&g, 0, 3, VconnNei::Error).expect("vc");
        assert_eq!(v, 0);
    }

    #[test]
    fn two_internally_vertex_disjoint_paths() {
        // 0 → 1 → 3 and 0 → 2 → 3 share no internal vertex.
        let mut g = Graph::new(4, true).expect("graph");
        for (u, v) in [(0u32, 1u32), (1, 3), (0, 2), (2, 3)] {
            g.add_edge(u, v).expect("edge");
        }
        let v = st_vertex_connectivity(&g, 0, 3, VconnNei::Error).expect("vc");
        assert_eq!(v, 2);
    }

    #[test]
    fn directed_anti_parallel_does_not_count_in_error_mode() {
        // 0 → 1 and 1 → 0: there *is* a direct arc 0→1, so ERROR mode
        // must raise.
        let mut g = Graph::new(2, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(1, 0).expect("edge");
        let err = st_vertex_connectivity(&g, 0, 1, VconnNei::Error).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn ignore_mode_subtracts_parallel_direct_arcs_directed() {
        // s → t exists (1 direct arc) plus 0→2→1 detour. With IGNORE
        // mode the direct arc is subtracted, leaving the single detour
        // → 1.
        let mut g = Graph::new(3, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(0, 2).expect("edge");
        g.add_edge(2, 1).expect("edge");
        let v = st_vertex_connectivity(&g, 0, 1, VconnNei::Ignore).expect("vc");
        assert_eq!(v, 1);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    //! Proptest invariants: s-t vertex connectivity is bounded above by
    //! s-t edge connectivity (Whitney 1932), and the function must
    //! never return a negative value in modes other than Negative.

    use super::*;
    use crate::algorithms::flow::st_edge_connectivity::st_edge_connectivity;
    use crate::core::Graph;
    use proptest::prelude::*;

    fn xorshift(mut r: u64) -> u64 {
        r ^= r << 13;
        r ^= r >> 7;
        r ^= r << 17;
        r
    }

    fn build_random(seed: u64, n: u32, m_max: u32, directed: bool) -> Graph {
        let mut g = Graph::new(n, directed).expect("graph");
        let mut state = seed | 1;
        for _ in 0..m_max {
            state = xorshift(state);
            let u = u32::try_from(state % u64::from(n)).expect("modulo fits");
            state = xorshift(state);
            let v = u32::try_from(state % u64::from(n)).expect("modulo fits");
            if u == v {
                continue;
            }
            g.add_edge(u, v).expect("edge");
        }
        g
    }

    proptest! {
        #[test]
        fn vc_le_ec_when_no_direct_edge(
            seed in any::<u64>(),
            n in 3u32..8,
            m in 1u32..16,
            directed in any::<bool>(),
        ) {
            // Whitney's theorem: kappa(s,t) <= lambda(s,t) when no
            // direct edge exists.
            let g = build_random(seed, n, m, directed);
            let s = u32::try_from(seed % u64::from(n)).expect("modulo fits");
            let t_raw = u32::try_from(xorshift(seed) % u64::from(n)).expect("modulo fits");
            let t = if t_raw == s { (s + 1) % n } else { t_raw };
            prop_assume!(s != t);

            // Skip cases with a direct edge for this invariant. For
            // undirected graphs `get_all_eids_between` covers both
            // directions; for directed graphs we want only s→t.
            let has_direct = if directed {
                g.find_eid(s, t).expect("find").is_some()
            } else {
                !g.get_all_eids_between(s, t).expect("eids").is_empty()
            };
            prop_assume!(!has_direct);

            let vc = st_vertex_connectivity(&g, s, t, VconnNei::Error)
                .expect("vc (no direct edge by prop_assume)");
            let ec = st_edge_connectivity(&g, s, t).expect("ec");

            prop_assert!(vc >= 0,
                "vc must be non-negative without direct edge, got {vc}");
            prop_assert!(vc <= ec,
                "Whitney violated: vc={vc} > ec={ec} (n={n}, m={m}, directed={directed}, seed={seed})");
            // And vc is also bounded by n - 2 (must leave s, t).
            prop_assert!(vc <= i64::from(n) - 2,
                "vc={vc} exceeds n-2={} (n={n})", i64::from(n) - 2);
        }

        #[test]
        fn ignore_mode_is_non_negative(
            seed in any::<u64>(),
            n in 2u32..6,
            m in 0u32..12,
            directed in any::<bool>(),
        ) {
            let g = build_random(seed, n, m, directed);
            let s = u32::try_from(seed % u64::from(n)).expect("modulo fits");
            let t_raw = u32::try_from(xorshift(seed) % u64::from(n)).expect("modulo fits");
            let t = if t_raw == s { (s + 1) % n } else { t_raw };
            prop_assume!(s != t);

            let vc = st_vertex_connectivity(&g, s, t, VconnNei::Ignore).expect("vc");
            prop_assert!(vc >= 0,
                "Ignore mode returned negative: vc={vc} (n={n}, m={m}, directed={directed}, seed={seed})");
        }
    }
}
