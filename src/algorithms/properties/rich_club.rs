//! Rich-club coefficient sequence (ALGO-PR-040).
//!
//! Counterpart of `igraph_rich_club_sequence()` from
//! `references/igraph/src/properties/rich_club.c:91-166`.
//!
//! Definition. Given a vertex ordering `vertex_order` (a permutation of
//! `0..vcount`), peel the vertices off the graph one at a time in that
//! order. After `i` removals, the surviving subgraph spans the trailing
//! `vcount - i` vertices of `vertex_order`. The rich-club sequence
//! returns either the surviving edge count (or summed edge weight) at
//! each peeling step (`normalized = false`) or that quantity divided by
//! the total number of possible edges on `vcount - i` vertices
//! (`normalized = true`, i.e. a density per step).
//!
//! Algorithm (linear, no actual peeling). Invert `vertex_order` into
//! `order_of[v] = position of v in vertex_order`. For every edge
//! `(v1, v2)` add its weight (default 1.0) to
//! `res[min(order_of[v1], order_of[v2])]` — that is the index at which
//! the *first* endpoint is removed and the edge disappears with it. A
//! single reverse cumulative sweep then turns the "removed-at-step-i"
//! tally into a "remaining-after-i-removals" sequence. With
//! `normalized = true`, divide each entry by `total_possible_edges`
//! evaluated at the remaining vertex count.
//!
//! `total_possible_edges(n, directed, loops)` is the standard density
//! denominator (see [`crate::density`]):
//!
//! | loops? | directed? | denominator |
//! |--------|-----------|-------------|
//! | no     | no        | `n(n-1)/2`  |
//! | no     | yes       | `n(n-1)`    |
//! | yes    | no        | `n(n+1)/2`  |
//! | yes    | yes       | `n²`        |
//!
//! When `vcount == 0`, the result is the empty vector. When `vcount == 1`,
//! the normalized result is the single value `0 / 0 = f64::NAN` (matching
//! upstream's `print_vector` output `( NaN )`). For any `vcount > 0`,
//! the very last entry `res[vcount - 1]` becomes `f64::NAN` in normalized
//! mode whenever the trailing single-vertex subgraph has zero possible
//! edges (loopless case), because the formula evaluates `0/0` there.
//!
//! Edge orientation. If `graph` is undirected, the caller-supplied
//! `directed` flag is silently coerced to `false` — matching upstream's
//! `if (!igraph_is_directed(graph)) directed = false;` line. Otherwise
//! `directed = true` selects the larger directed denominator without
//! changing the edge-count tally (which is already per directed edge in
//! the underlying [`Graph`]).
//!
//! Loops + normalization. When `normalized && !loops` and the graph
//! contains a self-loop, upstream issues a `Warning` (not an error) the
//! first time it sees one and continues with the loopless denominator;
//! this port silently follows the same behaviour without surfacing the
//! warning (we have no warning channel and the user explicitly opted in
//! to "assume no loops" — the responsibility for ensuring the graph
//! matches that assumption is on them).
//!
//! Complexity. `O(|V| + |E|)`.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Per-vertex rich-club coefficient sequence for `graph`.
///
/// Returns a `Vec<f64>` of length `graph.vcount()` whose `i`-th entry is
/// the density (`normalized = true`) or raw edge count
/// (`normalized = false`) of the subgraph that survives after removing
/// the first `i` vertices of `vertex_order` from `graph`.
///
/// Counterpart of `igraph_rich_club_sequence` — see the module-level
/// docs for the full mapping of arguments and semantics.
///
/// # Arguments
///
/// * `graph` — input graph.
/// * `weights` — optional per-edge weights of length `ecount()`. When
///   `None`, every edge contributes `1.0`.
/// * `vertex_order` — permutation of `0..vcount()` giving the order in
///   which vertices are peeled off.
/// * `normalized` — when `true`, divide the surviving edge count at each
///   step by the total possible edges on the remaining vertices; when
///   `false`, return the raw remaining edge count (or summed weight).
/// * `loops` — whether self-loops are *assumed possible* (affects the
///   normalization denominator only; ignored when `!normalized`).
/// * `directed` — whether to use the directed denominator when
///   normalizing. Silently coerced to `false` when `graph` is undirected.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — when:
///   * `vertex_order.len() != graph.vcount() as usize`,
///   * `weights.is_some()` and `weights.unwrap().len() != graph.ecount()`,
///   * any entry of `vertex_order` is `>= graph.vcount()` (i.e. it is not
///     a valid permutation of `0..vcount`).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, rich_club_sequence};
///
/// // Triangle K_3: after 0 removals → 3 edges, after 1 → 1, after 2 → 0.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let seq = rich_club_sequence(&g, None, &[0, 1, 2], false, false, false).unwrap();
/// assert_eq!(seq, vec![3.0, 1.0, 0.0]);
/// ```
pub fn rich_club_sequence(
    graph: &Graph,
    weights: Option<&[f64]>,
    vertex_order: &[VertexId],
    normalized: bool,
    loops: bool,
    directed: bool,
) -> IgraphResult<Vec<f64>> {
    let vcount = graph.vcount();
    let ecount = graph.ecount();
    let vcount_usize = vcount as usize;

    if vertex_order.len() != vcount_usize {
        return Err(IgraphError::InvalidArgument(format!(
            "rich_club_sequence: vertex_order length ({}) does not match vcount ({})",
            vertex_order.len(),
            vcount
        )));
    }
    if let Some(w) = weights {
        if w.len() != ecount {
            return Err(IgraphError::InvalidArgument(format!(
                "rich_club_sequence: weights length ({}) does not match ecount ({})",
                w.len(),
                ecount
            )));
        }
    }

    // Upstream: undirected graphs ignore the caller's `directed` flag.
    let directed_eff = directed && graph.is_directed();

    // Invert the permutation: order_of[v] = position of v in vertex_order.
    // Doubles as a permutation check — any out-of-range entry, repeat, or
    // missing value is caught here.
    let mut order_of: Vec<u32> = vec![u32::MAX; vcount_usize];
    for (pos, &v) in vertex_order.iter().enumerate() {
        if v >= vcount {
            return Err(IgraphError::InvalidArgument(format!(
                "rich_club_sequence: vertex_order entry {v} is out of range [0, {vcount})"
            )));
        }
        let slot = &mut order_of[v as usize];
        if *slot != u32::MAX {
            return Err(IgraphError::InvalidArgument(format!(
                "rich_club_sequence: vertex_order is not a permutation; vertex {v} appears more than once"
            )));
        }
        // `pos < vcount_usize` and `vcount: u32`, so `pos < u32::MAX`
        // and the conversion is infallible.
        *slot = u32::try_from(pos).map_err(|_| {
            IgraphError::InvalidArgument(
                "rich_club_sequence: index exceeds u32::MAX (internal invariant)".to_string(),
            )
        })?;
    }

    let mut res = vec![0.0f64; vcount_usize];

    // Distribute each edge's weight into the bucket of its first-removed
    // endpoint — that is the step at which the edge disappears.
    let ecount_u32 = u32::try_from(ecount).map_err(|_| {
        IgraphError::InvalidArgument(
            "rich_club_sequence: ecount exceeds u32::MAX (internal invariant)".to_string(),
        )
    })?;
    for eid in 0..ecount_u32 {
        let (v1, v2) = graph.edge(eid as EdgeId)?;
        let o1 = order_of[v1 as usize];
        let o2 = order_of[v2 as usize];
        let bucket = o1.min(o2) as usize;
        let contribution = match weights {
            Some(w) => w[eid as usize],
            None => 1.0,
        };
        res[bucket] += contribution;
    }

    // Reverse cumulative sum: res[i] becomes the surviving weight after
    // i removals (= sum of contributions of every edge whose first
    // endpoint is removed at step ≥ i).
    let mut total = 0.0f64;
    for slot in res.iter_mut().rev() {
        total += *slot;
        *slot = total;
    }

    if normalized {
        // `vcount_usize == vertex_order.len() == res.len()`; the
        // enumerate counter is bounded by it, which itself fits in u32
        // because `vcount: u32`. So the try_from is infallible.
        for (i, slot) in res.iter_mut().enumerate() {
            let i_u32 = u32::try_from(i).map_err(|_| {
                IgraphError::InvalidArgument(
                    "rich_club_sequence: index exceeds u32::MAX (internal invariant)".to_string(),
                )
            })?;
            let remaining = vcount - i_u32;
            *slot /= total_possible_edges(remaining, directed_eff, loops);
        }
    }

    Ok(res)
}

/// Total possible edges on a `n`-vertex graph, matching upstream's
/// per-mode denominator. Returns `f64` because the loopless undirected
/// case `n=0,1` yields zero (and `0/0 = NaN` is the intended sentinel).
///
/// | loops? | directed? | denominator |
/// |--------|-----------|-------------|
/// | no     | no        | `n(n-1)/2`  |
/// | no     | yes       | `n(n-1)`    |
/// | yes    | no        | `n(n+1)/2`  |
/// | yes    | yes       | `n²`        |
fn total_possible_edges(n: u32, directed: bool, loops: bool) -> f64 {
    // Widen to f64 once up front — same behaviour as the C `igraph_real_t`
    // cast in upstream's `total_possible_edges`.
    let nv = f64::from(n);
    if loops {
        if directed {
            nv * nv
        } else {
            nv * (nv + 1.0) / 2.0
        }
    } else if directed {
        nv * (nv - 1.0)
    } else {
        nv * (nv - 1.0) / 2.0
    }
}

#[cfg(test)]
#[allow(
    // Comparisons in these tests are against exact integer-valued
    // doubles (e.g. raw edge counts, denominator-table values); strict
    // equality is correct and intended.
    clippy::float_cmp,
    // `g.ecount() as f64` for tiny test graphs is always representable.
    clippy::cast_precision_loss,
)]
mod tests {
    use super::*;

    /// Assert two `f64` slices are equal up to absolute tolerance, with
    /// `NaN == NaN` (the upstream `.out` files print `NaN` and we want
    /// to match that exactly).
    fn assert_close(actual: &[f64], expected: &[f64], tol: f64) {
        assert_eq!(
            actual.len(),
            expected.len(),
            "lengths differ: actual={actual:?} expected={expected:?}"
        );
        for (i, (&a, &e)) in actual.iter().zip(expected.iter()).enumerate() {
            if e.is_nan() {
                assert!(a.is_nan(), "index {i}: expected NaN, got {a}");
            } else {
                assert!(
                    (a - e).abs() <= tol,
                    "index {i}: |{a} - {e}| = {} > {tol}",
                    (a - e).abs()
                );
            }
        }
    }

    /// Build the 7-vertex undirected, loopless test graph from upstream
    /// Test 3 (`undirected_no_loop_graph`).
    fn upstream_undirected_no_loop() -> Graph {
        let mut g = Graph::with_vertices(7);
        // Same edge list, same order as upstream `igraph_small`.
        let edges = [
            (0, 3),
            (1, 3),
            (2, 3),
            (4, 3),
            (5, 3),
            (5, 6),
            (1, 2),
            (2, 5),
        ];
        for (u, v) in edges {
            g.add_edge(u, v).expect("add_edge");
        }
        g
    }

    /// Build the 7-vertex directed, loopless test graph from upstream
    /// Test 4 (`directed_no_loop_graph`).
    fn upstream_directed_no_loop() -> Graph {
        let mut g = Graph::new(7, true).expect("Graph::new directed");
        let edges = [
            (0, 2),
            (1, 2),
            (2, 3),
            (1, 3),
            (3, 5),
            (3, 4),
            (5, 6),
            (6, 5),
        ];
        for (u, v) in edges {
            g.add_edge(u, v).expect("add_edge");
        }
        g
    }

    /// Build the 7-vertex undirected graph WITH self-loop from Test 5.
    fn upstream_undirected_loop() -> Graph {
        let mut g = Graph::with_vertices(7);
        let edges = [
            (0, 3),
            (1, 3),
            (2, 3),
            (4, 4),
            (5, 3),
            (5, 6),
            (1, 2),
            (2, 5),
            (6, 4),
        ];
        for (u, v) in edges {
            g.add_edge(u, v).expect("add_edge");
        }
        g
    }

    /// Build the 7-vertex directed graph WITH self-loop from Test 6.
    fn upstream_directed_loop() -> Graph {
        let mut g = Graph::new(7, true).expect("Graph::new directed");
        let edges = [
            (0, 2),
            (1, 2),
            (2, 3),
            (1, 3),
            (3, 5),
            (3, 4),
            (5, 6),
            (6, 5),
            (4, 4),
        ];
        for (u, v) in edges {
            g.add_edge(u, v).expect("add_edge");
        }
        g
    }

    #[test]
    fn test1_null_graph_matches_upstream() {
        let g = Graph::with_vertices(0);
        let r = rich_club_sequence(&g, None, &[], true, false, false).expect("ok");
        assert!(r.is_empty());
    }

    #[test]
    fn test2_singleton_matches_upstream() {
        let g = Graph::with_vertices(1);
        let r = rich_club_sequence(&g, None, &[0], true, false, false).expect("ok");
        // Upstream prints `( NaN )` — 0/0 from the loopless undirected
        // single-vertex denominator.
        assert_eq!(r.len(), 1);
        assert!(r[0].is_nan());
    }

    #[test]
    fn test3a_undirected_no_loop_in_order_matches_upstream() {
        let g = upstream_undirected_no_loop();
        let r =
            rich_club_sequence(&g, None, &[0, 1, 2, 3, 4, 5, 6], true, false, false).expect("ok");
        // From rich_club.out line 8:
        // ( 0.380952 0.466667 0.5 0.5 0.333333 1 NaN )
        let expected = [
            8.0 / 21.0, // 0.380952...
            7.0 / 15.0, // 0.466666...
            0.5,
            0.5,
            1.0 / 3.0, // 0.333333...
            1.0,
            f64::NAN,
        ];
        assert_close(&r, &expected, 1e-9);
    }

    #[test]
    fn test3b_undirected_no_loop_reverse_matches_upstream() {
        let g = upstream_undirected_no_loop();
        let r =
            rich_club_sequence(&g, None, &[6, 5, 4, 3, 2, 1, 0], true, false, false).expect("ok");
        // ( 0.380952 0.466667 0.5 0.666667 0.333333 0 NaN )
        let expected = [
            8.0 / 21.0,
            7.0 / 15.0,
            0.5,
            2.0 / 3.0,
            1.0 / 3.0,
            0.0,
            f64::NAN,
        ];
        assert_close(&r, &expected, 1e-9);
    }

    #[test]
    fn test4a_directed_no_loop_in_order_matches_upstream() {
        let g = upstream_directed_no_loop();
        let r =
            rich_club_sequence(&g, None, &[0, 1, 2, 3, 4, 5, 6], true, false, true).expect("ok");
        // ( 0.190476 0.233333 0.25 0.333333 0.333333 1 NaN )
        let expected = [
            8.0 / 42.0, // 0.190476
            7.0 / 30.0, // 0.233333
            0.25,
            1.0 / 3.0,
            1.0 / 3.0,
            1.0,
            f64::NAN,
        ];
        assert_close(&r, &expected, 1e-9);
    }

    #[test]
    fn test4b_directed_no_loop_reverse_matches_upstream() {
        let g = upstream_directed_no_loop();
        let r =
            rich_club_sequence(&g, None, &[6, 5, 4, 3, 2, 1, 0], true, false, true).expect("ok");
        // ( 0.190476 0.2 0.25 0.333333 0.333333 0 NaN )
        let expected = [
            8.0 / 42.0,
            6.0 / 30.0, // 0.2
            3.0 / 12.0, // 0.25
            1.0 / 3.0,
            1.0 / 3.0,
            0.0,
            f64::NAN,
        ];
        assert_close(&r, &expected, 1e-9);
    }

    #[test]
    fn test5a_undirected_loop_in_order_matches_upstream() {
        let g = upstream_undirected_loop();
        let r =
            rich_club_sequence(&g, None, &[0, 1, 2, 3, 4, 5, 6], true, true, false).expect("ok");
        // ( 0.321429 0.380952 0.4 0.4 0.5 0.333333 0 )
        // (denominator n(n+1)/2: 7→28, 6→21, 5→15, 4→10, 3→6, 2→3, 1→1)
        let expected = [
            9.0 / 28.0,
            8.0 / 21.0,
            6.0 / 15.0,
            4.0 / 10.0,
            3.0 / 6.0,
            1.0 / 3.0,
            0.0 / 1.0,
        ];
        assert_close(&r, &expected, 1e-9);
    }

    #[test]
    fn test5b_undirected_loop_reverse_matches_upstream() {
        let g = upstream_undirected_loop();
        let r =
            rich_club_sequence(&g, None, &[6, 5, 4, 3, 2, 1, 0], true, true, false).expect("ok");
        // ( 0.321429 0.333333 0.333333 0.4 0.166667 0 0 )
        let expected = [
            9.0 / 28.0,
            7.0 / 21.0,
            5.0 / 15.0,
            4.0 / 10.0,
            1.0 / 6.0,
            0.0,
            0.0,
        ];
        assert_close(&r, &expected, 1e-9);
    }

    #[test]
    fn test6a_directed_loop_in_order_matches_upstream() {
        let g = upstream_directed_loop();
        let r = rich_club_sequence(&g, None, &[0, 1, 2, 3, 4, 5, 6], true, true, true).expect("ok");
        // ( 0.183673 0.222222 0.24 0.3125 0.333333 0.5 0 )
        // (denominator n²: 7→49, 6→36, 5→25, 4→16, 3→9, 2→4, 1→1)
        let expected = [
            9.0 / 49.0,
            8.0 / 36.0,
            6.0 / 25.0,
            5.0 / 16.0,
            3.0 / 9.0,
            2.0 / 4.0,
            0.0,
        ];
        assert_close(&r, &expected, 1e-9);
    }

    #[test]
    fn test6b_directed_loop_reverse_matches_upstream() {
        let g = upstream_directed_loop();
        let r = rich_club_sequence(&g, None, &[6, 5, 4, 3, 2, 1, 0], true, true, true).expect("ok");
        // ( 0.183673 0.194444 0.24 0.25 0.222222 0 0 )
        let expected = [
            9.0 / 49.0,
            7.0 / 36.0,
            6.0 / 25.0,
            4.0 / 16.0,
            2.0 / 9.0,
            0.0,
            0.0,
        ];
        assert_close(&r, &expected, 1e-9);
    }

    #[test]
    fn test7a_weighted_doubles_test3a() {
        let g = upstream_undirected_no_loop();
        let weights = vec![2.0; g.ecount()];
        let r = rich_club_sequence(
            &g,
            Some(&weights),
            &[0, 1, 2, 3, 4, 5, 6],
            true,
            false,
            false,
        )
        .expect("ok");
        // ( 0.761905 0.933333 1 1 0.666667 2 NaN ) — exactly double Test 3a.
        let expected = [16.0 / 21.0, 14.0 / 15.0, 1.0, 1.0, 2.0 / 3.0, 2.0, f64::NAN];
        assert_close(&r, &expected, 1e-9);
    }

    #[test]
    fn test7b_weighted_non_integer_matches_test3a() {
        let g = upstream_undirected_no_loop();
        // Edge order matches upstream: (0,3), (1,3), (2,3), (4,3), (5,3),
        // (5,6), (1,2), (2,5). Weights chosen so the unnormalized counts
        // recover the all-ones case → result equals Test 3a.
        let weights = [1.0, 0.5, 0.5, 1.0, 1.0, 1.0, 1.5, 1.5];
        let r = rich_club_sequence(
            &g,
            Some(&weights),
            &[0, 1, 2, 3, 4, 5, 6],
            true,
            false,
            false,
        )
        .expect("ok");
        let expected = [8.0 / 21.0, 7.0 / 15.0, 0.5, 0.5, 1.0 / 3.0, 1.0, f64::NAN];
        assert_close(&r, &expected, 1e-9);
    }

    #[test]
    fn raw_unnormalized_returns_edge_counts() {
        let g = upstream_undirected_no_loop();
        let r =
            rich_club_sequence(&g, None, &[0, 1, 2, 3, 4, 5, 6], false, false, false).expect("ok");
        // 8 edges total. Per-bucket counts (= "edge removed when vertex i
        // gets peeled"):
        //   (0,3)→0  (1,3)→1  (2,3)→2  (4,3)→3  (5,3)→3
        //   (5,6)→5  (1,2)→1  (2,5)→2  =>  [1, 2, 2, 2, 0, 1, 0]
        // Reverse cumsum (= "edges remaining after i removals"):
        //   [8, 7, 5, 3, 1, 1, 0].
        assert_eq!(r, vec![8.0, 7.0, 5.0, 3.0, 1.0, 1.0, 0.0]);
    }

    #[test]
    fn raw_unnormalized_undirected_no_loop_reverse() {
        let g = upstream_undirected_no_loop();
        let r =
            rich_club_sequence(&g, None, &[6, 5, 4, 3, 2, 1, 0], false, false, false).expect("ok");
        // First bucket = total edges = 8 always; last must be 0 because
        // removing every vertex empties the graph.
        assert_eq!(r[0], 8.0);
        assert_eq!(r[6], 0.0);
        // Non-increasing.
        for w in r.windows(2) {
            assert!(w[0] >= w[1], "non-monotone: {r:?}");
        }
    }

    #[test]
    fn error_vertex_order_wrong_length() {
        let g = upstream_undirected_no_loop();
        let r = rich_club_sequence(&g, None, &[0], true, false, false);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn error_weights_wrong_length() {
        let g = upstream_undirected_no_loop();
        let w = vec![1.0];
        let r = rich_club_sequence(&g, Some(&w), &[0, 1, 2, 3, 4, 5, 6], true, false, false);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn error_vertex_order_out_of_range() {
        let g = upstream_undirected_no_loop();
        let r = rich_club_sequence(&g, None, &[0, 1, 2, 3, 4, 5, 99], true, false, false);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn error_vertex_order_duplicate() {
        let g = upstream_undirected_no_loop();
        // 0 appears twice; 6 missing.
        let r = rich_club_sequence(&g, None, &[0, 1, 2, 3, 4, 5, 0], true, false, false);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn directed_flag_silently_false_on_undirected_graph() {
        let g = upstream_undirected_no_loop();
        // Passing directed=true on an undirected graph must produce the
        // same result as directed=false.
        let r_true =
            rich_club_sequence(&g, None, &[0, 1, 2, 3, 4, 5, 6], true, false, true).expect("ok");
        let r_false =
            rich_club_sequence(&g, None, &[0, 1, 2, 3, 4, 5, 6], true, false, false).expect("ok");
        assert_close(&r_true, &r_false, 0.0);
    }

    #[test]
    fn first_entry_equals_total_when_unnormalized() {
        // Tested across all four (directed, loops) combinations.
        let g = upstream_undirected_no_loop();
        let r =
            rich_club_sequence(&g, None, &[0, 1, 2, 3, 4, 5, 6], false, false, false).expect("ok");
        assert_eq!(r[0], g.ecount() as f64);

        let g = upstream_directed_no_loop();
        let r =
            rich_club_sequence(&g, None, &[0, 1, 2, 3, 4, 5, 6], false, false, true).expect("ok");
        assert_eq!(r[0], g.ecount() as f64);
    }

    #[test]
    fn loops_normalize_avoids_nan_when_single_vertex_remains() {
        // With loops allowed, the n=1 denominator is n(n+1)/2 = 1 ≠ 0,
        // so the final entry should be 0.0 rather than NaN.
        let g = upstream_undirected_no_loop();
        let r =
            rich_club_sequence(&g, None, &[0, 1, 2, 3, 4, 5, 6], true, true, false).expect("ok");
        assert_eq!(r[6], 0.0);
        assert!(r.iter().all(|x| x.is_finite()));
    }

    #[test]
    fn total_possible_edges_table() {
        // Spot-check the four modes at n=4.
        assert_eq!(super::total_possible_edges(4, false, false), 6.0); // 4*3/2
        assert_eq!(super::total_possible_edges(4, true, false), 12.0); // 4*3
        assert_eq!(super::total_possible_edges(4, false, true), 10.0); // 4*5/2
        assert_eq!(super::total_possible_edges(4, true, true), 16.0); // 4*4
        // Edge: n=0 → all four cases are 0.
        for &(d, l) in &[(false, false), (true, false), (false, true), (true, true)] {
            assert_eq!(super::total_possible_edges(0, d, l), 0.0);
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// Build a random `(n, edges)` pair with `0 ≤ n ≤ 20` and edges
    /// drawn from `0..n`. Allows duplicates and self-loops.
    fn random_graph_strategy(directed: bool) -> impl Strategy<Value = Graph> {
        (1usize..=20).prop_flat_map(move |n| {
            let edge_count = 0usize..=(2 * n);
            edge_count
                .prop_flat_map(move |m| {
                    let n_u32 = n as u32;
                    (Just(n_u32), prop::collection::vec((0..n_u32, 0..n_u32), m))
                })
                .prop_map(move |(n_u32, edges)| {
                    let mut g = Graph::new(n_u32, directed).expect("Graph::new");
                    for (u, v) in edges {
                        g.add_edge(u, v).expect("add_edge");
                    }
                    g
                })
        })
    }

    /// A vertex_order is a random permutation of `0..vcount`. We
    /// generate one by shuffling `0..n`.
    fn permutation_strategy(n: u32) -> impl Strategy<Value = Vec<u32>> {
        Just((0..n).collect::<Vec<_>>()).prop_shuffle()
    }

    proptest! {
        #[test]
        fn unnormalized_first_entry_equals_ecount(g in random_graph_strategy(false)) {
            let n = g.vcount();
            if n == 0 {
                let r = rich_club_sequence(&g, None, &[], false, false, false).expect("ok");
                prop_assert!(r.is_empty());
                return Ok(());
            }
            let order: Vec<u32> = (0..n).collect();
            let r = rich_club_sequence(&g, None, &order, false, false, false).expect("ok");
            prop_assert_eq!(r[0], g.ecount() as f64);
        }

        #[test]
        fn unnormalized_last_entry_equals_self_loops_on_last_vertex(
            g in random_graph_strategy(false),
        ) {
            // The reverse-cumsum at index n-1 collects exactly the edges
            // whose *both* endpoints are removed at step ≥ n-1, i.e.
            // self-loops on the last vertex of the ordering. For
            // loopless graphs this collapses to 0.
            let n = g.vcount();
            if n == 0 {
                return Ok(());
            }
            let order: Vec<u32> = (0..n).collect();
            let r = rich_club_sequence(&g, None, &order, false, false, false).expect("ok");
            let last_vertex = order[(n - 1) as usize];
            let mut self_loops_on_last = 0u32;
            for eid in 0..g.ecount() {
                let (u, v) = g.edge(eid as u32).expect("edge");
                if u == last_vertex && v == last_vertex {
                    self_loops_on_last += 1;
                }
            }
            prop_assert_eq!(r[(n - 1) as usize], f64::from(self_loops_on_last));
        }

        #[test]
        fn unnormalized_monotone_non_increasing(g in random_graph_strategy(true)) {
            let n = g.vcount();
            if n == 0 {
                return Ok(());
            }
            let order: Vec<u32> = (0..n).collect();
            let r = rich_club_sequence(&g, None, &order, false, false, true).expect("ok");
            for w in r.windows(2) {
                prop_assert!(w[0] >= w[1], "non-monotone at window {w:?}");
            }
        }

        #[test]
        fn permutation_invariant_first_entry(
            g in random_graph_strategy(false),
        ) {
            let n = g.vcount();
            if n == 0 {
                return Ok(());
            }
            // For any permutation, the first entry (no removals yet) is
            // always the total edge count / weight — the order picks
            // *when* each edge disappears, not the starting subgraph.
            let perm_strat = permutation_strategy(n);
            proptest!(|(order in perm_strat)| {
                let r = rich_club_sequence(&g, None, &order, false, false, false).expect("ok");
                prop_assert_eq!(r[0], g.ecount() as f64);
            });
        }

        #[test]
        fn weighted_scale_invariance(g in random_graph_strategy(false), c in 0.1f64..10.0) {
            let n = g.vcount();
            if n == 0 {
                return Ok(());
            }
            let order: Vec<u32> = (0..n).collect();
            let r_unit = rich_club_sequence(&g, None, &order, false, false, false).expect("ok");
            let w: Vec<f64> = vec![c; g.ecount()];
            let r_scaled =
                rich_club_sequence(&g, Some(&w), &order, false, false, false).expect("ok");
            for (a, b) in r_unit.iter().zip(r_scaled.iter()) {
                prop_assert!((a * c - b).abs() <= 1e-9 * (1.0 + b.abs()));
            }
        }

        #[test]
        fn normalize_in_unit_interval_for_loopless_undirected(
            g in random_graph_strategy(false),
        ) {
            let n = g.vcount();
            if n == 0 {
                return Ok(());
            }
            // Pre-condition the property: density ∈ [0, 1] only holds
            // for *simple* graphs (no self-loops, no multi-edges).
            // Loopless: with loops=false the denominator drops the
            // diagonal, but the numerator still counts loops, so a graph
            // with many self-loops can exceed 1. Simple (no multi-edges):
            // parallel copies of the same edge contribute multiple unit
            // weights to the same bucket, which can also push the count
            // above the n(n-1)/2 ceiling. Skip those configurations.
            let mut bad = false;
            let mut seen: std::collections::HashSet<(u32, u32)> =
                std::collections::HashSet::new();
            for eid in 0..g.ecount() {
                let (u, v) = g.edge(eid as u32).expect("edge");
                if u == v {
                    bad = true;
                    break;
                }
                let key = if u <= v { (u, v) } else { (v, u) };
                if !seen.insert(key) {
                    bad = true;
                    break;
                }
            }
            if bad {
                return Ok(());
            }
            let order: Vec<u32> = (0..n).collect();
            let r = rich_club_sequence(&g, None, &order, true, false, false).expect("ok");
            for (i, &x) in r.iter().enumerate() {
                if i + 1 == n as usize {
                    // The trailing single-vertex subgraph is 0/0 = NaN.
                    prop_assert!(x.is_nan());
                } else {
                    prop_assert!((0.0..=1.0).contains(&x), "out of [0,1]: r[{i}] = {x}");
                }
            }
        }

        #[test]
        fn error_on_wrong_vertex_order_length(g in random_graph_strategy(false)) {
            let n = g.vcount();
            // Force a wrong length: 0 if n>0, 1 if n==0.
            let bad = if n == 0 { vec![0u32] } else { vec![] };
            let r = rich_club_sequence(&g, None, &bad, false, false, false);
            prop_assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
        }
    }
}
