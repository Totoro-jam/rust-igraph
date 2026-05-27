//! Clique algorithms (ALGO-CL-001).
//!
//! Bron-Kerbosch algorithm with pivot for enumerating maximal cliques,
//! plus convenience functions for clique number and largest cliques.

use crate::core::{Graph, IgraphResult, VertexId};

/// Returns the clique number of the graph (size of the largest clique).
///
/// A clique is a complete subgraph. The clique number is the vertex count
/// of the largest clique. For a graph with no edges, the clique number is 1
/// (each vertex is a clique of size 1). For an empty graph (no vertices),
/// returns 0.
///
/// Edge directions are ignored for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, clique_number};
///
/// // K4 (complete graph on 4 vertices)
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 {
///     for j in (i+1)..4 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// assert_eq!(clique_number(&g).unwrap(), 4);
/// ```
pub fn clique_number(graph: &Graph) -> IgraphResult<u32> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0);
    }

    let adj = build_neighbor_set(graph)?;
    let mut max_size: u32 = 0;

    let all_vertices: Vec<VertexId> = (0..n).collect();
    bron_kerbosch_max(
        &adj,
        &mut Vec::new(),
        &mut all_vertices.clone(),
        &mut Vec::new(),
        &mut max_size,
    );

    Ok(max_size)
}

/// Returns all maximal cliques in the graph.
///
/// A maximal clique is a clique that cannot be extended by adding another
/// adjacent vertex. Uses the Bron-Kerbosch algorithm with pivoting.
///
/// Edge directions are ignored for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, maximal_cliques};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let cliques = maximal_cliques(&g).unwrap();
/// // Path graph: each edge is a maximal clique
/// assert_eq!(cliques.len(), 3);
/// ```
pub fn maximal_cliques(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }

    let adj = build_neighbor_set(graph)?;
    let mut result: Vec<Vec<VertexId>> = Vec::new();

    let all_vertices: Vec<VertexId> = (0..n).collect();
    bron_kerbosch_all(
        &adj,
        &mut Vec::new(),
        &mut all_vertices.clone(),
        &mut Vec::new(),
        &mut result,
    );

    // Include isolated vertices as cliques of size 1
    for v in 0..n {
        if adj[v as usize].is_empty() {
            result.push(vec![v]);
        }
    }

    Ok(result)
}

/// Returns only the largest cliques in the graph.
///
/// A largest clique is a maximal clique whose size equals the clique
/// number. There may be multiple largest cliques.
///
/// Edge directions are ignored for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, largest_cliques};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let cliques = largest_cliques(&g).unwrap();
/// // The triangle {0,1,2} is the only largest clique
/// assert_eq!(cliques.len(), 1);
/// assert_eq!(cliques[0].len(), 3);
/// ```
pub fn largest_cliques(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let all = maximal_cliques(graph)?;
    let max_size = all.iter().map(Vec::len).max().unwrap_or(0);
    Ok(all.into_iter().filter(|c| c.len() == max_size).collect())
}

/// Bron-Kerbosch with pivot — only tracks maximum clique size.
fn bron_kerbosch_max(
    adj: &[Vec<VertexId>],
    r_clique: &mut Vec<VertexId>,
    p_candidates: &mut Vec<VertexId>,
    x_excluded: &mut Vec<VertexId>,
    max_size: &mut u32,
) {
    if p_candidates.is_empty() && x_excluded.is_empty() {
        #[allow(clippy::cast_possible_truncation)]
        let size = r_clique.len() as u32;
        if size > *max_size {
            *max_size = size;
        }
        return;
    }

    if p_candidates.is_empty() {
        return;
    }

    // Choose pivot: vertex in P ∪ X with most neighbors in P
    let pivot = choose_pivot(adj, p_candidates, x_excluded);
    let pivot_neighbors = &adj[pivot as usize];

    // Vertices in P that are NOT neighbors of pivot
    let candidates: Vec<VertexId> = p_candidates
        .iter()
        .filter(|&&v| !pivot_neighbors.contains(&v))
        .copied()
        .collect();

    for v in candidates {
        let v_neighbors = &adj[v as usize];

        r_clique.push(v);

        let mut new_p: Vec<VertexId> = p_candidates
            .iter()
            .filter(|&&u| v_neighbors.contains(&u))
            .copied()
            .collect();
        let mut new_x: Vec<VertexId> = x_excluded
            .iter()
            .filter(|&&u| v_neighbors.contains(&u))
            .copied()
            .collect();

        bron_kerbosch_max(adj, r_clique, &mut new_p, &mut new_x, max_size);

        r_clique.pop();

        p_candidates.retain(|&u| u != v);
        x_excluded.push(v);
    }
}

/// Bron-Kerbosch with pivot — collects all maximal cliques.
fn bron_kerbosch_all(
    adj: &[Vec<VertexId>],
    r_clique: &mut Vec<VertexId>,
    p_candidates: &mut Vec<VertexId>,
    x_excluded: &mut Vec<VertexId>,
    result: &mut Vec<Vec<VertexId>>,
) {
    if p_candidates.is_empty() && x_excluded.is_empty() {
        if r_clique.len() >= 2 {
            let mut clique = r_clique.clone();
            clique.sort_unstable();
            result.push(clique);
        }
        return;
    }

    if p_candidates.is_empty() {
        return;
    }

    let pivot = choose_pivot(adj, p_candidates, x_excluded);
    let pivot_neighbors = &adj[pivot as usize];

    let candidates: Vec<VertexId> = p_candidates
        .iter()
        .filter(|&&v| !pivot_neighbors.contains(&v))
        .copied()
        .collect();

    for v in candidates {
        let v_neighbors = &adj[v as usize];

        r_clique.push(v);

        let mut new_p: Vec<VertexId> = p_candidates
            .iter()
            .filter(|&&u| v_neighbors.contains(&u))
            .copied()
            .collect();
        let mut new_x: Vec<VertexId> = x_excluded
            .iter()
            .filter(|&&u| v_neighbors.contains(&u))
            .copied()
            .collect();

        bron_kerbosch_all(adj, r_clique, &mut new_p, &mut new_x, result);

        r_clique.pop();

        p_candidates.retain(|&u| u != v);
        x_excluded.push(v);
    }
}

/// Choose pivot vertex with maximum connections to P.
fn choose_pivot(
    adj: &[Vec<VertexId>],
    p_candidates: &[VertexId],
    x_excluded: &[VertexId],
) -> VertexId {
    let mut best = p_candidates[0];
    let mut best_count = 0usize;

    for &v in p_candidates.iter().chain(x_excluded.iter()) {
        let count = p_candidates
            .iter()
            .filter(|&&u| adj[v as usize].contains(&u))
            .count();
        if count > best_count {
            best_count = count;
            best = v;
        }
    }

    best
}

/// Build undirected neighbor lists (ignoring edge direction).
fn build_neighbor_set(graph: &Graph) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        if src != tgt {
            adj[src as usize].push(tgt);
            adj[tgt as usize].push(src);
        }
    }

    // Deduplicate (handles multi-edges)
    for neighbors in &mut adj {
        neighbors.sort_unstable();
        neighbors.dedup();
    }

    Ok(adj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clique_number_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(clique_number(&g).unwrap(), 0);
    }

    #[test]
    fn test_clique_number_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(clique_number(&g).unwrap(), 1);
    }

    #[test]
    fn test_clique_number_single_edge() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert_eq!(clique_number(&g).unwrap(), 2);
    }

    #[test]
    fn test_clique_number_triangle() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(clique_number(&g).unwrap(), 3);
    }

    #[test]
    fn test_clique_number_k5() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert_eq!(clique_number(&g).unwrap(), 5);
    }

    #[test]
    fn test_clique_number_directed_ignores_direction() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // Triangle when ignoring directions
        assert_eq!(clique_number(&g).unwrap(), 3);
    }

    #[test]
    fn test_maximal_cliques_path() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let cliques = maximal_cliques(&g).unwrap();
        assert_eq!(cliques.len(), 3);
        for c in &cliques {
            assert_eq!(c.len(), 2);
        }
    }

    #[test]
    fn test_maximal_cliques_triangle_plus_edge() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let cliques = maximal_cliques(&g).unwrap();
        // {0,1,2} and {2,3}
        assert_eq!(cliques.len(), 2);
        let sizes: Vec<usize> = cliques.iter().map(Vec::len).collect();
        assert!(sizes.contains(&3));
        assert!(sizes.contains(&2));
    }

    #[test]
    fn test_maximal_cliques_isolated_vertices() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        // vertices 2, 3, 4 are isolated

        let cliques = maximal_cliques(&g).unwrap();
        // {0,1} + 3 isolated = 4 cliques
        assert_eq!(cliques.len(), 4);
    }

    #[test]
    fn test_largest_cliques() {
        let mut g = Graph::with_vertices(6);
        // Two triangles sharing an edge
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(2, 4).unwrap();

        let cliques = largest_cliques(&g).unwrap();
        // Two triangles: {0,1,2} and {2,3,4}
        assert_eq!(cliques.len(), 2);
        for c in &cliques {
            assert_eq!(c.len(), 3);
        }
    }

    #[test]
    fn test_largest_cliques_k4() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }

        let cliques = largest_cliques(&g).unwrap();
        assert_eq!(cliques.len(), 1);
        assert_eq!(cliques[0].len(), 4);
    }

    #[test]
    fn test_clique_number_petersen() {
        // Petersen graph has clique number 2 (no triangles)
        let mut g = Graph::with_vertices(10);
        // Outer 5-cycle
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        // Inner pentagram
        g.add_edge(5, 7).unwrap();
        g.add_edge(7, 9).unwrap();
        g.add_edge(9, 6).unwrap();
        g.add_edge(6, 8).unwrap();
        g.add_edge(8, 5).unwrap();
        // Spokes
        g.add_edge(0, 5).unwrap();
        g.add_edge(1, 6).unwrap();
        g.add_edge(2, 7).unwrap();
        g.add_edge(3, 8).unwrap();
        g.add_edge(4, 9).unwrap();

        assert_eq!(clique_number(&g).unwrap(), 2);
    }
}
