//! Decompose a graph into its (weakly) connected components (ALGO-CC-003).
//!
//! Counterpart of `igraph_decompose()` from
//! `references/igraph/src/connectivity/components.c:603` — Phase-1
//! minimal slice covers the `IGRAPH_WEAK` branch only, which is
//! upstream's auto-selected mode for undirected graphs (see
//! `igraph_i_decompose_weak` at `components.c:620`).
//!
//! Each component becomes a freshly built [`Graph`] with its vertices
//! renumbered to `0..k` in BFS-from-`actstart` order, exactly matching
//! upstream's `igraph_i_induced_subgraph_map(..., IGRAPH_SUBGRAPH_AUTO)`
//! contract. Loops and parallel edges are preserved verbatim. Strong
//! decomposition is a follow-up AWU.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Decompose `graph` into its weakly connected components.
///
/// Returns one new [`Graph`] per component; the order matches the
/// natural BFS-from-lowest-unvisited-vertex traversal used by
/// `igraph_decompose(_, _, IGRAPH_WEAK, _, _)` (`components.c:620`).
/// The component graph's vertex IDs are remapped to `0..k`, where `k`
/// is the size of the component, in BFS visit order from the seed
/// vertex. Edges within the component (loops and parallel edges
/// included) are preserved with their original direction.
///
/// On directed input the decomposition still uses *weak* connectivity
/// (edge direction is ignored when discovering components, but each
/// component graph is built as directed and retains the original edge
/// orientations). Strong decomposition is a follow-up AWU.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, decompose};
///
/// // Two components: {0, 1, 2} (triangle) and {3, 4} (edge).
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(3, 4).unwrap();
///
/// let parts = decompose(&g).unwrap();
/// assert_eq!(parts.len(), 2);
/// // Triangle: 3 vertices, 3 edges.
/// assert_eq!(parts[0].vcount(), 3);
/// assert_eq!(parts[0].ecount(), 3);
/// // Edge component: 2 vertices, 1 edge.
/// assert_eq!(parts[1].vcount(), 2);
/// assert_eq!(parts[1].ecount(), 1);
/// ```
pub fn decompose(graph: &Graph) -> IgraphResult<Vec<Graph>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }

    // BFS-discovery order, remembered per vertex via `vid_in_comp`
    // (== local id within its component, or u32::MAX if unvisited).
    let mut vid_in_comp = vec![u32::MAX; n as usize];
    let mut comp_of = vec![u32::MAX; n as usize];
    // Per-component vertex lists, indexed in BFS-discovery order.
    let mut comp_vertices: Vec<Vec<VertexId>> = Vec::new();
    let mut queue: VecDeque<VertexId> = VecDeque::new();

    for actstart in 0..n {
        if vid_in_comp[actstart as usize] != u32::MAX {
            continue;
        }
        let comp_id = u32::try_from(comp_vertices.len())
            .map_err(|_| IgraphError::Internal("too many components in decompose"))?;
        let mut verts: Vec<VertexId> = Vec::new();

        vid_in_comp[actstart as usize] = 0;
        comp_of[actstart as usize] = comp_id;
        verts.push(actstart);
        queue.clear();
        queue.push_back(actstart);

        while let Some(v) = queue.pop_front() {
            for w in graph.neighbors(v)? {
                if vid_in_comp[w as usize] == u32::MAX {
                    let new_local = u32::try_from(verts.len())
                        .map_err(|_| IgraphError::Internal("component too large in decompose"))?;
                    vid_in_comp[w as usize] = new_local;
                    comp_of[w as usize] = comp_id;
                    verts.push(w);
                    queue.push_back(w);
                }
            }
        }
        comp_vertices.push(verts);
    }

    // Pre-construct an empty Graph per component (matching directedness).
    let directed = graph.is_directed();
    let mut parts: Vec<Graph> = comp_vertices
        .iter()
        .map(|verts| {
            // verts.len() ≤ vcount, fits u32 because vcount is u32.
            let k = u32::try_from(verts.len())
                .map_err(|_| IgraphError::Internal("component vertex count exceeds u32"))?;
            Graph::new(k, directed)
        })
        .collect::<IgraphResult<Vec<_>>>()?;

    // Single edge sweep: each edge belongs to exactly one weak
    // component, so we can place it directly in the right subgraph
    // using the `comp_of[s]` lookup. Endpoints are renumbered via
    // `vid_in_comp[v]` into the component's local id space.
    let m = graph.ecount();
    for e in 0..m {
        let eid = u32::try_from(e)
            .map_err(|_| IgraphError::Internal("edge id exceeds u32 in decompose"))?;
        let s = graph.edge_source(eid)?;
        let t = graph.edge_target(eid)?;
        let comp_s = comp_of[s as usize];
        // For weak components both endpoints share the same comp id.
        debug_assert_eq!(comp_s, comp_of[t as usize]);
        let local_s = vid_in_comp[s as usize];
        let local_t = vid_in_comp[t as usize];
        parts[comp_s as usize].add_edge(local_s, local_t)?;
    }

    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_yields_no_components() {
        let g = Graph::with_vertices(0);
        let parts = decompose(&g).unwrap();
        assert!(parts.is_empty());
    }

    #[test]
    fn null_graph_with_isolated_vertices() {
        let g = Graph::with_vertices(3);
        let parts = decompose(&g).unwrap();
        assert_eq!(parts.len(), 3);
        for p in &parts {
            assert_eq!(p.vcount(), 1);
            assert_eq!(p.ecount(), 0);
        }
    }

    #[test]
    fn single_component_preserves_size() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let parts = decompose(&g).unwrap();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].vcount(), 4);
        assert_eq!(parts[0].ecount(), 3);
    }

    #[test]
    fn two_disjoint_triangles() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        let parts = decompose(&g).unwrap();
        assert_eq!(parts.len(), 2);
        for p in &parts {
            assert_eq!(p.vcount(), 3);
            assert_eq!(p.ecount(), 3);
        }
    }

    #[test]
    fn vertex_remapping_starts_at_zero() {
        // Triangle {3, 4, 5} (no vertices 0, 1, 2 connected). Result
        // graph must have vertices 0, 1, 2 (renumbered).
        let mut g = Graph::with_vertices(6);
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        let parts = decompose(&g).unwrap();
        // 3 isolated singletons (0, 1, 2) + the triangle {3,4,5}.
        assert_eq!(parts.len(), 4);
        // First three are singletons (BFS-from-0, then 1, then 2).
        for p in parts.iter().take(3) {
            assert_eq!(p.vcount(), 1);
            assert_eq!(p.ecount(), 0);
        }
        // Last is the triangle, with renumbered ids 0..2.
        let tri = &parts[3];
        assert_eq!(tri.vcount(), 3);
        assert_eq!(tri.ecount(), 3);
        // BFS-from-3 visits 3 (→ local 0), then 4 (→ 1), then 5 (→ 2).
        assert_eq!(tri.edge_source(0).unwrap(), 0);
        assert_eq!(tri.edge_target(0).unwrap(), 1);
    }

    #[test]
    fn directed_graph_preserves_edge_orientation() {
        // 0 → 1 → 2 (chain) + 3 isolated. Weak components: {0,1,2},
        // {3}. The chain component's edges keep their direction.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let parts = decompose(&g).unwrap();
        assert_eq!(parts.len(), 2);
        let chain = &parts[0];
        assert!(chain.is_directed());
        assert_eq!(chain.vcount(), 3);
        assert_eq!(chain.ecount(), 2);
        assert_eq!(chain.edge_source(0).unwrap(), 0);
        assert_eq!(chain.edge_target(0).unwrap(), 1);
        assert_eq!(chain.edge_source(1).unwrap(), 1);
        assert_eq!(chain.edge_target(1).unwrap(), 2);
    }

    #[test]
    fn loops_and_parallel_edges_preserved() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap(); // self-loop
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap(); // parallel
        let parts = decompose(&g).unwrap();
        // Vertex 2 is isolated → second component.
        assert_eq!(parts.len(), 2);
        let main = &parts[0];
        assert_eq!(main.vcount(), 2);
        assert_eq!(main.ecount(), 3);
    }

    #[test]
    fn component_count_matches_connected_components() {
        // Cross-check against ALGO-CC-001 on a multi-component graph.
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        // Vertex 5, 6 isolated.
        let parts = decompose(&g).unwrap();
        let cc = crate::algorithms::connectivity::components::connected_components(&g).unwrap();
        assert_eq!(u32::try_from(parts.len()).unwrap(), cc.count);
    }

    #[test]
    fn round_trip_edge_count_total_matches() {
        // The total edge count across all subgraphs must equal the
        // input edge count (every edge belongs to exactly one weak
        // component).
        let mut g = Graph::with_vertices(8);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(6, 7).unwrap();
        let parts = decompose(&g).unwrap();
        let total_e: usize = parts.iter().map(Graph::ecount).sum();
        assert_eq!(total_e, g.ecount());
        let total_v: u32 = parts.iter().map(Graph::vcount).sum();
        assert_eq!(total_v, g.vcount());
    }
}
