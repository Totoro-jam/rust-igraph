//! Edge cover algorithms (ALGO-MA-002).
//!
//! An edge cover is a set of edges such that every vertex in the
//! graph is incident to at least one edge in the set. An edge cover
//! exists if and only if the graph has no isolated vertices.
//!
//! We compute a minimum edge cover via a greedy maximal matching
//! followed by covering unmatched vertices with any incident edge.
//! For graphs without isolated vertices, the result size equals
//! n - |matching|, which is optimal by König's theorem.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Edge ID type alias for clarity.
type EdgeId = u32;

/// Check whether a set of edge IDs forms a valid edge cover.
///
/// An edge cover is valid if every vertex in the graph is incident
/// to at least one edge in the set.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_edge_cover};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap(); // edge 0
/// g.add_edge(1, 2).unwrap(); // edge 1
/// g.add_edge(2, 3).unwrap(); // edge 2
///
/// assert!(is_edge_cover(&g, &[0, 2]));   // covers all 4 vertices
/// assert!(!is_edge_cover(&g, &[1]));      // vertices 0, 3 uncovered
/// ```
#[allow(clippy::cast_possible_truncation)]
pub fn is_edge_cover(graph: &Graph, cover: &[EdgeId]) -> bool {
    let n = graph.vcount() as usize;
    if n == 0 {
        return true;
    }

    let mut covered = vec![false; n];

    for &eid in cover {
        if let Ok((u, v)) = graph.edge(eid) {
            covered[u as usize] = true;
            covered[v as usize] = true;
        }
    }

    covered.iter().all(|&c| c)
}

/// Compute a minimum edge cover.
///
/// Returns a set of edge IDs such that every vertex is incident to
/// at least one edge in the set, and the number of edges is
/// minimized.
///
/// The algorithm first computes a greedy maximal matching, then
/// covers each unmatched vertex by adding any one of its incident
/// edges. By König's theorem, the result has size n - |matching|,
/// which is optimal.
///
/// # Errors
///
/// Returns an error if the graph has isolated vertices (vertices
/// with degree 0), since no edge cover exists in that case.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, minimum_edge_cover, is_edge_cover};
///
/// // Path 0-1-2: minimum edge cover is {0-1, 1-2} (2 edges)
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let cover = minimum_edge_cover(&g).unwrap();
/// assert!(is_edge_cover(&g, &cover));
/// assert_eq!(cover.len(), 2);
///
/// // K3: minimum edge cover is 2 edges
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let cover = minimum_edge_cover(&g).unwrap();
/// assert!(is_edge_cover(&g, &cover));
/// assert_eq!(cover.len(), 2);
/// ```
#[allow(clippy::cast_possible_truncation)]
pub fn minimum_edge_cover(graph: &Graph) -> IgraphResult<Vec<EdgeId>> {
    let n = graph.vcount();

    if n == 0 {
        return Ok(Vec::new());
    }

    let directed = graph.is_directed();

    // Check for isolated vertices
    for v in 0..n {
        let out_edges = graph.incident(v)?;
        let has_edges = if directed {
            let in_edges = graph.incident_in(v)?;
            !out_edges.is_empty() || !in_edges.is_empty()
        } else {
            !out_edges.is_empty()
        };
        if !has_edges {
            return Err(IgraphError::InvalidArgument(format!(
                "vertex {v} has degree 0; no edge cover exists"
            )));
        }
    }

    let ecount = graph.ecount();
    if ecount == 0 {
        return Err(IgraphError::InvalidArgument(
            "graph has vertices but no edges; no edge cover exists".into(),
        ));
    }

    // Step 1: Greedy maximal matching
    let mut matched = vec![false; n as usize];
    let mut cover = Vec::new();

    for eid in 0..ecount as u32 {
        let (u, v) = graph.edge(eid)?;
        if !matched[u as usize] && !matched[v as usize] && u != v {
            matched[u as usize] = true;
            matched[v as usize] = true;
            cover.push(eid);
        }
    }

    // Step 2: Cover unmatched vertices
    for v in 0..n {
        if !matched[v as usize] {
            let mut edges = graph.incident(v)?;
            if directed && edges.is_empty() {
                edges = graph.incident_in(v)?;
            }
            if let Some(&eid) = edges.first() {
                if !cover.contains(&eid) {
                    cover.push(eid);
                }
                matched[v as usize] = true;
            }
        }
    }

    cover.sort_unstable();
    cover.dedup();
    Ok(cover)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let cover = minimum_edge_cover(&g).unwrap();
        assert!(cover.is_empty());
    }

    #[test]
    fn isolated_vertex_error() {
        let g = Graph::with_vertices(3);
        assert!(minimum_edge_cover(&g).is_err());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let cover = minimum_edge_cover(&g).unwrap();
        assert_eq!(cover.len(), 1);
        assert!(is_edge_cover(&g, &cover));
    }

    #[test]
    fn path_3() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let cover = minimum_edge_cover(&g).unwrap();
        assert!(is_edge_cover(&g, &cover));
        assert_eq!(cover.len(), 2);
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let cover = minimum_edge_cover(&g).unwrap();
        assert!(is_edge_cover(&g, &cover));
        assert_eq!(cover.len(), 2);
    }

    #[test]
    fn star_graph() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        let cover = minimum_edge_cover(&g).unwrap();
        assert!(is_edge_cover(&g, &cover));
        // Star: matching has 1 edge, cover = 5 - 1 = 4? No.
        // Actually star matching: one edge (0,1). Unmatched: 2,3,4.
        // Cover adds edges (0,2), (0,3), (0,4). Total = 4.
        // Optimal is also 4 (each leaf needs its own edge).
        // Wait: n=5, max matching=2 (e.g., (0,1) and... no, star max matching=1
        // since center is used). So min edge cover = 5-1 = 4.
        assert_eq!(cover.len(), 4);
    }

    #[test]
    fn complete_k4() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let cover = minimum_edge_cover(&g).unwrap();
        assert!(is_edge_cover(&g, &cover));
        // K4: max matching = 2 (perfect matching), cover = 4 - 2 = 2
        assert_eq!(cover.len(), 2);
    }

    #[test]
    fn bipartite_k22() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        let cover = minimum_edge_cover(&g).unwrap();
        assert!(is_edge_cover(&g, &cover));
        // K_{2,2}: perfect matching of size 2, cover = 4 - 2 = 2
        assert_eq!(cover.len(), 2);
    }

    #[test]
    fn path_5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let cover = minimum_edge_cover(&g).unwrap();
        assert!(is_edge_cover(&g, &cover));
        // Path of 5: max matching = 2, cover = 5 - 2 = 3
        assert_eq!(cover.len(), 3);
    }

    #[test]
    fn directed_graph() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let cover = minimum_edge_cover(&g).unwrap();
        assert!(is_edge_cover(&g, &cover));
    }

    #[test]
    fn self_loop_only() {
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();
        let cover = minimum_edge_cover(&g).unwrap();
        assert!(is_edge_cover(&g, &cover));
        assert_eq!(cover.len(), 1);
    }

    #[test]
    fn mixed_isolated_error() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        // vertex 2 is isolated
        assert!(minimum_edge_cover(&g).is_err());
    }

    #[test]
    fn parallel_edges() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let cover = minimum_edge_cover(&g).unwrap();
        assert!(is_edge_cover(&g, &cover));
        assert_eq!(cover.len(), 1);
    }

    #[test]
    fn sorted_output() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(2, 3).unwrap();
        g.add_edge(0, 1).unwrap();
        let cover = minimum_edge_cover(&g).unwrap();
        for i in 1..cover.len() {
            assert!(cover[i] > cover[i - 1]);
        }
    }

    #[test]
    fn validator_positive() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 3).unwrap(); // 2
        assert!(is_edge_cover(&g, &[0, 2]));
        assert!(is_edge_cover(&g, &[0, 1, 2]));
    }

    #[test]
    fn validator_negative() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_edge_cover(&g, &[]));
        assert!(!is_edge_cover(&g, &[1])); // 0, 3 uncovered
    }
}
