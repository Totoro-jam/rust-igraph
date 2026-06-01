//! Pseudo-forest predicate (ALGO-PR-075).
//!
//! A pseudo-forest is an undirected graph where every connected
//! component contains at most one cycle. Equivalently, for each
//! connected component, |E| <= |V|.
//!
//! A forest (|E| = |V| - 1 per component) is a special case.
//! A 1-tree (unicyclic component, |E| = |V|) is a pseudo-forest
//! component with exactly one cycle.
//!
//! For directed graphs, the function returns `false` (the concept
//! is defined for undirected graphs).

use crate::algorithms::connectivity::components::connected_components;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a pseudo-forest.
///
/// A pseudo-forest has at most one cycle per connected component.
/// This is equivalent to requiring |E| <= |V| for every component.
///
/// Returns `false` for directed graphs.
///
/// An empty graph (0 vertices) is a pseudo-forest (vacuously).
/// An edgeless graph is a pseudo-forest (forest with no edges).
/// A single cycle is a pseudo-forest.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_pseudo_forest};
///
/// // Triangle: one component with |E|=3, |V|=3 → pseudo-forest
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_pseudo_forest(&g).unwrap());
///
/// // K4: |E|=6, |V|=4 → NOT pseudo-forest
/// let mut g = Graph::with_vertices(4);
/// for u in 0..4u32 {
///     for v in (u + 1)..4 {
///         g.add_edge(u, v).unwrap();
///     }
/// }
/// assert!(!is_pseudo_forest(&g).unwrap());
/// ```
pub fn is_pseudo_forest(graph: &Graph) -> IgraphResult<bool> {
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
        let (u, _v) = graph.edge(eid_u32)?;
        let c = cc.membership[u as usize] as usize;
        comp_edges[c] = comp_edges[c].saturating_add(1);
    }

    for c in 0..comp_count {
        if comp_edges[c] > comp_verts[c] {
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
        assert!(is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn edgeless_graph() {
        let g = Graph::with_vertices(5);
        assert!(is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn path_is_pseudo_forest() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn tree_is_pseudo_forest() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn triangle_is_pseudo_forest() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn cycle_4_is_pseudo_forest() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn unicyclic_with_tail() {
        // Triangle 0-1-2 + tail 2-3-4: |V|=5, |E|=4 per component → ok
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn two_disjoint_cycles() {
        // Two triangles as separate components: each has |E|=|V|=3
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        assert!(is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn k4_not_pseudo_forest() {
        // K4: |E|=6, |V|=4 → two cycles → not pseudo-forest
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(!is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn diamond_not_pseudo_forest() {
        // Diamond = K4 - edge: |V|=4, |E|=5 → not pseudo-forest
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(!is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn theta_graph_not_pseudo_forest() {
        // Theta: two vertices connected by 3 internally disjoint paths
        // e.g., 0-1, 0-2-1, 0-3-1: |V|=4, |E|=5 → not pseudo-forest
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 1).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(3, 1).unwrap();
        assert!(!is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn self_loop_counts() {
        // Self-loop: component has |V|=1, |E|=1 → ok (one cycle)
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();
        assert!(is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn two_self_loops_not_pseudo_forest() {
        // Two self-loops on same vertex: |V|=1, |E|=2 → not pseudo-forest
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 0).unwrap();
        assert!(!is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn parallel_edges_not_pseudo_forest() {
        // Two parallel edges + one more: |V|=2, |E|=3 → not pseudo-forest
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn parallel_edge_is_pseudo_forest() {
        // Two parallel edges: |V|=2, |E|=2 → pseudo-forest (one cycle)
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(is_pseudo_forest(&g).unwrap());
    }

    #[test]
    fn mixed_components() {
        // Component 1: tree {0,1,2} with edges 0-1, 1-2 (|E|=2, |V|=3 → ok)
        // Component 2: K4 {3,4,5,6} (|E|=6, |V|=4 → not ok)
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        for u in 3..7u32 {
            for v in (u + 1)..7 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(!is_pseudo_forest(&g).unwrap());
    }
}
