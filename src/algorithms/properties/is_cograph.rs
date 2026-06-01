//! Cograph predicate (ALGO-PR-080).
//!
//! A cograph (complement-reducible graph) is a graph with no induced
//! P4 (path on 4 vertices).  Equivalently, a graph G is a cograph
//! iff:
//! - G has a single vertex, or
//! - G is disconnected and each component is a cograph, or
//! - the complement of G is disconnected and each component of
//!   the complement is a cograph.
//!
//! This module uses the recursive characterization with induced
//! subgraph extraction and complementation.
//!
//! For directed graphs, the function returns `false`.

use crate::algorithms::connectivity::components::connected_components;
use crate::algorithms::operators::complementer::complementer;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a cograph (P4-free).
///
/// A cograph contains no induced path on 4 vertices. Cographs are
/// also known as complement-reducible graphs.
///
/// Uses the recursive characterization: a graph is a cograph iff
/// either it or its complement is disconnected, and every component
/// (of whichever is disconnected) is a cograph.
///
/// Returns `false` for directed graphs.
///
/// An empty graph (0 vertices) is a cograph (vacuously).
/// A single vertex is a cograph.
/// Any complete graph is a cograph.
/// Any edgeless graph is a cograph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_cograph};
///
/// // K3: cograph
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_cograph(&g).unwrap());
///
/// // P4 (path on 4 vertices) is NOT a cograph
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(!is_cograph(&g).unwrap());
/// ```
pub fn is_cograph(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n <= 3 {
        return Ok(true);
    }

    is_cograph_recursive(graph)
}

fn is_cograph_recursive(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    if n <= 3 {
        return Ok(true);
    }

    let cc = connected_components(graph)?;

    if cc.count > 1 {
        // G is disconnected — each component must be a cograph
        return check_components_are_cographs(graph, &cc);
    }

    // G is connected — check if complement is disconnected
    let comp = complementer(graph, false)?;
    let cc_comp = connected_components(&comp)?;

    if cc_comp.count <= 1 {
        // Both G and complement(G) are connected → not a cograph
        return Ok(false);
    }

    // Complement is disconnected — each component of complement must be a cograph
    check_components_are_cographs(&comp, &cc_comp)
}

fn check_components_are_cographs(
    graph: &Graph,
    cc: &crate::algorithms::connectivity::components::ConnectedComponents,
) -> IgraphResult<bool> {
    for comp_id in 0..cc.count {
        let comp_verts: Vec<u32> = (0..graph.vcount())
            .filter(|&v| cc.membership[v as usize] == comp_id)
            .collect();

        if comp_verts.len() <= 3 {
            continue;
        }

        let subgraph = extract_induced_subgraph(graph, &comp_verts)?;
        if !is_cograph_recursive(&subgraph)? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn extract_induced_subgraph(graph: &Graph, vertices: &[u32]) -> IgraphResult<Graph> {
    let n = vertices.len();
    let mut sub = Graph::with_vertices(u32::try_from(n).unwrap_or(u32::MAX));

    // Map old vertex IDs to new IDs
    let mut old_to_new = vec![u32::MAX; graph.vcount() as usize];
    for (new_id, &old_id) in vertices.iter().enumerate() {
        old_to_new[old_id as usize] = u32::try_from(new_id).unwrap_or(u32::MAX);
    }

    // Add edges between vertices in the subset
    for &v in vertices {
        let nbrs = graph.neighbors(v)?;
        for w in nbrs {
            if w > v && old_to_new[w as usize] != u32::MAX {
                sub.add_edge(old_to_new[v as usize], old_to_new[w as usize])?;
            }
        }
    }

    Ok(sub)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_cograph(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_cograph(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_cograph(&g).unwrap());
    }

    #[test]
    fn triangle_is_cograph() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_cograph(&g).unwrap());
    }

    #[test]
    fn k4_is_cograph() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(is_cograph(&g).unwrap());
    }

    #[test]
    fn k5_is_cograph() {
        let mut g = Graph::with_vertices(5);
        for u in 0..5u32 {
            for v in (u + 1)..5 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(is_cograph(&g).unwrap());
    }

    #[test]
    fn edgeless_is_cograph() {
        let g = Graph::with_vertices(5);
        assert!(is_cograph(&g).unwrap());
    }

    #[test]
    fn p4_not_cograph() {
        // P4: 0-1-2-3 (the defining forbidden subgraph)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_cograph(&g).unwrap());
    }

    #[test]
    fn c4_is_cograph() {
        // C4 has no induced P4 (every 4-vertex path has a chord)
        // Complement of C4 is 2K2, disconnected into cograph components
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_cograph(&g).unwrap());
    }

    #[test]
    fn c5_not_cograph() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!is_cograph(&g).unwrap());
    }

    #[test]
    fn star_is_cograph() {
        // K_{1,n} is a cograph for any n
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_cograph(&g).unwrap());
    }

    #[test]
    fn path_3_is_cograph() {
        // P3: 0-1-2 (3 vertices, always cograph)
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_cograph(&g).unwrap());
    }

    #[test]
    fn disjoint_k3_k2_is_cograph() {
        // K3 + K2: both components are cographs → cograph
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_cograph(&g).unwrap());
    }

    #[test]
    fn complement_of_p4_not_cograph() {
        // Complement of P4 is also P4 (self-complementary) → not cograph
        let mut g = Graph::with_vertices(4);
        // P4: 0-1, 1-2, 2-3
        // Complement: 0-2, 0-3, 1-3 → which is path 2-0-3-1, also P4
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(!is_cograph(&g).unwrap());
    }

    #[test]
    fn k2_union_k2_is_cograph() {
        // Two disjoint edges: cograph (complement is C4, but each
        // component of this graph is K2 which is cograph)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_cograph(&g).unwrap());
    }

    #[test]
    fn petersen_not_cograph() {
        // Petersen graph contains many induced P4s
        let mut g = Graph::with_vertices(10);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        g.add_edge(5, 7).unwrap();
        g.add_edge(7, 9).unwrap();
        g.add_edge(9, 6).unwrap();
        g.add_edge(6, 8).unwrap();
        g.add_edge(8, 5).unwrap();
        g.add_edge(0, 5).unwrap();
        g.add_edge(1, 6).unwrap();
        g.add_edge(2, 7).unwrap();
        g.add_edge(3, 8).unwrap();
        g.add_edge(4, 9).unwrap();
        assert!(!is_cograph(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_cograph(&g).unwrap());
    }

    #[test]
    fn join_of_two_edgeless_is_cograph() {
        // K_{2,3} is NOT a cograph (contains induced P4: 0-2-1-3)
        // But disjoint union of edgeless graphs IS cograph
        let g = Graph::with_vertices(5);
        assert!(is_cograph(&g).unwrap());
    }

    #[test]
    fn k23_is_cograph() {
        // K_{2,3}: complement is K2 + K3, all complete bipartite graphs are cographs
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(is_cograph(&g).unwrap());
    }

    #[test]
    fn threshold_graph_is_cograph() {
        // Threshold graphs are a subclass of cographs
        // Build one: start with K1, add isolated, add dominating, add isolated
        // 0: initial vertex
        // 1: isolated (no edges)
        // 2: dominating (edges to 0, 1)
        // 3: isolated (no edges)
        // 4: dominating (edges to 0, 1, 2, 3)
        let mut g = Graph::with_vertices(5);
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 1).unwrap();
        g.add_edge(4, 0).unwrap();
        g.add_edge(4, 1).unwrap();
        g.add_edge(4, 2).unwrap();
        g.add_edge(4, 3).unwrap();
        assert!(is_cograph(&g).unwrap());
    }
}
