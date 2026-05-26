//! `gomory_hu_tree` (ALGO-FL-020) — Gomory-Hu cut tree.
//!
//! Counterpart of `igraph_gomory_hu_tree` in
//! `references/igraph/src/flow/flow.c:2479-2616`. The Gomory-Hu tree is
//! a compact representation of the *all-pairs* minimum s-t cut: it is
//! an undirected weighted tree on the same vertex set as the input
//! graph, with `V - 1` edges, such that the minimum cut value between
//! any two vertices `u, v` in the original graph equals the *minimum*
//! edge weight on the unique tree path from `u` to `v`.
//!
//! We use Gusfield's algorithm (1990), which computes the tree with
//! exactly `V - 1` calls to s-t max-flow / min-cut. Each call pulls a
//! `(value, source-side-partition)` pair from the FL-018 `st_mincut`
//! backend and rewrites a parent-pointer array in place; once the
//! loop finishes, the parent pointers describe the tree edges and the
//! saved flow values become the tree edge weights.
//!
//! # Reference
//!
//! Gusfield D., *Very simple methods for all pairs network flow
//! analysis*. SIAM J Comput 19(1):143-155, 1990.
//! <https://doi.org/10.1137/0219009>

use crate::core::{Graph, IgraphError, IgraphResult};

use super::st_mincut::st_mincut;

/// Output of [`gomory_hu_tree`] — a weighted undirected tree on the
/// same vertex set as the input graph, plus one flow value per tree
/// edge.
///
/// The flow vector is indexed by tree edge id in the order they were
/// added (`tree.edge_endpoints(eid)` gives the corresponding endpoints).
/// `flows.len() == tree.ecount() == max(vcount - 1, 0)`.
#[derive(Debug, Clone)]
pub struct GomoryHuTree {
    /// Undirected tree with the same vertex count as the input graph
    /// and exactly `max(vcount - 1, 0)` edges.
    pub tree: Graph,
    /// Tree-edge weights, in the same order as the edges of [`tree`].
    /// Each entry is the value of the s-t min cut between the two
    /// endpoints of that tree edge in the *original* graph (and, by
    /// the Gomory-Hu property, the minimum cut value between every
    /// pair separated by that edge in the tree).
    ///
    /// [`tree`]: GomoryHuTree::tree
    pub flows: Vec<f64>,
}

/// Gomory-Hu cut tree of an undirected graph (Gusfield 1990).
///
/// Returns a [`GomoryHuTree`] whose `tree` field is an undirected
/// weighted tree on the same vertex set as `graph` with `V - 1` edges,
/// and whose `flows` field gives one weight per tree edge. The
/// minimum s-t cut value between any two vertices `u, v` in the
/// original graph is exactly the *minimum* edge weight along the
/// unique tree path from `u` to `v`.
///
/// # Arguments
///
/// * `graph` — input graph. **Must be undirected**; directed graphs
///   are rejected with [`IgraphError::InvalidArgument`].
/// * `capacity` — optional per-edge capacity in the graph's edge-id
///   order. When `None`, each edge contributes unit capacity. When
///   `Some(c)`, `c.len()` must equal `graph.ecount()`, and every
///   entry must be finite and `≥ 0` (same contract as
///   [`crate::st_mincut`]).
///
/// # Returns
///
/// A [`GomoryHuTree`] with `tree.vcount() == graph.vcount()` and
/// `flows.len() == tree.ecount() == max(graph.vcount() - 1, 0)`.
///
/// For `graph.vcount() == 0` the returned tree is empty and `flows`
/// is empty. For `graph.vcount() == 1` the tree has one isolated
/// vertex and `flows` is empty.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] if `graph` is directed, the
///   capacity slice length differs from `ecount()`, or any capacity
///   entry is negative / non-finite.
///
/// [`IgraphError::InvalidArgument`]: crate::core::IgraphError::InvalidArgument
///
/// # Examples
///
/// `K_4` with unit capacities — every pair has min-cut 3 (each vertex's
/// degree), so every tree edge weight is 3.
///
/// ```
/// use rust_igraph::{Graph, gomory_hu_tree};
///
/// let mut k4 = Graph::new(4, false).unwrap();
/// for u in 0u32..4 {
///     for v in (u + 1)..4 {
///         k4.add_edge(u, v).unwrap();
///     }
/// }
///
/// let gh = gomory_hu_tree(&k4, None).unwrap();
/// assert_eq!(gh.tree.vcount(), 4);
/// assert_eq!(gh.tree.ecount(), 3);
/// assert_eq!(gh.flows.len(), 3);
/// for &w in &gh.flows {
///     assert_eq!(w, 3.0);
/// }
/// ```
///
/// # References
///
/// Gusfield D., *Very simple methods for all pairs network flow
/// analysis*. SIAM J Comput 19(1):143-155, 1990.
pub fn gomory_hu_tree(graph: &Graph, capacity: Option<&[f64]>) -> IgraphResult<GomoryHuTree> {
    // flow.c:2533 — Gomory-Hu is defined only on undirected graphs.
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "Gomory-Hu tree can only be calculated for undirected graphs".into(),
        ));
    }

    let n = graph.vcount();

    // n=0 → empty tree, empty flows. We bypass capacity validation here
    // because there are no max-flow calls to surface it through; an
    // empty graph with a non-empty capacity slice is a caller bug, so
    // we still surface it as InvalidArgument for consistency with FL-002.
    if n == 0 {
        if let Some(c) = capacity {
            if !c.is_empty() {
                return Err(IgraphError::InvalidArgument(format!(
                    "capacity length {} does not match edge count 0",
                    c.len()
                )));
            }
        }
        return Ok(GomoryHuTree {
            tree: Graph::new(0, false)?,
            flows: Vec::new(),
        });
    }

    // n=1 → one-vertex tree, empty flows. Same capacity sanity check.
    if n == 1 {
        if let Some(c) = capacity {
            let m = graph.ecount();
            if c.len() != m {
                return Err(IgraphError::InvalidArgument(format!(
                    "capacity length {} does not match edge count {m}",
                    c.len()
                )));
            }
        }
        return Ok(GomoryHuTree {
            tree: Graph::new(1, false)?,
            flows: Vec::new(),
        });
    }

    // Parent-pointer array (`neighbors[i]` = current tree-neighbor of i)
    // and per-vertex flow values. Both start at 0 — mirroring the
    // implicit "every edge points to node 0" init at flow.c:2544-2546.
    let n_usize = n as usize;
    let mut neighbors: Vec<u32> = vec![0; n_usize];
    let mut flow_values: Vec<f64> = vec![0.0; n_usize];

    // Main Gusfield loop (flow.c:2549-2581). Capacity validation is
    // delegated to the first st_mincut call — if the slice is malformed,
    // that call returns InvalidArgument and we propagate via `?`.
    for source in 1..n {
        let target = neighbors[source as usize];

        let cut = st_mincut(graph, source, target, capacity)?;
        let value = cut.value;
        flow_values[source as usize] = value;

        // Tree update (flow.c:2568-2580). `partition` is the source-side
        // vertex set returned by st_mincut, guaranteed to contain
        // `source` and not contain `target`. For every other vertex
        // `mid` in `partition` we either re-parent `mid` to `source` or
        // perform the swap-rotate that keeps the tree consistent when
        // `target`'s parent crosses into the source side.
        for &mid in &cut.partition {
            if mid == source {
                continue;
            }
            let mid_us = mid as usize;
            let target_us = target as usize;
            if neighbors[mid_us] == target {
                neighbors[mid_us] = source;
            } else if neighbors[target_us] == mid {
                // flow.c:2573 swap branch: rotate the tree so that
                // (source, mid) and (target, source) replace
                // (source, target) and (target, mid). The previous
                // flow_values[source] = value assignment above is
                // overwritten by flow_values[target]'s value, and
                // flow_values[target] receives `value`.
                neighbors[target_us] = source;
                neighbors[source as usize] = mid;
                flow_values[source as usize] = flow_values[target_us];
                flow_values[target_us] = value;
            }
        }
    }

    // Assemble the output tree (undirected, vcount unchanged, V-1 edges).
    let mut tree = Graph::new(n, false)?;
    let mut flows: Vec<f64> = Vec::with_capacity(n_usize - 1);
    for i in 1..n {
        // Edge (i, neighbors[i]); both endpoints are valid vertex ids
        // by construction of `neighbors` (always within [0, n)).
        tree.add_edge(i, neighbors[i as usize])?;
        flows.push(flow_values[i as usize]);
    }

    Ok(GomoryHuTree { tree, flows })
}

#[cfg(test)]
mod tests {
    //! Step-4 sanity tests — the full unit suite (boundary, error
    //! variants, multigraph, `K_4` issue #1810, …) is filled in at
    //! Step-5 by `/awu-test`. The cases here are the minimum needed
    //! to confirm Gusfield's loop produces a correct tree, not a
    //! comprehensive spec.

    use super::*;
    use crate::core::IgraphError;
    use crate::max_flow_value;

    const TOL: f64 = 1e-9;

    /// Validate the Gomory-Hu property: for every `(i, j)` pair, the
    /// minimum edge weight on the unique tree path equals the s-t
    /// max-flow in the original graph.
    fn validate_tree(graph: &Graph, gh: &GomoryHuTree, caps: Option<&[f64]>) {
        let n = gh.tree.vcount();
        // Build adjacency of the tree (vertex → (neighbor, flow value)).
        let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n as usize];
        for eid in 0..gh.tree.ecount() {
            let eid_u32 = u32::try_from(eid).expect("tree.ecount() within u32 in validate_tree");
            let (u, v) = gh.tree.edge(eid_u32).expect("tree edge");
            let weight = gh.flows[eid];
            adj[u as usize].push((v, weight));
            adj[v as usize].push((u, weight));
        }
        for src in 0..n {
            for dst in (src + 1)..n {
                // BFS in the tree from src to dst; the tree is acyclic
                // so there is a unique path; track the minimum edge
                // weight along it.
                let mut parent: Vec<i64> = vec![-1; n as usize];
                let mut parent_w: Vec<f64> = vec![0.0; n as usize];
                let mut queue: Vec<u32> = vec![src];
                let mut head = 0_usize;
                parent[src as usize] = i64::from(src);
                while head < queue.len() {
                    let u = queue[head];
                    head += 1;
                    if u == dst {
                        break;
                    }
                    for &(w, weight) in &adj[u as usize] {
                        if parent[w as usize] < 0 {
                            parent[w as usize] = i64::from(u);
                            parent_w[w as usize] = weight;
                            queue.push(w);
                        }
                    }
                }
                let mut min_w = f64::INFINITY;
                let mut cur = dst;
                while cur != src {
                    if parent_w[cur as usize] < min_w {
                        min_w = parent_w[cur as usize];
                    }
                    cur = u32::try_from(parent[cur as usize])
                        .expect("parent within u32 (set by BFS above)");
                }
                let mf = max_flow_value(graph, src, dst, caps).expect("max_flow_value");
                assert!(
                    (min_w - mf).abs() <= TOL,
                    "Gomory-Hu property failed for (src={src}, dst={dst}): tree min={min_w}, max-flow={mf}"
                );
            }
        }
    }

    #[test]
    fn empty_graph_returns_empty_tree() {
        let g = Graph::new(0, false).expect("graph");
        let gh = gomory_hu_tree(&g, None).expect("gh");
        assert_eq!(gh.tree.vcount(), 0);
        assert_eq!(gh.tree.ecount(), 0);
        assert!(gh.flows.is_empty());
    }

    #[test]
    fn single_vertex_returns_one_vertex_tree() {
        let g = Graph::new(1, false).expect("graph");
        let gh = gomory_hu_tree(&g, None).expect("gh");
        assert_eq!(gh.tree.vcount(), 1);
        assert_eq!(gh.tree.ecount(), 0);
        assert!(gh.flows.is_empty());
    }

    #[test]
    fn rejects_directed_graph() {
        let mut g = Graph::new(3, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        let err = gomory_hu_tree(&g, None).expect_err("must reject directed");
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn six_vertex_weighted_matches_c_reference() {
        // From references/igraph/tests/unit/igraph_gomory_hu_tree.c —
        // edges (0,1)(0,2)(1,2)(1,3)(1,4)(2,4)(3,4)(3,5)(4,5),
        // caps [1, 7, 1, 3, 2, 4, 1, 6, 2]. Validate via the
        // all-pairs Gomory-Hu property.
        let mut g = Graph::new(6, false).expect("graph");
        let edges = [
            (0u32, 1u32),
            (0, 2),
            (1, 2),
            (1, 3),
            (1, 4),
            (2, 4),
            (3, 4),
            (3, 5),
            (4, 5),
        ];
        for (u, v) in edges {
            g.add_edge(u, v).expect("edge");
        }
        let caps = [1.0, 7.0, 1.0, 3.0, 2.0, 4.0, 1.0, 6.0, 2.0];
        let gh = gomory_hu_tree(&g, Some(&caps)).expect("gh");
        assert_eq!(gh.tree.vcount(), 6);
        assert_eq!(gh.tree.ecount(), 5);
        assert_eq!(gh.flows.len(), 5);
        validate_tree(&g, &gh, Some(&caps));
    }

    #[test]
    fn k4_unit_capacities_matches_c_reference() {
        // K_4 unit caps (regression for issue #1810). Every pair has
        // min cut = 3, so every tree edge weight = 3.
        let mut g = Graph::new(4, false).expect("graph");
        for i in 0..4u32 {
            for j in (i + 1)..4u32 {
                g.add_edge(i, j).expect("edge");
            }
        }
        let gh = gomory_hu_tree(&g, None).expect("gh");
        assert_eq!(gh.tree.vcount(), 4);
        assert_eq!(gh.tree.ecount(), 3);
        for &w in &gh.flows {
            assert!((w - 3.0).abs() <= TOL, "expected weight 3, got {w}");
        }
        validate_tree(&g, &gh, None);
    }

    #[test]
    fn two_vertices_no_edge_zero_flow() {
        // Disconnected pair → only tree edge, weight 0.
        let g = Graph::new(2, false).expect("graph");
        let gh = gomory_hu_tree(&g, None).expect("gh");
        assert_eq!(gh.tree.vcount(), 2);
        assert_eq!(gh.tree.ecount(), 1);
        assert!(gh.flows[0].abs() <= TOL);
    }

    #[test]
    fn two_vertices_one_edge_unit_flow() {
        let mut g = Graph::new(2, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        let gh = gomory_hu_tree(&g, None).expect("gh");
        assert_eq!(gh.tree.ecount(), 1);
        assert!((gh.flows[0] - 1.0).abs() <= TOL);
        validate_tree(&g, &gh, None);
    }

    #[test]
    fn two_vertices_parallel_edges_multigraph() {
        // Multigraph: parallel edges raise the min cut.
        let mut g = Graph::new(2, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(0, 1).expect("edge");
        let gh = gomory_hu_tree(&g, None).expect("gh");
        assert!((gh.flows[0] - 3.0).abs() <= TOL);
        validate_tree(&g, &gh, None);
    }

    #[test]
    fn path_5v_unit_caps_all_pairs_one() {
        // Path 0-1-2-3-4 has min cut 1 for every pair (the cheapest
        // single edge disconnects them), so every tree edge weight = 1.
        let mut g = Graph::new(5, false).expect("graph");
        for i in 0..4u32 {
            g.add_edge(i, i + 1).expect("edge");
        }
        let gh = gomory_hu_tree(&g, None).expect("gh");
        assert_eq!(gh.tree.ecount(), 4);
        for &w in &gh.flows {
            assert!((w - 1.0).abs() <= TOL, "expected 1.0, got {w}");
        }
        validate_tree(&g, &gh, None);
    }

    #[test]
    fn ring_5v_unit_caps_all_pairs_two() {
        // Cycle 0-1-2-3-4-0: every pair has min cut 2 (must remove
        // two edges to disconnect).
        let mut g = Graph::new(5, false).expect("graph");
        for i in 0..5u32 {
            g.add_edge(i, (i + 1) % 5).expect("edge");
        }
        let gh = gomory_hu_tree(&g, None).expect("gh");
        for &w in &gh.flows {
            assert!((w - 2.0).abs() <= TOL, "expected 2.0, got {w}");
        }
        validate_tree(&g, &gh, None);
    }

    #[test]
    fn disconnected_two_components_zero_cross_flows() {
        // 0-1 and 2-3, no cross edges. min cut(0,2) = 0, etc. The tree
        // must still be a single tree on 4 vertices (Gusfield always
        // produces one tree regardless of connectivity); some weights
        // will be 0.
        let mut g = Graph::new(4, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(2, 3).expect("edge");
        let gh = gomory_hu_tree(&g, None).expect("gh");
        assert_eq!(gh.tree.vcount(), 4);
        assert_eq!(gh.tree.ecount(), 3);
        validate_tree(&g, &gh, None);
    }

    #[test]
    fn weighted_bridge_dominates_min_cut() {
        // Two triangles 0-1-2 and 3-4-5 joined by a single weak edge
        // (2,3) of capacity 0.5. min cut(any in {0,1,2}, any in {3,4,5})
        // is 0.5 — the bridge. Tree must contain that bridge with
        // weight 0.5; all other tree edges must have weight ≥ 0.5.
        let mut g = Graph::new(6, false).expect("graph");
        for (u, v) in [(0u32, 1u32), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
            g.add_edge(u, v).expect("edge");
        }
        let caps = [10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 0.5];
        let gh = gomory_hu_tree(&g, Some(&caps)).expect("gh");
        let min_flow = gh.flows.iter().copied().fold(f64::INFINITY, f64::min);
        assert!(
            (min_flow - 0.5).abs() <= TOL,
            "min flow should be 0.5, got {min_flow}"
        );
        validate_tree(&g, &gh, Some(&caps));
    }

    #[test]
    fn capacity_wrong_length_errors() {
        let mut g = Graph::new(3, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(1, 2).expect("edge");
        let bad = vec![1.0_f64; 99];
        let err = gomory_hu_tree(&g, Some(&bad)).expect_err("must err");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn capacity_negative_errors() {
        let mut g = Graph::new(3, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(1, 2).expect("edge");
        let caps = [1.0_f64, -0.5];
        let err = gomory_hu_tree(&g, Some(&caps)).expect_err("must err");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn capacity_nan_errors() {
        let mut g = Graph::new(3, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(1, 2).expect("edge");
        let caps = [1.0_f64, f64::NAN];
        let err = gomory_hu_tree(&g, Some(&caps)).expect_err("must err");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn empty_graph_rejects_nonempty_capacity() {
        let g = Graph::new(0, false).expect("graph");
        let caps = [1.0_f64];
        let err = gomory_hu_tree(&g, Some(&caps)).expect_err("must err");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn unit_caps_match_explicit_unit_capacity_vector() {
        // None should produce the same tree topology + weights as a
        // fully-unit capacity vector. (Order of edges may differ, but
        // both must satisfy validate_tree against the same graph, and
        // the multisets of weights must match.)
        let mut g = Graph::new(5, false).expect("graph");
        for i in 0..5u32 {
            for j in (i + 1)..5u32 {
                g.add_edge(i, j).expect("edge");
            }
        }
        let gh_none = gomory_hu_tree(&g, None).expect("gh_none");
        let caps = vec![1.0_f64; g.ecount()];
        let gh_unit = gomory_hu_tree(&g, Some(&caps)).expect("gh_unit");
        let mut a = gh_none.flows.clone();
        let mut b = gh_unit.flows.clone();
        a.sort_by(|x, y| x.partial_cmp(y).expect("no NaN"));
        b.sort_by(|x, y| x.partial_cmp(y).expect("no NaN"));
        for (x, y) in a.iter().zip(b.iter()) {
            assert!((x - y).abs() <= TOL, "weight mismatch: {x} vs {y}");
        }
    }

    #[test]
    fn signature_symbol_is_stable() {
        // Frozen signature guard — prevents accidental signature drift.
        const SYMBOL: fn(&Graph, Option<&[f64]>) -> IgraphResult<GomoryHuTree> = gomory_hu_tree;
        let _ = SYMBOL;
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    //! Proptest invariants for Gomory-Hu tree:
    //!
    //! 1. **Shape**: `tree.vcount() == graph.vcount()`,
    //!    `tree.ecount() == max(vcount - 1, 0)`, `flows.len() ==
    //!    tree.ecount()`, and every flow value is finite and `≥ 0`.
    //! 2. **Gomory-Hu property**: for every `(i, j)` pair, the minimum
    //!    edge weight on the unique tree path equals
    //!    `max_flow_value(graph, i, j, capacity)` within tolerance.
    //! 3. **Tree connectivity**: the produced tree is acyclic and (for
    //!    `vcount ≥ 1`) connected — a BFS from vertex 0 reaches all
    //!    vertices.
    //! 4. **Unit-caps parity**: `gomory_hu_tree(g, None)` and
    //!    `gomory_hu_tree(g, Some(unit caps))` produce equal flow
    //!    multisets.

    use super::*;
    use crate::max_flow_value;
    use proptest::prelude::*;

    const TOL: f64 = 1e-9;

    fn xorshift(mut r: u64) -> u64 {
        r ^= r << 13;
        r ^= r >> 7;
        r ^= r << 17;
        r
    }

    fn build_random_undirected(seed: u64, n: u32, m_max: u32) -> Graph {
        let mut g = Graph::new(n, false).expect("graph");
        let mut state = seed | 1;
        for _ in 0..m_max {
            state = xorshift(state);
            let u = u32::try_from(state % u64::from(n)).expect("modulo fits");
            state = xorshift(state);
            let v = u32::try_from(state % u64::from(n)).expect("modulo fits");
            if u == v {
                continue;
            }
            g.add_edge(u, v).expect("edge");
        }
        g
    }

    fn tree_min_weight_on_path(gh: &GomoryHuTree, src: u32, dst: u32) -> f64 {
        let n = gh.tree.vcount();
        let mut adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n as usize];
        for eid in 0..gh.tree.ecount() {
            let eid_u32 = u32::try_from(eid).expect("ecount fits in u32");
            let (u, v) = gh.tree.edge(eid_u32).expect("edge");
            let w = gh.flows[eid];
            adj[u as usize].push((v, w));
            adj[v as usize].push((u, w));
        }
        let mut parent: Vec<i64> = vec![-1; n as usize];
        let mut parent_w: Vec<f64> = vec![0.0; n as usize];
        let mut queue: Vec<u32> = vec![src];
        let mut head = 0_usize;
        parent[src as usize] = i64::from(src);
        while head < queue.len() {
            let u = queue[head];
            head += 1;
            if u == dst {
                break;
            }
            for &(w, weight) in &adj[u as usize] {
                if parent[w as usize] < 0 {
                    parent[w as usize] = i64::from(u);
                    parent_w[w as usize] = weight;
                    queue.push(w);
                }
            }
        }
        let mut min_w = f64::INFINITY;
        let mut cur = dst;
        while cur != src {
            if parent_w[cur as usize] < min_w {
                min_w = parent_w[cur as usize];
            }
            cur = u32::try_from(parent[cur as usize]).expect("parent within u32");
        }
        min_w
    }

    proptest! {
        #[test]
        fn shape_invariants(
            seed in any::<u64>(),
            n in 1u32..7,
            m in 0u32..12,
        ) {
            let g = build_random_undirected(seed, n, m);
            let gh = gomory_hu_tree(&g, None).expect("gh");
            prop_assert_eq!(gh.tree.vcount(), n);
            prop_assert_eq!(gh.tree.ecount(), (n - 1) as usize);
            prop_assert_eq!(gh.flows.len(), (n - 1) as usize);
            for &w in &gh.flows {
                prop_assert!(w.is_finite(), "flow {w} not finite");
                prop_assert!(w >= 0.0, "flow {w} negative");
            }
        }

        #[test]
        fn gomory_hu_property_holds(
            seed in any::<u64>(),
            n in 2u32..6,
            m in 0u32..10,
        ) {
            let g = build_random_undirected(seed, n, m);
            let gh = gomory_hu_tree(&g, None).expect("gh");
            for src in 0..n {
                for dst in (src + 1)..n {
                    let tree_min = tree_min_weight_on_path(&gh, src, dst);
                    let mf = max_flow_value(&g, src, dst, None).expect("mf");
                    prop_assert!(
                        (tree_min - mf).abs() <= TOL,
                        "Gomory-Hu property violated for (src={}, dst={}): tree min={}, max-flow={}, seed={}",
                        src, dst, tree_min, mf, seed
                    );
                }
            }
        }

        #[test]
        fn tree_is_connected(
            seed in any::<u64>(),
            n in 1u32..7,
            m in 0u32..12,
        ) {
            let g = build_random_undirected(seed, n, m);
            let gh = gomory_hu_tree(&g, None).expect("gh");
            // BFS from vertex 0 — should reach every other vertex
            // because the tree has n-1 edges and is connected by
            // construction.
            let mut seen = vec![false; n as usize];
            seen[0] = true;
            let mut queue: Vec<u32> = vec![0];
            let mut head = 0_usize;
            let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n as usize];
            for eid in 0..gh.tree.ecount() {
                let eid_u32 = u32::try_from(eid).expect("ecount fits");
                let (u, v) = gh.tree.edge(eid_u32).expect("edge");
                adj[u as usize].push(v);
                adj[v as usize].push(u);
            }
            while head < queue.len() {
                let v = queue[head];
                head += 1;
                for &w in &adj[v as usize] {
                    if !seen[w as usize] {
                        seen[w as usize] = true;
                        queue.push(w);
                    }
                }
            }
            for (v, &s) in seen.iter().enumerate() {
                prop_assert!(s, "vertex {v} not reachable from 0 in the tree (seed={seed})");
            }
        }

        #[test]
        fn unit_caps_parity(
            seed in any::<u64>(),
            n in 1u32..6,
            m in 0u32..10,
        ) {
            let g = build_random_undirected(seed, n, m);
            let caps = vec![1.0_f64; g.ecount()];
            let gh_none = gomory_hu_tree(&g, None).expect("gh_none");
            let gh_unit = gomory_hu_tree(&g, Some(&caps)).expect("gh_unit");
            let mut a = gh_none.flows.clone();
            let mut b = gh_unit.flows.clone();
            a.sort_by(|x, y| x.partial_cmp(y).expect("no NaN in a"));
            b.sort_by(|x, y| x.partial_cmp(y).expect("no NaN in b"));
            prop_assert_eq!(a.len(), b.len());
            for (x, y) in a.iter().zip(b.iter()) {
                prop_assert!((x - y).abs() <= TOL, "weight mismatch: {} vs {}", x, y);
            }
        }
    }
}
