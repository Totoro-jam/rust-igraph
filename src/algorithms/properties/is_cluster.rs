//! Cluster graph predicate (ALGO-PR-077).
//!
//! A cluster graph (disjoint union of cliques) is a graph where every
//! connected component is a complete graph. Equivalently, the graph
//! is P3-free (no induced path on 3 vertices).
//!
//! Recognition: decompose into connected components and verify that
//! each component with k vertices has exactly k(k-1)/2 edges.
//!
//! Unlike block graph, cluster graph does NOT require overall
//! connectivity — a disconnected graph can be a cluster graph.

use crate::algorithms::connectivity::components::connected_components;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a cluster graph (disjoint union of cliques).
///
/// Every connected component must be a complete graph. This is
/// equivalent to the graph being P3-free.
///
/// Returns `false` for directed graphs (the concept is defined for
/// undirected graphs). Also returns `false` if the graph has
/// self-loops or parallel edges, since components would not be
/// simple cliques.
///
/// An empty graph (0 vertices) is a cluster graph (vacuously).
/// An edgeless graph is a cluster graph (each vertex is K1).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_cluster_graph};
///
/// // K3 + K2 + isolated vertex
/// let mut g = Graph::with_vertices(6);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(3, 4).unwrap();
/// assert!(is_cluster_graph(&g).unwrap());
///
/// // Path P3 is NOT a cluster graph (contains induced P3)
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert!(!is_cluster_graph(&g).unwrap());
/// ```
pub fn is_cluster_graph(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n == 0 {
        return Ok(true);
    }

    let cc = connected_components(graph)?;
    let comp_count = cc.count as usize;

    let mut comp_verts = vec![0u64; comp_count];
    let mut comp_edges = vec![0u64; comp_count];

    for v in 0..n {
        let c = cc.membership[v as usize] as usize;
        comp_verts[c] = comp_verts[c].saturating_add(1);
    }

    let ecount = graph.ecount();
    for eid in 0..ecount {
        let eid_u32 = u32::try_from(eid).unwrap_or(u32::MAX);
        let (u, v) = graph.edge(eid_u32)?;
        if u == v {
            return Ok(false);
        }
        let c = cc.membership[u as usize] as usize;
        comp_edges[c] = comp_edges[c].saturating_add(1);
    }

    for c in 0..comp_count {
        let k = comp_verts[c];
        let expected = k.saturating_mul(k.saturating_sub(1)) / 2;
        if comp_edges[c] != expected {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn edgeless_graph() {
        let g = Graph::with_vertices(5);
        assert!(is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn k4() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn k3_plus_k2_plus_isolated() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn two_disjoint_k3() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        assert!(is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn path_3_not_cluster() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn path_4_not_cluster() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn star_not_cluster() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(!is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn cycle_4_not_cluster() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn k4_minus_edge_not_cluster() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(!is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn self_loop_returns_false() {
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();
        assert!(!is_cluster_graph(&g).unwrap());
    }

    #[test]
    fn parallel_edges_returns_false() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_cluster_graph(&g).unwrap());
    }
}
