//! Per-edge trussness via k-truss decomposition (ALGO-PR-036).
//!
//! Counterpart of `igraph_trussness()` from
//! `references/igraph/src/centrality/truss.cpp` (Wang-Cheng 2012).
//!
//! A *k-truss* is a maximal subgraph in which every edge participates
//! in at least k-2 triangles. The *trussness* of an edge is the
//! largest k such that the edge belongs to a k-truss.
//!
//! Edges that do not participate in any triangle have trussness 2
//! (by convention they belong to the 2-truss, which is the entire
//! graph). Self-loops also receive trussness 2.

use std::collections::HashSet;

use crate::algorithms::properties::multiplicity::has_multiple;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Compute the trussness of every edge in an undirected graph.
///
/// Returns a `Vec<u32>` of length `graph.ecount()` where entry `i`
/// is the trussness of edge `i`. Multigraphs are rejected with an
/// error; self-loops are accepted and receive trussness 2.
///
/// Counterpart of `igraph_trussness()` from
/// `references/igraph/src/centrality/truss.cpp`.
///
/// # Errors
///
/// - `IgraphError::Unsupported` if the graph has parallel edges
///   (multigraph).
/// - `IgraphError::InvalidArgument` if the graph is directed.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, trussness};
///
/// // K4 (complete graph on 4 vertices): every edge is in 2
/// // triangles → trussness = 4 for all 6 edges.
/// let mut g = Graph::with_vertices(4);
/// for u in 0..4u32 {
///     for v in (u + 1)..4 {
///         g.add_edge(u, v).unwrap();
///     }
/// }
/// let t = trussness(&g).unwrap();
/// assert!(t.iter().all(|&x| x == 4));
///
/// // Triangle (K3): every edge is in 1 triangle → trussness = 3.
/// let mut g = Graph::with_vertices(3);
/// for (u, v) in [(0, 1), (0, 2), (1, 2)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let t = trussness(&g).unwrap();
/// assert!(t.iter().all(|&x| x == 3));
///
/// // Path 0-1-2: no triangles → trussness = 2 for both edges.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let t = trussness(&g).unwrap();
/// assert!(t.iter().all(|&x| x == 2));
/// ```
pub fn trussness(graph: &Graph) -> IgraphResult<Vec<u32>> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "trussness is only defined for undirected graphs".into(),
        ));
    }

    let edge_count = graph.ecount();
    if edge_count == 0 {
        return Ok(Vec::new());
    }

    if has_multiple(graph)? {
        return Err(IgraphError::Unsupported(
            "trussness does not support multigraphs",
        ));
    }

    let adj = build_simple_adj(graph)?;
    let mut support = compute_support(graph, &adj)?;
    peel_trussness(graph, &adj, &mut support)
}

fn build_simple_adj(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let vert_count = graph.vcount();
    let mut adj: Vec<Vec<VertexId>> = Vec::with_capacity(vert_count as usize);
    for vid in 0..vert_count {
        let raw = graph.neighbors(vid)?;
        let mut simple: Vec<VertexId> = raw.into_iter().filter(|&nb| nb != vid).collect();
        simple.sort_unstable();
        simple.dedup();
        adj.push(simple);
    }
    Ok(adj)
}

fn compute_support(graph: &Graph, adj: &[Vec<VertexId>]) -> IgraphResult<Vec<u32>> {
    let vert_count = graph.vcount();
    let mut support: Vec<u32> = vec![0; graph.ecount()];
    let mut mark: Vec<u32> = vec![0; vert_count as usize];

    for v1 in 0..vert_count {
        let nei1 = &adj[v1 as usize];
        if nei1.len() < 2 {
            continue;
        }
        let v1_mark = v1
            .checked_add(1)
            .ok_or(IgraphError::Internal("vertex id overflow"))?;

        for &v2 in nei1 {
            if v2 >= v1 {
                break;
            }
            mark[v2 as usize] = v1_mark;
        }

        for &v2 in nei1 {
            if v2 >= v1 {
                break;
            }
            for &v3 in &adj[v2 as usize] {
                if v3 >= v2 {
                    break;
                }
                if mark[v3 as usize] == v1_mark {
                    if let Some(eid) = graph.find_eid(v1, v2)? {
                        support[eid as usize] = support[eid as usize].saturating_add(1);
                    }
                    if let Some(eid) = graph.find_eid(v1, v3)? {
                        support[eid as usize] = support[eid as usize].saturating_add(1);
                    }
                    if let Some(eid) = graph.find_eid(v2, v3)? {
                        support[eid as usize] = support[eid as usize].saturating_add(1);
                    }
                }
            }
        }
    }
    Ok(support)
}

#[allow(clippy::similar_names)]
fn peel_trussness(
    graph: &Graph,
    adj: &[Vec<VertexId>],
    support: &mut [u32],
) -> IgraphResult<Vec<u32>> {
    let edge_count = support.len();
    let max_support = support.iter().copied().max().unwrap_or(0);
    let bucket_count = (max_support as usize).saturating_add(1);

    let mut truss: Vec<u32> = vec![0; edge_count];
    let mut completed: Vec<bool> = vec![false; edge_count];
    let mut buckets: Vec<HashSet<u32>> = vec![HashSet::new(); bucket_count];

    for eid in 0..edge_count {
        let eid_u32 = u32::try_from(eid).map_err(|_| IgraphError::Internal("edge id overflow"))?;
        buckets[support[eid] as usize].insert(eid_u32);
    }

    for level in 0..bucket_count {
        let level_u32 =
            u32::try_from(level).map_err(|_| IgraphError::Internal("level overflow"))?;

        while let Some(&eid) = buckets[level].iter().next() {
            buckets[level].remove(&eid);
            completed[eid as usize] = true;
            truss[eid as usize] = level_u32
                .checked_add(2)
                .ok_or(IgraphError::Internal("trussness overflow"))?;

            let (src, dst) = graph.edge(eid)?;

            if src == dst {
                continue;
            }

            let common = sorted_intersection(&adj[src as usize], &adj[dst as usize]);

            for nb in common {
                let Some(edge_sn) = graph.find_eid(src, nb)? else {
                    continue;
                };
                let Some(edge_dn) = graph.find_eid(dst, nb)? else {
                    continue;
                };

                if completed[edge_sn as usize] || completed[edge_dn as usize] {
                    continue;
                }

                for &edge in &[edge_sn, edge_dn] {
                    let old_sup = support[edge as usize];
                    if old_sup > level_u32 {
                        buckets[old_sup as usize].remove(&edge);
                        let new_sup = old_sup.saturating_sub(1);
                        support[edge as usize] = new_sup;
                        let target = (new_sup as usize).max(level);
                        buckets[target].insert(edge);
                    }
                }
            }
        }
    }

    Ok(truss)
}

fn sorted_intersection(a: &[VertexId], b: &[VertexId]) -> Vec<VertexId> {
    let mut result = Vec::new();
    let (mut ia, mut ib) = (0, 0);
    while ia < a.len() && ib < b.len() {
        match a[ia].cmp(&b[ib]) {
            std::cmp::Ordering::Less => ia += 1,
            std::cmp::Ordering::Greater => ib += 1,
            std::cmp::Ordering::Equal => {
                result.push(a[ia]);
                ia += 1;
                ib += 1;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_undirected(n: u32, edges: &[(u32, u32)]) -> Graph {
        let mut g = Graph::with_vertices(n);
        for &(u, v) in edges {
            g.add_edge(u, v).unwrap();
        }
        g
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let t = trussness(&g).unwrap();
        assert!(t.is_empty());
    }

    #[test]
    fn singleton_no_edges() {
        let g = Graph::with_vertices(1);
        let t = trussness(&g).unwrap();
        assert!(t.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(10);
        let t = trussness(&g).unwrap();
        assert!(t.is_empty());
    }

    #[test]
    fn path_graph() {
        let g = make_undirected(3, &[(0, 1), (1, 2)]);
        let t = trussness(&g).unwrap();
        assert_eq!(t, vec![2, 2]);
    }

    #[test]
    fn triangle_k3() {
        let g = make_undirected(3, &[(0, 1), (0, 2), (1, 2)]);
        let t = trussness(&g).unwrap();
        assert_eq!(t, vec![3, 3, 3]);
    }

    #[test]
    fn k4_complete() {
        let g = make_undirected(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
        let t = trussness(&g).unwrap();
        assert!(t.iter().all(|&x| x == 4), "K4 trussness: {t:?}");
    }

    #[test]
    fn k5_complete() {
        let g = make_undirected(
            5,
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 4),
                (1, 2),
                (1, 3),
                (1, 4),
                (2, 3),
                (2, 4),
                (3, 4),
            ],
        );
        let t = trussness(&g).unwrap();
        assert!(t.iter().all(|&x| x == 5), "K5 trussness: {t:?}");
    }

    #[test]
    fn igraph_c_test_graph() {
        let g = make_undirected(
            12,
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 4),
                (1, 2),
                (1, 3),
                (1, 4),
                (2, 3),
                (2, 4),
                (3, 4),
                (3, 6),
                (3, 11),
                (4, 5),
                (4, 6),
                (5, 6),
                (5, 7),
                (5, 8),
                (5, 9),
                (6, 7),
                (6, 10),
                (6, 11),
                (7, 8),
                (7, 9),
                (8, 9),
                (8, 10),
            ],
        );
        let t = trussness(&g).unwrap();
        #[rustfmt::skip]
        let expected = vec![
            5, 5, 5, 5,         // 0-1, 0-2, 0-3, 0-4
            5, 5, 5, 5, 5, 5,   // 1-2, 1-3, 1-4, 2-3, 2-4, 3-4
            3, 3,                // 3-6, 3-11
            3, 3, 3,             // 4-5, 4-6, 5-6
            4, 4, 4,             // 5-7, 5-8, 5-9
            3, 2, 3,             // 6-7, 6-10, 6-11
            4, 4, 4, 2,          // 7-8, 7-9, 8-9, 8-10
        ];
        assert_eq!(t, expected);
    }

    #[test]
    fn graph_with_self_loops() {
        let mut g = make_undirected(3, &[(0, 1), (0, 2), (1, 2)]);
        g.add_edge(0, 0).unwrap();
        g.add_edge(2, 2).unwrap();
        let t = trussness(&g).unwrap();
        assert_eq!(t, vec![3, 3, 3, 2, 2]);
    }

    #[test]
    fn directed_graph_rejected() {
        let g = Graph::new(3, true).unwrap();
        let err = trussness(&g).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn multigraph_rejected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let err = trussness(&g).unwrap_err();
        assert!(matches!(err, IgraphError::Unsupported(_)));
    }

    #[test]
    fn star_graph() {
        let g = make_undirected(5, &[(0, 1), (0, 2), (0, 3), (0, 4)]);
        let t = trussness(&g).unwrap();
        assert!(t.iter().all(|&x| x == 2));
    }

    #[test]
    fn two_triangles_sharing_edge() {
        let g = make_undirected(4, &[(0, 1), (0, 2), (1, 2), (0, 3), (1, 3)]);
        let t = trussness(&g).unwrap();
        assert!(
            t.iter().all(|&x| x == 3),
            "two triangles sharing edge: {t:?}"
        );
    }

    #[test]
    fn single_edge() {
        let g = make_undirected(2, &[(0, 1)]);
        let t = trussness(&g).unwrap();
        assert_eq!(t, vec![2]);
    }

    #[test]
    fn disconnected_triangles() {
        let g = make_undirected(6, &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5)]);
        let t = trussness(&g).unwrap();
        assert!(t.iter().all(|&x| x == 3));
    }

    #[test]
    fn bridge_between_triangles() {
        let g = make_undirected(6, &[(0, 1), (0, 2), (1, 2), (2, 3), (3, 4), (3, 5), (4, 5)]);
        let t = trussness(&g).unwrap();
        assert_eq!(t[0], 3); // 0-1
        assert_eq!(t[1], 3); // 0-2
        assert_eq!(t[2], 3); // 1-2
        assert_eq!(t[3], 2); // 2-3 (bridge)
        assert_eq!(t[4], 3); // 3-4
        assert_eq!(t[5], 3); // 3-5
        assert_eq!(t[6], 3); // 4-5
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_simple_undirected(max_n: u32, max_e: usize) -> impl Strategy<Value = Graph> {
        (3..=max_n).prop_flat_map(move |n| {
            let possible = ((n as u64) * (n as u64 - 1) / 2) as usize;
            let cap = max_e.min(possible);
            proptest::collection::hash_set((0..n, 0..n), 0..=cap).prop_map(move |pairs| {
                let mut g = Graph::with_vertices(n);
                for (a, b) in pairs {
                    if a != b {
                        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
                        if g.find_eid(lo, hi).ok().flatten().is_none() {
                            g.add_edge(lo, hi).unwrap();
                        }
                    }
                }
                g
            })
        })
    }

    proptest! {
        #[test]
        fn trussness_at_least_two(g in arb_simple_undirected(20, 40)) {
            let t = trussness(&g).unwrap();
            for &val in &t {
                prop_assert!(val >= 2, "trussness must be >= 2, got {val}");
            }
        }

        #[test]
        fn trussness_k_complete(n in 3u32..=8) {
            let mut g = Graph::with_vertices(n);
            for u in 0..n {
                for v in (u + 1)..n {
                    g.add_edge(u, v).unwrap();
                }
            }
            let t = trussness(&g).unwrap();
            for &val in &t {
                prop_assert_eq!(val, n, "K{} trussness should be {}, got {}", n, n, val);
            }
        }

        #[test]
        fn trussness_bounded_by_graph_size(g in arb_simple_undirected(15, 30)) {
            let t = trussness(&g).unwrap();
            let n = g.vcount();
            for &val in &t {
                prop_assert!(val <= n, "trussness {val} exceeds vertex count {n}");
            }
        }
    }
}
