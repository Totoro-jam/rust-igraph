//! Wheel graph constructor (ALGO-CN-003).
//!
//! Counterpart of `igraph_wheel()` in
//! `references/igraph/src/constructors/regular.c:145-287`.
//!
//! A *wheel graph* on `n` vertices is the union of a star and a cycle:
//! one vertex (the **centre**) is connected to every other vertex via
//! star-spokes, and those `n - 1` rim vertices are linked by a cycle.
//! Wheel layout is controlled by [`WheelMode`], whose four cases line
//! up with [`crate::StarMode`]:
//!
//! * [`WheelMode::Out`] — directed, every spoke flows **from** the
//!   centre to a leaf, every rim edge is `prev → next` (visited in the
//!   `[0, center) ∪ (center, n)` raw vertex order).
//! * [`WheelMode::In`] — directed, every spoke flows **from each leaf**
//!   to the centre, rim direction `prev → next` matches `Out`.
//! * [`WheelMode::Mutual`] — directed, every spoke and every rim edge
//!   is emitted in **both** directions. Star-spokes are added first
//!   (forward arc before back-arc, per leaf); the rim is then appended
//!   as `e_0, e_1, …, e_{n-2}, rev(e_{n-2}), rev(e_{n-3}), …, rev(e_0)`
//!   — i.e. forward rim sweep followed by the reverse of each forward
//!   arc in reverse-discovery order, matching the upstream C loop
//!   verbatim.
//! * [`WheelMode::Undirected`] — undirected; each spoke and each rim
//!   edge is added once, `Graph::add_edges` canonicalises endpoints to
//!   `(min, max)`.
//!
//! Edge counts:
//!
//! * `n = 0` or `n = 1` — wheel ≡ star; no edges.
//! * `n ≥ 2`, mode ∈ {Out, In, Undirected} — `2(n − 1)` edges
//!   (`n − 1` spokes plus `n − 1` rim edges).
//! * `n ≥ 2`, mode = Mutual — `4(n − 1)` edges.
//!
//! Degenerate two- and three-vertex wheels are intentionally non-simple
//! (upstream `igraph_wheel` documents this): the two-vertex wheel
//! contains a self-loop on the only rim vertex (the rim collapses to a
//! 1-cycle), and the three-vertex wheel produces parallel edges between
//! the two rim vertices (rim collapses to a 2-cycle).
//!
//! Time complexity: O(|V|).

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

use super::star::{StarMode, star_graph};

/// Direction policy for [`wheel_graph`].
///
/// Mirrors `igraph_wheel_mode_t` in the C reference. The four variants
/// map one-to-one onto [`StarMode`] for the spoke layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WheelMode {
    /// Directed wheel, all spokes and rim arcs flow forward.
    Out,
    /// Directed wheel, all spokes flow from rim to centre; rim arcs
    /// still flow `prev → next`.
    In,
    /// Directed wheel with both arcs on every spoke and every rim edge.
    Mutual,
    /// Undirected wheel.
    Undirected,
}

impl WheelMode {
    #[cfg(test)]
    fn is_directed(self) -> bool {
        !matches!(self, WheelMode::Undirected)
    }

    fn star_mode(self) -> StarMode {
        match self {
            WheelMode::Out => StarMode::Out,
            WheelMode::In => StarMode::In,
            WheelMode::Mutual => StarMode::Mutual,
            WheelMode::Undirected => StarMode::Undirected,
        }
    }
}

/// Build a wheel graph on `n` vertices with the given `center` and arc
/// policy `mode`.
///
/// See the module-level docs for the precise role of each mode and the
/// degenerate two/three-vertex shapes.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — `center >= n` for `n > 0` (the
///   spoke layer rejects it via [`star_graph`]).
/// * [`IgraphError::Internal`] — implied rim edge count would overflow
///   `usize` (only reachable on 32-bit targets with absurdly large `n`).
///
/// # Examples
///
/// ```
/// use rust_igraph::{wheel_graph, WheelMode};
///
/// // Undirected wheel W6 (5 rim vertices around centre 0).
/// let w = wheel_graph(6, WheelMode::Undirected, 0).unwrap();
/// assert_eq!(w.vcount(), 6);
/// assert_eq!(w.ecount(), 2 * (6 - 1)); // 5 spokes + 5 rim edges
/// assert!(!w.is_directed());
/// ```
pub fn wheel_graph(n: u32, mode: WheelMode, center: u32) -> IgraphResult<Graph> {
    // Spoke layer + parameter validation (center bounds + n=0 short-circuit)
    // share `star_graph`, mirroring the upstream "create star, then add rim"
    // composition.
    let mut graph = star_graph(n, mode.star_mode(), center)?;

    // n <= 1 → wheel ≡ star (no rim possible).
    if n <= 1 {
        return Ok(graph);
    }

    let rim_count = (n as usize) - 1;
    let forward_count = rim_count;
    let per_edge = if matches!(mode, WheelMode::Mutual) {
        2
    } else {
        1
    };
    let total_rim_edges = forward_count
        .checked_mul(per_edge)
        .ok_or(IgraphError::Internal("wheel_graph rim edge count overflow"))?;
    let mut rim: Vec<(VertexId, VertexId)> = Vec::with_capacity(total_rim_edges);

    // First (n - 2) rim arcs, then the wrap-around — matching the C
    // index arithmetic in `regular.c` lines 244-269. The branch skips
    // `center` when stepping `i → i+1` so that `center` never appears
    // on the rim.
    if n >= 3 {
        for i in 0..(n - 2) {
            let (a, b) = if i < center {
                let head = i;
                let tail = if i + 1 < center { i + 1 } else { i + 2 };
                (head, tail)
            } else {
                (i + 1, i + 2)
            };
            rim.push((a, b));
        }
    }

    // Wrap-around: last rim arc connects the largest rim vertex back to
    // the smallest rim vertex.
    let wrap_head = if n - 2 < center { n - 2 } else { n - 1 };
    let wrap_tail = u32::from(center == 0);
    rim.push((wrap_head, wrap_tail));

    // Mutual mode: append the reverse of every forward arc in
    // reverse-discovery order (forward arc i appended at "end - i"
    // upstream → reversal flips both endpoint order and arc order).
    if matches!(mode, WheelMode::Mutual) {
        let forwards: Vec<(VertexId, VertexId)> = rim.clone();
        for &(u, v) in forwards.iter().rev() {
            rim.push((v, u));
        }
    }

    graph.add_edges(rim)?;
    Ok(graph)
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
            WheelMode::Out,
            WheelMode::In,
            WheelMode::Mutual,
            WheelMode::Undirected,
        ] {
            let g = wheel_graph(0, mode, 0).expect("wheel n=0");
            assert_eq!(g.vcount(), 0);
            assert_eq!(g.ecount(), 0);
            assert_eq!(g.is_directed(), mode.is_directed());
        }
    }

    #[test]
    fn singleton_is_star() {
        for mode in [
            WheelMode::Out,
            WheelMode::In,
            WheelMode::Mutual,
            WheelMode::Undirected,
        ] {
            let g = wheel_graph(1, mode, 0).expect("wheel n=1");
            assert_eq!(g.vcount(), 1);
            assert_eq!(g.ecount(), 0);
        }
    }

    #[test]
    fn two_vertex_wheel_has_self_loop_on_rim() {
        // For n=2 the rim collapses to a 1-cycle (wrap-around endpoints
        // coincide on vertex 1 when center=0). Star contributes 1 edge,
        // rim contributes 1 self-loop edge.
        let g = wheel_graph(2, WheelMode::Out, 0).expect("W2");
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 2);
        let edges = collect_edges(&g);
        assert_eq!(edges, vec![(0, 1), (1, 1)]);
    }

    #[test]
    fn three_vertex_wheel_has_parallel_rim_edge() {
        // For n=3 the rim collapses to a 2-cycle: forward (1,2) plus
        // wrap (2,1). Star contributes (0,1), (0,2).
        let g = wheel_graph(3, WheelMode::Out, 0).expect("W3");
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 4);
        let edges = collect_edges(&g);
        assert_eq!(edges, vec![(0, 1), (0, 2), (1, 2), (2, 1)]);
    }

    #[test]
    fn w5_out_mode_edges_in_upstream_order() {
        // n=5, center=0:
        //   star spokes: (0,1) (0,2) (0,3) (0,4)
        //   rim forward: i=0: i<center? false → (1,2)
        //                i=1: false → (2,3)
        //                i=2: false → (3,4)
        //   rim wrap: n-2=3, center=0 → head=n-1=4; center>0? false → tail=1
        //            → (4, 1)
        let g = wheel_graph(5, WheelMode::Out, 0).expect("W5 out");
        assert!(g.is_directed());
        assert_eq!(g.ecount(), 8);
        let edges = collect_edges(&g);
        assert_eq!(
            edges,
            vec![
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 4),
                (1, 2),
                (2, 3),
                (3, 4),
                (4, 1),
            ]
        );
    }

    #[test]
    fn w5_undirected_mode_canon_edges() {
        // Same n=5, center=0 but undirected — endpoints get
        // canonicalised to (min, max). Star-spokes from In/Undirected
        // are emitted as (leaf, center) which canonicalises to
        // (center, leaf).
        let g = wheel_graph(5, WheelMode::Undirected, 0).expect("W5 undir");
        assert!(!g.is_directed());
        assert_eq!(g.ecount(), 8);
        let edges = collect_edges(&g);
        assert_eq!(
            edges,
            vec![
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 4),
                (1, 2),
                (2, 3),
                (3, 4),
                (1, 4),
            ]
        );
    }

    #[test]
    fn w5_mutual_mode_double_count() {
        // n=5 mutual: star contributes 2*(n-1)=8 arcs, rim contributes
        // 2*(n-1)=8 arcs (forward sweep + reverse-discovery sweep).
        let g = wheel_graph(5, WheelMode::Mutual, 0).expect("W5 mutual");
        assert!(g.is_directed());
        assert_eq!(g.ecount(), 16);
        let edges = collect_edges(&g);
        // Star: forward (0,leaf) then back (leaf,0) per leaf.
        // Rim forward: (1,2) (2,3) (3,4) (4,1)
        // Rim reverse-discovery: (1,4) (4,3) (3,2) (2,1)
        assert_eq!(
            edges,
            vec![
                // star
                (0, 1),
                (1, 0),
                (0, 2),
                (2, 0),
                (0, 3),
                (3, 0),
                (0, 4),
                (4, 0),
                // rim forward
                (1, 2),
                (2, 3),
                (3, 4),
                (4, 1),
                // rim reverse-discovery
                (1, 4),
                (4, 3),
                (3, 2),
                (2, 1),
            ]
        );
    }

    #[test]
    fn w5_center_two_skips_center_on_rim() {
        // n=5, center=2 — rim must visit 0,1,3,4 in that order.
        // Loop iterations (i in 0..n-2 = 0..3):
        //   i=0: i<center (0<2) → head=0; i+1<center (1<2) → tail=1 ⇒ (0,1)
        //   i=1: i<center (1<2) → head=1; i+1<center (2<2)? false → tail=i+2=3 ⇒ (1,3)
        //   i=2: i<center? (2<2 false) → (i+1, i+2) = (3, 4)
        // Wrap: n-2=3, center=2 → head=n-1=4 (3<2 false); center>0 → tail=0 ⇒ (4, 0)
        let g = wheel_graph(5, WheelMode::Out, 2).expect("W5 center=2");
        assert_eq!(g.ecount(), 8);
        let edges = collect_edges(&g);
        assert_eq!(
            edges,
            vec![
                // star (out from centre 2)
                (2, 0),
                (2, 1),
                (2, 3),
                (2, 4),
                // rim
                (0, 1),
                (1, 3),
                (3, 4),
                (4, 0),
            ]
        );
    }

    #[test]
    fn center_out_of_range_errors() {
        let err = wheel_graph(3, WheelMode::Out, 3).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
        let err = wheel_graph(3, WheelMode::Out, 5).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rim_vertices_form_cycle_for_n_large() {
        // For n ≥ 4 the rim should be a simple cycle through all
        // (n - 1) leaves, each leaf with rim-degree exactly 2.
        let n = 10u32;
        let center = 4u32;
        let g = wheel_graph(n, WheelMode::Undirected, center).expect("W10");
        // Leaf degrees in the full wheel = 1 (spoke) + 2 (rim) = 3.
        // Centre degree = n - 1 = 9.
        for v in 0..n {
            let deg = g.neighbors(v).expect("neighbors").len();
            let expected = if v == center { (n - 1) as usize } else { 3 };
            assert_eq!(deg, expected, "vertex {v}");
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_mode() -> impl Strategy<Value = WheelMode> {
        prop_oneof![
            Just(WheelMode::Out),
            Just(WheelMode::In),
            Just(WheelMode::Mutual),
            Just(WheelMode::Undirected),
        ]
    }

    proptest! {
        #[test]
        fn edge_count_matches_formula(
            n in 0u32..30,
            mode in arb_mode(),
            center_raw in 0u32..30,
        ) {
            let center = if n == 0 { 0 } else { center_raw % n };
            let g = wheel_graph(n, mode, center).unwrap();
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(g.is_directed(), mode.is_directed());

            let per_layer = n.saturating_sub(1);
            let star_total = per_layer * (if matches!(mode, WheelMode::Mutual) { 2 } else { 1 });
            let rim_total = per_layer * (if matches!(mode, WheelMode::Mutual) { 2 } else { 1 });
            let expected = star_total + rim_total;
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
            prop_assert!(wheel_graph(n, mode, bad_center).is_err());
        }

        #[test]
        fn rim_avoids_center_for_n_at_least_three(
            n in 3u32..30,
            mode in arb_mode(),
            center_raw in 0u32..30,
        ) {
            let center = center_raw % n;
            let g = wheel_graph(n, mode, center).unwrap();
            let m = u32::try_from(g.ecount()).unwrap();
            // Star occupies the first (n-1) or 2(n-1) edge slots; the
            // remaining slots are the rim. Mutual doubles both counts.
            let mul = if matches!(mode, WheelMode::Mutual) { 2 } else { 1 };
            let star_count = (n - 1) * mul;
            for eid in star_count..m {
                let (u, v) = g.edge(eid).unwrap();
                prop_assert_ne!(u, center, "rim edge touches centre");
                prop_assert_ne!(v, center, "rim edge touches centre");
            }
        }
    }
}
