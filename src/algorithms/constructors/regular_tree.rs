//! Regular tree constructor (ALGO-CN-006).
//!
//! Counterpart of `igraph_regular_tree()` in
//! `references/igraph/src/constructors/regular.c:806-865`.
//!
//! A *regular tree* (a.k.a. **Bethe lattice**) of height `h` and degree
//! `k` is a rooted tree where every non-leaf vertex has total degree
//! exactly `k`. Concretely:
//!
//! * the root has `k` children (its total degree is `k`),
//! * every internal non-root vertex has `k - 1` children plus its parent,
//!   so its total degree is also `k`,
//! * leaves sit at depth `h` and have degree 1.
//!
//! This is different from a k-ary tree (where every internal vertex —
//! including the root — has the same number of *children*), and from a
//! symmetric tree (where the branching factor varies per level).
//!
//! Upstream C builds it as a thin wrapper over `igraph_symmetric_tree`
//! by constructing `branches = [k, k - 1, k - 1, ...]` of length `h` and
//! delegating. The Rust port mirrors that.
//!
//! Vertex count: `1 + k * Σ_{i=0..h} (k-1)^i = 1 + k * ((k-1)^h - 1) / (k - 2)`
//! for `k >= 3`; `1 + h` for `k = 2` (degenerate linear chain).
//!
//! Edge count: `n - 1` (a tree).
//!
//! Time complexity: O(|V|).
//!
//! See also [`kary_tree`] / [`symmetric_tree`] for the related shapes.
//!
//! [`kary_tree`]: super::kary_tree::kary_tree
//! [`symmetric_tree`]: super::symmetric_tree::symmetric_tree

use super::kary_tree::TreeMode;
use super::symmetric_tree::symmetric_tree;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Build a regular tree (Bethe lattice) of height `h` and degree `k`.
///
/// Vertex `0` is the root and has `k` children; every internal non-root
/// vertex has `k - 1` children plus its parent, giving total degree `k`.
/// Leaves sit at depth `h`.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — `h < 1` (height must be at least
///   one; height zero would be a singleton, which is not a Bethe
///   lattice — upstream rejects the same input).
/// * [`IgraphError::InvalidArgument`] — `k < 2` (degree must be at least
///   two; degree one would mean every internal vertex has exactly one
///   neighbour, which is degenerate).
/// * [`IgraphError::InvalidArgument`] — vertex count overflows `u32`
///   (propagated through the underlying [`symmetric_tree`] call).
///
/// # Examples
///
/// ```
/// use rust_igraph::{regular_tree, TreeMode};
///
/// // h=2, k=3 — root has 3 kids, each internal has 2 kids: 1 + 3 + 6 = 10.
/// let t = regular_tree(2, 3, TreeMode::Undirected).unwrap();
/// assert_eq!(t.vcount(), 10);
/// assert_eq!(t.ecount(), 9);
/// assert!(!t.is_directed());
/// ```
pub fn regular_tree(h: u32, k: u32, mode: TreeMode) -> IgraphResult<Graph> {
    if h < 1 {
        return Err(IgraphError::InvalidArgument(format!(
            "regular_tree: height must be at least 1, got {h}"
        )));
    }
    if k < 2 {
        return Err(IgraphError::InvalidArgument(format!(
            "regular_tree: degree must be at least 2, got {k}"
        )));
    }

    // branches = [k, k-1, k-1, ...] of length h.
    // The root contributes k children; every non-root non-leaf level
    // contributes k-1 children per parent (the missing one is the parent
    // edge), keeping total degree at exactly k.
    let mut branches: Vec<u32> = vec![k - 1; h as usize];
    branches[0] = k;

    symmetric_tree(&branches, mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dump_edges(g: &Graph) -> Vec<(u32, u32)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32 in tests");
        (0..m)
            .map(|e| g.edge(e).expect("edge id in bounds in tests"))
            .collect()
    }

    #[test]
    fn h1_k3_is_star_k1_3() {
        // h=1, k=3 — root with 3 leaves. Equivalent to star K1,3.
        let t = regular_tree(1, 3, TreeMode::Out).expect("star-like");
        assert_eq!(t.vcount(), 4);
        assert_eq!(t.ecount(), 3);
        assert_eq!(dump_edges(&t), vec![(0, 1), (0, 2), (0, 3)]);
    }

    #[test]
    fn h2_k3_root_has_three_kids_each_has_two() {
        // h=2, k=3 — root has 3 children, each child has 2 children.
        // 1 + 3 + 6 = 10 vertices.
        let t = regular_tree(2, 3, TreeMode::Out).expect("h2 k3");
        assert_eq!(t.vcount(), 10);
        assert_eq!(t.ecount(), 9);
        // Same as symmetric_tree([3,2]).
        assert_eq!(
            dump_edges(&t),
            vec![
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 4),
                (1, 5),
                (2, 6),
                (2, 7),
                (3, 8),
                (3, 9),
            ]
        );
    }

    #[test]
    fn h3_k2_is_linear_chain() {
        // k=2 forces branches=[2,1,1] — root has 2 kids, then a chain.
        let t = regular_tree(3, 2, TreeMode::Out).expect("h3 k2");
        assert_eq!(t.vcount(), 1 + 2 + 2 + 2);
        assert_eq!(t.ecount(), 6);
    }

    #[test]
    fn h1_k2_is_single_edge_pair() {
        // h=1, k=2: branches=[2] — root with 2 leaves (path P3 rooted
        // at centre). 1 + 2 = 3 vertices.
        let t = regular_tree(1, 2, TreeMode::Undirected).expect("h1 k2");
        assert_eq!(t.vcount(), 3);
        assert_eq!(t.ecount(), 2);
    }

    #[test]
    fn in_mode_reverses_arcs() {
        let t = regular_tree(1, 3, TreeMode::In).expect("in h1 k3");
        assert_eq!(dump_edges(&t), vec![(1, 0), (2, 0), (3, 0)]);
    }

    #[test]
    fn undirected_is_undirected() {
        let t = regular_tree(2, 3, TreeMode::Undirected).expect("undirected");
        assert!(!t.is_directed());
    }

    #[test]
    fn h_zero_rejected() {
        let err = regular_tree(0, 3, TreeMode::Out).expect_err("h=0");
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("height")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn k_less_than_two_rejected() {
        let err = regular_tree(3, 1, TreeMode::Out).expect_err("k=1");
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("degree")),
            other => panic!("unexpected error: {other:?}"),
        }
        let err0 = regular_tree(3, 0, TreeMode::Out).expect_err("k=0");
        match err0 {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("degree")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn vertex_count_overflow_propagates() {
        // h=32, k=3 → branches=[3, 2; 31] → 3 * 2^31 = ~6.4 billion leaves
        // alone, well over u32::MAX. Should bubble up as InvalidArgument.
        let err = regular_tree(40, 3, TreeMode::Out).expect_err("overflow");
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("overflow")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn root_has_degree_k() {
        // In the directed (Out) tree, root's out-degree equals k.
        let t = regular_tree(3, 4, TreeMode::Out).expect("h3 k4");
        let mut root_out_degree = 0u32;
        let m = u32::try_from(t.ecount()).unwrap();
        for e in 0..m {
            let (u, _v) = t.edge(e).unwrap();
            if u == 0 {
                root_out_degree += 1;
            }
        }
        assert_eq!(root_out_degree, 4);
    }

    #[test]
    fn internal_non_root_vertex_has_total_degree_k_undirected() {
        // For k=4, h=3, each non-leaf non-root vertex must have 4
        // incident edges (1 parent + 3 children).
        let t = regular_tree(3, 4, TreeMode::Undirected).expect("h3 k4");
        let m = u32::try_from(t.ecount()).unwrap();
        // Vertex 1 is depth 1 (non-root, non-leaf). Count its incident
        // edges.
        let mut deg1 = 0u32;
        for e in 0..m {
            let (u, v) = t.edge(e).unwrap();
            if u == 1 || v == 1 {
                deg1 += 1;
            }
        }
        assert_eq!(deg1, 4);
    }

    #[test]
    fn h2_k4_vertex_and_edge_counts() {
        // h=2, k=4 — branches=[4, 3]. 1 + 4 + 12 = 17 vertices, 16 edges.
        let t = regular_tree(2, 4, TreeMode::Out).expect("h2 k4");
        assert_eq!(t.vcount(), 17);
        assert_eq!(t.ecount(), 16);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Always a tree: ecount = n - 1 for any valid (h, k).
        #[test]
        fn edge_count_is_n_minus_one(h in 1u32..=4, k in 2u32..=4) {
            let t = regular_tree(h, k, TreeMode::Out).unwrap();
            let n = t.vcount();
            prop_assert_eq!(t.ecount(), (n as usize) - 1);
        }

        // Mode round-trip Out vs In: same edge count, endpoints swapped.
        #[test]
        fn out_and_in_endpoints_swap(h in 1u32..=3, k in 2u32..=3) {
            let out = regular_tree(h, k, TreeMode::Out).unwrap();
            let inn = regular_tree(h, k, TreeMode::In).unwrap();
            prop_assert_eq!(out.vcount(), inn.vcount());
            prop_assert_eq!(out.ecount(), inn.ecount());
            let m = u32::try_from(out.ecount()).unwrap();
            for e in 0..m {
                let (a, b) = out.edge(e).unwrap();
                let (c, d) = inn.edge(e).unwrap();
                prop_assert_eq!((a, b), (d, c));
            }
        }

        // For k>=2, root out-degree equals k in the Out mode.
        #[test]
        fn root_out_degree_equals_k(h in 1u32..=3, k in 2u32..=4) {
            let t = regular_tree(h, k, TreeMode::Out).unwrap();
            let m = u32::try_from(t.ecount()).unwrap();
            let mut root_deg = 0u32;
            for e in 0..m {
                let (u, _v) = t.edge(e).unwrap();
                if u == 0 { root_deg += 1; }
            }
            prop_assert_eq!(root_deg, k);
        }
    }
}
