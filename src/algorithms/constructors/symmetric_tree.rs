//! Symmetric tree constructor (ALGO-CN-005).
//!
//! Counterpart of `igraph_symmetric_tree()` in
//! `references/igraph/src/constructors/regular.c:706-808`.
//!
//! A *symmetric tree* is a rooted tree where every vertex at depth `d`
//! has exactly `branches[d]` children. Unlike [`kary_tree`], where every
//! internal vertex has the same number of children, a symmetric tree
//! lets the branching factor vary per level — for example
//! `branches = [3, 2, 2]` produces a tree where the root has 3 kids,
//! each of those has 2, and each of those has 2 again (so the leaves
//! sit at depth 3).
//!
//! Vertex layout follows the same BFS scheme as `kary_tree`: vertex `0`
//! is the root, then all depth-1 vertices in arrival order, then all
//! depth-2 vertices in their arrival order, and so on. Arc direction is
//! controlled by the shared [`TreeMode`] enum:
//!
//! * [`TreeMode::Out`] — directed, every edge flows **from parent to
//!   child** (`parent → child`).
//! * [`TreeMode::In`] — directed, every edge flows **from child to
//!   parent** (`child → parent`).
//! * [`TreeMode::Undirected`] — undirected; raw endpoints are
//!   `(parent, child)` but stored as canonical `(min, max)` by
//!   `Graph::add_edges`.
//!
//! The C implementation walks one BFS level at a time, expanding every
//! parent of that level in turn, pushing exactly `branches[k]` outgoing
//! arcs per parent. The Rust port mirrors that level-by-level loop.
//!
//! Vertex count is `1 + Σ_{i=0..d} Π_{j=0..=i} branches[j]`. Empty
//! `branches` collapses to the singleton root (n = 1, 0 edges).
//!
//! Edge counts:
//!
//! * `branches.is_empty()` — singleton root, no edges.
//! * non-empty `branches` — `n - 1` edges (always a tree).
//!
//! Time complexity: O(|V|).
//!
//! [`kary_tree`]: super::kary_tree::kary_tree

use super::kary_tree::TreeMode;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Build a symmetric tree where each vertex at depth `d` has
/// `branches[d]` children.
///
/// Vertex `0` is the root. Children at each level are added in BFS
/// order, and the per-level branching count is read from `branches`.
/// Empty `branches` yields the singleton tree on one vertex.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — any entry of `branches` is
///   zero (a zero branching count yields a degenerate tree with no
///   vertices at the next level, which upstream rejects).
/// * [`IgraphError::InvalidArgument`] — vertex count overflows `u32`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{symmetric_tree, TreeMode};
///
/// // Root + 2 kids + 4 grandkids = 7 vertices (perfect binary depth-2).
/// let t = symmetric_tree(&[2, 2], TreeMode::Undirected).unwrap();
/// assert_eq!(t.vcount(), 7);
/// assert_eq!(t.ecount(), 6);
/// assert!(!t.is_directed());
/// ```
pub fn symmetric_tree(branches: &[u32], mode: TreeMode) -> IgraphResult<Graph> {
    let directed = matches!(mode, TreeMode::Out | TreeMode::In);

    if branches.is_empty() {
        return Graph::new(1, directed);
    }

    for &b in branches {
        if b == 0 {
            return Err(IgraphError::InvalidArgument(
                "symmetric_tree: number of branches must be positive at each level".to_string(),
            ));
        }
    }

    // Vertex count: 1 + b0 + b0*b1 + b0*b1*b2 + ... — checked-mul the
    // running level width so overflow surfaces as an InvalidArgument
    // rather than panicking.
    let mut n: u32 = 1;
    let mut level_width: u32 = 1;
    for &b in branches {
        level_width = level_width.checked_mul(b).ok_or_else(|| {
            IgraphError::InvalidArgument("symmetric_tree: vertex count overflows u32".to_string())
        })?;
        n = n.checked_add(level_width).ok_or_else(|| {
            IgraphError::InvalidArgument("symmetric_tree: vertex count overflows u32".to_string())
        })?;
    }

    let edge_count = (n as usize) - 1;
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(edge_count);

    // BFS level walk mirroring the upstream C loop: at each level k,
    // expand every parent in [parent .. level_end), pushing exactly
    // branches[k] arcs apiece. `level_end` is the future value of
    // `child` once that level finishes, so we snapshot it before the
    // inner loop and only advance `parent` past it after.
    let mut child: u32 = 1;
    let mut parent: u32 = 0;
    for &b in branches {
        let level_end = child;
        while parent < level_end {
            for _ in 0..b {
                match mode {
                    TreeMode::Out | TreeMode::Undirected => edges.push((parent, child)),
                    TreeMode::In => edges.push((child, parent)),
                }
                child += 1;
            }
            parent += 1;
        }
    }

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
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
    fn empty_branches_yields_singleton() {
        let t = symmetric_tree(&[], TreeMode::Out).expect("empty branches");
        assert_eq!(t.vcount(), 1);
        assert_eq!(t.ecount(), 0);
        assert!(t.is_directed());
        let u = symmetric_tree(&[], TreeMode::Undirected).expect("empty branches undirected");
        assert!(!u.is_directed());
    }

    #[test]
    fn single_level_three_out_is_star_k1_3() {
        // [3] → root with 3 leaves, 4 vertices, 3 edges. Equivalent to
        // an out-star on 4 vertices centred at 0.
        let t = symmetric_tree(&[3], TreeMode::Out).expect("star-like");
        assert_eq!(t.vcount(), 4);
        assert_eq!(t.ecount(), 3);
        assert_eq!(dump_edges(&t), vec![(0, 1), (0, 2), (0, 3)]);
    }

    #[test]
    fn single_level_three_in_reverses_arcs() {
        let t = symmetric_tree(&[3], TreeMode::In).expect("star-like in");
        assert_eq!(dump_edges(&t), vec![(1, 0), (2, 0), (3, 0)]);
    }

    #[test]
    fn binary_depth_two_matches_kary_tree_seven_two() {
        // [2, 2] → 1 + 2 + 4 = 7 vertices, identical to kary_tree(7, 2).
        let t = symmetric_tree(&[2, 2], TreeMode::Out).expect("binary depth 2");
        assert_eq!(t.vcount(), 7);
        assert_eq!(t.ecount(), 6);
        assert_eq!(
            dump_edges(&t),
            vec![(0, 1), (0, 2), (1, 3), (1, 4), (2, 5), (2, 6)]
        );
    }

    #[test]
    fn ternary_then_binary_layout() {
        // [3, 2] → 1 + 3 + 6 = 10 vertices.
        // Root 0 has children 1,2,3; each of those has 2 kids.
        let t = symmetric_tree(&[3, 2], TreeMode::Out).expect("ternary then binary");
        assert_eq!(t.vcount(), 10);
        assert_eq!(t.ecount(), 9);
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
    fn chain_branches_one_one_one_is_linear_path() {
        // [1, 1, 1] → 1 + 1 + 1 + 1 = 4 vertices in a line.
        let t = symmetric_tree(&[1, 1, 1], TreeMode::Undirected).expect("chain");
        assert_eq!(t.vcount(), 4);
        assert_eq!(t.ecount(), 3);
        let mut got = dump_edges(&t);
        got.sort_unstable();
        assert_eq!(got, vec![(0, 1), (1, 2), (2, 3)]);
    }

    #[test]
    fn three_levels_mixed_three_two_one() {
        // [3, 2, 1] → 1 + 3 + 6 + 6 = 16 vertices.
        let t = symmetric_tree(&[3, 2, 1], TreeMode::Out).expect("3-level mixed");
        assert_eq!(t.vcount(), 16);
        assert_eq!(t.ecount(), 15);
        // Spot-check: depth-2 vertices 4..=9 each have exactly one
        // grandchild (10..=15) attached one-to-one in BFS order.
        let edges = dump_edges(&t);
        for (i, parent) in (4..=9u32).enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let child = 10u32 + i as u32;
            assert!(edges.contains(&(parent, child)));
        }
    }

    #[test]
    fn zero_branch_rejected() {
        let err = symmetric_tree(&[2, 0, 2], TreeMode::Out).expect_err("zero branch");
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("positive")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn vertex_count_overflow_rejected() {
        // 1 + 2^32 → overflow. branches=[2; 32] yields 2^32 leaves
        // alone, well past u32::MAX.
        let bad = [2u32; 32];
        let err = symmetric_tree(&bad, TreeMode::Out).expect_err("overflow");
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("overflow")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn undirected_canonicalises_edges() {
        let t = symmetric_tree(&[2, 2], TreeMode::Undirected).expect("undirected");
        for e in 0..u32::try_from(t.ecount()).unwrap() {
            let (u, v) = t.edge(e).unwrap();
            assert!(u < v, "edge ({u},{v}) is not canonical (min, max)");
        }
    }

    #[test]
    fn single_leaf_branches_one() {
        // [1] → root + 1 leaf.
        let t = symmetric_tree(&[1], TreeMode::Out).expect("single leaf");
        assert_eq!(t.vcount(), 2);
        assert_eq!(t.ecount(), 1);
        assert_eq!(dump_edges(&t), vec![(0, 1)]);
    }

    #[test]
    fn out_and_undirected_share_edge_endpoints() {
        // Same (parent, child) tuples regardless of mode — the only
        // difference for undirected is canonical ordering inside the
        // Graph storage.
        let out = symmetric_tree(&[2, 3], TreeMode::Out).expect("out");
        let und = symmetric_tree(&[2, 3], TreeMode::Undirected).expect("und");
        assert_eq!(out.vcount(), und.vcount());
        assert_eq!(out.ecount(), und.ecount());
        let mut out_pairs: Vec<(u32, u32)> = dump_edges(&out)
            .into_iter()
            .map(|(u, v)| (u.min(v), u.max(v)))
            .collect();
        let mut und_pairs = dump_edges(&und);
        out_pairs.sort_unstable();
        und_pairs.sort_unstable();
        assert_eq!(out_pairs, und_pairs);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn branches_strategy() -> impl Strategy<Value = Vec<u32>> {
        prop::collection::vec(1u32..=3, 0..=4)
    }

    proptest! {
        // Edge count is always n - 1 for non-empty branches and 0 for
        // an empty branches slice.
        #[test]
        fn edge_count_matches_n_minus_one(branches in branches_strategy()) {
            let t = symmetric_tree(&branches, TreeMode::Out).unwrap();
            let n: u32 = t.vcount();
            let m: usize = t.ecount();
            if branches.is_empty() {
                prop_assert_eq!(n, 1);
                prop_assert_eq!(m, 0);
            } else {
                prop_assert_eq!(m, (n as usize).saturating_sub(1));
            }
        }

        // Vertex count equals the cumulative-product BFS sum.
        #[test]
        fn vertex_count_matches_cumulative_product(branches in branches_strategy()) {
            let t = symmetric_tree(&branches, TreeMode::Undirected).unwrap();
            let mut expected: u32 = 1;
            let mut level: u32 = 1;
            for &b in &branches {
                level = level.checked_mul(b).expect("no overflow in small proptest range");
                expected = expected.checked_add(level).expect("no overflow");
            }
            prop_assert_eq!(t.vcount(), expected);
        }

        // Mode round-trip: Out and In must yield the same |E| and the
        // same parent-child multiset modulo flipping endpoints.
        #[test]
        fn out_and_in_have_swapped_endpoints(branches in branches_strategy()) {
            let out = symmetric_tree(&branches, TreeMode::Out).unwrap();
            let inn = symmetric_tree(&branches, TreeMode::In).unwrap();
            prop_assert_eq!(out.vcount(), inn.vcount());
            prop_assert_eq!(out.ecount(), inn.ecount());
            let m = u32::try_from(out.ecount()).unwrap();
            for e in 0..m {
                let (a, b) = out.edge(e).unwrap();
                let (c, d) = inn.edge(e).unwrap();
                prop_assert_eq!((a, b), (d, c));
            }
        }
    }
}
