//! Complete multipartite graph predicate (ALGO-PR-090).
//!
//! A complete multipartite graph `K_{n1,n2,...,nk}` has vertices
//! partitioned into k independent sets (parts), with every pair of
//! vertices in different parts connected by an edge. The complement
//! of a complete multipartite graph is a disjoint union of cliques.
//!
//! For directed graphs, the function returns `false`.

use crate::algorithms::connectivity::connected_components;
use crate::algorithms::operators::complementer::complementer;
use crate::algorithms::properties::is_simple::is_simple;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a complete multipartite graph.
///
/// A complete multipartite graph has vertices partitioned into
/// independent sets where every two vertices in different parts are
/// adjacent. Equivalently, the complement is a disjoint union of
/// cliques.
///
/// Returns `false` for directed graphs and non-simple graphs.
/// Returns `true` for the empty graph (vacuously), single vertex,
/// complete graphs (single part of size 0 with all vertices in other
/// parts, or equivalently k parts of size 1), and edgeless graphs
/// (single part containing all vertices).
///
/// If the graph is complete multipartite, returns `Some(parts)` where
/// `parts` is a sorted vector of part sizes. Returns `None` otherwise.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_complete_multipartite};
///
/// // K_{2,3}: complete bipartite
/// let mut g = Graph::with_vertices(5);
/// for i in 0..2u32 {
///     for j in 2..5u32 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// let parts = is_complete_multipartite(&g).unwrap();
/// assert_eq!(parts, Some(vec![2, 3]));
///
/// // Triangle K_3 = K_{1,1,1}
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let parts = is_complete_multipartite(&g).unwrap();
/// assert_eq!(parts, Some(vec![1, 1, 1]));
/// ```
pub fn is_complete_multipartite(graph: &Graph) -> IgraphResult<Option<Vec<u32>>> {
    if graph.is_directed() {
        return Ok(None);
    }

    let n = graph.vcount();
    if n == 0 {
        return Ok(Some(vec![]));
    }

    if !is_simple(graph)? {
        return Ok(None);
    }

    // Edgeless graph: one part containing all vertices
    if graph.ecount() == 0 {
        return Ok(Some(vec![n]));
    }

    // Complement of a complete multipartite graph is a disjoint union of cliques.
    let comp = complementer(graph, false)?;
    let comps = connected_components(&comp)?;

    let membership = &comps.membership;
    let num_comps = comps.count as usize;

    // Collect vertices per component
    let mut parts: Vec<Vec<u32>> = vec![vec![]; num_comps];
    for (v, &comp_id) in membership.iter().enumerate() {
        parts[comp_id as usize].push(u32::try_from(v).unwrap_or(u32::MAX));
    }

    // Verify each part is a clique in the complement (= independent set in original)
    for part in &parts {
        if part.len() > 1 {
            // Check that this set is a clique in the complement
            let subgraph_comp = is_clique_in_graph(&comp, part)?;
            if !subgraph_comp {
                return Ok(None);
            }
        }
    }

    // Verify total edge count matches expected for complete multipartite
    let mut expected_edges: u64 = 0;
    let part_sizes: Vec<u32> = parts
        .iter()
        .map(|p| u32::try_from(p.len()).unwrap_or(u32::MAX))
        .collect();
    let total_n = u64::from(n);
    for &size in &part_sizes {
        let s = u64::from(size);
        // Each vertex in this part connects to all vertices NOT in this part
        expected_edges = expected_edges.saturating_add(s.saturating_mul(total_n.saturating_sub(s)));
    }
    // Each edge counted twice (once from each endpoint's part)
    expected_edges /= 2;

    if graph.ecount() as u64 != expected_edges {
        return Ok(None);
    }

    let mut result: Vec<u32> = part_sizes;
    result.sort_unstable();
    Ok(Some(result))
}

fn is_clique_in_graph(graph: &Graph, vertices: &[u32]) -> IgraphResult<bool> {
    let k = vertices.len();
    if k <= 1 {
        return Ok(true);
    }

    for (i, &vi) in vertices.iter().enumerate() {
        let nbrs = graph.neighbors(vi)?;
        let count = vertices
            .iter()
            .enumerate()
            .filter(|&(j, &vj)| i != j && nbrs.contains(&vj))
            .count();
        if count != k - 1 {
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
        assert_eq!(is_complete_multipartite(&g).unwrap(), Some(vec![]));
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert_eq!(is_complete_multipartite(&g).unwrap(), Some(vec![1]));
    }

    #[test]
    fn edgeless_3() {
        let g = Graph::with_vertices(3);
        assert_eq!(is_complete_multipartite(&g).unwrap(), Some(vec![3]));
    }

    #[test]
    fn single_edge_k11() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert_eq!(is_complete_multipartite(&g).unwrap(), Some(vec![1, 1]));
    }

    #[test]
    fn triangle_k111() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(is_complete_multipartite(&g).unwrap(), Some(vec![1, 1, 1]));
    }

    #[test]
    fn k23() {
        let mut g = Graph::with_vertices(5);
        for i in 0..2u32 {
            for j in 2..5u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert_eq!(is_complete_multipartite(&g).unwrap(), Some(vec![2, 3]));
    }

    #[test]
    fn k22() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert_eq!(is_complete_multipartite(&g).unwrap(), Some(vec![2, 2]));
    }

    #[test]
    fn k_1_1_1_1() {
        // K4 = K_{1,1,1,1}
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(
            is_complete_multipartite(&g).unwrap(),
            Some(vec![1, 1, 1, 1])
        );
    }

    #[test]
    fn star_k13() {
        // K_{1,3}: center 0 connected to 1,2,3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert_eq!(is_complete_multipartite(&g).unwrap(), Some(vec![1, 3]));
    }

    #[test]
    fn k_2_2_2() {
        // Complete tripartite K_{2,2,2}: 6 vertices, parts {0,1}, {2,3}, {4,5}
        let mut g = Graph::with_vertices(6);
        let parts = [vec![0u32, 1], vec![2, 3], vec![4, 5]];
        for i in 0..3 {
            for j in (i + 1)..3 {
                for &u in &parts[i] {
                    for &v in &parts[j] {
                        g.add_edge(u, v).unwrap();
                    }
                }
            }
        }
        assert_eq!(is_complete_multipartite(&g).unwrap(), Some(vec![2, 2, 2]));
    }

    #[test]
    fn path_p3_is_k12() {
        // P3: 0-1-2 = K_{1,2} (star with 2 leaves)
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(is_complete_multipartite(&g).unwrap(), Some(vec![1, 2]));
    }

    #[test]
    fn path_p4_not_multipartite() {
        // P4: 0-1-2-3 — vertex 0 and 2 are non-adjacent but in different
        // parts, yet 1 and 2 are adjacent in the same "should-be" part
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(is_complete_multipartite(&g).unwrap(), None);
    }

    #[test]
    fn cycle_c4_is_k22() {
        // C4 = K_{2,2}
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert_eq!(is_complete_multipartite(&g).unwrap(), Some(vec![2, 2]));
    }

    #[test]
    fn cycle_c5_not_multipartite() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert_eq!(is_complete_multipartite(&g).unwrap(), None);
    }

    #[test]
    fn directed_returns_none() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(is_complete_multipartite(&g).unwrap(), None);
    }

    #[test]
    fn disconnected_not_multipartite() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(is_complete_multipartite(&g).unwrap(), None);
    }

    #[test]
    fn petersen_not_multipartite() {
        // Petersen graph is 3-regular on 10 vertices, not complete multipartite
        let mut g = Graph::with_vertices(10);
        // Outer cycle
        for i in 0..5u32 {
            g.add_edge(i, (i + 1) % 5).unwrap();
        }
        // Inner pentagram
        for i in 0..5u32 {
            g.add_edge(i + 5, 5 + (i + 2) % 5).unwrap();
        }
        // Spokes
        for i in 0..5u32 {
            g.add_edge(i, i + 5).unwrap();
        }
        assert_eq!(is_complete_multipartite(&g).unwrap(), None);
    }
}
