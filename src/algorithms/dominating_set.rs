//! Dominating set algorithms (ALGO-VC-003).
//!
//! A dominating set is a subset D of vertices such that every vertex
//! is either in D or adjacent to at least one vertex in D. Finding a
//! minimum dominating set is NP-hard; we provide a greedy
//! O(ln n)-approximation.
//!
//! The greedy algorithm repeatedly picks the vertex that dominates
//! the most currently-undominated vertices (including itself).

use crate::core::{Graph, IgraphResult, VertexId};

/// Check whether a set of vertices forms a valid dominating set.
///
/// A dominating set is valid if every vertex in the graph is either
/// in the set or adjacent to at least one vertex in the set.
///
/// For directed graphs, a vertex v is considered dominated by a
/// vertex u in the set if there is an edge in either direction
/// between u and v.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_dominating_set};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
///
/// assert!(is_dominating_set(&g, &[1, 3]));
/// assert!(!is_dominating_set(&g, &[0, 4]));
/// ```
pub fn is_dominating_set(graph: &Graph, dom_set: &[VertexId]) -> bool {
    let n = graph.vcount() as usize;
    if n == 0 {
        return true;
    }

    let mut dominated = vec![false; n];
    let directed = graph.is_directed();

    for &v in dom_set {
        if (v as usize) >= n {
            continue;
        }
        dominated[v as usize] = true;

        if let Ok(neighbors) = graph.neighbors(v) {
            for &w in &neighbors {
                dominated[w as usize] = true;
            }
        }
        if directed {
            if let Ok(in_neighbors) = graph.in_neighbors_vec(v) {
                for &w in &in_neighbors {
                    dominated[w as usize] = true;
                }
            }
        }
    }

    dominated.iter().all(|&d| d)
}

/// Compute an approximate minimum dominating set using a greedy
/// heuristic.
///
/// Repeatedly picks the vertex that dominates the most
/// currently-undominated vertices (a vertex dominates itself and
/// all its neighbors). This greedy approach yields an
/// O(ln n)-approximation to the optimal minimum dominating set.
///
/// For directed graphs, adjacency considers edges in both
/// directions.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, minimum_dominating_set, is_dominating_set};
///
/// // Path 0-1-2-3-4: {1, 3} dominates all
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// let ds = minimum_dominating_set(&g).unwrap();
/// assert!(is_dominating_set(&g, &ds));
/// assert!(ds.len() <= 4); // greedy should do much better
///
/// // Isolated vertices: every vertex must be in the set
/// let g = Graph::with_vertices(3);
/// let ds = minimum_dominating_set(&g).unwrap();
/// assert_eq!(ds.len(), 3);
/// ```
#[allow(clippy::cast_possible_truncation)] // vcount ≤ u32::MAX by graph invariant
pub fn minimum_dominating_set(graph: &Graph) -> IgraphResult<Vec<VertexId>> {
    let n = graph.vcount();
    let directed = graph.is_directed();

    if n == 0 {
        return Ok(Vec::new());
    }

    let mut dominated = vec![false; n as usize];
    let mut undominated_count = n;
    let mut result = Vec::new();

    // Precompute closed neighborhoods (vertex + all neighbors)
    let mut closed_neighborhoods: Vec<Vec<VertexId>> = Vec::with_capacity(n as usize);
    for v in 0..n {
        let mut cn = vec![v];
        let out_n = graph.neighbors(v)?;
        for &w in &out_n {
            if !cn.contains(&w) {
                cn.push(w);
            }
        }
        if directed {
            let in_n = graph.in_neighbors_vec(v)?;
            for &w in &in_n {
                if !cn.contains(&w) {
                    cn.push(w);
                }
            }
        }
        closed_neighborhoods.push(cn);
    }

    while undominated_count > 0 {
        let mut best_v = 0u32;
        let mut best_gain = 0u32;

        for v in 0..n {
            if dominated[v as usize] {
                // Already dominated, but could still dominate others
            }
            let gain = closed_neighborhoods[v as usize]
                .iter()
                .filter(|&&w| !dominated[w as usize])
                .count() as u32;
            if gain > best_gain || (gain == best_gain && v < best_v) {
                best_gain = gain;
                best_v = v;
            }
        }

        if best_gain == 0 {
            break;
        }

        result.push(best_v);
        for &w in &closed_neighborhoods[best_v as usize] {
            if !dominated[w as usize] {
                dominated[w as usize] = true;
                undominated_count -= 1;
            }
        }
    }

    result.sort_unstable();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let ds = minimum_dominating_set(&g).unwrap();
        assert!(ds.is_empty());
        assert!(is_dominating_set(&g, &ds));
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let ds = minimum_dominating_set(&g).unwrap();
        assert_eq!(ds.len(), 1);
        assert!(is_dominating_set(&g, &ds));
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(5);
        let ds = minimum_dominating_set(&g).unwrap();
        assert_eq!(ds.len(), 5);
        assert!(is_dominating_set(&g, &ds));
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let ds = minimum_dominating_set(&g).unwrap();
        assert_eq!(ds.len(), 1);
        assert!(is_dominating_set(&g, &ds));
    }

    #[test]
    fn path_5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let ds = minimum_dominating_set(&g).unwrap();
        assert!(is_dominating_set(&g, &ds));
        assert!(ds.len() <= 3); // optimal is 2
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let ds = minimum_dominating_set(&g).unwrap();
        assert!(is_dominating_set(&g, &ds));
        assert_eq!(ds.len(), 1);
    }

    #[test]
    fn star_graph() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        let ds = minimum_dominating_set(&g).unwrap();
        assert!(is_dominating_set(&g, &ds));
        assert_eq!(ds.len(), 1); // center dominates all
    }

    #[test]
    fn complete_k4() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let ds = minimum_dominating_set(&g).unwrap();
        assert!(is_dominating_set(&g, &ds));
        assert_eq!(ds.len(), 1);
    }

    #[test]
    fn bipartite_k22() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        let ds = minimum_dominating_set(&g).unwrap();
        assert!(is_dominating_set(&g, &ds));
        assert!(ds.len() <= 2);
    }

    #[test]
    fn directed_graph() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let ds = minimum_dominating_set(&g).unwrap();
        assert!(is_dominating_set(&g, &ds));
    }

    #[test]
    fn self_loop() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        let ds = minimum_dominating_set(&g).unwrap();
        assert!(is_dominating_set(&g, &ds));
    }

    #[test]
    fn isolated_with_edges() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        // vertex 4 is isolated
        let ds = minimum_dominating_set(&g).unwrap();
        assert!(is_dominating_set(&g, &ds));
        assert!(ds.contains(&4)); // isolated must be in DS
    }

    #[test]
    fn sorted_output() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(4, 5).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let ds = minimum_dominating_set(&g).unwrap();
        for i in 1..ds.len() {
            assert!(ds[i] > ds[i - 1]);
        }
    }

    #[test]
    fn validator_positive() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_dominating_set(&g, &[1, 3]));
        assert!(is_dominating_set(&g, &[0, 2, 4]));
        assert!(is_dominating_set(&g, &[0, 1, 2, 3, 4]));
    }

    #[test]
    fn validator_negative() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(!is_dominating_set(&g, &[]));
        assert!(!is_dominating_set(&g, &[0, 4])); // vertex 2 not dominated
        assert!(!is_dominating_set(&g, &[2])); // vertices 0, 4 not dominated
    }

    #[test]
    fn petersen_graph() {
        let mut g = Graph::with_vertices(10);
        for i in 0..5u32 {
            g.add_edge(i, (i + 1) % 5).unwrap();
        }
        for i in 0..5u32 {
            g.add_edge(5 + i, 5 + (i + 2) % 5).unwrap();
        }
        for i in 0..5u32 {
            g.add_edge(i, 5 + i).unwrap();
        }
        let ds = minimum_dominating_set(&g).unwrap();
        assert!(is_dominating_set(&g, &ds));
        // Petersen domination number is 3
        assert!(ds.len() <= 5); // greedy should get ≤ 5
    }

    #[test]
    fn directed_domination_both_directions() {
        // 0→1, 2→1: vertex 1 is dominated by both 0 and 2 (via in-edges)
        // {1} should dominate all since 0 is in-neighbor of 1, 2 is in-neighbor of 1
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 1).unwrap();
        assert!(is_dominating_set(&g, &[1]));
    }
}
