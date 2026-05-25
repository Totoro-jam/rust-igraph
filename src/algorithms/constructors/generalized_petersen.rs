//! Generalized Petersen graph constructor (ALGO-CN-010).
//!
//! Counterpart of `igraph_generalized_petersen()` in
//! `references/igraph/src/constructors/generalized_petersen.c:58-95`.
//!
//! The generalized Petersen graph `G(n, k)` has `2n` vertices in two
//! layers:
//!
//! * **outer cycle** `v_0, v_1, …, v_{n-1}` — a cycle `C_n` over ids
//!   `0..n`.
//! * **inner circulant** `u_0, u_1, …, u_{n-1}` — circulant graph over
//!   ids `n..2n` where `u_i` is connected to `u_{(i + k) mod n}`.
//! * **rungs** — every `v_i` is connected to its inner counterpart
//!   `u_i`.
//!
//! It has exactly `3n` edges: `n` outer-cycle edges, `n` rung edges,
//! and `n` inner-circulant edges.
//!
//! `G(5, 2)` is the classic Petersen graph. Other famous specializations
//! include `G(8, 3)` (Möbius–Kantor), `G(10, 3)` (Desargues), and
//! `G(12, 5)` (Nauru).
//!
//! Constraints (mirroring upstream):
//!
//! * `n >= 3`.
//! * `0 < k < n / 2` (note: strict — `2k < n`).
//!
//! Returns an **undirected** graph in all cases — the upstream C
//! constructor is fixed-direction (`IGRAPH_UNDIRECTED`).
//!
//! Reference: M. E. Watkins, *A Theorem on Tait Colorings with an
//! Application to the Generalized Petersen Graphs*, Journal of
//! Combinatorial Theory 6, 152–164 (1969).
//!
//! Time complexity: `O(|V|) = O(n)`.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Build the generalized Petersen graph `G(n, k)`.
///
/// The result is **undirected** with `2n` vertices and `3n` edges:
/// outer cycle, inner circulant of shift `k`, and one rung per index.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — `n < 3`.
/// * [`IgraphError::InvalidArgument`] — `k == 0`, or `k >= n / 2`
///   (equivalently `2 * k >= n`).
/// * [`IgraphError::InvalidArgument`] — `2 * n` overflows `u32`
///   (theoretical: would require `n > u32::MAX / 2`).
///
/// # Examples
///
/// ```
/// use rust_igraph::generalized_petersen;
///
/// // The classic Petersen graph G(5, 2).
/// let g = generalized_petersen(5, 2).unwrap();
/// assert_eq!(g.vcount(), 10);
/// assert_eq!(g.ecount(), 15);
/// assert!(!g.is_directed());
/// ```
pub fn generalized_petersen(n: u32, k: u32) -> IgraphResult<Graph> {
    if n < 3 {
        return Err(IgraphError::InvalidArgument(format!(
            "generalized_petersen: n = {n} must be at least 3"
        )));
    }

    // 0 < k < n/2 — the strict upper bound 2*k < n avoids parallel
    // inner edges (e.g. k = n/2 with n even gives a perfect matching
    // that doubles edges). Upstream rejects 2*k == n as well.
    if k == 0 || k.checked_mul(2).is_none_or(|two_k| two_k >= n) {
        return Err(IgraphError::InvalidArgument(format!(
            "generalized_petersen: k = {k} must satisfy 0 < k < n/2 with n = {n}"
        )));
    }

    let vcount: u32 = n.checked_mul(2).ok_or_else(|| {
        IgraphError::InvalidArgument(format!(
            "generalized_petersen: 2*n overflows u32 for n = {n}"
        ))
    })?;
    let ecount: usize = usize::try_from(n)
        .ok()
        .and_then(|nu| nu.checked_mul(3))
        .ok_or_else(|| {
            IgraphError::InvalidArgument(format!(
                "generalized_petersen: 3*n overflows usize for n = {n}"
            ))
        })?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);
    for i in 0..n {
        // Outer cycle edge v_i — v_{(i+1) mod n}.
        edges.push((i, (i + 1) % n));
        // Rung v_i — u_i.
        edges.push((i, i + n));
        // Inner circulant edge u_i — u_{(i+k) mod n}.
        edges.push((i + n, ((i + k) % n) + n));
    }

    let mut g = Graph::new(vcount, false)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dump_edges(g: &Graph) -> Vec<(u32, u32)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..m)
            .map(|e| g.edge(e).expect("edge id in bounds"))
            .collect()
    }

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    #[test]
    fn petersen_graph_g_5_2() {
        // The eponymous Petersen graph: 10 vertices, 15 edges, 3-regular.
        let g = generalized_petersen(5, 2).expect("Petersen graph");
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 15);
        assert!(!g.is_directed());
        for v in 0..g.vcount() {
            assert_eq!(
                g.degree(v).expect("in range"),
                3,
                "vertex {v} should be 3-regular"
            );
        }
    }

    #[test]
    fn mobius_kantor_g_8_3() {
        // Möbius–Kantor graph: 16 vertices, 24 edges, 3-regular, bipartite, girth 6.
        let g = generalized_petersen(8, 3).expect("Möbius–Kantor");
        assert_eq!(g.vcount(), 16);
        assert_eq!(g.ecount(), 24);
        for v in 0..g.vcount() {
            assert_eq!(g.degree(v).expect("in range"), 3);
        }
    }

    #[test]
    fn desargues_g_10_3() {
        let g = generalized_petersen(10, 3).expect("Desargues");
        assert_eq!(g.vcount(), 20);
        assert_eq!(g.ecount(), 30);
        for v in 0..g.vcount() {
            assert_eq!(g.degree(v).expect("in range"), 3);
        }
    }

    #[test]
    fn nauru_g_12_5() {
        let g = generalized_petersen(12, 5).expect("Nauru");
        assert_eq!(g.vcount(), 24);
        assert_eq!(g.ecount(), 36);
        for v in 0..g.vcount() {
            assert_eq!(g.degree(v).expect("in range"), 3);
        }
    }

    #[test]
    fn three_regular_for_all_valid_n_k() {
        // Every G(n, k) is 3-regular by construction; check a sweep.
        for n in 3..=20u32 {
            // valid k: 1 <= k < n/2 i.e. 2*k < n
            let max_k = (n - 1) / 2;
            for k in 1..=max_k {
                let g = generalized_petersen(n, k).expect("valid (n,k)");
                assert_eq!(g.vcount(), 2 * n);
                assert_eq!(g.ecount(), 3 * (n as usize));
                for v in 0..g.vcount() {
                    assert_eq!(
                        g.degree(v).expect("in range"),
                        3,
                        "G({n},{k}): vertex {v} should have degree 3"
                    );
                }
            }
        }
    }

    #[test]
    fn edge_emission_order_matches_upstream_g_5_2() {
        // Emission order: for each i in 0..n, push outer, rung, inner.
        // For G(5, 2):
        //   i=0: (0,1)(0,5)(5,7)
        //   i=1: (1,2)(1,6)(6,8)
        //   i=2: (2,3)(2,7)(7,9)
        //   i=3: (3,4)(3,8)(8,5)
        //   i=4: (4,0)(4,9)(9,6)
        // Undirected Graph::add_edges canonicalises (lo, hi).
        let g = generalized_petersen(5, 2).expect("G(5,2)");
        let expected = vec![
            (0, 1),
            (0, 5),
            (5, 7),
            (1, 2),
            (1, 6),
            (6, 8),
            (2, 3),
            (2, 7),
            (7, 9),
            (3, 4),
            (3, 8),
            (5, 8),
            (0, 4),
            (4, 9),
            (6, 9),
        ];
        assert_eq!(dump_edges(&g), expected);
    }

    #[test]
    fn outer_cycle_present() {
        // Every (i, (i+1) % n) edge with i, i+1 < n must exist.
        let g = generalized_petersen(7, 2).expect("G(7,2)");
        let edges: std::collections::HashSet<(u32, u32)> = dump_edges(&g).into_iter().collect();
        for i in 0..7u32 {
            assert!(edges.contains(&canon(i, (i + 1) % 7)));
        }
    }

    #[test]
    fn rung_edges_present() {
        let g = generalized_petersen(7, 2).expect("G(7,2)");
        let edges: std::collections::HashSet<(u32, u32)> = dump_edges(&g).into_iter().collect();
        for i in 0..7u32 {
            assert!(
                edges.contains(&canon(i, i + 7)),
                "missing rung {i}-{}",
                i + 7
            );
        }
    }

    #[test]
    fn inner_circulant_edges_present() {
        let g = generalized_petersen(7, 2).expect("G(7,2)");
        let edges: std::collections::HashSet<(u32, u32)> = dump_edges(&g).into_iter().collect();
        for i in 0..7u32 {
            let lhs = i + 7;
            let rhs = ((i + 2) % 7) + 7;
            assert!(
                edges.contains(&canon(lhs, rhs)),
                "missing inner edge {lhs}-{rhs}"
            );
        }
    }

    #[test]
    fn rejects_n_less_than_three() {
        for n in 0..3u32 {
            let err = generalized_petersen(n, 1).unwrap_err();
            assert!(matches!(err, IgraphError::InvalidArgument(_)));
        }
    }

    #[test]
    fn rejects_k_zero() {
        let err = generalized_petersen(5, 0).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rejects_k_at_half_n() {
        // k = n/2 (with n even) is rejected — 2*k == n.
        let err = generalized_petersen(6, 3).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
        // k just below n/2 (n=6, k=2) is fine.
        assert!(generalized_petersen(6, 2).is_ok());
    }

    #[test]
    fn rejects_k_too_large() {
        let err = generalized_petersen(7, 4).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
        // 7/2 = 3 (integer), 2*3 = 6 < 7 → k=3 is valid.
        assert!(generalized_petersen(7, 3).is_ok());
    }

    #[test]
    fn no_self_loops_no_parallel_edges() {
        // 3-regular, simple — verify by canonical-set dedup.
        for n in 3..=15u32 {
            let max_k = (n - 1) / 2;
            for k in 1..=max_k {
                let g = generalized_petersen(n, k).expect("valid");
                let edges = dump_edges(&g);
                let mut set: std::collections::HashSet<(u32, u32)> =
                    std::collections::HashSet::new();
                for (u, v) in &edges {
                    assert_ne!(u, v, "G({n},{k}) has self-loop {u}-{v}");
                    let key = canon(*u, *v);
                    assert!(set.insert(key), "G({n},{k}) has duplicate edge {u}-{v}");
                }
                assert_eq!(set.len(), 3 * (n as usize));
            }
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_invariants {
    use super::*;
    use proptest::prelude::*;

    fn dump_edges(g: &Graph) -> Vec<(u32, u32)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32");
        (0..m)
            .map(|e| g.edge(e).expect("edge id in bounds"))
            .collect()
    }

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            max_shrink_iters: 1000,
            .. ProptestConfig::default()
        })]

        /// Vertex and edge counts always match the closed-form `2n` and
        /// `3n`.
        #[test]
        fn vcount_ecount_match_formula(n in 3u32..=80, k_seed in 1u32..=200) {
            let max_k = (n - 1) / 2;
            prop_assume!(max_k >= 1);
            let k = ((k_seed - 1) % max_k) + 1;
            let g = generalized_petersen(n, k).expect("valid (n,k)");
            prop_assert_eq!(g.vcount(), 2 * n);
            prop_assert_eq!(g.ecount(), 3 * (n as usize));
        }

        /// Every emitted graph is 3-regular and simple.
        #[test]
        fn three_regular_and_simple(n in 3u32..=60, k_seed in 1u32..=200) {
            let max_k = (n - 1) / 2;
            prop_assume!(max_k >= 1);
            let k = ((k_seed - 1) % max_k) + 1;
            let g = generalized_petersen(n, k).expect("valid (n,k)");
            for v in 0..g.vcount() {
                prop_assert_eq!(g.degree(v).expect("in range"), 3);
            }
            // simplicity (no loops, no parallels)
            let mut set: std::collections::HashSet<(u32, u32)> =
                std::collections::HashSet::new();
            for (u, w) in dump_edges(&g) {
                prop_assert_ne!(u, w);
                prop_assert!(set.insert(canon(u, w)));
            }
        }

        /// The outer-cycle subgraph (vertices 0..n) is exactly a cycle
        /// C_n: connecting (i, (i+1) % n) for every i.
        #[test]
        fn outer_layer_is_a_cycle(n in 3u32..=50, k_seed in 1u32..=200) {
            let max_k = (n - 1) / 2;
            prop_assume!(max_k >= 1);
            let k = ((k_seed - 1) % max_k) + 1;
            let g = generalized_petersen(n, k).expect("valid (n,k)");
            let edges: std::collections::HashSet<(u32, u32)> =
                dump_edges(&g).into_iter().collect();
            for i in 0..n {
                prop_assert!(edges.contains(&canon(i, (i + 1) % n)));
            }
        }

        /// Rungs: every i in 0..n is connected to i + n.
        #[test]
        fn rungs_exist(n in 3u32..=50, k_seed in 1u32..=200) {
            let max_k = (n - 1) / 2;
            prop_assume!(max_k >= 1);
            let k = ((k_seed - 1) % max_k) + 1;
            let g = generalized_petersen(n, k).expect("valid (n,k)");
            let edges: std::collections::HashSet<(u32, u32)> =
                dump_edges(&g).into_iter().collect();
            for i in 0..n {
                prop_assert!(edges.contains(&canon(i, i + n)));
            }
        }

        /// Inner layer (vertices n..2n) is exactly the circulant with
        /// shift k.
        #[test]
        fn inner_layer_is_circulant_k(n in 3u32..=50, k_seed in 1u32..=200) {
            let max_k = (n - 1) / 2;
            prop_assume!(max_k >= 1);
            let k = ((k_seed - 1) % max_k) + 1;
            let g = generalized_petersen(n, k).expect("valid (n,k)");
            let edges: std::collections::HashSet<(u32, u32)> =
                dump_edges(&g).into_iter().collect();
            for i in 0..n {
                let lhs = i + n;
                let rhs = ((i + k) % n) + n;
                prop_assert!(edges.contains(&canon(lhs, rhs)));
            }
        }
    }
}
