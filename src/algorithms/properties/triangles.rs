//! Triangles and global transitivity (ALGO-PR-002 / 002b / 002c / 002d).
//!
//! Counterparts of:
//! - `igraph_count_triangles()`             from `references/igraph/src/properties/triangles.c:501`
//! - `igraph_count_adjacent_triangles()`    from `references/igraph/src/properties/triangles.c:522`
//! - `igraph_transitivity_undirected()`     from `references/igraph/src/properties/triangles.c:615`
//! - `igraph_transitivity_local_undirected()` from `references/igraph/src/properties/triangles.c:369`
//! - `igraph_transitivity_barrat()`         from `references/igraph/src/properties/triangles.c:874`
//!
//! The first three share a private helper that computes triangles AND
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
//! Barrat's weighted local variant uses a different inner loop driven
//! by the `transitivity_barrat1` helper at `triangles.c:632`.

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

/// Per-vertex adjacent-triangle count. Entry `i` is the number of
/// triangles vertex `i` participates in. Parallel edges and self-loops
/// are ignored — the simple graph induced by the OUT-neighbour view is
/// used (consistent with [`count_triangles`]). For directed graphs that
/// is not yet the underlying-undirected projection that upstream uses;
/// see ALGO-PR-002 for the deferred fix.
///
/// Counterpart of `igraph_count_adjacent_triangles()` from
/// `references/igraph/src/properties/triangles.c:522`. Always runs the
/// "all vertices" path (`adjacent_triangles4`); the C version's
/// per-vertex `adjacent_triangles1` short-circuit is moot when querying
/// every vertex.
///
/// Invariants: `result.iter().sum::<u64>() == 3 * count_triangles(g)`,
/// and each entry is bounded by `C(deg, 2)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, count_adjacent_triangles};
///
/// // K4: every vertex sees the 3 triangles meeting at it.
/// let mut g = Graph::with_vertices(4);
/// for u in 0..4u32 {
///     for v in (u + 1)..4 {
///         g.add_edge(u, v).unwrap();
///     }
/// }
/// assert_eq!(count_adjacent_triangles(&g).unwrap(), vec![3, 3, 3, 3]);
///
/// // Star centre meets 0 triangles; leaves all 0 as well.
/// let mut g = Graph::with_vertices(4);
/// for v in 1..4 { g.add_edge(0, v).unwrap(); }
/// assert_eq!(count_adjacent_triangles(&g).unwrap(), vec![0, 0, 0, 0]);
/// ```
pub fn count_adjacent_triangles(graph: &Graph) -> IgraphResult<Vec<u64>> {
    let (per_vertex_triangles, _) = per_vertex_triangle_stats(graph)?;
    Ok(per_vertex_triangles)
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

/// Local transitivity (clustering coefficient) per vertex.
///
/// For vertex `v` with simple-degree `d` and `t` adjacent triangles,
/// returns `2t / (d * (d - 1))`. `None` when `d < 2` (no closed triple
/// possible — upstream's `IGRAPH_TRANSITIVITY_NAN` mode); use
/// `result.iter().map(|o| o.unwrap_or(0.0))` for `IGRAPH_TRANSITIVITY_ZERO`
/// behaviour.
///
/// Counterpart of `igraph_transitivity_local_undirected()` from
/// `references/igraph/src/properties/triangles.c:369`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, transitivity_local_undirected};
///
/// // Triangle: every vertex has clustering 1.0.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert_eq!(
///     transitivity_local_undirected(&g).unwrap(),
///     vec![Some(1.0), Some(1.0), Some(1.0)],
/// );
///
/// // Star centre: 3 neighbours but 0 triangles → 0.0.
/// // Leaves: degree 1 → None (no closed triple possible).
/// let mut g = Graph::with_vertices(4);
/// for v in 1..4 { g.add_edge(0, v).unwrap(); }
/// let r = transitivity_local_undirected(&g).unwrap();
/// assert_eq!(r, vec![Some(0.0), None, None, None]);
/// ```
pub fn transitivity_local_undirected(graph: &Graph) -> IgraphResult<Vec<Option<f64>>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let (per_vertex_triangles, simple_degrees) = per_vertex_triangle_stats(graph)?;
    let mut out: Vec<Option<f64>> = Vec::with_capacity(n_us);
    for v in 0..n_us {
        let d = simple_degrees[v];
        if d < 2 {
            out.push(None);
            continue;
        }
        let t = per_vertex_triangles[v];
        // d ≤ n ≤ 2^32; t ≤ n^2; both fit in f64 exactly for any
        // graph that survives u32 vertex ids in practice.
        #[allow(clippy::cast_precision_loss)]
        let val = 2.0 * (t as f64) / ((d as f64) * ((d - 1) as f64));
        out.push(Some(val));
    }
    Ok(out)
}

/// How to handle vertices with degree < 2 when averaging local transitivity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitivityMode {
    /// Exclude low-degree vertices from the average; return NaN if none qualify.
    Nan,
    /// Treat low-degree vertices as having transitivity 0.
    Zero,
}

/// Average local transitivity (clustering coefficient).
///
/// Computes the local transitivity for each vertex and returns the mean.
///
/// - `mode`: how to handle vertices with degree < 2.
///   - [`TransitivityMode::Nan`]: exclude them (result is NaN if no vertex qualifies).
///   - [`TransitivityMode::Zero`]: include them with transitivity 0.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, transitivity_avglocal_undirected, TransitivityMode};
///
/// // Triangle: all vertices have local transitivity 1.0
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 2).unwrap();
/// let avg = transitivity_avglocal_undirected(&g, TransitivityMode::Nan).unwrap();
/// assert!((avg - 1.0).abs() < 1e-10);
/// ```
pub fn transitivity_avglocal_undirected(
    graph: &Graph,
    mode: TransitivityMode,
) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(if mode == TransitivityMode::Zero {
            0.0
        } else {
            f64::NAN
        });
    }

    let local = transitivity_local_undirected(graph)?;
    let mut sum = 0.0_f64;
    let mut count = 0_u64;

    for val in &local {
        match val {
            Some(t) => {
                sum += t;
                count += 1;
            }
            None => {
                if mode == TransitivityMode::Zero {
                    count += 1;
                }
            }
        }
    }

    if count == 0 {
        Ok(f64::NAN)
    } else {
        #[allow(clippy::cast_precision_loss)]
        Ok(sum / count as f64)
    }
}

/// Adjacent-triangle count per vertex (length `vcount`) plus the simple
/// degree (no loops, no multi) of each vertex.
fn per_vertex_triangle_stats(graph: &Graph) -> IgraphResult<(Vec<u64>, Vec<u64>)> {
    let n = graph.vcount();
    let n_us = n as usize;

    let mut adj: Vec<Vec<VertexId>> = Vec::with_capacity(n_us);
    for v in 0..n {
        let raw = graph.neighbors(v)?;
        let mut simple: Vec<VertexId> = raw.into_iter().filter(|&u| u != v).collect();
        simple.sort_unstable();
        simple.dedup();
        adj.push(simple);
    }

    let degrees: Vec<u64> = adj.iter().map(|nei| nei.len() as u64).collect();
    let mut tri_counts: Vec<u64> = vec![0; n_us];
    let mut mark: Vec<u32> = vec![0; n_us];

    for v1 in 0..n {
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
            let nei2 = &adj[v2 as usize];
            for &v3 in nei2 {
                if v3 >= v2 {
                    break;
                }
                if mark[v3 as usize] == v1_mark {
                    tri_counts[v1 as usize] += 1;
                    tri_counts[v2 as usize] += 1;
                    tri_counts[v3 as usize] += 1;
                }
            }
        }
    }

    Ok((tri_counts, degrees))
}

/// Barrat's weighted local transitivity, per vertex.
///
/// For each vertex `v`, returns
/// `(Σ over triangles (v, u, w) (w(v,u) + w(v,w))) /
///  (s_v * (deg_v - 1))`,
/// where `s_v = Σ_{u ∼ v} w(v, u)` is `v`'s strength and `deg_v` is
/// `v`'s simple degree. Returns `None` for vertices with `triples = 0`
/// (degree < 2 or strength 0) — matches upstream's
/// `IGRAPH_TRANSITIVITY_NAN` mode.
///
/// Counterpart of `igraph_transitivity_barrat()` from
/// `references/igraph/src/properties/triangles.c:874`. The graph must
/// be simple (no multi-edges, no self-loops); otherwise this returns
/// `IgraphError::Internal`. Edge directions are ignored — directed
/// graphs are currently rejected (Phase-1 minimal slice).
///
/// `weights` must have length `graph.ecount()` and contain only finite,
/// non-negative values.
///
/// Reference: Barrat, Barthélemy, Pastor-Satorras, Vespignani (2004),
/// "The architecture of complex weighted networks", PNAS 101 3747,
/// equation (5).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, transitivity_barrat};
///
/// // Triangle 0-1-2 with all unit weights — every vertex sees the
/// // single triangle, so weighted transitivity equals the unweighted
/// // value of 1.0.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let r = transitivity_barrat(&g, &[1.0, 1.0, 1.0]).unwrap();
/// assert_eq!(r, vec![Some(1.0), Some(1.0), Some(1.0)]);
/// ```
pub fn transitivity_barrat(graph: &Graph, weights: &[f64]) -> IgraphResult<Vec<Option<f64>>> {
    if graph.is_directed() {
        return Err(IgraphError::Internal(
            "transitivity_barrat: directed graphs not supported in Phase-1 minimal slice",
        ));
    }
    let n = graph.vcount();
    let n_us = n as usize;
    let m = graph.ecount();
    if weights.len() != m {
        return Err(IgraphError::Internal(
            "weights length does not match ecount in transitivity_barrat",
        ));
    }
    for &w in weights {
        if !w.is_finite() || w < 0.0 {
            return Err(IgraphError::Internal(
                "weights must be finite and non-negative in transitivity_barrat",
            ));
        }
    }
    // Upstream rejects multi-edges; loops yield undefined output, so
    // require simple graphs here — matches the documented contract
    // ("the function does not work for non-simple graphs").
    if !crate::algorithms::properties::is_simple::is_simple(graph)? {
        return Err(IgraphError::Internal(
            "transitivity_barrat requires a simple graph (no multi-edges, no self-loops)",
        ));
    }

    if n_us == 0 {
        return Ok(Vec::new());
    }

    // Sentinel-based neighbour marker: `nei_mark[u] == i+1` means u is
    // a neighbour of the i-th vertex being processed. Avoids reset cost
    // between iterations. `actw[u]` records the weight of the edge
    // from the current vertex to u.
    let mut nei_mark: Vec<u64> = vec![0; n_us];
    let mut actw: Vec<f64> = vec![0.0; n_us];
    let mut out: Vec<Option<f64>> = Vec::with_capacity(n_us);

    for v in 0..n {
        let i_mark = u64::from(v) + 1;
        let inc_v = graph.incident(v)?;
        let mut strength = 0.0_f64;
        for &e in &inc_v {
            let u = graph.edge_other(e, v)?;
            nei_mark[u as usize] = i_mark;
            actw[u as usize] = weights[e as usize];
            strength += weights[e as usize];
        }
        let deg_v = inc_v.len();
        if deg_v < 2 {
            out.push(None);
            continue;
        }
        // triples = strength_v * (deg_v - 1). For a simple graph, deg_v
        // is the simple degree, matching upstream.
        #[allow(clippy::cast_precision_loss)]
        let triples = strength * (deg_v as f64 - 1.0);
        let mut triangles_w = 0.0_f64;

        for &e1 in &inc_v {
            let u = graph.edge_other(e1, v)?;
            let w1 = weights[e1 as usize];
            let inc_u = graph.incident(u)?;
            for &e2 in &inc_u {
                let u2 = graph.edge_other(e2, u)?;
                if nei_mark[u2 as usize] == i_mark {
                    triangles_w += f64::midpoint(actw[u2 as usize], w1);
                }
            }
        }

        if triples > 0.0 {
            out.push(Some(triangles_w / triples));
        } else {
            out.push(None);
        }
    }

    Ok(out)
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
    fn local_transitivity_triangle_is_all_ones() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(
            transitivity_local_undirected(&g).unwrap(),
            vec![Some(1.0), Some(1.0), Some(1.0)]
        );
    }

    #[test]
    fn local_transitivity_star_centre_zero_leaves_none() {
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        assert_eq!(
            transitivity_local_undirected(&g).unwrap(),
            vec![Some(0.0), None, None, None]
        );
    }

    #[test]
    fn local_transitivity_isolated_vertices_all_none() {
        let g = Graph::with_vertices(3);
        assert_eq!(
            transitivity_local_undirected(&g).unwrap(),
            vec![None, None, None]
        );
    }

    #[test]
    fn local_transitivity_diamond_per_vertex() {
        // K4 minus edge (0,3). Adjacent triangles per vertex: 0→1, 1→2, 2→2, 3→1.
        // Simple degrees: 0→2, 1→3, 2→3, 3→2.
        // Expected: 2*1/(2*1)=1.0; 2*2/(3*2)=2/3; 2*2/(3*2)=2/3; 2*1/(2*1)=1.0.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = transitivity_local_undirected(&g).unwrap();
        assert_eq!(r[0], Some(1.0));
        assert_eq!(r[3], Some(1.0));
        // 2/3 isn't exactly representable; compare via approximate equality
        // with f64 epsilon (matches python-igraph's exact computation).
        let two_thirds = 2.0_f64 / 3.0;
        assert_eq!(r[1], Some(two_thirds));
        assert_eq!(r[2], Some(two_thirds));
    }

    #[test]
    fn barrat_unit_weights_match_unweighted_on_k4() {
        // K4: every triple is a triangle → local clustering = 1.0 per
        // vertex. Barrat with unit weights must reduce to that value.
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let r = transitivity_barrat(&g, &[1.0; 6]).unwrap();
        assert_eq!(r, vec![Some(1.0); 4]);
    }

    #[test]
    fn barrat_triangle_unit_weights_all_ones() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let r = transitivity_barrat(&g, &[1.0; 3]).unwrap();
        assert_eq!(r, vec![Some(1.0), Some(1.0), Some(1.0)]);
    }

    #[test]
    fn barrat_triangle_unequal_weights_hand_check() {
        // Triangle 0-1-2 with edges (0,1)=1, (1,2)=2, (2,0)=4.
        // strength: s_0=1+4=5, s_1=1+2=3, s_2=2+4=6. degrees=2 each.
        // Vertex 0: triples = 5*1 = 5;
        //   triangle (0,1,2): w(0,1)=1, w(0,2)=4. Sum = 1+4 = 5.
        //   Barrat = 5/5 = 1.0.
        // Vertex 1: triples = 3; triangle sum = w(1,0)+w(1,2)=1+2=3 → 1.0.
        // Vertex 2: triples = 6; sum = w(2,0)+w(2,1)=4+2=6 → 1.0.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let r = transitivity_barrat(&g, &[1.0, 2.0, 4.0]).unwrap();
        assert_eq!(r, vec![Some(1.0), Some(1.0), Some(1.0)]);
    }

    #[test]
    fn barrat_path_no_triangles_yields_zero_inner_none_endpoints() {
        // Path 0-1-2: vertex 1 has 2 neighbours, no triangle → 0.0.
        // Vertices 0, 2 have degree 1 → triples = 0 → None.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = transitivity_barrat(&g, &[1.0, 1.0]).unwrap();
        assert_eq!(r, vec![None, Some(0.0), None]);
    }

    #[test]
    fn barrat_isolated_vertex_yields_none() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let r = transitivity_barrat(&g, &[1.0]).unwrap();
        assert_eq!(r, vec![None, None, None]);
    }

    #[test]
    fn barrat_empty_graph_empty_result() {
        let g = Graph::with_vertices(0);
        let r = transitivity_barrat(&g, &[]).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn barrat_diamond_unit_weights() {
        // K4 minus edge (0,3): same as unweighted local with unit weights.
        // Expected: 1.0, 2/3, 2/3, 1.0.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = transitivity_barrat(&g, &[1.0; 5]).unwrap();
        let two_thirds = 2.0_f64 / 3.0;
        assert_eq!(r[0], Some(1.0));
        assert_eq!(r[3], Some(1.0));
        match (r[1], r[2]) {
            (Some(a), Some(b)) => {
                assert!((a - two_thirds).abs() < 1e-12);
                assert!((b - two_thirds).abs() < 1e-12);
            }
            _ => panic!("middle vertices must be Some"),
        }
    }

    #[test]
    fn barrat_invalid_weight_length_errors() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(transitivity_barrat(&g, &[1.0]).is_err());
    }

    #[test]
    fn barrat_negative_weight_errors() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(transitivity_barrat(&g, &[1.0, -1.0]).is_err());
    }

    #[test]
    fn barrat_self_loop_rejected() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(transitivity_barrat(&g, &[1.0, 1.0]).is_err());
    }

    #[test]
    fn barrat_multi_edge_rejected() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(transitivity_barrat(&g, &[1.0, 1.0]).is_err());
    }

    #[test]
    fn barrat_directed_rejected() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(transitivity_barrat(&g, &[1.0]).is_err());
    }

    // ---------- count_adjacent_triangles (PR-002d) ----------

    #[test]
    fn adjacent_triangles_empty_graph() {
        let g = Graph::with_vertices(0);
        assert_eq!(count_adjacent_triangles(&g).unwrap(), Vec::<u64>::new());
    }

    #[test]
    fn adjacent_triangles_isolated_vertices() {
        let g = Graph::with_vertices(5);
        assert_eq!(count_adjacent_triangles(&g).unwrap(), vec![0; 5]);
    }

    #[test]
    fn adjacent_triangles_single_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(count_adjacent_triangles(&g).unwrap(), vec![1, 1, 1]);
    }

    #[test]
    fn adjacent_triangles_k4_each_vertex_sees_3() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        // K4 has 4 triangles; each vertex is in 3 of them.
        assert_eq!(count_adjacent_triangles(&g).unwrap(), vec![3, 3, 3, 3]);
    }

    #[test]
    fn adjacent_triangles_diamond_k4_minus_edge() {
        // Triangles (0,1,2) and (1,2,3); deg-by-vertex {0,3} see one
        // triangle, deg-by-vertex {1,2} see both.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(count_adjacent_triangles(&g).unwrap(), vec![1, 2, 2, 1]);
    }

    #[test]
    fn adjacent_triangles_star_all_zero() {
        let mut g = Graph::with_vertices(5);
        for v in 1..5u32 {
            g.add_edge(0, v).unwrap();
        }
        assert_eq!(count_adjacent_triangles(&g).unwrap(), vec![0; 5]);
    }

    #[test]
    fn adjacent_triangles_self_loops_ignored() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(1, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // Self-loops don't contribute; the (0,1,2) triangle remains.
        assert_eq!(count_adjacent_triangles(&g).unwrap(), vec![1, 1, 1]);
    }

    #[test]
    fn adjacent_triangles_parallel_edges_ignored() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(count_adjacent_triangles(&g).unwrap(), vec![1, 1, 1]);
    }

    #[test]
    fn adjacent_triangles_two_disjoint_triangles() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        assert_eq!(
            count_adjacent_triangles(&g).unwrap(),
            vec![1, 1, 1, 1, 1, 1]
        );
    }

    #[test]
    fn adjacent_triangles_sum_equals_three_times_count_triangles() {
        // K4: count_triangles=4 → sum should equal 12.
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let per_vertex = count_adjacent_triangles(&g).unwrap();
        let total = count_triangles(&g).unwrap();
        assert_eq!(per_vertex.iter().sum::<u64>(), 3 * total);
    }

    #[test]
    fn adjacent_triangles_consistent_with_local_transitivity() {
        // For a triangle, each vertex has one adjacent triangle and
        // simple-degree 2 → local transitivity 2*1/(2*1) = 1.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let adj = count_adjacent_triangles(&g).unwrap();
        let local = transitivity_local_undirected(&g).unwrap();
        for v in 0..3 {
            assert_eq!(adj[v], 1);
            assert_eq!(local[v], Some(1.0));
        }
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
