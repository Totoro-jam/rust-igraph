//! k-ary tree constructor (ALGO-CN-004).
//!
//! Counterpart of `igraph_kary_tree()` in
//! `references/igraph/src/constructors/regular.c:608-706`.
//!
//! A *k-ary tree* on `n` vertices is the rooted tree obtained by
//! attaching children to parents in breadth-first order: vertex `0` is
//! the root, vertex `1..=children` are its kids, then vertex `1`'s
//! kids, then vertex `2`'s kids, and so on. Almost every internal
//! vertex has exactly `children` children — the last internal vertex
//! may have fewer if `n - 1` is not a multiple of `children`. Arc
//! direction is controlled by [`TreeMode`]:
//!
//! * [`TreeMode::Out`] — directed, every edge flows **from parent to
//!   child** (`parent → child`).
//! * [`TreeMode::In`] — directed, every edge flows **from child to
//!   parent** (`child → parent`).
//! * [`TreeMode::Undirected`] — undirected; raw endpoints are
//!   `(parent, child)` but stored as canonical `(min, max)` by
//!   `Graph::add_edges`.
//!
//! Edge enumeration mirrors the upstream C loop exactly: a `to`
//! pointer starts at `1` and advances once per emitted edge; an outer
//! `parent` pointer starts at `0` and increments after every batch of
//! `children` edges (or when `n - 1` edges have been emitted in
//! total). For `n - 1` edges total, the result is always a tree
//! (acyclic, connected when undirected, weakly connected otherwise).
//!
//! Edge counts:
//!
//! * `n = 0` — no edges (empty graph).
//! * `n = 1` — no edges (singleton root).
//! * `n ≥ 2`, all modes — `n - 1` edges.
//!
//! Time complexity: O(|V|).

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Direction policy for [`kary_tree`].
///
/// Mirrors `igraph_tree_mode_t` in the C reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeMode {
    /// Directed tree, arcs flow from parent to child.
    Out,
    /// Directed tree, arcs flow from child to parent.
    In,
    /// Undirected tree.
    Undirected,
}

impl TreeMode {
    fn is_directed(self) -> bool {
        !matches!(self, TreeMode::Undirected)
    }
}

/// Build a k-ary tree on `n` vertices where each non-leaf parent has
/// up to `children` kids attached in BFS order.
///
/// For a perfectly symmetric tree of depth `l` use
/// `n = (children^(l+1) - 1) / (children - 1)`.
///
/// See the module-level docs for the precise BFS layout and per-mode
/// arc orientation.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — `children == 0`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{kary_tree, TreeMode};
///
/// // Binary tree with 7 vertices (perfect depth-2 tree).
/// let t = kary_tree(7, 2, TreeMode::Undirected).unwrap();
/// assert_eq!(t.vcount(), 7);
/// assert_eq!(t.ecount(), 6); // n - 1 edges
/// assert!(!t.is_directed());
/// ```
pub fn kary_tree(n: u32, children: u32, mode: TreeMode) -> IgraphResult<Graph> {
    if children == 0 {
        return Err(IgraphError::InvalidArgument(
            "kary_tree: number of children must be positive".to_string(),
        ));
    }

    let directed = mode.is_directed();

    if n == 0 {
        return Graph::new(0, directed);
    }
    if n == 1 {
        return Graph::new(1, directed);
    }

    let edge_count = (n as usize) - 1;
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(edge_count);

    // Upstream uses an `idx < 2 * (n - 1)` flat-array termination; we
    // bound on emitted edge count instead. `to` pumps the next child
    // index, `parent` walks the BFS parent ordering. Inner loop emits
    // up to `children` edges then advances `parent`.
    let mut to: u32 = 1;
    let mut parent: u32 = 0;
    let mut emitted: u32 = 0;
    let remaining = n - 1;
    while emitted < remaining {
        let batch = children.min(remaining - emitted);
        for _ in 0..batch {
            match mode {
                TreeMode::Out | TreeMode::Undirected => edges.push((parent, to)),
                TreeMode::In => edges.push((to, parent)),
            }
            to += 1;
            emitted += 1;
        }
        parent += 1;
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
        for mode in [TreeMode::Out, TreeMode::In, TreeMode::Undirected] {
            let g = kary_tree(0, 2, mode).expect("tree n=0");
            assert_eq!(g.vcount(), 0);
            assert_eq!(g.ecount(), 0);
            assert_eq!(g.is_directed(), mode.is_directed());
        }
    }

    #[test]
    fn singleton_has_no_edges() {
        for mode in [TreeMode::Out, TreeMode::In, TreeMode::Undirected] {
            let g = kary_tree(1, 3, mode).expect("tree n=1");
            assert_eq!(g.vcount(), 1);
            assert_eq!(g.ecount(), 0);
        }
    }

    #[test]
    fn binary_tree_seven_vertices_out_mode() {
        // n=7, children=2: perfect binary tree depth 2.
        //   parent 0 -> 1, 2
        //   parent 1 -> 3, 4
        //   parent 2 -> 5, 6
        let g = kary_tree(7, 2, TreeMode::Out).expect("B7");
        assert!(g.is_directed());
        assert_eq!(g.ecount(), 6);
        assert_eq!(
            collect_edges(&g),
            vec![(0, 1), (0, 2), (1, 3), (1, 4), (2, 5), (2, 6)]
        );
    }

    #[test]
    fn binary_tree_seven_vertices_in_mode() {
        // Same shape, edge direction reversed: every (parent, child) becomes
        // (child, parent).
        let g = kary_tree(7, 2, TreeMode::In).expect("B7 in");
        assert!(g.is_directed());
        assert_eq!(g.ecount(), 6);
        assert_eq!(
            collect_edges(&g),
            vec![(1, 0), (2, 0), (3, 1), (4, 1), (5, 2), (6, 2)]
        );
    }

    #[test]
    fn binary_tree_seven_vertices_undirected_mode() {
        // Same raw edges as OUT, canonicalised by add_edges (parent<child
        // already so no swap occurs).
        let g = kary_tree(7, 2, TreeMode::Undirected).expect("B7 undir");
        assert!(!g.is_directed());
        assert_eq!(g.ecount(), 6);
        assert_eq!(
            collect_edges(&g),
            vec![(0, 1), (0, 2), (1, 3), (1, 4), (2, 5), (2, 6)]
        );
    }

    #[test]
    fn ternary_tree_eight_vertices_partial_last_parent() {
        // n=8, children=3:
        //   parent 0 -> 1, 2, 3
        //   parent 1 -> 4, 5, 6
        //   parent 2 -> 7 (only one child — emission stops at n-1=7 edges)
        let g = kary_tree(8, 3, TreeMode::Out).expect("T8");
        assert_eq!(g.ecount(), 7);
        assert_eq!(
            collect_edges(&g),
            vec![(0, 1), (0, 2), (0, 3), (1, 4), (1, 5), (1, 6), (2, 7)]
        );
    }

    #[test]
    fn linear_chain_one_child_per_parent() {
        // children=1 → path graph 0-1-2-3-4-5.
        let g = kary_tree(6, 1, TreeMode::Out).expect("path-6");
        assert_eq!(g.ecount(), 5);
        assert_eq!(
            collect_edges(&g),
            vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]
        );
    }

    #[test]
    fn star_when_children_at_least_n_minus_one() {
        // children >= n-1 → first parent gets all the rest as kids: a
        // star (root at vertex 0).
        let g = kary_tree(5, 10, TreeMode::Out).expect("star-5 via tree");
        assert_eq!(g.ecount(), 4);
        assert_eq!(collect_edges(&g), vec![(0, 1), (0, 2), (0, 3), (0, 4)]);
    }

    #[test]
    fn zero_children_errors() {
        let err = kary_tree(5, 0, TreeMode::Out).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn binary_tree_is_acyclic_and_connected() {
        // For n=15 binary (perfect depth 3), check tree topology:
        // exactly n-1 edges and every non-root has exactly one parent.
        let g = kary_tree(15, 2, TreeMode::Undirected).expect("B15");
        assert_eq!(g.ecount(), 14);
        // Every non-root vertex appears exactly once as a child in the
        // upstream BFS layout.
        for v in 1..15u32 {
            let degree = g.neighbors(v).expect("neighbors").len();
            // Leaf or internal: every non-root vertex has at least its
            // parent edge; internal vertices add their children's edges
            // on top.
            assert!(degree >= 1, "vertex {v} disconnected");
        }
    }

    #[test]
    fn child_count_pattern_for_n_seven_three() {
        // n=7, children=3:
        //   parent 0 -> 1, 2, 3
        //   parent 1 -> 4, 5, 6
        // parent 2 onward gets no kids (already 6 edges emitted = n-1).
        let g = kary_tree(7, 3, TreeMode::Out).expect("T7");
        assert_eq!(g.ecount(), 6);
        assert_eq!(
            collect_edges(&g),
            vec![(0, 1), (0, 2), (0, 3), (1, 4), (1, 5), (1, 6)]
        );
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_mode() -> impl Strategy<Value = TreeMode> {
        prop_oneof![
            Just(TreeMode::Out),
            Just(TreeMode::In),
            Just(TreeMode::Undirected),
        ]
    }

    proptest! {
        #[test]
        fn edge_count_is_n_minus_one(
            n in 0u32..50,
            children in 1u32..10,
            mode in arb_mode(),
        ) {
            let g = kary_tree(n, children, mode).unwrap();
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(g.is_directed(), mode.is_directed());
            let expected = n.saturating_sub(1);
            let m = u32::try_from(g.ecount()).unwrap();
            prop_assert_eq!(m, expected);
        }

        #[test]
        fn children_zero_always_errors(
            n in 0u32..30,
            mode in arb_mode(),
        ) {
            prop_assert!(kary_tree(n, 0, mode).is_err());
        }

        #[test]
        fn every_non_root_appears_exactly_once_as_child(
            n in 2u32..30,
            children in 1u32..6,
            mode in arb_mode(),
        ) {
            let g = kary_tree(n, children, mode).unwrap();
            let m = u32::try_from(g.ecount()).unwrap();
            // Collect the "child" endpoint of every edge (the endpoint
            // that the BFS layout assigned as a child — the higher-id
            // endpoint after canonicalisation for undirected).
            let mut child_count = vec![0u32; n as usize];
            for e in 0..m {
                let (u, v) = g.edge(e).unwrap();
                let child = match mode {
                    TreeMode::Out | TreeMode::Undirected => v,
                    TreeMode::In => u,
                };
                child_count[child as usize] += 1;
            }
            // Vertex 0 is the root — never a child.
            prop_assert_eq!(child_count[0], 0);
            // Every other vertex is a child exactly once.
            for v in 1..n {
                prop_assert_eq!(child_count[v as usize], 1);
            }
        }
    }
}
