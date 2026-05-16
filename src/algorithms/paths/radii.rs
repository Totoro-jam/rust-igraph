//! Eccentricity / radius / diameter (ALGO-SP-020).
//!
//! Counterpart of:
//! - `igraph_eccentricity()` from `references/igraph/src/paths/distances.c:257`
//! - `igraph_radius()`       from `references/igraph/src/paths/distances.c:345`
//! - `igraph_diameter()`     from `references/igraph/src/paths/shortest_paths.c:1259`
//!
//! All three are BFS-based on unweighted graphs and share the same
//! "distances from each vertex" inner loop. Phase-1 minimal slice ships
//! the unweighted, undirected (or `IGRAPH_OUT` for directed) variants
//! that consume the existing [`distances`] primitive.
//!
//! Conventions (matching upstream):
//! - **Eccentricity** of a vertex `v` = max shortest-path distance from
//!   `v` to any reachable vertex; `0` for isolated vertices. Unreachable
//!   vertex pairs are ignored (`unconn = true` semantics).
//! - **Radius** = min eccentricity over all vertices; `None` for n = 0.
//! - **Diameter** = max eccentricity over all vertices; `None` for n = 0.

use crate::algorithms::paths::distances::distances;
use crate::core::{Graph, IgraphResult};

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
}
