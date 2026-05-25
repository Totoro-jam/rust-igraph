//! Parent-vector tree/forest decoder (ALGO-CN-017).
//!
//! Counterpart of `igraph_tree_from_parent_vector()` in
//! `references/igraph/src/constructors/trees.c:70-161`.
//!
//! A *parent vector* of length `n` represents a rooted forest on `n`
//! vertices: `parents[v]` is the vertex id of `v`'s parent, or any
//! negative value to mark `v` as a root (no parent). The decoder walks
//! every vertex once and follows its parent chain, emitting one edge per
//! `(v, parent[v])` link. Per-round colouring detects both self-loops
//! (`parent[v] == v`) and longer cycles (`v → … → v`) in linear time.
//!
//! [`TreeMode`] is re-exported from [`crate::kary_tree`] and controls
//! arc orientation:
//!
//! * [`TreeMode::Out`] — directed, every arc flows **parent → child**.
//! * [`TreeMode::In`] — directed, every arc flows **child → parent**.
//! * [`TreeMode::Undirected`] — undirected, raw `(child, parent)`
//!   stored as canonical `(min, max)` by `Graph::add_edges`.
//!
//! The output is guaranteed to be a forest (or a tree, when there is
//! exactly one root); validation rejects negative-or-large parent ids,
//! self-loops, and longer cycles up-front.
//!
//! Time complexity: `O(n)` where `n = parents.len()`. Each vertex is
//! visited exactly once across all rounds (once as `i`, once as the
//! cascading `u`).

use crate::algorithms::constructors::kary_tree::TreeMode;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Decode a parent vector into the tree or forest it represents.
///
/// `parents[v]` is the vertex id of `v`'s parent. Any negative value
/// marks `v` as a root. Roots produce no edge (singletons), so the
/// emitted graph has between `0` and `n - 1` edges.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — some non-negative `parents[v]`
///   is `>= parents.len()`, or the chain starting at some `v` forms a
///   self-loop or longer cycle, or `parents.len()` overflows [`u32`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{TreeMode, tree_from_parent_vector};
///
/// // Two roots (-1) produce a forest of two paths.
/// let g = tree_from_parent_vector(&[-1, 0, 1, -1, 3], TreeMode::Out).unwrap();
/// assert_eq!(g.vcount(), 5);
/// assert_eq!(g.ecount(), 3);
/// assert!(g.is_directed());
/// ```
pub fn tree_from_parent_vector(parents: &[i64], mode: TreeMode) -> IgraphResult<Graph> {
    let n_usize = parents.len();
    let n = u32::try_from(n_usize).map_err(|_| {
        IgraphError::InvalidArgument(
            "tree_from_parent_vector: parents.len() overflows u32".to_string(),
        )
    })?;

    let directed = !matches!(mode, TreeMode::Undirected);
    let intree = !matches!(mode, TreeMode::Out);

    if n == 0 {
        return Graph::new(0, directed);
    }

    // 0 means "never seen"; round counter c starts at 1.
    let mut seen: Vec<u32> = vec![0; n_usize];
    // Per upstream, hint the allocator with 2*(n-1) for small graphs,
    // but reserve only 2048 entries when n > 1024 so extracting a
    // small subtree of a huge graph does not over-allocate. Our edge
    // tuples carry 8 bytes each rather than two ints, so divide by 2
    // when scaling to the (VertexId, VertexId) buffer.
    let edge_cap = if n_usize > 1024 { 1024 } else { n_usize - 1 };
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(edge_cap);

    let n_i64 = i64::from(n);
    let mut c: u32 = 1;
    for i in 0..n {
        if seen[i as usize] != 0 {
            c = c.checked_add(1).ok_or_else(|| {
                IgraphError::InvalidArgument(
                    "tree_from_parent_vector: round counter overflows u32".to_string(),
                )
            })?;
            continue;
        }
        let mut v = i;
        loop {
            seen[v as usize] = c;
            let u_raw = parents[v as usize];
            if u_raw < 0 {
                break; // v is a root
            }
            if u_raw >= n_i64 {
                return Err(IgraphError::InvalidArgument(format!(
                    "tree_from_parent_vector: invalid parent id {u_raw} for vertex {v} (must be < {n})",
                )));
            }
            // Bounds above guarantee 0 <= u_raw < n <= u32::MAX. The
            // try_from cannot fail in practice; map_err keeps the path
            // free of unwrap/expect per project conventions.
            let u = u32::try_from(u_raw).map_err(|_| {
                IgraphError::InvalidArgument(format!(
                    "tree_from_parent_vector: invalid parent id {u_raw} for vertex {v}",
                ))
            })?;

            if intree {
                edges.push((v, u));
            } else {
                edges.push((u, v));
            }

            if seen[u as usize] != 0 {
                if seen[u as usize] == c {
                    return Err(IgraphError::InvalidArgument(if u == v {
                        format!(
                            "tree_from_parent_vector: self-loop at vertex {v} (parents[{v}] = {v})",
                        )
                    } else {
                        format!(
                            "tree_from_parent_vector: cycle detected reaching vertex {u} from vertex {v}",
                        )
                    }));
                }
                break; // u was seen in a previous round, stop traversal
            }

            v = u;
        }
        c = c.checked_add(1).ok_or_else(|| {
            IgraphError::InvalidArgument(
                "tree_from_parent_vector: round counter overflows u32".to_string(),
            )
        })?;
    }

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn collect_edges_ordered(g: &Graph) -> Vec<(u32, u32)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..m).map(|e| g.edge(e).expect("edge in range")).collect()
    }

    fn collect_edges_canonical(g: &Graph) -> BTreeSet<(u32, u32)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..m)
            .map(|e| g.edge(e).expect("edge"))
            .map(|(a, b)| if a <= b { (a, b) } else { (b, a) })
            .collect()
    }

    fn uf_find(parent: &mut [u32], mut node: u32) -> u32 {
        while parent[node as usize] != node {
            let grand = parent[parent[node as usize] as usize];
            parent[node as usize] = grand;
            node = grand;
        }
        node
    }

    fn is_forest(g: &Graph) -> bool {
        // Acyclic when treated as undirected.
        let vcount = g.vcount();
        let mut parent: Vec<u32> = (0..vcount).collect();
        let m = u32::try_from(g.ecount()).expect("m fits u32");
        for e in 0..m {
            let (a, b) = g.edge(e).expect("edge");
            let ra = uf_find(&mut parent, a);
            let rb = uf_find(&mut parent, b);
            if ra == rb {
                return false;
            }
            parent[ra as usize] = rb;
        }
        true
    }

    #[test]
    fn upstream_fixture_out_mode() {
        // From references/igraph/tests/unit/igraph_tree_from_parent_vector.out:
        // parents = [4, 4, 1, -2, 3], OUT mode emits (parent, child).
        let g = tree_from_parent_vector(&[4, 4, 1, -2, 3], TreeMode::Out).expect("ok");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 4);
        assert!(g.is_directed());
        // Expected ordered emission (matches upstream .out):
        // (4,0) (3,4) (4,1) (1,2)
        let expected = vec![(4, 0), (3, 4), (4, 1), (1, 2)];
        assert_eq!(collect_edges_ordered(&g), expected);
        assert!(is_forest(&g));
    }

    #[test]
    fn upstream_fixture_in_mode() {
        let g = tree_from_parent_vector(&[4, 4, 1, -2, 3], TreeMode::In).expect("ok");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 4);
        assert!(g.is_directed());
        // Expected ordered emission (child, parent):
        // (0,4) (4,3) (1,4) (2,1)
        let expected = vec![(0, 4), (4, 3), (1, 4), (2, 1)];
        assert_eq!(collect_edges_ordered(&g), expected);
        assert!(is_forest(&g));
    }

    #[test]
    fn upstream_fixture_undirected_mode() {
        let g = tree_from_parent_vector(&[4, 4, 1, -2, 3], TreeMode::Undirected).expect("ok");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 4);
        assert!(!g.is_directed());
        // After (min, max) canonicalisation by Graph::add_edges:
        // (0,4) (3,4) (1,4) (1,2)
        let edges = collect_edges_canonical(&g);
        let expected: BTreeSet<(u32, u32)> = [(0, 4), (3, 4), (1, 4), (1, 2)].into_iter().collect();
        assert_eq!(edges, expected);
        assert!(is_forest(&g));
    }

    #[test]
    fn upstream_fixture_forest_two_roots() {
        // [-1, 4, 1, -2, 3] — vertex 0 and vertex 3 are both roots.
        let g = tree_from_parent_vector(&[-1, 4, 1, -2, 3], TreeMode::Out).expect("ok");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 3);
        assert!(g.is_directed());
        let expected = vec![(4, 1), (3, 4), (1, 2)];
        assert_eq!(collect_edges_ordered(&g), expected);
        assert!(is_forest(&g));
    }

    #[test]
    fn null_graph_empty_parents() {
        let g = tree_from_parent_vector(&[], TreeMode::Out).expect("ok");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn edgeless_graph_all_roots() {
        let g = tree_from_parent_vector(&[-1, -1, -1, -1, -1], TreeMode::Out).expect("ok");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn undirected_isolated_roots() {
        let g = tree_from_parent_vector(&[-3, -1], TreeMode::Undirected).expect("ok");
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn invalid_parent_id_out_of_range() {
        let err = tree_from_parent_vector(&[5, 4, 1, -2, 3], TreeMode::Out).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn invalid_self_loop_rejected() {
        // parents[0] = 0 ⇒ self-loop.
        let err = tree_from_parent_vector(&[0, 4, 1, -2, 3], TreeMode::Out).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => {
                assert!(
                    msg.contains("self-loop"),
                    "expected self-loop msg, got {msg}"
                );
            }
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn invalid_longer_cycle_rejected() {
        // 0 → 4 → 3 → 0 forms a 3-cycle.
        let err = tree_from_parent_vector(&[4, 4, 1, 0, 3], TreeMode::Out).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => {
                assert!(msg.contains("cycle"), "expected cycle msg, got {msg}");
            }
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn singleton_root() {
        let g = tree_from_parent_vector(&[-1], TreeMode::Out).expect("ok");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn star_centred_on_zero() {
        // Vertex 0 is root, every other vertex's parent is 0.
        let g = tree_from_parent_vector(&[-1, 0, 0, 0, 0], TreeMode::Undirected).expect("ok");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 4);
        let deg0 = g.neighbors(0).expect("neighbors").len();
        assert_eq!(deg0, 4, "centre has degree 4");
        for v in 1..5u32 {
            let d = g.neighbors(v).expect("neighbors").len();
            assert_eq!(d, 1, "leaf {v} has degree 1");
        }
    }

    #[test]
    fn chain_from_parent_vector() {
        // parents = [-1, 0, 1, 2, 3] ⇒ chain 0 — 1 — 2 — 3 — 4
        // (undirected so we can read full degree via neighbors()).
        let g = tree_from_parent_vector(&[-1, 0, 1, 2, 3], TreeMode::Undirected).expect("ok");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 4);
        for v in 0..5u32 {
            let deg = g.neighbors(v).expect("neighbors").len();
            let expected = if v == 0 || v == 4 { 1 } else { 2 };
            assert_eq!(deg, expected, "vertex {v} degree");
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    /// Build a guaranteed-valid parent vector: every non-root entry
    /// points to a strictly-smaller vertex id, which is a topological
    /// order ⇒ cycle-free by construction.
    fn arb_valid_parents() -> impl Strategy<Value = Vec<i64>> {
        (1usize..30).prop_flat_map(|n| {
            let mut strategies: Vec<BoxedStrategy<i64>> = Vec::with_capacity(n);
            strategies.push(Just(-1_i64).boxed()); // vertex 0 is always a root
            for i in 1..n {
                // either a root (negative) or a smaller vertex id.
                strategies
                    .push(prop_oneof![Just(-1_i64), (0i64..(i as i64)).prop_map(|p| p),].boxed());
            }
            strategies
        })
    }

    fn arb_mode() -> impl Strategy<Value = TreeMode> {
        prop_oneof![
            Just(TreeMode::Out),
            Just(TreeMode::In),
            Just(TreeMode::Undirected),
        ]
    }

    fn uf_find(parent: &mut [u32], mut node: u32) -> u32 {
        while parent[node as usize] != node {
            let grand = parent[parent[node as usize] as usize];
            parent[node as usize] = grand;
            node = grand;
        }
        node
    }

    proptest! {
        #[test]
        fn always_a_forest((parents, mode) in (arb_valid_parents(), arb_mode())) {
            let g = tree_from_parent_vector(&parents, mode).unwrap();
            let n = u32::try_from(parents.len()).unwrap();
            prop_assert_eq!(g.vcount(), n);
            // ecount = number of non-root entries
            let non_roots = parents.iter().filter(|&&p| p >= 0).count();
            prop_assert_eq!(g.ecount(), non_roots);
            // Acyclic (undirected sense): union-find without same-root collisions.
            let mut uf: Vec<u32> = (0..n).collect();
            let m = u32::try_from(g.ecount()).unwrap();
            for e in 0..m {
                let (a, b) = g.edge(e).unwrap();
                let ra = uf_find(&mut uf, a);
                let rb = uf_find(&mut uf, b);
                prop_assert_ne!(ra, rb, "found cycle in supposedly forest output");
                uf[ra as usize] = rb;
            }
        }

        #[test]
        fn no_self_loops((parents, mode) in (arb_valid_parents(), arb_mode())) {
            let g = tree_from_parent_vector(&parents, mode).unwrap();
            let m = u32::try_from(g.ecount()).unwrap();
            for e in 0..m {
                let (a, b) = g.edge(e).unwrap();
                prop_assert_ne!(a, b);
            }
        }

        #[test]
        fn no_duplicate_undirected_edges(parents in arb_valid_parents()) {
            let g = tree_from_parent_vector(&parents, TreeMode::Undirected).unwrap();
            let m = u32::try_from(g.ecount()).unwrap();
            let mut seen: BTreeSet<(u32, u32)> = BTreeSet::new();
            for e in 0..m {
                let (a, b) = g.edge(e).unwrap();
                let canon = if a <= b { (a, b) } else { (b, a) };
                prop_assert!(seen.insert(canon), "duplicate edge {:?}", canon);
            }
        }

        #[test]
        fn directedness_matches_mode((parents, mode) in (arb_valid_parents(), arb_mode())) {
            let g = tree_from_parent_vector(&parents, mode).unwrap();
            match mode {
                TreeMode::Out | TreeMode::In => prop_assert!(g.is_directed()),
                TreeMode::Undirected => prop_assert!(!g.is_directed()),
            }
        }

        #[test]
        fn in_and_out_modes_reverse_each_other(parents in arb_valid_parents()) {
            let g_out = tree_from_parent_vector(&parents, TreeMode::Out).unwrap();
            let g_in = tree_from_parent_vector(&parents, TreeMode::In).unwrap();
            prop_assert_eq!(g_out.ecount(), g_in.ecount());
            let m = u32::try_from(g_out.ecount()).unwrap();
            for e in 0..m {
                let (a, b) = g_out.edge(e).unwrap();
                let (c, d) = g_in.edge(e).unwrap();
                prop_assert_eq!((a, b), (d, c), "edge {} not reversed", e);
            }
        }

        #[test]
        fn out_of_range_parent_errors(
            n in 3u32..15,
            bad in 100i64..200,
        ) {
            let mut parents = vec![-1_i64; n as usize];
            parents[0] = bad; // bad >= 100 > n
            let err = tree_from_parent_vector(&parents, TreeMode::Out).unwrap_err();
            prop_assert!(matches!(err, IgraphError::InvalidArgument(_)));
        }

        #[test]
        fn self_loop_errors(n in 1u32..15) {
            let mut parents = vec![-1_i64; n as usize];
            parents[0] = 0; // self-loop on vertex 0
            let err = tree_from_parent_vector(&parents, TreeMode::Out).unwrap_err();
            prop_assert!(matches!(err, IgraphError::InvalidArgument(_)));
        }
    }
}
