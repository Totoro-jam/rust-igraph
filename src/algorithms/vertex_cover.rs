//! Vertex cover algorithms (ALGO-VC-001).
//!
//! A vertex cover is a set of vertices such that every edge in the
//! graph is incident to at least one vertex in the set. Finding a
//! minimum vertex cover is NP-hard; we provide a greedy
//! 2-approximation.
//!
//! The greedy algorithm repeatedly picks an uncovered edge and adds
//! both endpoints to the cover. This guarantees the result is at most
//! twice the size of the optimal cover (König 1931 / Gavril 1974).

use crate::core::{Graph, IgraphResult, VertexId};

/// Check whether a set of vertices forms a valid vertex cover.
///
/// A vertex cover is valid if every edge in the graph has at least
/// one endpoint in the set.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_vertex_cover};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// assert!(is_vertex_cover(&g, &[1, 2]));
/// assert!(!is_vertex_cover(&g, &[0, 3]));
/// ```
pub fn is_vertex_cover(graph: &Graph, cover: &[VertexId]) -> bool {
    let n = graph.vcount() as usize;
    let mut in_cover = vec![false; n];
    for &v in cover {
        if (v as usize) < n {
            in_cover[v as usize] = true;
        }
    }

    let ecount = graph.ecount();
    #[allow(clippy::cast_possible_truncation)] // ecount ≤ u32::MAX by graph invariant
    for eid in 0..ecount as u32 {
        if let Ok((u, v)) = graph.edge(eid) {
            if !in_cover[u as usize] && !in_cover[v as usize] {
                return false;
            }
        }
    }
    true
}

/// Compute a minimum vertex cover using a greedy 2-approximation.
///
/// Repeatedly picks an uncovered edge and adds both endpoints to
/// the cover. The result is guaranteed to be at most twice the
/// size of the optimal minimum vertex cover.
///
/// For undirected graphs, direction is ignored. For directed graphs,
/// each arc counts as an edge that must be covered.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, minimum_vertex_cover, is_vertex_cover};
///
/// // Path 0-1-2-3: optimal cover is {1, 2}, greedy gives at most 4
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let cover = minimum_vertex_cover(&g).unwrap();
/// assert!(is_vertex_cover(&g, &cover));
/// assert!(cover.len() <= 4); // 2-approx of opt=2
///
/// // Empty graph: no edges, empty cover
/// let g = Graph::with_vertices(5);
/// let cover = minimum_vertex_cover(&g).unwrap();
/// assert!(cover.is_empty());
/// ```
#[allow(clippy::cast_possible_truncation)] // vcount/ecount ≤ u32::MAX by graph invariant
pub fn minimum_vertex_cover(graph: &Graph) -> IgraphResult<Vec<VertexId>> {
    let n = graph.vcount();
    let ecount = graph.ecount();

    if ecount == 0 {
        return Ok(Vec::new());
    }

    let mut in_cover = vec![false; n as usize];
    let mut covered = vec![false; ecount];

    for eid in 0..ecount {
        if covered[eid] {
            continue;
        }

        let (u, v) = graph.edge(eid as u32)?;
        if in_cover[u as usize] || in_cover[v as usize] {
            covered[eid] = true;
            continue;
        }

        in_cover[u as usize] = true;
        in_cover[v as usize] = true;

        mark_incident_covered(graph, u, &mut covered);
        mark_incident_covered(graph, v, &mut covered);
    }

    let mut result: Vec<VertexId> = (0..n).filter(|&v| in_cover[v as usize]).collect();
    result.sort_unstable();
    Ok(result)
}

fn mark_incident_covered(graph: &Graph, v: VertexId, covered: &mut [bool]) {
    if let Ok(edges) = graph.incident(v) {
        for &eid in &edges {
            covered[eid as usize] = true;
        }
    }
    if graph.is_directed() {
        if let Ok(edges) = graph.incident_in(v) {
            for &eid in &edges {
                covered[eid as usize] = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let cover = minimum_vertex_cover(&g).unwrap();
        assert!(cover.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(5);
        let cover = minimum_vertex_cover(&g).unwrap();
        assert!(cover.is_empty());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let cover = minimum_vertex_cover(&g).unwrap();
        assert_eq!(cover.len(), 2);
        assert!(is_vertex_cover(&g, &cover));
    }

    #[test]
    fn path_3() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let cover = minimum_vertex_cover(&g).unwrap();
        assert!(is_vertex_cover(&g, &cover));
        assert!(cover.len() <= 4); // 2-approx of opt=1
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let cover = minimum_vertex_cover(&g).unwrap();
        assert!(is_vertex_cover(&g, &cover));
        assert!(cover.len() >= 2); // min cover of triangle is 2
        assert!(cover.len() <= 4); // 2-approx
    }

    #[test]
    fn star_graph() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        let cover = minimum_vertex_cover(&g).unwrap();
        assert!(is_vertex_cover(&g, &cover));
        // Optimal is {0} (size 1), 2-approx gives at most 2
        assert!(cover.len() <= 2);
    }

    #[test]
    fn complete_k4() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let cover = minimum_vertex_cover(&g).unwrap();
        assert!(is_vertex_cover(&g, &cover));
        assert!(cover.len() >= 3); // K4 minimum cover is 3
    }

    #[test]
    fn bipartite_matching_example() {
        // K_{2,2}: 0-2, 0-3, 1-2, 1-3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        let cover = minimum_vertex_cover(&g).unwrap();
        assert!(is_vertex_cover(&g, &cover));
        assert!(cover.len() >= 2); // minimum cover is 2
        assert!(cover.len() <= 4); // 2-approx
    }

    #[test]
    fn directed_graph() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let cover = minimum_vertex_cover(&g).unwrap();
        assert!(is_vertex_cover(&g, &cover));
    }

    #[test]
    fn self_loop() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        let cover = minimum_vertex_cover(&g).unwrap();
        assert!(is_vertex_cover(&g, &cover));
        assert!(cover.contains(&0)); // self-loop needs vertex 0
    }

    #[test]
    fn isolated_with_edges() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        // vertex 4 is isolated
        let cover = minimum_vertex_cover(&g).unwrap();
        assert!(is_vertex_cover(&g, &cover));
        assert!(!cover.contains(&4));
    }

    #[test]
    fn two_approx_guarantee() {
        // Path of length 5: 0-1-2-3-4-5
        // Optimal cover: {1, 3, 5} (size 3) or {1, 3} won't work...
        // Actually opt is {1, 3, 5} or {0, 2, 4} — size 3
        // Greedy picks first edge 0-1 → both in cover
        // 2-approx → at most 6
        let mut g = Graph::with_vertices(6);
        for i in 0..5u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let cover = minimum_vertex_cover(&g).unwrap();
        assert!(is_vertex_cover(&g, &cover));
        assert!(cover.len() <= 6); // 2 * opt(3) = 6
    }

    #[test]
    fn is_cover_validator_positive() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_vertex_cover(&g, &[1]));
        assert!(is_vertex_cover(&g, &[0, 2]));
        assert!(is_vertex_cover(&g, &[0, 1, 2]));
    }

    #[test]
    fn is_cover_validator_negative() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_vertex_cover(&g, &[]));
        assert!(!is_vertex_cover(&g, &[0]));
        assert!(!is_vertex_cover(&g, &[0, 3]));
    }

    #[test]
    fn sorted_output() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(4, 5).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let cover = minimum_vertex_cover(&g).unwrap();
        for i in 1..cover.len() {
            assert!(cover[i] > cover[i - 1]);
        }
    }

    #[test]
    fn parallel_edges() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let cover = minimum_vertex_cover(&g).unwrap();
        assert!(is_vertex_cover(&g, &cover));
        assert_eq!(cover.len(), 2);
    }
}
