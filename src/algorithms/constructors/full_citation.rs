//! Full citation graph constructor (ALGO-CN-025).
//!
//! Counterpart of `igraph_full_citation()` in
//! `references/igraph/src/constructors/full.c:348-376`.
//!
//! A *full citation graph* is the complete DAG on `n` vertices in which
//! every vertex `i` cites every earlier vertex `j < i`. Equivalently:
//!
//! * `directed = true`  → directed acyclic graph with `n·(n−1)/2` arcs
//!   `(i, j)` for every `0 ≤ j < i < n`. The induced topological order
//!   is `0, 1, 2, …, n−1` (in-degree of vertex `k` equals `k`,
//!   out-degree equals `n − 1 − k`).
//! * `directed = false` → undirected `K_n` with `n·(n−1)/2` edges. The
//!   edge *multiset* matches [`full_graph(n, false, false)`][super::full::full_graph],
//!   but the *emission order* is row-major over the destination's index
//!   `j < i` (upstream emits `(1,0), (2,0), (2,1), (3,0), (3,1), …`),
//!   so the edge ids assigned by [`Graph::add_edges`] differ from
//!   `full_graph`'s undirected branch.
//!
//! Time complexity: `O(|V|² ) = O(|E|)`.
//!
//! Degenerate inputs:
//!
//! * `n == 0` → empty graph (vcount = 0, ecount = 0).
//! * `n == 1` → singleton, no edges.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Build the full citation graph on `n` vertices.
///
/// See the module documentation for the precise semantics of the
/// `directed` flag.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — the `n·(n−1)/2` edge count
///   cannot be sized in `usize` (computed via [`usize::checked_mul`] so
///   the failure mode is deterministic).
///
/// # Examples
///
/// ```
/// use rust_igraph::full_citation;
///
/// // Directed complete DAG on 4 vertices: 6 arcs forming a transitive
/// // closure where vertex i cites every j < i.
/// let dag = full_citation(4, true).unwrap();
/// assert_eq!(dag.vcount(), 4);
/// assert_eq!(dag.ecount(), 6);
/// assert!(dag.is_directed());
///
/// // Undirected → K_n (same edge multiset as full_graph(4, false, false)).
/// let k4 = full_citation(4, false).unwrap();
/// assert_eq!(k4.vcount(), 4);
/// assert_eq!(k4.ecount(), 6);
/// assert!(!k4.is_directed());
/// ```
pub fn full_citation(n: u32, directed: bool) -> IgraphResult<Graph> {
    if n == 0 {
        return Graph::new(0, directed);
    }
    if n == 1 {
        return Graph::new(1, directed);
    }

    let n_us = usize::try_from(n).map_err(|_| {
        IgraphError::InvalidArgument(format!("full_citation: n {n} cannot fit usize"))
    })?;
    // Total edges: n·(n−1)/2 — and n·(n−1) for the overflow-safe product
    // path mirrors the upstream `IGRAPH_SAFE_MULT(n, n-1, …)` check.
    let prod = n_us.checked_mul(n_us - 1).ok_or_else(|| {
        IgraphError::InvalidArgument(format!("full_citation: n·(n−1) overflows usize (n = {n})"))
    })?;
    let ecount = prod / 2;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);
    // Upstream `for i in 1..n { for j in 0..i { push (i, j) } }`, byte
    // for byte — same emission order whether `directed` is true or false.
    for i in 1..n {
        for j in 0..i {
            edges.push((i, j));
        }
    }
    debug_assert_eq!(edges.len(), ecount);

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn dump_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let ec = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..ec)
            .map(|e| g.edge(e).expect("edge id in range"))
            .collect()
    }

    fn canonical_undirected(g: &Graph) -> BTreeSet<(VertexId, VertexId)> {
        dump_edges(g)
            .into_iter()
            .map(|(u, v)| if u <= v { (u, v) } else { (v, u) })
            .collect()
    }

    #[test]
    fn null_graph_zero_vertices() {
        let g = full_citation(0, true).expect("ok");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn singleton_no_edges() {
        let g = full_citation(1, true).expect("ok");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn directed_4_emission_order_matches_upstream() {
        // Upstream emission walks (i = 1..4, j = 0..i):
        // (1, 0), (2, 0), (2, 1), (3, 0), (3, 1), (3, 2).
        let g = full_citation(4, true).expect("ok");
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 6);
        assert!(g.is_directed());
        let want = vec![(1, 0), (2, 0), (2, 1), (3, 0), (3, 1), (3, 2)];
        assert_eq!(dump_edges(&g), want);
    }

    fn in_out_counts(g: &Graph) -> (Vec<usize>, Vec<usize>) {
        let n = g.vcount() as usize;
        let mut in_count = vec![0usize; n];
        let mut out_count = vec![0usize; n];
        for (u, v) in dump_edges(g) {
            out_count[u as usize] += 1;
            in_count[v as usize] += 1;
        }
        (in_count, out_count)
    }

    #[test]
    fn directed_5_in_out_degree_profile() {
        // In a complete DAG with citation order 0..n-1: vertex i has
        // out-arcs to every j < i, so the arcs *into* k come from i > k.
        // That means in-degree(k) = n - 1 - k and out-degree(k) = k.
        let n = 5u32;
        let g = full_citation(n, true).expect("ok");
        let (in_d, out_d) = in_out_counts(&g);
        for k in 0..n {
            let want_in = (n - 1 - k) as usize;
            let want_out = k as usize;
            assert_eq!(out_d[k as usize], want_out, "vertex {k} out-degree");
            assert_eq!(in_d[k as usize], want_in, "vertex {k} in-degree");
        }
    }

    #[test]
    fn undirected_5_multiset_matches_k5() {
        let g = full_citation(5, false).expect("ok");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 10);
        let want: BTreeSet<(u32, u32)> = (0..5u32)
            .flat_map(|i| ((i + 1)..5u32).map(move |j| (i, j)))
            .collect();
        assert_eq!(canonical_undirected(&g), want);
    }

    #[test]
    fn undirected_10_matches_k10_count() {
        let g = full_citation(10, false).expect("ok");
        assert_eq!(g.ecount(), 45);
    }

    #[test]
    fn no_self_loops_in_directed() {
        let g = full_citation(8, true).expect("ok");
        for (u, v) in dump_edges(&g) {
            assert_ne!(u, v, "directed full_citation must be loop-free");
        }
    }

    #[test]
    fn no_self_loops_in_undirected() {
        let g = full_citation(8, false).expect("ok");
        for (u, v) in dump_edges(&g) {
            assert_ne!(u, v, "undirected full_citation must be loop-free");
        }
    }

    #[test]
    fn directed_arcs_are_canonically_descending() {
        // Every arc has src > dst (the citation invariant).
        let g = full_citation(7, true).expect("ok");
        for (u, v) in dump_edges(&g) {
            assert!(u > v, "arc ({u}, {v}) violates citation invariant");
        }
    }

    #[test]
    fn directed_ecount_closed_form() {
        for n in 0..=12u32 {
            let g = full_citation(n, true).expect("ok");
            let want = if n == 0 {
                0
            } else {
                (n as usize) * (n as usize - 1) / 2
            };
            assert_eq!(g.ecount(), want, "n = {n}");
        }
    }

    #[test]
    fn undirected_ecount_matches_directed() {
        for n in 0..=12u32 {
            let du = full_citation(n, false).expect("ok");
            let dd = full_citation(n, true).expect("ok");
            assert_eq!(du.ecount(), dd.ecount(), "n = {n}");
        }
    }

    #[test]
    fn directed_flag_round_trips() {
        assert!(full_citation(5, true).expect("ok").is_directed());
        assert!(!full_citation(5, false).expect("ok").is_directed());
    }

    #[cfg(all(test, feature = "proptest-harness"))]
    mod prop {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(64))]

            #[test]
            fn ecount_is_triangular_for_directed(n in 0u32..=40) {
                let g = full_citation(n, true).expect("ok");
                let want = (n as usize) * (n as usize).saturating_sub(1) / 2;
                prop_assert_eq!(g.ecount(), want);
            }

            #[test]
            fn ecount_is_triangular_for_undirected(n in 0u32..=40) {
                let g = full_citation(n, false).expect("ok");
                let want = (n as usize) * (n as usize).saturating_sub(1) / 2;
                prop_assert_eq!(g.ecount(), want);
            }

            #[test]
            fn vcount_round_trips(n in 0u32..=80) {
                let g = full_citation(n, true).expect("ok");
                prop_assert_eq!(g.vcount(), n);
            }

            #[test]
            fn directed_arcs_descending(n in 0u32..=40) {
                let g = full_citation(n, true).expect("ok");
                let ec = u32::try_from(g.ecount()).expect("ec");
                for e in 0..ec {
                    let (u, v) = g.edge(e).expect("edge");
                    prop_assert!(u > v, "non-descending arc ({}, {})", u, v);
                }
            }

            #[test]
            fn in_degree_profile_directed(n in 1u32..=40) {
                let g = full_citation(n, true).expect("ok");
                let (in_d, out_d) = in_out_counts(&g);
                for k in 0..n {
                    let want_in = (n - 1 - k) as usize;
                    let want_out = k as usize;
                    prop_assert_eq!(in_d[k as usize], want_in);
                    prop_assert_eq!(out_d[k as usize], want_out);
                }
            }

            #[test]
            fn undirected_is_simple_no_loops(n in 0u32..=40) {
                let g = full_citation(n, false).expect("ok");
                let ec = u32::try_from(g.ecount()).expect("ec");
                let mut seen = std::collections::HashSet::new();
                for e in 0..ec {
                    let (u, v) = g.edge(e).expect("edge");
                    prop_assert_ne!(u, v, "self loop ({}, {}) emitted", u, v);
                    let key = if u <= v { (u, v) } else { (v, u) };
                    prop_assert!(seen.insert(key), "parallel edge {:?}", key);
                }
            }
        }
    }
}
