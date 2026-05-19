//! Eccentricity / radius / diameter (ALGO-SP-020 + SP-021abc mode-aware).
//!
//! Counterpart of:
//! - `igraph_eccentricity()` from `references/igraph/src/paths/distances.c:257`
//! - `igraph_radius()`       from `references/igraph/src/paths/distances.c:345`
//! - `igraph_diameter()`     from `references/igraph/src/paths/shortest_paths.c:1259`
//!
//! All three are BFS-based on unweighted graphs and share the same
//! "distances from each vertex" inner loop. The Phase-1 SP-020 minimal
//! slice ships the unweighted, undirected (or `IGRAPH_OUT` for directed)
//! variants. SP-021abc adds the mode-aware `*_with_mode` family — the
//! `mode` parameter selects which adjacency the BFS follows on directed
//! graphs (`Out` / `In` / `All`); for undirected graphs every mode
//! reduces to `All` (every edge is bidirectional).
//!
//! Conventions (matching upstream):
//! - **Eccentricity** of a vertex `v` = max shortest-path distance from
//!   `v` to any reachable vertex; `0` for isolated vertices. Unreachable
//!   vertex pairs are ignored (`unconn = true` semantics).
//! - **Radius** = min eccentricity over all vertices; `None` for n = 0.
//! - **Diameter** = max eccentricity over all vertices; `None` for n = 0.

use std::collections::VecDeque;

use crate::algorithms::paths::distances::distances;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Mode for direction-aware eccentricity / radius / diameter on directed
/// graphs. For undirected graphs every mode reduces to [`EccMode::All`].
///
/// Counterpart of `igraph_neimode_t` in
/// `references/igraph/include/igraph_constants.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EccMode {
    /// Follow outgoing edges (`IGRAPH_OUT`). Default semantics for
    /// upstream `igraph_eccentricity` / `igraph_radius` /
    /// `igraph_diameter` and what the legacy [`eccentricity`] / [`radius`]
    /// / [`diameter`] APIs use.
    Out,
    /// Follow incoming edges (`IGRAPH_IN`).
    In,
    /// Ignore direction — treat every edge as bidirectional
    /// (`IGRAPH_ALL`).
    All,
}

/// BFS distances from `source` following edges in `mode` direction.
/// Pure helper — does not allocate the full distance vector twice and
/// matches [`distances`] when `mode == EccMode::Out`.
fn bfs_distances(graph: &Graph, source: VertexId, mode: EccMode) -> IgraphResult<Vec<Option<u32>>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }
    if source >= n {
        return Err(IgraphError::VertexOutOfRange { id: source, n });
    }

    let n_us = n as usize;
    let mut dist: Vec<Option<u32>> = vec![None; n_us];
    let mut queue: VecDeque<VertexId> = VecDeque::new();
    dist[source as usize] = Some(0);
    queue.push_back(source);

    let directed = graph.is_directed();
    while let Some(v) = queue.pop_front() {
        let next = dist[v as usize]
            .expect("dequeued vertex has been visited")
            .checked_add(1)
            .ok_or(IgraphError::Internal(
                "distance overflow (graph diameter exceeds u32::MAX)",
            ))?;
        let neighbours: Vec<VertexId> = if directed {
            match mode {
                EccMode::Out => graph.out_neighbors_vec(v)?,
                EccMode::In => graph.in_neighbors_vec(v)?,
                EccMode::All => {
                    let mut out = graph.out_neighbors_vec(v)?;
                    out.extend(graph.in_neighbors_vec(v)?);
                    out
                }
            }
        } else {
            // Undirected graphs: every mode is `All`, and `Graph::neighbors`
            // already returns the merged list.
            graph.neighbors(v)?
        };
        for w in neighbours {
            if dist[w as usize].is_none() {
                dist[w as usize] = Some(next);
                queue.push_back(w);
            }
        }
    }

    Ok(dist)
}

// ---------------------------------------------------------------
// Mode-default (OUT) variants — kept as the public legacy surface
// from SP-020. They simply delegate to the with-mode flavour.
// ---------------------------------------------------------------

/// Eccentricity of every vertex (length `vcount`). Result `r[v]` is the
/// maximum shortest-path distance from `v` to any reachable vertex.
/// Isolated vertices have eccentricity `0`.
///
/// Counterpart of `igraph_eccentricity(_, NULL_weights, _, igraph_vss_all(), IGRAPH_OUT)`.
pub fn eccentricity(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount();
    let mut out = vec![0u32; n as usize];
    for v in 0..n {
        let d = distances(graph, v)?;
        let max = d.iter().filter_map(|x| *x).max().unwrap_or(0);
        out[v as usize] = max;
    }
    Ok(out)
}

/// Radius of `graph` — the minimum vertex eccentricity. `None` for a
/// graph with no vertices (matches upstream's `IGRAPH_NAN` for the
/// null graph).
///
/// Counterpart of `igraph_radius(_, NULL_weights, _, IGRAPH_OUT)`.
pub fn radius(graph: &Graph) -> IgraphResult<Option<u32>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(None);
    }
    let ecc = eccentricity(graph)?;
    Ok(ecc.into_iter().min())
}

/// Diameter of `graph` — the maximum vertex eccentricity. `None` for a
/// graph with no vertices.
///
/// Counterpart of
/// `igraph_diameter(_, NULL_weights, _, NULL, NULL, NULL, NULL, _, /*unconn=*/true)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, diameter, radius, eccentricity};
///
/// // Path 0-1-2-3-4: longest geodesic is 0→4 of length 4.
/// let mut g = Graph::with_vertices(5);
/// for i in 0..4 { g.add_edge(i, i + 1).unwrap(); }
/// assert_eq!(diameter(&g).unwrap(), Some(4));
/// // Centre of the path (vertex 2) has eccentricity 2 → radius 2.
/// assert_eq!(radius(&g).unwrap(), Some(2));
/// assert_eq!(eccentricity(&g).unwrap(), vec![4, 3, 2, 3, 4]);
/// ```
pub fn diameter(graph: &Graph) -> IgraphResult<Option<u32>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(None);
    }
    let ecc = eccentricity(graph)?;
    Ok(ecc.into_iter().max())
}

// ---------------------------------------------------------------
// SP-021abc: mode-aware variants. For undirected graphs every mode
// behaves identically (matches upstream).
// ---------------------------------------------------------------

/// Eccentricity of every vertex following `mode`-direction edges.
///
/// Counterpart of `igraph_eccentricity(_, NULL_weights, _,
/// igraph_vss_all(), mode)`. For undirected graphs every mode reduces
/// to [`EccMode::All`] (every edge is bidirectional).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, eccentricity_with_mode, EccMode};
///
/// // Directed path 0→1→2: out-mode says ecc[2]=0; in-mode says ecc[0]=0;
/// // all-mode treats it like an undirected path.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert_eq!(eccentricity_with_mode(&g, EccMode::Out).unwrap(), vec![2, 1, 0]);
/// assert_eq!(eccentricity_with_mode(&g, EccMode::In).unwrap(),  vec![0, 1, 2]);
/// assert_eq!(eccentricity_with_mode(&g, EccMode::All).unwrap(), vec![2, 1, 2]);
/// ```
pub fn eccentricity_with_mode(graph: &Graph, mode: EccMode) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount();
    let mut out = vec![0u32; n as usize];
    for v in 0..n {
        let d = bfs_distances(graph, v, mode)?;
        let max = d.iter().filter_map(|x| *x).max().unwrap_or(0);
        out[v as usize] = max;
    }
    Ok(out)
}

/// Radius of `graph` under the given `mode`. `None` for `vcount == 0`.
///
/// Counterpart of `igraph_radius(_, NULL_weights, _, mode)`.
pub fn radius_with_mode(graph: &Graph, mode: EccMode) -> IgraphResult<Option<u32>> {
    if graph.vcount() == 0 {
        return Ok(None);
    }
    let ecc = eccentricity_with_mode(graph, mode)?;
    Ok(ecc.into_iter().min())
}

/// Diameter of `graph` under the given `mode`. `None` for `vcount == 0`.
///
/// Counterpart of `igraph_diameter(_, NULL_weights, _, NULL, NULL, NULL,
/// NULL, mode == directed ? IGRAPH_OUT : IGRAPH_ALL, /*unconn=*/true)`.
pub fn diameter_with_mode(graph: &Graph, mode: EccMode) -> IgraphResult<Option<u32>> {
    if graph.vcount() == 0 {
        return Ok(None);
    }
    let ecc = eccentricity_with_mode(graph, mode)?;
    Ok(ecc.into_iter().max())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_radii_are_none() {
        let g = Graph::with_vertices(0);
        assert_eq!(radius(&g).unwrap(), None);
        assert_eq!(diameter(&g).unwrap(), None);
        assert!(eccentricity(&g).unwrap().is_empty());
    }

    #[test]
    fn singleton_has_zero_eccentricity() {
        let g = Graph::with_vertices(1);
        assert_eq!(eccentricity(&g).unwrap(), vec![0]);
        assert_eq!(radius(&g).unwrap(), Some(0));
        assert_eq!(diameter(&g).unwrap(), Some(0));
    }

    #[test]
    fn isolated_vertices_each_have_eccentricity_zero() {
        let g = Graph::with_vertices(5);
        assert_eq!(eccentricity(&g).unwrap(), vec![0; 5]);
        assert_eq!(radius(&g).unwrap(), Some(0));
        assert_eq!(diameter(&g).unwrap(), Some(0));
    }

    #[test]
    fn path_5_diameter_4_radius_2() {
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        assert_eq!(eccentricity(&g).unwrap(), vec![4, 3, 2, 3, 4]);
        assert_eq!(radius(&g).unwrap(), Some(2));
        assert_eq!(diameter(&g).unwrap(), Some(4));
    }

    #[test]
    fn cycle_4_eccentricity_uniform_2() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        assert_eq!(eccentricity(&g).unwrap(), vec![2, 2, 2, 2]);
        assert_eq!(radius(&g).unwrap(), Some(2));
        assert_eq!(diameter(&g).unwrap(), Some(2));
    }

    #[test]
    fn star_centre_has_eccentricity_1_leaves_have_2() {
        // 0-1, 0-2, 0-3 → centre 0 has ecc 1; leaves have ecc 2 (via centre).
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        assert_eq!(eccentricity(&g).unwrap(), vec![1, 2, 2, 2]);
        assert_eq!(radius(&g).unwrap(), Some(1));
        assert_eq!(diameter(&g).unwrap(), Some(2));
    }

    #[test]
    fn disconnected_components_use_max_within_components() {
        // Two paths: 0-1-2 (diameter 2) and 3-4 (diameter 1).
        // Per upstream's `unconn=true` semantics, unreachable pairs are
        // ignored, so eccentricity[v] is the max over v's component only.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        assert_eq!(eccentricity(&g).unwrap(), vec![2, 1, 2, 1, 1]);
        assert_eq!(radius(&g).unwrap(), Some(1));
        assert_eq!(diameter(&g).unwrap(), Some(2));
    }

    #[test]
    fn directed_path_uses_out_edges() {
        // 0 -> 1 -> 2: from 0, ecc = 2; from 2, ecc = 0 (no outgoing).
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(eccentricity(&g).unwrap(), vec![2, 1, 0]);
        assert_eq!(diameter(&g).unwrap(), Some(2));
    }

    #[test]
    fn self_loop_does_not_inflate_eccentricity() {
        // 0-self + 0-1: ecc[0] = 1, ecc[1] = 1.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        assert_eq!(eccentricity(&g).unwrap(), vec![1, 1]);
        assert_eq!(diameter(&g).unwrap(), Some(1));
    }

    // ---- SP-021abc mode-aware tests -----

    #[test]
    fn legacy_apis_match_with_mode_out() {
        // For undirected graphs every mode is identical; ensure the
        // legacy default-mode wrappers agree with `_with_mode(Out)`.
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        assert_eq!(
            eccentricity(&g).unwrap(),
            eccentricity_with_mode(&g, EccMode::Out).unwrap()
        );
        assert_eq!(
            radius(&g).unwrap(),
            radius_with_mode(&g, EccMode::Out).unwrap()
        );
        assert_eq!(
            diameter(&g).unwrap(),
            diameter_with_mode(&g, EccMode::Out).unwrap()
        );
    }

    #[test]
    fn undirected_all_modes_agree() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        for m in [EccMode::Out, EccMode::In, EccMode::All] {
            assert_eq!(
                eccentricity_with_mode(&g, m).unwrap(),
                vec![3, 2, 2, 3],
                "mode {m:?}"
            );
            assert_eq!(radius_with_mode(&g, m).unwrap(), Some(2));
            assert_eq!(diameter_with_mode(&g, m).unwrap(), Some(3));
        }
    }

    #[test]
    fn directed_path_in_mode_reverses() {
        // 0→1→2: out-mode reaches forward, in-mode reaches backward.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(
            eccentricity_with_mode(&g, EccMode::Out).unwrap(),
            vec![2, 1, 0]
        );
        assert_eq!(
            eccentricity_with_mode(&g, EccMode::In).unwrap(),
            vec![0, 1, 2]
        );
    }

    #[test]
    fn directed_path_all_mode_treats_undirected() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // All-mode = ignore direction → undirected path, ecc = [2,1,2].
        assert_eq!(
            eccentricity_with_mode(&g, EccMode::All).unwrap(),
            vec![2, 1, 2]
        );
        assert_eq!(radius_with_mode(&g, EccMode::All).unwrap(), Some(1));
        assert_eq!(diameter_with_mode(&g, EccMode::All).unwrap(), Some(2));
    }

    #[test]
    fn directed_cycle_all_modes_uniform() {
        // 0→1→2→0: every mode visits the whole cycle.
        // Out / In: max distance from a vertex on a directed 3-cycle is 2.
        // All: the underlying graph is a triangle (undirected K3) — ecc = 1.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(
            eccentricity_with_mode(&g, EccMode::Out).unwrap(),
            vec![2, 2, 2]
        );
        assert_eq!(
            eccentricity_with_mode(&g, EccMode::In).unwrap(),
            vec![2, 2, 2]
        );
        assert_eq!(
            eccentricity_with_mode(&g, EccMode::All).unwrap(),
            vec![1, 1, 1]
        );
    }

    #[test]
    fn directed_disconnected_in_out_is_zero_for_sinks_sources() {
        // 0→1; isolated vertex 2.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        // Out: 0 reaches 1 (1); 1 reaches nothing (0); 2 isolated (0).
        assert_eq!(
            eccentricity_with_mode(&g, EccMode::Out).unwrap(),
            vec![1, 0, 0]
        );
        // In: 0 reaches nothing in reverse (0); 1 reaches 0 (1); 2 (0).
        assert_eq!(
            eccentricity_with_mode(&g, EccMode::In).unwrap(),
            vec![0, 1, 0]
        );
        // All: 0 and 1 reach each other (1); 2 still isolated (0).
        assert_eq!(
            eccentricity_with_mode(&g, EccMode::All).unwrap(),
            vec![1, 1, 0]
        );
    }

    #[test]
    fn radius_diameter_match_min_max_of_eccentricity() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        for m in [EccMode::Out, EccMode::In, EccMode::All] {
            let ecc = eccentricity_with_mode(&g, m).unwrap();
            assert_eq!(radius_with_mode(&g, m).unwrap(), ecc.iter().copied().min());
            assert_eq!(
                diameter_with_mode(&g, m).unwrap(),
                ecc.iter().copied().max()
            );
        }
    }

    #[test]
    fn empty_graph_with_mode_is_none() {
        let g_u = Graph::with_vertices(0);
        let g_d = Graph::new(0, true).unwrap();
        for m in [EccMode::Out, EccMode::In, EccMode::All] {
            assert_eq!(radius_with_mode(&g_u, m).unwrap(), None);
            assert_eq!(diameter_with_mode(&g_u, m).unwrap(), None);
            assert!(eccentricity_with_mode(&g_u, m).unwrap().is_empty());
            assert_eq!(radius_with_mode(&g_d, m).unwrap(), None);
            assert_eq!(diameter_with_mode(&g_d, m).unwrap(), None);
            assert!(eccentricity_with_mode(&g_d, m).unwrap().is_empty());
        }
    }
}
