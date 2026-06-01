//! Independent set algorithms (ALGO-VC-002).
//!
//! An independent set is a set of vertices such that no two vertices
//! in the set are adjacent. Finding a maximum independent set is
//! NP-hard; we provide a greedy heuristic.
//!
//! The greedy algorithm repeatedly picks the vertex with minimum
//! degree (among remaining vertices), adds it to the independent set,
//! and removes it and all its neighbors from the candidate pool.

use crate::core::{Graph, IgraphResult, VertexId};

/// Compute an approximate maximum independent set using a greedy
/// heuristic.
///
/// Repeatedly picks the vertex with the smallest degree among
/// remaining candidates, adds it to the independent set, then
/// removes it and all its neighbors from the candidate pool.
///
/// This is a simple greedy heuristic — it does not guarantee an
/// optimal solution, but tends to produce good results in practice.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, maximum_independent_set, is_independent_vertex_set};
///
/// // Path 0-1-2-3: maximum independent set is {0, 2} or {1, 3}
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let mis = maximum_independent_set(&g).unwrap();
/// assert!(is_independent_vertex_set(&g, &mis).unwrap());
/// assert!(mis.len() >= 2);
///
/// // Empty graph: all vertices are independent
/// let g = Graph::with_vertices(5);
/// let mis = maximum_independent_set(&g).unwrap();
/// assert_eq!(mis.len(), 5);
/// ```
#[allow(clippy::cast_possible_truncation)] // vcount/ecount ≤ u32::MAX by graph invariant
pub fn maximum_independent_set(graph: &Graph) -> IgraphResult<Vec<VertexId>> {
    let n = graph.vcount();
    let directed = graph.is_directed();

    if n == 0 {
        return Ok(Vec::new());
    }

    if graph.ecount() == 0 {
        return Ok((0..n).collect());
    }

    let mut removed = vec![false; n as usize];

    // Pre-exclude self-loop vertices (they can never be in an IS)
    for v in 0..n {
        let neighbors = all_neighbors(graph, v, directed)?;
        if neighbors.contains(&v) {
            removed[v as usize] = true;
        }
    }

    let mut result = Vec::new();
    let mut degrees = vec![0u32; n as usize];

    for v in 0..n {
        if !removed[v as usize] {
            degrees[v as usize] = effective_degree(graph, v, &removed, directed)?;
        }
    }

    loop {
        let mut best_v: Option<VertexId> = None;
        let mut best_deg = u32::MAX;

        for v in 0..n {
            if removed[v as usize] {
                continue;
            }
            let deg = degrees[v as usize];
            if deg < best_deg || (deg == best_deg && best_v.is_none_or(|b| v < b)) {
                best_deg = deg;
                best_v = Some(v);
            }
        }

        let Some(v) = best_v else { break };

        result.push(v);
        removed[v as usize] = true;

        let neighbors = all_neighbors(graph, v, directed)?;
        for &w in &neighbors {
            if !removed[w as usize] {
                removed[w as usize] = true;
            }
        }

        for u in 0..n {
            if !removed[u as usize] {
                degrees[u as usize] = effective_degree(graph, u, &removed, directed)?;
            }
        }
    }

    result.sort_unstable();
    Ok(result)
}

fn all_neighbors(graph: &Graph, v: VertexId, directed: bool) -> IgraphResult<Vec<VertexId>> {
    let mut neighbors = graph.neighbors(v)?;
    if directed {
        let in_n = graph.in_neighbors_vec(v)?;
        for w in in_n {
            if !neighbors.contains(&w) {
                neighbors.push(w);
            }
        }
    }
    Ok(neighbors)
}

fn effective_degree(
    graph: &Graph,
    v: VertexId,
    removed: &[bool],
    directed: bool,
) -> IgraphResult<u32> {
    let neighbors = all_neighbors(graph, v, directed)?;
    #[allow(clippy::cast_possible_truncation)]
    let deg = neighbors
        .iter()
        .filter(|&&w| w != v && !removed[w as usize])
        .count() as u32;
    Ok(deg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::properties::is_clique::is_independent_vertex_set;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let mis = maximum_independent_set(&g).unwrap();
        assert!(mis.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(5);
        let mis = maximum_independent_set(&g).unwrap();
        assert_eq!(mis.len(), 5);
        assert!(is_independent_vertex_set(&g, &mis).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let mis = maximum_independent_set(&g).unwrap();
        assert_eq!(mis.len(), 1);
        assert!(is_independent_vertex_set(&g, &mis).unwrap());
    }

    #[test]
    fn path_4() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let mis = maximum_independent_set(&g).unwrap();
        assert!(is_independent_vertex_set(&g, &mis).unwrap());
        assert!(mis.len() >= 2); // optimal is 2
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let mis = maximum_independent_set(&g).unwrap();
        assert!(is_independent_vertex_set(&g, &mis).unwrap());
        assert_eq!(mis.len(), 1); // triangle MIS is 1
    }

    #[test]
    fn star_graph() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        let mis = maximum_independent_set(&g).unwrap();
        assert!(is_independent_vertex_set(&g, &mis).unwrap());
        // Optimal: {1,2,3,4} (leaves) = 4; greedy picks min-degree leaf first
        assert!(mis.len() >= 2);
    }

    #[test]
    fn complete_k4() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let mis = maximum_independent_set(&g).unwrap();
        assert!(is_independent_vertex_set(&g, &mis).unwrap());
        assert_eq!(mis.len(), 1); // K4 MIS is 1
    }

    #[test]
    fn bipartite_k22() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        let mis = maximum_independent_set(&g).unwrap();
        assert!(is_independent_vertex_set(&g, &mis).unwrap());
        assert_eq!(mis.len(), 2); // K_{2,2} MIS is 2
    }

    #[test]
    fn isolated_with_edges() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        // vertex 4 is isolated
        let mis = maximum_independent_set(&g).unwrap();
        assert!(is_independent_vertex_set(&g, &mis).unwrap());
        assert!(mis.contains(&4)); // isolated vertex always in MIS
        assert!(mis.len() >= 3); // at least one from each component + isolated
    }

    #[test]
    fn directed_graph() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let mis = maximum_independent_set(&g).unwrap();
        assert!(is_independent_vertex_set(&g, &mis).unwrap());
    }

    #[test]
    fn self_loop() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        let mis = maximum_independent_set(&g).unwrap();
        assert!(is_independent_vertex_set(&g, &mis).unwrap());
        assert!(!mis.contains(&0)); // self-loop vertex can't be in IS
    }

    #[test]
    fn sorted_output() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(4, 5).unwrap();
        g.add_edge(0, 1).unwrap();
        let mis = maximum_independent_set(&g).unwrap();
        for i in 1..mis.len() {
            assert!(mis[i] > mis[i - 1]);
        }
    }

    #[test]
    fn petersen_graph() {
        // Petersen graph: 10 vertices, MIS = 4
        let mut g = Graph::with_vertices(10);
        // Outer cycle
        for i in 0..5u32 {
            g.add_edge(i, (i + 1) % 5).unwrap();
        }
        // Inner pentagram
        for i in 0..5u32 {
            g.add_edge(5 + i, 5 + (i + 2) % 5).unwrap();
        }
        // Spokes
        for i in 0..5u32 {
            g.add_edge(i, 5 + i).unwrap();
        }
        let mis = maximum_independent_set(&g).unwrap();
        assert!(is_independent_vertex_set(&g, &mis).unwrap());
        assert!(mis.len() >= 3); // optimal is 4, greedy should get at least 3
    }

    #[test]
    fn complement_of_vertex_cover() {
        // For any graph: V \ vertex_cover is an independent set
        use crate::algorithms::vertex_cover::minimum_vertex_cover;
        let mut g = Graph::with_vertices(6);
        for i in 0..5u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let cover = minimum_vertex_cover(&g).unwrap();
        let n = g.vcount();
        let complement: Vec<VertexId> = (0..n).filter(|v| !cover.contains(v)).collect();
        assert!(is_independent_vertex_set(&g, &complement).unwrap());
    }
}
