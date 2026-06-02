//! Strongly chordal graph predicate (ALGO-PR-113).
//!
//! A graph is strongly chordal if it is chordal and every even cycle
//! of length ≥ 6 has an odd chord. Equivalently, a chordal graph
//! that admits a strong elimination ordering (SEO): a perfect
//! elimination ordering where for each vertex v, the later neighbors
//! of v have their neighborhoods (restricted to later vertices)
//! totally ordered by inclusion.
//!
//! Directed graphs are treated as undirected.

use crate::algorithms::chordality::{is_chordal, maximum_cardinality_search};
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is strongly chordal.
///
/// A strongly chordal graph is chordal with every even cycle having
/// an odd chord. Uses maximum cardinality search to find a PEO, then
/// verifies the strong elimination property.
///
/// Directed graphs are treated as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_strongly_chordal};
///
/// // Complete graph is strongly chordal
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 {
///     for j in (i+1)..4 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// assert!(is_strongly_chordal(&g).unwrap());
///
/// // Sun graph `S_3` is chordal but NOT strongly chordal
/// let mut g = Graph::with_vertices(6);
/// // Inner triangle: 0-1-2
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// // Outer vertices 3,4,5 each adjacent to one edge of triangle
/// g.add_edge(3, 0).unwrap();
/// g.add_edge(3, 1).unwrap();
/// g.add_edge(4, 1).unwrap();
/// g.add_edge(4, 2).unwrap();
/// g.add_edge(5, 2).unwrap();
/// g.add_edge(5, 0).unwrap();
/// assert!(!is_strongly_chordal(&g).unwrap());
/// ```
pub fn is_strongly_chordal(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    if n <= 3 {
        let chordal = is_chordal(graph, None)?;
        return Ok(chordal.chordal);
    }

    let chordal = is_chordal(graph, None)?;
    if !chordal.chordal {
        return Ok(false);
    }

    let n_usize = n as usize;
    let mut adj = vec![vec![false; n_usize]; n_usize];
    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            adj[v as usize][w as usize] = true;
            adj[w as usize][v as usize] = true;
        }
    }

    // Get PEO from maximum cardinality search
    let mcs = maximum_cardinality_search(graph)?;
    // alpham1[i] is the vertex with rank i; visiting in reverse gives PEO
    let order = &mcs.alpham1;

    // Build position map: position[v] = index in elimination order
    let mut position = vec![0usize; n_usize];
    for (i, &v) in order.iter().enumerate() {
        position[v as usize] = i;
    }

    // Check strong elimination property: for each vertex v at position i,
    // the later neighbors of v (those with position > i) must have their
    // neighborhoods (restricted to vertices with position > i) totally
    // ordered by inclusion.
    for (i, &vi) in order.iter().enumerate() {
        let v = vi as usize;
        let later_nbrs: Vec<usize> = (0..n_usize)
            .filter(|&u| adj[v][u] && position[u] > i)
            .collect();

        if later_nbrs.len() <= 1 {
            continue;
        }

        // For each pair of later neighbors, check inclusion of their
        // later neighborhoods
        for (a_idx, &a) in later_nbrs.iter().enumerate() {
            for &b in &later_nbrs[(a_idx + 1)..] {
                let nbrs_a: Vec<usize> = (0..n_usize)
                    .filter(|&x| adj[a][x] && position[x] > i)
                    .collect();
                let nbrs_b: Vec<usize> = (0..n_usize)
                    .filter(|&x| adj[b][x] && position[x] > i)
                    .collect();

                // Check if nbrs_a ⊆ nbrs_b or nbrs_b ⊆ nbrs_a
                let a_sub_b = nbrs_a.iter().all(|x| adj[b][*x] || *x == b);
                let b_sub_a = nbrs_b.iter().all(|x| adj[a][*x] || *x == a);

                if !a_sub_b && !b_sub_a {
                    return Ok(false);
                }
            }
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
        assert!(is_strongly_chordal(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_strongly_chordal(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_strongly_chordal(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_strongly_chordal(&g).unwrap());
    }

    #[test]
    fn complete_k4() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_strongly_chordal(&g).unwrap());
    }

    #[test]
    fn complete_k5() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_strongly_chordal(&g).unwrap());
    }

    #[test]
    fn tree() {
        // Trees are strongly chordal (chordal + no cycles at all)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_strongly_chordal(&g).unwrap());
    }

    #[test]
    fn sun_3_not_strongly_chordal() {
        // Sun graph S_3: inner triangle 0-1-2, outer vertices 3,4,5
        // 3 adj to 0,1; 4 adj to 1,2; 5 adj to 2,0
        // This is chordal but NOT strongly chordal
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(3, 1).unwrap();
        g.add_edge(4, 1).unwrap();
        g.add_edge(4, 2).unwrap();
        g.add_edge(5, 2).unwrap();
        g.add_edge(5, 0).unwrap();
        assert!(!is_strongly_chordal(&g).unwrap());
    }

    #[test]
    fn c4_not_chordal() {
        // C_4 is not chordal → not strongly chordal
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_strongly_chordal(&g).unwrap());
    }

    #[test]
    fn star() {
        // Star is a tree → strongly chordal
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_strongly_chordal(&g).unwrap());
    }

    #[test]
    fn path() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_strongly_chordal(&g).unwrap());
    }

    #[test]
    fn diamond_strongly_chordal() {
        // Diamond = K_4 minus one edge. It's chordal and strongly chordal.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        // missing 2-3
        assert!(is_strongly_chordal(&g).unwrap());
    }

    #[test]
    fn edgeless() {
        let g = Graph::with_vertices(4);
        assert!(is_strongly_chordal(&g).unwrap());
    }

    #[test]
    fn directed_treated_as_undirected() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_strongly_chordal(&g).unwrap());
    }
}
