//! Chordality algorithms.
//!
//! ALGO-CL-002: `maximum_cardinality_search` — Tarjan-Yannakakis 1984
//! maximum cardinality search ordering, and `is_chordal` — chordal
//! graph test with optional fill-in computation.
//!
//! Reference: `references/igraph/src/misc/chordality.c` (477 lines).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::module_name_repetitions
)]

use crate::core::error::{IgraphError, IgraphResult};
use crate::core::graph::{Graph, VertexId};

/// Result of [`maximum_cardinality_search`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McsResult {
    /// `alpha[v]` is the rank of vertex `v` in the MCS ordering, in `0..n`.
    /// Higher rank means the vertex was visited earlier.
    pub alpha: Vec<u32>,
    /// `alpham1[i]` is the vertex with rank `i`. This is the inverse of
    /// `alpha`: visiting vertices in order `alpham1[n-1], alpham1[n-2], ..., alpham1[0]`
    /// gives the MCS visitation order.
    pub alpham1: Vec<u32>,
}

/// Result of [`is_chordal`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChordalResult {
    /// Whether the graph is chordal.
    pub chordal: bool,
    /// Fill-in edges: pairs `(u, w)` that would need to be added to make
    /// the graph chordal. Empty if the graph is already chordal.
    /// Note: the fill-in may not be minimal.
    pub fill_in: Vec<(u32, u32)>,
}

/// Computes a maximum cardinality search ordering.
///
/// Assigns a rank to each vertex such that visiting vertices in
/// decreasing rank order corresponds to always choosing the vertex
/// with the most already-visited neighbors next. Edge directions are
/// ignored. Self-loops and multi-edges are stripped internally.
///
/// # Returns
///
/// An [`McsResult`] with `alpha` (vertex-to-rank) and `alpham1`
/// (rank-to-vertex, the inverse).
///
/// # Complexity
///
/// O(|V| + |E|) using bucket-list data structures.
///
/// # References
///
/// R. E. Tarjan, M. Yannakakis: "Simple linear-time algorithms to
/// test chordality of graphs, test acyclicity of hypergraphs, and
/// selectively reduce acyclic hypergraphs." SIAM J. Comput. 13,
/// 566--579, 1984.
///
/// # Examples
///
/// ```
/// use rust_igraph::{maximum_cardinality_search, create};
///
/// let g = create(&[(0, 1), (1, 2), (2, 0)], 3, false).unwrap();
/// let mcs = maximum_cardinality_search(&g).unwrap();
/// assert_eq!(mcs.alpha.len(), 3);
/// assert_eq!(mcs.alpham1.len(), 3);
/// // alpham1 is the inverse of alpha
/// for (v, &rank) in mcs.alpha.iter().enumerate() {
///     assert_eq!(mcs.alpham1[rank as usize], v as u32);
/// }
/// ```
pub fn maximum_cardinality_search(graph: &Graph) -> IgraphResult<McsResult> {
    let n = graph.vcount() as usize;

    if n == 0 {
        return Ok(McsResult {
            alpha: vec![],
            alpham1: vec![],
        });
    }

    // Build simple adjacency (no loops, no multi-edges, undirected).
    let adj = build_simple_adj(graph)?;

    let mut alpha = vec![0u32; n];
    let mut alpham1 = vec![0u32; n];

    // Doubly-linked bucket lists indexed by "number of already-visited
    // neighbours". head[s] = 1-based first vertex in bucket s (0 = empty).
    // next[v] / prev[v] are 1-based pointers (0 = end / no predecessor).
    let mut size = vec![0i32; n];
    let mut head = vec![0u32; n];
    let mut next = vec![0u32; n];
    let mut prev = vec![0u32; n];

    // Initially all vertices in bucket 0.
    head[0] = 1; // 1-based: vertex 0
    for v in 0..n {
        next[v] = u32::try_from(v + 2).unwrap_or(0);
        prev[v] = v as u32;
    }
    next[n - 1] = 0;

    let mut i = n; // ranks assigned from n-1 down to 0
    let mut j: i32 = 0; // current max bucket

    while i >= 1 {
        // Pick any vertex from bucket j (the highest non-empty bucket).
        let v = (head[j as usize])
            .checked_sub(1)
            .ok_or_else(|| IgraphError::InvalidArgument("MCS: empty bucket unexpectedly".into()))?
            as usize;

        let x = next[v];
        head[j as usize] = x;
        if x != 0 {
            prev[(x - 1) as usize] = 0;
        }

        i -= 1;
        alpha[v] = i as u32;
        alpham1[i] = v as u32;
        size[v] = -1; // mark as visited

        // Update neighbours: move each unvisited neighbour from
        // bucket size[w] to bucket size[w]+1.
        for &w in &adj[v] {
            let wu = w as usize;
            let ws = size[wu];
            if ws < 0 {
                continue;
            }

            // Delete w from its current bucket.
            let nw = next[wu];
            let pw = prev[wu];
            if nw != 0 {
                prev[(nw - 1) as usize] = pw;
            }
            if pw != 0 {
                next[(pw - 1) as usize] = nw;
            } else {
                head[ws as usize] = nw;
            }

            size[wu] += 1;
            let ws_new = size[wu] as usize;

            // Insert w at the head of the new bucket.
            let nw_new = head[ws_new];
            next[wu] = nw_new;
            prev[wu] = 0;
            if nw_new != 0 {
                prev[(nw_new - 1) as usize] = w + 1;
            }
            head[ws_new] = w + 1;
        }

        j += 1;

        // Find the next non-empty bucket at or below j.
        if (j as usize) < n {
            while j >= 0 && head[j as usize] == 0 {
                j -= 1;
            }
        }
    }

    Ok(McsResult { alpha, alpham1 })
}

/// Tests whether a graph is chordal.
///
/// A graph is chordal if every cycle of four or more nodes has a
/// chord (an edge joining two non-adjacent nodes in the cycle).
/// Equivalently, all chordless cycles have at most three nodes.
///
/// Uses the MCS ordering from [`maximum_cardinality_search`] and the
/// zero-fill-in test of Tarjan-Yannakakis 1984. Optionally computes
/// the fill-in edges needed to make the graph chordal (note: the
/// fill-in is not guaranteed to be minimal).
///
/// Edge directions are ignored. Self-loops and multi-edges are
/// stripped internally.
///
/// # Arguments
///
/// * `graph` — the input graph.
/// * `alpha` — optional pre-computed MCS ordering (`alpha[v]` = rank).
///   If `None`, MCS is computed internally.
///
/// # Returns
///
/// A [`ChordalResult`] with `chordal` flag and `fill_in` edges.
///
/// # Complexity
///
/// O(|V| + |E|).
///
/// # Examples
///
/// ```
/// use rust_igraph::{is_chordal, create};
///
/// // Triangle is chordal
/// let g = create(&[(0, 1), (1, 2), (2, 0)], 3, false).unwrap();
/// let r = is_chordal(&g, None).unwrap();
/// assert!(r.chordal);
/// assert!(r.fill_in.is_empty());
///
/// // 4-cycle is not chordal (needs one diagonal)
/// let g4 = create(&[(0, 1), (1, 2), (2, 3), (3, 0)], 4, false).unwrap();
/// let r4 = is_chordal(&g4, None).unwrap();
/// assert!(!r4.chordal);
/// assert!(!r4.fill_in.is_empty());
/// ```
pub fn is_chordal(graph: &Graph, alpha: Option<&[u32]>) -> IgraphResult<ChordalResult> {
    let n = graph.vcount() as usize;

    if n == 0 {
        return Ok(ChordalResult {
            chordal: true,
            fill_in: vec![],
        });
    }

    if let Some(a) = alpha {
        if a.len() != n {
            return Err(IgraphError::InvalidArgument(format!(
                "is_chordal: alpha length {} != vcount {}",
                a.len(),
                n
            )));
        }
        let mut inv = vec![0u32; n];
        for (v, &rank) in a.iter().enumerate() {
            let ri = rank as usize;
            if ri >= n {
                return Err(IgraphError::InvalidArgument(format!(
                    "is_chordal: alpha[{v}] = {rank} out of range [0, {n})"
                )));
            }
            inv[ri] = v as u32;
        }
        is_chordal_impl(graph, n, a, &inv)
    } else {
        let mcs = maximum_cardinality_search(graph)?;
        is_chordal_impl(graph, n, &mcs.alpha, &mcs.alpham1)
    }
}

fn is_chordal_impl(
    graph: &Graph,
    n: usize,
    alpha: &[u32],
    alpham1: &[u32],
) -> IgraphResult<ChordalResult> {
    let adj = build_simple_adj(graph)?;

    let mut f = vec![0u32; n]; // path-compression forest
    let mut index = vec![0u32; n];
    let mut mark = vec![0u32; n]; // mark[v] = w+1 if v is a neighbour of w
    let mut fill_in: Vec<(u32, u32)> = Vec::new();
    let mut chordal = true;

    for (i, &alpham1_i) in alpham1.iter().enumerate() {
        let w = alpham1_i as usize;
        f[w] = w as u32;
        index[w] = i as u32;

        // Mark all neighbours of w.
        for &nei in &adj[w] {
            mark[nei as usize] = (w + 1) as u32;
        }

        // Walk neighbours with alpha < i.
        for &nei in &adj[w] {
            let v = nei as usize;
            if alpha[v] as usize >= i {
                continue;
            }

            let mut x = v;
            while (index[x] as usize) < i {
                index[x] = i as u32;

                if mark[x] != (w + 1) as u32 {
                    chordal = false;
                    fill_in.push((x as u32, w as u32));
                }

                x = f[x] as usize;
            }

            if f[x] == x as u32 {
                f[x] = w as u32;
            }
        }
    }

    Ok(ChordalResult { chordal, fill_in })
}

/// Build simple undirected adjacency lists (no self-loops, no multi-edges).
fn build_simple_adj(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount() as usize;
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];

    let ec = graph.ecount();
    for e in 0..ec {
        let eid = u32::try_from(e)
            .map_err(|_| IgraphError::InvalidArgument("edge count exceeds u32".into()))?;
        let (u, v) = graph.edge(eid)?;
        if u == v {
            continue;
        }
        adj[u as usize].push(v);
        adj[v as usize].push(u);
    }

    // Dedup each list.
    for list in &mut adj {
        list.sort_unstable();
        list.dedup();
    }

    Ok(adj)
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create;

    fn make(n: u32, edges: &[(u32, u32)]) -> Graph {
        create(edges, n, false).expect("make")
    }

    // ---- maximum_cardinality_search ----

    #[test]
    fn test_mcs_empty() {
        let g = make(0, &[]);
        let r = maximum_cardinality_search(&g).expect("ok");
        assert!(r.alpha.is_empty());
        assert!(r.alpham1.is_empty());
    }

    #[test]
    fn test_mcs_singleton() {
        let g = make(1, &[]);
        let r = maximum_cardinality_search(&g).expect("ok");
        assert_eq!(r.alpha, vec![0]);
        assert_eq!(r.alpham1, vec![0]);
    }

    #[test]
    fn test_mcs_triangle() {
        let g = make(3, &[(0, 1), (1, 2), (2, 0)]);
        let r = maximum_cardinality_search(&g).expect("ok");
        assert_eq!(r.alpha.len(), 3);
        assert_eq!(r.alpham1.len(), 3);
        // Inverse property
        for (v, &rank) in r.alpha.iter().enumerate() {
            assert_eq!(r.alpham1[rank as usize], v as u32);
        }
    }

    #[test]
    fn test_mcs_path() {
        let g = make(4, &[(0, 1), (1, 2), (2, 3)]);
        let r = maximum_cardinality_search(&g).expect("ok");
        for (v, &rank) in r.alpha.iter().enumerate() {
            assert_eq!(r.alpham1[rank as usize], v as u32);
        }
    }

    #[test]
    fn test_mcs_disconnected() {
        let g = make(4, &[(0, 1)]);
        let r = maximum_cardinality_search(&g).expect("ok");
        assert_eq!(r.alpha.len(), 4);
        for (v, &rank) in r.alpha.iter().enumerate() {
            assert_eq!(r.alpham1[rank as usize], v as u32);
        }
    }

    #[test]
    fn test_mcs_self_loops() {
        let g = make(3, &[(0, 0), (0, 1), (1, 2), (2, 2)]);
        let r = maximum_cardinality_search(&g).expect("ok");
        assert_eq!(r.alpha.len(), 3);
        for (v, &rank) in r.alpha.iter().enumerate() {
            assert_eq!(r.alpham1[rank as usize], v as u32);
        }
    }

    // ---- is_chordal ----

    #[test]
    fn test_chordal_empty() {
        let g = make(0, &[]);
        let r = is_chordal(&g, None).expect("ok");
        assert!(r.chordal);
        assert!(r.fill_in.is_empty());
    }

    #[test]
    fn test_chordal_singleton() {
        let g = make(1, &[]);
        let r = is_chordal(&g, None).expect("ok");
        assert!(r.chordal);
        assert!(r.fill_in.is_empty());
    }

    #[test]
    fn test_chordal_triangle() {
        let g = make(3, &[(0, 1), (1, 2), (2, 0)]);
        let r = is_chordal(&g, None).expect("ok");
        assert!(r.chordal);
        assert!(r.fill_in.is_empty());
    }

    #[test]
    fn test_chordal_complete_k4() {
        let g = make(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
        let r = is_chordal(&g, None).expect("ok");
        assert!(r.chordal);
        assert!(r.fill_in.is_empty());
    }

    #[test]
    fn test_not_chordal_4cycle() {
        let g = make(4, &[(0, 1), (1, 2), (2, 3), (3, 0)]);
        let r = is_chordal(&g, None).expect("ok");
        assert!(!r.chordal);
        assert!(!r.fill_in.is_empty());
    }

    #[test]
    fn test_not_chordal_5cycle() {
        let g = make(5, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)]);
        let r = is_chordal(&g, None).expect("ok");
        assert!(!r.chordal);
    }

    #[test]
    fn test_chordal_tree() {
        let g = make(5, &[(0, 1), (0, 2), (1, 3), (1, 4)]);
        let r = is_chordal(&g, None).expect("ok");
        assert!(r.chordal);
        assert!(r.fill_in.is_empty());
    }

    #[test]
    fn test_chordal_isolated_vertices() {
        let g = make(5, &[]);
        let r = is_chordal(&g, None).expect("ok");
        assert!(r.chordal);
    }

    #[test]
    fn test_chordal_with_alpha() {
        let g = make(4, &[(0, 1), (1, 2), (2, 3), (3, 0)]);
        let mcs = maximum_cardinality_search(&g).expect("mcs");
        let r = is_chordal(&g, Some(&mcs.alpha)).expect("ok");
        assert!(!r.chordal);
    }

    #[test]
    fn test_chordal_wrong_alpha_size() {
        let g = make(3, &[(0, 1)]);
        assert!(is_chordal(&g, Some(&[0, 1])).is_err());
    }

    #[test]
    fn test_chordal_igraph_lmu_graph() {
        // From igraph test: 6 vertices, edges 0-1, 0-2, 1-1, 1-3, 2-0, 2-0, 2-3, 3-4, 3-4
        let g = make(
            6,
            &[
                (0, 1),
                (0, 2),
                (1, 1),
                (1, 3),
                (2, 0),
                (2, 0),
                (2, 3),
                (3, 4),
                (3, 4),
            ],
        );
        let r = is_chordal(&g, None).expect("ok");
        assert!(!r.chordal);
        // Fill-in should be (3, 0) — the missing chord
        assert!(!r.fill_in.is_empty());
    }

    #[test]
    fn test_chordal_directed_ignored() {
        let g = create(&[(0, 1), (1, 2), (2, 0)], 3, true).expect("directed");
        let r = is_chordal(&g, None).expect("ok");
        assert!(r.chordal);
    }

    #[test]
    fn test_chordal_fan_graph() {
        // Fan: 0 connected to all, plus path 1-2-3-4
        let g = make(5, &[(0, 1), (0, 2), (0, 3), (0, 4), (1, 2), (2, 3), (3, 4)]);
        let r = is_chordal(&g, None).expect("ok");
        assert!(r.chordal);
    }

    #[test]
    fn test_fill_in_makes_chordal() {
        // 4-cycle: add the fill-in edge to make it chordal
        let g = make(4, &[(0, 1), (1, 2), (2, 3), (3, 0)]);
        let r = is_chordal(&g, None).expect("ok");
        assert!(!r.chordal);

        // Add fill-in edges to the graph
        let mut all_edges: Vec<(u32, u32)> = vec![(0, 1), (1, 2), (2, 3), (3, 0)];
        all_edges.extend(r.fill_in.iter());
        let g2 = make(4, &all_edges);
        let r2 = is_chordal(&g2, None).expect("ok");
        assert!(r2.chordal);
    }
}

// =================================================================
// Proptest
// =================================================================

#[cfg(test)]
#[cfg(feature = "proptest-harness")]
mod proptests {
    use super::*;
    use crate::create;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_mcs_inverse_property(n in 1u32..30, edge_count in 0usize..60) {
            let mut edges = Vec::new();
            let mut rng_state = 0x1234_5678u64;
            for _ in 0..edge_count {
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let u = (rng_state >> 32) as u32 % n;
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let v = (rng_state >> 32) as u32 % n;
                if u != v {
                    edges.push((u, v));
                }
            }
            let g = create(&edges, n, false).expect("graph");
            let r = maximum_cardinality_search(&g).expect("mcs");
            prop_assert_eq!(r.alpha.len(), n as usize);
            prop_assert_eq!(r.alpham1.len(), n as usize);
            for (v, &rank) in r.alpha.iter().enumerate() {
                prop_assert!((rank as usize) < n as usize, "rank out of range");
                prop_assert_eq!(r.alpham1[rank as usize], v as u32, "inverse mismatch");
            }
        }

        #[test]
        fn prop_mcs_ranks_are_permutation(n in 1u32..30, edge_count in 0usize..60) {
            let mut edges = Vec::new();
            let mut rng_state = 0xABCD_EF01u64;
            for _ in 0..edge_count {
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let u = (rng_state >> 32) as u32 % n;
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let v = (rng_state >> 32) as u32 % n;
                if u != v {
                    edges.push((u, v));
                }
            }
            let g = create(&edges, n, false).expect("graph");
            let r = maximum_cardinality_search(&g).expect("mcs");
            let mut sorted_ranks: Vec<u32> = r.alpha.clone();
            sorted_ranks.sort_unstable();
            let expected: Vec<u32> = (0..n).collect();
            prop_assert_eq!(sorted_ranks, expected, "ranks should be a permutation of 0..n");
        }

        #[test]
        fn prop_fill_in_makes_chordal(n in 3u32..20, edge_count in 3usize..40) {
            let mut edges = Vec::new();
            let mut rng_state = 0xDEAD_BEEFu64;
            for _ in 0..edge_count {
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let u = (rng_state >> 32) as u32 % n;
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let v = (rng_state >> 32) as u32 % n;
                if u != v {
                    edges.push((u, v));
                }
            }
            let g = create(&edges, n, false).expect("graph");
            let r = is_chordal(&g, None).expect("chordal");
            if !r.chordal {
                let mut all_edges = edges.clone();
                all_edges.extend(r.fill_in.iter());
                let g2 = create(&all_edges, n, false).expect("graph2");
                let r2 = is_chordal(&g2, None).expect("chordal2");
                prop_assert!(r2.chordal, "adding fill-in should make graph chordal");
            }
        }

        #[test]
        fn prop_tree_is_chordal(n in 2u32..30) {
            // Build a random tree (parent = random vertex < v)
            let mut edges = Vec::new();
            let mut rng_state = 0xCAFE_BABEu64;
            for v in 1..n {
                rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let parent = (rng_state >> 32) as u32 % v;
                edges.push((parent, v));
            }
            let g = create(&edges, n, false).expect("tree");
            let r = is_chordal(&g, None).expect("chordal");
            prop_assert!(r.chordal, "every tree is chordal");
            prop_assert!(r.fill_in.is_empty(), "tree needs no fill-in");
        }

        #[test]
        fn prop_complete_graph_is_chordal(n in 2u32..15) {
            let mut edges = Vec::new();
            for u in 0..n {
                for v in (u + 1)..n {
                    edges.push((u, v));
                }
            }
            let g = create(&edges, n, false).expect("complete");
            let r = is_chordal(&g, None).expect("chordal");
            prop_assert!(r.chordal, "complete graph is chordal");
        }
    }
}
