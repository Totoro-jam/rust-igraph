//! Trivially perfect graph predicate (ALGO-PR-094).
//!
//! A graph is trivially perfect (also called quasi-threshold) iff
//! every connected induced subgraph has a universal vertex (a vertex
//! adjacent to all others in the component). Equivalently, it has no
//! induced `P_4` or `C_4`.
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is trivially perfect.
///
/// A trivially perfect graph (quasi-threshold graph) has no induced
/// `P_4` or `C_4`. Equivalently, every connected induced subgraph
/// has a universal vertex.
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_trivially_perfect};
///
/// // Star `K_{1,3}` is trivially perfect
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// assert!(is_trivially_perfect(&g).unwrap());
///
/// // `P_4` (path of 4 vertices) is NOT trivially perfect
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(!is_trivially_perfect(&g).unwrap());
/// ```
pub fn is_trivially_perfect(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n <= 2 {
        return Ok(true);
    }

    // Check for induced P_4: for each edge (u, v), check if there exists
    // w adjacent to v but not u, and x adjacent to w but not v and not u.
    // This gives an induced path u - v - w - x.
    //
    // Also check for induced C_4: for each pair (u, v) at distance 2
    // (sharing a common neighbor), check if they have two distinct
    // common non-neighbors that form the other two vertices of a C_4.
    //
    // We use the universal-vertex characterization instead for clarity:
    // For each connected component, verify there's a vertex adjacent to
    // all others in the component, then recurse on the remaining vertices.

    // Build adjacency sets for fast lookup
    let n_usize = n as usize;
    let mut adj = vec![vec![false; n_usize]; n_usize];
    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            adj[v as usize][w as usize] = true;
        }
    }

    // Find connected components via BFS on our adjacency matrix
    let mut visited = vec![false; n_usize];
    let mut component = Vec::new();

    for start in 0..n {
        if visited[start as usize] {
            continue;
        }

        // BFS to find this component
        component.clear();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        visited[start as usize] = true;

        while let Some(v) = queue.pop_front() {
            component.push(v);
            for w in 0..n {
                if !visited[w as usize] && adj[v as usize][w as usize] {
                    visited[w as usize] = true;
                    queue.push_back(w);
                }
            }
        }

        // Check this component recursively
        if !check_component_trivially_perfect(&adj, &component) {
            return Ok(false);
        }
    }

    Ok(true)
}

fn check_component_trivially_perfect(adj: &[Vec<bool>], component: &[u32]) -> bool {
    let size = component.len();
    if size <= 2 {
        return true;
    }

    // Find a universal vertex in this component
    let mut universal = None;
    for &v in component {
        let deg_in_comp = component
            .iter()
            .filter(|&&w| w != v && adj[v as usize][w as usize])
            .count();
        if deg_in_comp == size - 1 {
            universal = Some(v);
            break;
        }
    }

    let Some(univ) = universal else {
        return false;
    };

    // Remove the universal vertex and recurse on each resulting component
    let remaining: Vec<u32> = component.iter().copied().filter(|&v| v != univ).collect();

    // Find connected components among remaining vertices
    let mut rem_visited = vec![false; remaining.len()];
    let mut sub_component = Vec::new();

    for i in 0..remaining.len() {
        if rem_visited[i] {
            continue;
        }

        sub_component.clear();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(i);
        rem_visited[i] = true;

        while let Some(idx) = queue.pop_front() {
            sub_component.push(remaining[idx]);
            for j in 0..remaining.len() {
                if !rem_visited[j] && adj[remaining[idx] as usize][remaining[j] as usize] {
                    rem_visited[j] = true;
                    queue.push_back(j);
                }
            }
        }

        if !check_component_trivially_perfect(adj, &sub_component) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn star_k14() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn complete_k4() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn p4_not_trivially_perfect() {
        // P_4: 0-1-2-3 — has an induced P_4
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn c4_not_trivially_perfect() {
        // C_4: 0-1-2-3-0 — has an induced C_4
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn c5_not_trivially_perfect() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn disconnected_trivially_perfect() {
        // Two isolated edges: trivially perfect
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn disconnected_with_p4() {
        // Triangle + P_4
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 6).unwrap();
        assert!(!is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn fan_with_p4_not_trivially_perfect() {
        // Fan: universal vertex 0 connected to path 1-2-3-4
        // Removing 0 leaves P_4 → not trivially perfect
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(!is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn fan_with_p3_trivially_perfect() {
        // Universal vertex 0 connected to path 1-2-3
        // Removing 0 leaves P_3; vertex 2 is universal in P_3.
        // Removing 2 leaves {1, 3} disconnected, each trivially perfect.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn nested_stars() {
        // 0 connected to all; 1 connected to {2, 3}
        // This is a "threshold graph" = trivially perfect
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn bull_graph_not_trivially_perfect() {
        // Bull: triangle 0-1-2 with pendant edges 1-3 and 2-4
        // Induced path: 3-1-2-4 → P_4
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 4).unwrap();
        assert!(!is_trivially_perfect(&g).unwrap());
    }

    #[test]
    fn isolated_vertices() {
        let g = Graph::with_vertices(5);
        assert!(is_trivially_perfect(&g).unwrap());
    }
}
