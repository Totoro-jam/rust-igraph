//! Triangles and global transitivity (ALGO-PR-002).
//!
//! Counterparts of:
//! - `igraph_count_triangles()`            from `references/igraph/src/properties/triangles.c:501`
//! - `igraph_transitivity_undirected()`    from `references/igraph/src/properties/triangles.c:615`
//!
//! Both share the same private helper that computes triangles AND
//! connected triples in a single sweep. We port that helper directly:
//!
//! 1. Build simple adjacency lists (no self-loops, no parallel edges).
//! 2. For each vertex `v1`, scan its neighbours `v2 < v1` (acyclic
//!    orientation trick — counts each undirected triangle exactly
//!    once). Mark each such `v2` with the sentinel `v1 + 1`.
//! 3. For each `v2 < v1`, scan `v2`'s neighbours `v3 < v2`. If
//!    `mark[v3] == v1 + 1`, then `(v1, v2, v3)` is a triangle.
//! 4. Connected triples per vertex `v` = `C(deg, 2) = deg*(deg-1)/2`.
//! 5. Global transitivity = `3 * triangles / triples` (or `None` if no
//!    triples).
//!
//! Phase-1 minimal slice — `transitivity_local_undirected` (per-vertex)
//! and weighted Barrat's variant ship in PR-002b/c.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Count the number of triangles in `graph`. Edge directions, parallel
/// edges, and self-loops are ignored.
///
/// Counterpart of `igraph_count_triangles()`. Returns the number of
/// triangles as a `u64` (fits comfortably for graphs up to about
/// `n = 600 000` cliques).
pub fn count_triangles(graph: &Graph) -> IgraphResult<u64> {
    let (triangles, _) = triangles_and_triples(graph)?;
    Ok(triangles)
}

/// Global transitivity (clustering coefficient) of `graph` —
/// `3 * triangles / connected_triples`. Returns `None` when there are
/// no connected triples (matches upstream's `IGRAPH_TRANSITIVITY_NAN`
/// mode); use `.unwrap_or(0.0)` for the `IGRAPH_TRANSITIVITY_ZERO`
/// behaviour.
///
/// Edge directions, parallel edges, and self-loops are ignored.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, transitivity_undirected};
///
/// // K4: every triple is a triangle → transitivity 1.0.
/// let mut g = Graph::with_vertices(4);
/// for u in 0..4u32 {
///     for v in (u + 1)..4 {
///         g.add_edge(u, v).unwrap();
///     }
/// }
/// let t = transitivity_undirected(&g).unwrap();
/// assert_eq!(t, Some(1.0));
///
/// // 4-cycle: 4 connected triples (one per vertex) but no triangles.
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 { g.add_edge(i, (i + 1) % 4).unwrap(); }
/// let t = transitivity_undirected(&g).unwrap();
/// assert_eq!(t, Some(0.0));
/// ```
pub fn transitivity_undirected(graph: &Graph) -> IgraphResult<Option<f64>> {
    let (triangles, triples) = triangles_and_triples(graph)?;
    if triples == 0 {
        return Ok(None);
    }
    // f64 mantissa is 52 bits. triangles + triples both fit exactly for any
    // graph that survives the u32 vertex-id encoding (n ≤ 2^32 → triples
    // ≤ n^2/2 ≤ 2^63, but in practice n ≤ ~2^25 keeps the cast lossless).
    #[allow(clippy::cast_precision_loss)]
    let t = (triangles as f64) * 3.0 / (triples as f64);
    Ok(Some(t))
}

fn triangles_and_triples(graph: &Graph) -> IgraphResult<(u64, u64)> {
    let n = graph.vcount();
    let n_us = n as usize;

    // Build simple adjacency lists (no loops, no multi). We sort and
    // dedupe each list so the `if v2 >= v1: break` early-out works.
    let mut adj: Vec<Vec<VertexId>> = Vec::with_capacity(n_us);
    for v in 0..n {
        let raw = graph.neighbors(v)?;
        let mut simple: Vec<VertexId> = raw.into_iter().filter(|&u| u != v).collect();
        simple.sort_unstable();
        simple.dedup();
        adj.push(simple);
    }

    // mark[v3] == v1+1 means "v3 is a neighbour of v1 that we've seen
    // in the current outer iteration". Sentinel 0 = unmarked.
    let mut mark: Vec<u32> = vec![0; n_us];
    let mut triangles: u64 = 0;
    let mut triples: u64 = 0;

    for v1 in 0..n {
        let nei1 = &adj[v1 as usize];
        let d1 = nei1.len();
        if d1 < 2 {
            continue;
        }
        // Each pair of neighbours of v1 forms one connected triple.
        triples = triples.saturating_add((d1 as u64) * ((d1 - 1) as u64) / 2);

        // Mark v1's lower-id neighbours.
        let v1_mark = v1
            .checked_add(1)
            .ok_or(IgraphError::Internal("vertex id overflow"))?;
        for &v2 in nei1 {
            if v2 >= v1 {
                break;
            }
            mark[v2 as usize] = v1_mark;
        }

        // For each lower-id neighbour v2, scan v2's lower-id neighbours.
        for &v2 in nei1 {
            if v2 >= v1 {
                break;
            }
            let nei2 = &adj[v2 as usize];
            for &v3 in nei2 {
                if v3 >= v2 {
                    break;
                }
                if mark[v3 as usize] == v1_mark {
                    triangles = triangles.saturating_add(1);
                }
            }
        }
    }

    Ok((triangles, triples))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_has_no_triangles_or_transitivity() {
        let g = Graph::with_vertices(0);
        assert_eq!(count_triangles(&g).unwrap(), 0);
        assert_eq!(transitivity_undirected(&g).unwrap(), None);
    }

    #[test]
    fn isolated_vertices_give_no_triples() {
        let g = Graph::with_vertices(5);
        assert_eq!(count_triangles(&g).unwrap(), 0);
        assert_eq!(transitivity_undirected(&g).unwrap(), None);
    }

    #[test]
    fn triangle_count_is_one_transitivity_is_one() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(count_triangles(&g).unwrap(), 1);
        assert_eq!(transitivity_undirected(&g).unwrap(), Some(1.0));
    }

    #[test]
    fn k4_has_4_triangles_transitivity_1() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert_eq!(count_triangles(&g).unwrap(), 4);
        assert_eq!(transitivity_undirected(&g).unwrap(), Some(1.0));
    }

    #[test]
    fn cycle_4_has_no_triangles_transitivity_zero() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        assert_eq!(count_triangles(&g).unwrap(), 0);
        assert_eq!(transitivity_undirected(&g).unwrap(), Some(0.0));
    }

    #[test]
    fn star_has_no_triangles_transitivity_zero() {
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        assert_eq!(count_triangles(&g).unwrap(), 0);
        // Centre has C(3,2) = 3 connected triples, no triangles → 0.0.
        assert_eq!(transitivity_undirected(&g).unwrap(), Some(0.0));
    }

    #[test]
    fn path_has_one_triple_no_triangle() {
        // Path 0-1-2: vertex 1 has 2 neighbours → 1 triple, 0 triangles.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(count_triangles(&g).unwrap(), 0);
        assert_eq!(transitivity_undirected(&g).unwrap(), Some(0.0));
    }

    #[test]
    fn self_loop_is_ignored() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap(); // self-loop
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // The self-loop must not change the triangle/triple count.
        assert_eq!(count_triangles(&g).unwrap(), 1);
        assert_eq!(transitivity_undirected(&g).unwrap(), Some(1.0));
    }

    #[test]
    fn parallel_edges_are_ignored() {
        // Triangle with one duplicated edge.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap(); // duplicate
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(count_triangles(&g).unwrap(), 1);
        assert_eq!(transitivity_undirected(&g).unwrap(), Some(1.0));
    }

    #[test]
    fn two_disjoint_triangles_count_as_two() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        assert_eq!(count_triangles(&g).unwrap(), 2);
        // Each component contributes 3 triples → 6 triples, 2 triangles → 1.0.
        assert_eq!(transitivity_undirected(&g).unwrap(), Some(1.0));
    }

    #[test]
    fn diamond_k4_minus_edge_transitivity_below_one() {
        // K4 minus the edge (0, 3). Triangles: (0,1,2), (1,2,3) → 2.
        // Triples: deg(0)=2 → 1; deg(1)=3 → 3; deg(2)=3 → 3; deg(3)=2 → 1.
        // Total triples = 8, transitivity = 6/8 = 0.75.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(count_triangles(&g).unwrap(), 2);
        assert_eq!(transitivity_undirected(&g).unwrap(), Some(0.75));
    }
}
