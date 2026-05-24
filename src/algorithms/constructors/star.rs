//! Star graph constructor (ALGO-CN-002).
//!
//! Counterpart of `igraph_star()` in
//! `references/igraph/src/constructors/regular.c:75-141`.
//!
//! A star graph on `n` vertices designates one vertex as the *centre*
//! (often vertex `0`); every other vertex is a leaf connected only to
//! the centre. The shape of the edge is controlled by a four-way
//! [`StarMode`]:
//!
//! * [`StarMode::Out`] — directed, edges flow **from** the centre to
//!   each leaf (`center → leaf`).
//! * [`StarMode::In`] — directed, edges flow **from each leaf** to the
//!   centre (`leaf → center`).
//! * [`StarMode::Mutual`] — directed, both arcs `center → leaf` and
//!   `leaf → center` are emitted for every leaf.
//! * [`StarMode::Undirected`] — undirected; the underlying graph is
//!   directed = `false` and a single edge `{center, leaf}` is added per
//!   leaf.
//!
//! Edge enumeration mirrors the upstream C loop exactly: leaves are
//! visited in vertex-id order `[0, center) ∪ (center, n)` and the arc
//! direction is determined by the mode. For `Mutual` the forward arc
//! `center → leaf` is always emitted before the back-arc `leaf → center`
//! for that leaf.
//!
//! Edge counts:
//!
//! * `n = 0` or `n = 1` — no edges.
//! * `n ≥ 2`, mode ∈ {Out, In, Undirected} — `n - 1` edges.
//! * `n ≥ 2`, mode = Mutual — `2(n - 1)` edges.
//!
//! Time complexity: O(|V|).

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Direction policy for [`star_graph`].
///
/// Mirrors `igraph_star_mode_t` in the C reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StarMode {
    /// Directed star, arcs flow from the centre to each leaf.
    Out,
    /// Directed star, arcs flow from each leaf to the centre.
    In,
    /// Directed star with both arcs per leaf.
    Mutual,
    /// Undirected star.
    Undirected,
}

impl StarMode {
    fn is_directed(self) -> bool {
        !matches!(self, StarMode::Undirected)
    }
}

/// Build a star graph on `n` vertices with the given `center` and arc
/// policy `mode`.
///
/// See the module-level docs for the precise role of each mode.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — `center >= n` for `n > 0`.
/// * [`IgraphError::Internal`] — implied edge count would overflow
///   `usize` (only reachable on 32-bit targets with absurdly large `n`).
///
/// # Examples
///
/// ```
/// use rust_igraph::{star_graph, StarMode};
///
/// // Undirected star with 5 leaves around vertex 0.
/// let s = star_graph(6, StarMode::Undirected, 0).unwrap();
/// assert_eq!(s.vcount(), 6);
/// assert_eq!(s.ecount(), 5);
/// assert!(!s.is_directed());
/// ```
pub fn star_graph(n: u32, mode: StarMode, center: u32) -> IgraphResult<Graph> {
    if n == 0 {
        return Graph::new(0, mode.is_directed());
    }

    if center >= n {
        return Err(IgraphError::InvalidArgument(format!(
            "star_graph: center vertex {center} is out of range for n = {n}"
        )));
    }

    let directed = mode.is_directed();
    let leaves = (n as usize) - 1;
    let per_leaf = if matches!(mode, StarMode::Mutual) {
        2
    } else {
        1
    };
    let total_edges = leaves
        .checked_mul(per_leaf)
        .ok_or(IgraphError::Internal("star_graph edge count overflow"))?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(total_edges);

    for leaf in 0..n {
        if leaf == center {
            continue;
        }
        match mode {
            StarMode::Out => edges.push((center, leaf)),
            StarMode::In | StarMode::Undirected => edges.push((leaf, center)),
            StarMode::Mutual => {
                edges.push((center, leaf));
                edges.push((leaf, center));
            }
        }
    }

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..m)
            .map(|eid| g.edge(eid).expect("edge id in bounds"))
            .collect()
    }

    #[test]
    fn empty_graph_all_modes() {
        for mode in [
            StarMode::Out,
            StarMode::In,
            StarMode::Mutual,
            StarMode::Undirected,
        ] {
            let g = star_graph(0, mode, 0).expect("star n=0");
            assert_eq!(g.vcount(), 0);
            assert_eq!(g.ecount(), 0);
            assert_eq!(g.is_directed(), mode.is_directed());
        }
    }

    #[test]
    fn singleton_has_no_edges() {
        for mode in [
            StarMode::Out,
            StarMode::In,
            StarMode::Mutual,
            StarMode::Undirected,
        ] {
            let g = star_graph(1, mode, 0).expect("star n=1");
            assert_eq!(g.vcount(), 1);
            assert_eq!(g.ecount(), 0);
        }
    }

    #[test]
    fn out_mode_emits_center_to_leaf_arcs() {
        let g = star_graph(5, StarMode::Out, 0).expect("out star");
        assert!(g.is_directed());
        assert_eq!(g.ecount(), 4);
        assert_eq!(collect_edges(&g), vec![(0, 1), (0, 2), (0, 3), (0, 4)]);
    }

    #[test]
    fn in_mode_emits_leaf_to_center_arcs() {
        let g = star_graph(5, StarMode::In, 0).expect("in star");
        assert!(g.is_directed());
        assert_eq!(g.ecount(), 4);
        assert_eq!(collect_edges(&g), vec![(1, 0), (2, 0), (3, 0), (4, 0)]);
    }

    #[test]
    fn mutual_mode_emits_both_arcs_per_leaf() {
        let g = star_graph(4, StarMode::Mutual, 0).expect("mutual star");
        assert!(g.is_directed());
        assert_eq!(g.ecount(), 6);
        // Forward arc first, then back-arc — per upstream loop.
        assert_eq!(
            collect_edges(&g),
            vec![(0, 1), (1, 0), (0, 2), (2, 0), (0, 3), (3, 0)]
        );
    }

    #[test]
    fn undirected_mode_emits_one_edge_per_leaf() {
        let g = star_graph(5, StarMode::Undirected, 0).expect("undirected star");
        assert!(!g.is_directed());
        assert_eq!(g.ecount(), 4);
        // Undirected canonicalisation sorts endpoints as (min, max).
        assert_eq!(collect_edges(&g), vec![(0, 1), (0, 2), (0, 3), (0, 4)]);
    }

    #[test]
    fn center_can_be_any_vertex_out_mode() {
        let g = star_graph(5, StarMode::Out, 2).expect("centred at 2");
        // Leaves visited in vertex-id order [0, 1] then [3, 4].
        assert_eq!(collect_edges(&g), vec![(2, 0), (2, 1), (2, 3), (2, 4)]);
    }

    #[test]
    fn center_can_be_any_vertex_undirected_mode() {
        let g = star_graph(5, StarMode::Undirected, 3).expect("centred at 3");
        // For undirected mode the raw arc `(leaf, center)` is canonicalised
        // to `(min(leaf, center), max(leaf, center))` by Graph::add_edges.
        assert_eq!(collect_edges(&g), vec![(0, 3), (1, 3), (2, 3), (3, 4)]);
    }

    #[test]
    fn center_equal_to_n_minus_one_visits_only_lower_leaves() {
        let g = star_graph(4, StarMode::Out, 3).expect("centred at last");
        assert_eq!(g.ecount(), 3);
        assert_eq!(collect_edges(&g), vec![(3, 0), (3, 1), (3, 2)]);
    }

    #[test]
    fn center_out_of_range_errors() {
        let err = star_graph(3, StarMode::Out, 3).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
        let err = star_graph(3, StarMode::Out, 5).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn center_zero_with_n_zero_is_empty_not_error() {
        // n=0 short-circuits before the center bounds check (matches upstream).
        let g = star_graph(0, StarMode::Out, 0).expect("n=0 star");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn leaf_degrees_undirected_are_one_each() {
        let g = star_graph(10, StarMode::Undirected, 0).expect("undirected K1,9");
        for v in 0..10u32 {
            let deg = g.neighbors(v).expect("neighbors").len();
            let expected = if v == 0 { 9 } else { 1 };
            assert_eq!(deg, expected, "vertex {v}");
        }
    }

    #[test]
    fn out_arcs_give_center_out_degree_and_leaves_in_degree_one() {
        let g = star_graph(6, StarMode::Out, 0).expect("out star K1,5");
        // In a directed graph, neighbors() returns OUT-neighbours.
        let center_out = g.neighbors(0).expect("center out").len();
        assert_eq!(center_out, 5);
        for v in 1..6u32 {
            assert_eq!(g.neighbors(v).expect("leaf out").len(), 0);
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_mode() -> impl Strategy<Value = StarMode> {
        prop_oneof![
            Just(StarMode::Out),
            Just(StarMode::In),
            Just(StarMode::Mutual),
            Just(StarMode::Undirected),
        ]
    }

    proptest! {
        #[test]
        fn edge_count_matches_formula(
            n in 0u32..40,
            mode in arb_mode(),
            center_raw in 0u32..40,
        ) {
            // Clamp center within [0, n) when n > 0; otherwise the value is unused.
            let center = if n == 0 { 0 } else { center_raw % n };
            let g = star_graph(n, mode, center).unwrap();
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(g.is_directed(), mode.is_directed());

            let leaves = if n == 0 { 0 } else { n - 1 };
            let per_leaf = if matches!(mode, StarMode::Mutual) { 2 } else { 1 };
            let expected = leaves * per_leaf;
            let m = u32::try_from(g.ecount()).unwrap();
            prop_assert_eq!(m, expected);
        }

        #[test]
        fn invalid_center_returns_error(
            n in 1u32..20,
            mode in arb_mode(),
            offset in 0u32..20,
        ) {
            let bad_center = n + offset;
            prop_assert!(star_graph(n, mode, bad_center).is_err());
        }

        #[test]
        fn every_leaf_is_connected_to_center(
            n in 2u32..30,
            mode in arb_mode(),
            center_raw in 0u32..30,
        ) {
            let center = center_raw % n;
            let g = star_graph(n, mode, center).unwrap();
            // Total degree at the centre = leaves count for directed-asymmetric
            // modes, 2·leaves for mutual (both arcs touch centre), leaves for
            // undirected. In every case, every leaf must share an edge with
            // the centre — i.e. there exists at least one edge whose endpoint
            // set is {leaf, center} for each leaf.
            let m = u32::try_from(g.ecount()).unwrap();
            let edges: Vec<(VertexId, VertexId)> =
                (0..m).map(|e| g.edge(e).unwrap()).collect();
            for leaf in 0..n {
                if leaf == center {
                    continue;
                }
                let found = edges.iter().any(|&(u, v)| {
                    (u == center && v == leaf) || (u == leaf && v == center)
                });
                prop_assert!(found, "leaf {leaf} missing edge with center {center}");
            }
        }
    }
}
