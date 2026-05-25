//! Basic edge-list graph constructor (ALGO-CN-022).
//!
//! Counterpart of `igraph_create()` in
//! `references/igraph/src/constructors/basic_constructors.c:53-81`.
//! Build a graph from a flat edge list, automatically extending the
//! vertex count to accommodate the largest endpoint when `n == 0` or
//! when `n` is smaller than `max(edges) + 1` (upstream behaviour —
//! callers passing `0` get the smallest graph that holds every edge).
//!
//! Upstream `igraph_create` accepts a flat `igraph_vector_int_t` of
//! length `2·|E|` (`[u0, v0, u1, v1, …]`) and reports `IGRAPH_EINVAL`
//! on an odd length and `IGRAPH_EINVVID` on a negative vertex id. The
//! Rust signature is `&[(u32, u32)]`, which eliminates both error
//! paths at the type-system level (pairs cannot be odd; `u32` cannot
//! be negative), so the only remaining runtime check is the silent
//! `n := max(max_edge_endpoint + 1, n)` upper-bound logic.
//!
//! The `igraph_small` varargs convenience is intentionally not
//! ported: Rust call sites use a `Vec`/slice literal like
//! `create(&[(0, 1), (1, 2), (2, 3)], 4, false)` which is already
//! terse, and Rust has no `va_list`.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Build a graph from a flat edge list.
///
/// `edges` is a slice of `(source, target)` pairs interpreted under
/// `directed`. If `n == 0` the vertex count is inferred from the
/// largest endpoint (`max + 1`). If `n > 0` but smaller than
/// `max + 1`, the vertex count is silently raised to `max + 1`
/// (matches upstream `igraph_create` which calls `igraph_add_vertices`
/// to extend).
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — if the implied vertex count
///   would overflow `u32` (only possible when the maximum edge
///   endpoint is `u32::MAX`).
///
/// # Examples
///
/// ```
/// use rust_igraph::create;
///
/// // n = 0 infers vcount from the edges.
/// let g = create(&[(0, 1), (1, 2), (2, 3)], 0, false).unwrap();
/// assert_eq!((g.vcount(), g.ecount()), (4, 3));
///
/// // n > max+1 keeps the requested vertex count.
/// let h = create(&[(0, 1)], 5, false).unwrap();
/// assert_eq!((h.vcount(), h.ecount()), (5, 1));
///
/// // Directed graphs preserve arc orientation.
/// let d = create(&[(0, 1), (1, 0)], 2, true).unwrap();
/// assert_eq!((d.vcount(), d.ecount()), (2, 2));
/// ```
pub fn create(edges: &[(u32, u32)], n: u32, directed: bool) -> IgraphResult<Graph> {
    let inferred = if edges.is_empty() {
        0u32
    } else {
        let mut m = 0u32;
        for &(u, v) in edges {
            if u > m {
                m = u;
            }
            if v > m {
                m = v;
            }
        }
        m.checked_add(1).ok_or_else(|| {
            IgraphError::InvalidArgument(
                "create: vertex id u32::MAX cannot be promoted to a vcount".into(),
            )
        })?
    };

    let vcount = n.max(inferred);
    let mut g = Graph::new(vcount, directed)?;
    g.add_edges(edges.iter().copied())?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn canonical_undirected_edges(g: &Graph) -> BTreeSet<(u32, u32)> {
        let mut s = BTreeSet::new();
        for v in 0..g.vcount() {
            for &u in &g.neighbors(v).expect("neighbors") {
                let key = if v <= u { (v, u) } else { (u, v) };
                s.insert(key);
            }
        }
        s
    }

    fn directed_arc_list(g: &Graph) -> Vec<(u32, u32)> {
        (0..u32::try_from(g.ecount()).expect("ecount fits u32"))
            .map(|eid| g.edge(eid).expect("edge"))
            .collect()
    }

    #[test]
    fn empty_edges_n_zero_yields_null_graph() {
        let g = create(&[], 0, false).expect("create");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn empty_edges_n_positive_yields_isolated_vertices() {
        let g = create(&[], 5, true).expect("create");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn upstream_simple_example_n_zero_infers_four_vertices() {
        // From references/igraph/examples/simple/igraph_create.c:
        // edges = [0,1, 1,2, 2,3, 2,2] with n=0 should give 4 vertices.
        let g = create(&[(0, 1), (1, 2), (2, 3), (2, 2)], 0, false).expect("create");
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 4);
    }

    #[test]
    fn upstream_example_n_ten_keeps_ten_vertices() {
        // Same edges with n=10 should keep all 10 vertices.
        let g = create(&[(0, 1), (1, 2), (2, 3), (2, 2)], 10, false).expect("create");
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 4);
    }

    #[test]
    fn n_smaller_than_max_silently_extends() {
        // Upstream: if n is smaller than max(edges)+1, add the missing vertices.
        let g = create(&[(0, 1), (5, 6)], 3, false).expect("create");
        assert_eq!(g.vcount(), 7); // max edge id is 6, so 7 vertices
        assert_eq!(g.ecount(), 2);
    }

    #[test]
    fn directed_preserves_arc_order() {
        // Arc emission must match input order for directed graphs.
        let arcs = vec![(0, 1), (1, 0), (1, 2), (2, 1)];
        let g = create(&arcs, 3, true).expect("create");
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 4);
        assert_eq!(directed_arc_list(&g), arcs);
    }

    #[test]
    fn undirected_canonicalises_edges() {
        let g = create(&[(2, 1), (3, 0)], 0, false).expect("create");
        let want: BTreeSet<(u32, u32)> = [(0, 3), (1, 2)].into_iter().collect();
        assert_eq!(canonical_undirected_edges(&g), want);
    }

    #[test]
    fn self_loops_accepted() {
        let g = create(&[(0, 0), (1, 1), (0, 1)], 0, false).expect("create");
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 3);
    }

    #[test]
    fn parallel_edges_preserved() {
        let g = create(&[(0, 1), (0, 1), (0, 1)], 0, false).expect("create");
        assert_eq!(g.ecount(), 3);
        // All three should land on the same vertex pair.
        assert_eq!(g.degree(0).unwrap(), 3);
        assert_eq!(g.degree(1).unwrap(), 3);
    }

    #[test]
    fn u32_max_endpoint_errors_rather_than_wraps() {
        // max edge id == u32::MAX would need vcount = u32::MAX + 1 → overflow.
        let r = create(&[(0, u32::MAX)], 0, false);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn n_max_vcount_works_when_max_endpoint_lt_n() {
        // n=10 with max=2 — no extension needed, succeeds.
        let g = create(&[(0, 1), (1, 2)], 10, false).expect("create");
        assert_eq!(g.vcount(), 10);
    }

    #[test]
    fn star_via_create_matches_handrolled_star() {
        let n: u32 = 8;
        let edges: Vec<(u32, u32)> = (1..n).map(|i| (0, i)).collect();
        let g = create(&edges, n, false).expect("create");
        assert_eq!(g.vcount(), n);
        assert_eq!(g.ecount(), (n - 1) as usize);
        assert_eq!(g.degree(0).unwrap(), (n - 1) as usize);
        for v in 1..n {
            assert_eq!(g.degree(v).unwrap(), 1);
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_pairs(max_n: u32) -> impl Strategy<Value = Vec<(u32, u32)>> {
        prop::collection::vec((0..max_n, 0..max_n), 0..30)
    }

    proptest! {
        #[test]
        fn vcount_at_least_n_and_at_least_max_plus_one(
            edges in arb_pairs(20),
            n in 0u32..30,
            directed: bool,
        ) {
            let g = create(&edges, n, directed).expect("create");
            let inferred = edges
                .iter()
                .flat_map(|&(u, v)| [u, v])
                .max()
                .map_or(0u32, |m| m + 1);
            prop_assert!(g.vcount() >= n);
            prop_assert!(g.vcount() >= inferred);
            prop_assert_eq!(g.vcount(), n.max(inferred));
        }

        #[test]
        fn ecount_equals_input_length(
            edges in arb_pairs(20),
            n in 0u32..30,
            directed: bool,
        ) {
            let g = create(&edges, n, directed).expect("create");
            prop_assert_eq!(g.ecount(), edges.len());
        }

        #[test]
        fn directed_preserves_arc_order_property(
            edges in arb_pairs(15),
        ) {
            let g = create(&edges, 0, true).expect("create");
            let got: Vec<(u32, u32)> = (0..u32::try_from(g.ecount()).unwrap())
                .map(|eid| g.edge(eid).unwrap())
                .collect();
            prop_assert_eq!(got, edges);
        }

        #[test]
        fn directedness_flag_round_trips(
            edges in arb_pairs(15),
            n in 0u32..20,
            directed: bool,
        ) {
            let g = create(&edges, n, directed).expect("create");
            prop_assert_eq!(g.is_directed(), directed);
        }
    }
}
