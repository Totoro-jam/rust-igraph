//! `vertex_disjoint_paths` (ALGO-FL-014) — maximum number of
//! internally vertex-disjoint `source → target` paths.
//!
//! Counterpart of `igraph_vertex_disjoint_paths` in
//! `references/igraph/src/flow/flow.c` (lines 2374-2405). The C source
//! is a 30-line wrapper that rejects `source == target`, computes the
//! count of direct `source → target` edges via
//! `igraph_get_all_eids_between`, then delegates to one of
//! `igraph_i_st_vertex_connectivity_{directed, undirected}` with
//! `IGRAPH_VCONN_NEI_IGNORE` (the splitting reduction ignores direct
//! edges) and adds the direct-edge count back to the result.
//!
//! By Menger's theorem (1927) the maximum number of pairwise
//! internally vertex-disjoint `s → t` paths equals the cardinality of
//! the minimum s-t internal-vertex cut whenever `s` and `t` are not
//! adjacent. When they are, the direct-edge count is added on top:
//! every parallel arc contributes one trivially internally-disjoint
//! path of length 1.
//!
//! Note that `vertex_disjoint_paths(g, s, t) ==
//! st_vertex_connectivity(g, s, t, VconnNei::Ignore)` exactly when no
//! direct `source → target` edge exists. When direct edges exist,
//! `vertex_disjoint_paths` exceeds the `Ignore` value by the
//! direct-edge count — `VconnNei::NumberOfNodes` is *not* an
//! equivalent spelling (it returns the sentinel `vcount()` instead of
//! the additive count). We keep `vertex_disjoint_paths` as a
//! separate Rust function for naming parity with igraph C, so call
//! sites can pick the spelling that matches their intent ("paths"
//! vs. "connectivity").

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

use super::st_vertex_connectivity::{VconnNei, st_vertex_connectivity};

/// Maximum number of pairwise internally vertex-disjoint paths from
/// `source` to `target`.
///
/// Counterpart of `igraph_vertex_disjoint_paths` in
/// `references/igraph/src/flow/flow.c:2374`. By Menger's theorem
/// (1927) on internal-vertex cuts, this equals the s-t vertex
/// connectivity (under the `Ignore` convention) plus the count of
/// direct `source → target` edges. See
/// [`st_vertex_connectivity`](super::st_vertex_connectivity::st_vertex_connectivity)
/// for the connectivity-flavoured spelling.
///
/// # Arguments
///
/// * `graph` — input graph (directed or undirected).
/// * `source` — source vertex id (`0 ≤ source < vcount()`).
/// * `target` — sink vertex id (`0 ≤ target < vcount()`,
///   `target != source`).
///
/// # Returns
///
/// The maximum number of internally vertex-disjoint `source → target`
/// paths as `i64`, plus the count of direct `source → target` edges.
/// Returns `0` when no path exists at all.
///
/// # Errors
///
/// Same as
/// [`st_vertex_connectivity`](super::st_vertex_connectivity::st_vertex_connectivity):
///
/// * [`IgraphError::VertexOutOfRange`] if `source` or `target` is
///   outside `[0, vcount())`.
/// * [`IgraphError::InvalidArgument`] if `source == target` (igraph C
///   returns `IGRAPH_UNIMPLEMENTED` for this case; we use
///   `InvalidArgument` for parity with the rest of our flow module),
///   or if `vcount() < 2`.
/// * [`IgraphError::Internal`] if the sum `vc + direct_count`
///   overflows `i64` (unreachable in practice — both summands are
///   bounded above by `ecount() ≤ u32::MAX`).
///
/// [`IgraphError::VertexOutOfRange`]: crate::core::IgraphError::VertexOutOfRange
/// [`IgraphError::InvalidArgument`]: crate::core::IgraphError::InvalidArgument
/// [`IgraphError::Internal`]: crate::core::IgraphError::Internal
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, vertex_disjoint_paths};
///
/// // Two parallel directed paths 0→1→3 and 0→2→3 → 2 vertex-disjoint paths.
/// let mut g = Graph::new(4, true).unwrap();
/// for (s, t) in [(0u32, 1u32), (1, 3), (0, 2), (2, 3)] {
///     g.add_edge(s, t).unwrap();
/// }
/// assert_eq!(vertex_disjoint_paths(&g, 0, 3).unwrap(), 2);
/// ```
pub fn vertex_disjoint_paths(
    graph: &Graph,
    source: VertexId,
    target: VertexId,
) -> IgraphResult<i64> {
    let vc = st_vertex_connectivity(graph, source, target, VconnNei::Ignore)?;
    let direct = graph.get_all_eids_between(source, target)?.len();
    let direct_i64 = i64::try_from(direct)
        .map_err(|_| IgraphError::Internal("direct-edge count exceeds i64::MAX"))?;
    vc.checked_add(direct_i64).ok_or(IgraphError::Internal(
        "vertex_disjoint_paths sum overflows i64",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::IgraphError;

    #[test]
    fn rejects_source_equals_target() {
        let mut g = Graph::new(2, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        let err = vertex_disjoint_paths(&g, 0, 0).unwrap_err();
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn rejects_out_of_range_source() {
        let g = Graph::new(2, true).expect("graph");
        let err = vertex_disjoint_paths(&g, 5, 0).unwrap_err();
        match err {
            IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!(id, 5);
                assert_eq!(n, 2);
            }
            other => panic!("expected VertexOutOfRange, got {other:?}"),
        }
    }

    #[test]
    fn rejects_out_of_range_target() {
        let g = Graph::new(2, true).expect("graph");
        let err = vertex_disjoint_paths(&g, 0, 5).unwrap_err();
        match err {
            IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!(id, 5);
                assert_eq!(n, 2);
            }
            other => panic!("expected VertexOutOfRange, got {other:?}"),
        }
    }

    #[test]
    fn isolated_endpoints_have_no_paths() {
        let g = Graph::new(4, true).expect("graph");
        assert_eq!(vertex_disjoint_paths(&g, 0, 3).expect("paths"), 0);
    }

    #[test]
    fn single_direct_edge_one_path() {
        let mut g = Graph::new(2, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        assert_eq!(vertex_disjoint_paths(&g, 0, 1).expect("paths"), 1);
    }

    #[test]
    fn two_parallel_directed_paths() {
        // 0→1→3 and 0→2→3 → 2 internally vertex-disjoint paths
        // (vertices 1 and 2 each on one path, no shared interior).
        let mut g = Graph::new(4, true).expect("graph");
        for (s, t) in [(0u32, 1u32), (1, 3), (0, 2), (2, 3)] {
            g.add_edge(s, t).expect("edge");
        }
        assert_eq!(vertex_disjoint_paths(&g, 0, 3).expect("paths"), 2);
    }

    #[test]
    fn direct_edge_plus_one_via_interior() {
        // 0→1 direct, 0→2→1 via vertex 2. 1 direct + 1 internally-disjoint = 2.
        let mut g = Graph::new(3, true).expect("graph");
        for (s, t) in [(0u32, 1u32), (0, 2), (2, 1)] {
            g.add_edge(s, t).expect("edge");
        }
        assert_eq!(vertex_disjoint_paths(&g, 0, 1).expect("paths"), 2);
    }

    #[test]
    fn parallel_direct_arcs() {
        // 4 parallel arcs 0→1 → 4 trivially-disjoint paths.
        let mut g = Graph::new(2, true).expect("graph");
        for _ in 0..4 {
            g.add_edge(0, 1).expect("edge");
        }
        assert_eq!(vertex_disjoint_paths(&g, 0, 1).expect("paths"), 4);
    }

    #[test]
    fn directed_no_reverse_path() {
        // 0 → 1 → 2 → 3 directed chain.
        let mut g = Graph::new(4, true).expect("graph");
        for (s, t) in [(0u32, 1u32), (1, 2), (2, 3)] {
            g.add_edge(s, t).expect("edge");
        }
        assert_eq!(vertex_disjoint_paths(&g, 0, 3).expect("paths"), 1);
        // Reverse direction has no paths.
        assert_eq!(vertex_disjoint_paths(&g, 3, 0).expect("paths"), 0);
    }

    #[test]
    fn c_unit_fixture_directed() {
        // Mirrors igraph_vertex_disjoint_paths.c lines 28-39 — directed
        // 7-vertex graph with self-loop, mutual arcs, and a direct s→t
        // arc. vdp(0,5)=3, vdp(1,3)=2, vdp(4,0)=0.
        let mut g = Graph::new(7, true).expect("graph");
        let arcs = [
            (0u32, 1u32),
            (0, 2),
            (1, 2),
            (1, 3),
            (2, 4),
            (3, 4),
            (3, 5),
            (4, 5),
            (0, 5),
            (3, 3),
            (5, 2),
            (1, 3),
            (3, 1),
        ];
        for (s, t) in arcs {
            g.add_edge(s, t).expect("edge");
        }
        assert_eq!(vertex_disjoint_paths(&g, 0, 5).expect("paths"), 3);
        assert_eq!(vertex_disjoint_paths(&g, 1, 3).expect("paths"), 2);
        assert_eq!(vertex_disjoint_paths(&g, 4, 0).expect("paths"), 0);
    }

    #[test]
    fn c_unit_fixture_undirected() {
        // Mirrors igraph_vertex_disjoint_paths.c lines 41-47 — same
        // graph but converted to undirected. vdp(4,0)=3, vdp(1,3)=5.
        let mut g = Graph::new(7, false).expect("graph");
        // The C test calls igraph_to_undirected(IGRAPH_TO_UNDIRECTED_EACH)
        // which keeps every directed arc as a separate undirected edge.
        // We faithfully replay the original arc list.
        let arcs = [
            (0u32, 1u32),
            (0, 2),
            (1, 2),
            (1, 3),
            (2, 4),
            (3, 4),
            (3, 5),
            (4, 5),
            (0, 5),
            (3, 3),
            (5, 2),
            (1, 3),
            (3, 1),
        ];
        for (s, t) in arcs {
            g.add_edge(s, t).expect("edge");
        }
        assert_eq!(vertex_disjoint_paths(&g, 4, 0).expect("paths"), 3);
        assert_eq!(vertex_disjoint_paths(&g, 1, 3).expect("paths"), 5);
    }

    #[test]
    fn vdp_equals_vc_ignore_plus_direct_count_with_direct_edge() {
        // When a direct s→t edge exists, vdp == vc(Ignore) + direct_count.
        // VconnNei::NumberOfNodes returns the sentinel vcount() in this
        // case, so it is NOT equivalent (verified inline below).
        let mut g = Graph::new(3, true).expect("graph");
        for (s, t) in [(0u32, 1u32), (0, 2), (2, 1)] {
            g.add_edge(s, t).expect("edge");
        }
        let vdp = vertex_disjoint_paths(&g, 0, 1).expect("vdp");
        let vc_ign = st_vertex_connectivity(&g, 0, 1, VconnNei::Ignore).expect("vc");
        let direct =
            i64::try_from(g.get_all_eids_between(0, 1).expect("eids").len()).expect("direct fits");
        assert_eq!(vdp, vc_ign + direct);

        // Sentinel sanity-check: NumberOfNodes returns vcount() (=3) here,
        // confirming it is not interchangeable with vertex_disjoint_paths.
        let vc_non = st_vertex_connectivity(&g, 0, 1, VconnNei::NumberOfNodes).expect("vc");
        assert_eq!(vc_non, 3);
    }

    #[test]
    fn matches_st_vertex_connectivity_ignore_when_no_direct_edge() {
        // When no direct s→t edge exists, vdp must equal
        // st_vertex_connectivity(Ignore) since the direct-edge count is 0.
        let mut g = Graph::new(6, false).expect("graph");
        for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 5)] {
            g.add_edge(u, v).expect("edge");
        }
        let vdp = vertex_disjoint_paths(&g, 0, 5).expect("vdp");
        let vc_ign = st_vertex_connectivity(&g, 0, 5, VconnNei::Ignore).expect("vc");
        assert_eq!(vdp, vc_ign);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    //! Proptest cross-validates the Menger equality
    //! `vertex_disjoint_paths(g, s, t) ==
    //!  st_vertex_connectivity(g, s, t, Ignore) + |direct(s, t)|`
    //! across random unit-cap graphs. FL-013 itself proptests against
    //! `st_edge_connectivity` (Whitney bound), so this transitively
    //! inherits independent oracles.

    use super::*;
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
        fn vdp_equals_vc_ignore_plus_direct(
            seed in any::<u64>(),
            n in 2u32..7,
            m in 1u32..12,
            directed in any::<bool>(),
        ) {
            let g = build_random(seed, n, m, directed);
            let s = u32::try_from(seed % u64::from(n)).expect("modulo fits");
            let t_raw = u32::try_from(xorshift(seed) % u64::from(n)).expect("modulo fits");
            let t = if t_raw == s { (s + 1) % n } else { t_raw };
            prop_assume!(s != t);

            let vdp = vertex_disjoint_paths(&g, s, t).expect("vdp");
            let vc = st_vertex_connectivity(&g, s, t, VconnNei::Ignore).expect("vc");
            let direct = i64::try_from(g.get_all_eids_between(s, t).expect("eids").len())
                .expect("direct fits");
            prop_assert_eq!(
                vdp,
                vc + direct,
                "vdp={} vc(Ignore)={} direct={} (n={}, m={}, directed={}, seed={})",
                vdp, vc, direct, n, m, directed, seed
            );
        }

        #[test]
        fn vdp_le_n(
            seed in any::<u64>(),
            n in 2u32..7,
            m in 1u32..12,
            directed in any::<bool>(),
        ) {
            // Trivially-disjoint paths can't exceed parallel-edge count
            // plus internal vertex slack, both bounded by graph size.
            // Use ecount as a slack-free upper bound (an edge per path).
            let g = build_random(seed, n, m, directed);
            let s = u32::try_from(seed % u64::from(n)).expect("modulo fits");
            let t_raw = u32::try_from(xorshift(seed) % u64::from(n)).expect("modulo fits");
            let t = if t_raw == s { (s + 1) % n } else { t_raw };
            prop_assume!(s != t);

            let vdp = vertex_disjoint_paths(&g, s, t).expect("vdp");
            let ec_bound = i64::try_from(g.ecount()).expect("ecount fits i64");
            prop_assert!(vdp <= ec_bound, "vdp={} > ecount={}", vdp, ec_bound);
        }
    }
}
