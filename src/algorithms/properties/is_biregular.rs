//! Biregular graph predicate (ALGO-PR-126).
//!
//! A graph is biregular (also called semiregular bipartite) if it is
//! bipartite and all vertices in the same partition have the same
//! degree. The two partitions may have different degrees.
//!
//! Every complete bipartite graph `K_{m,n}` is biregular.
//! Every regular bipartite graph is biregular with both partition
//! degrees equal.
//!
//! Directed graphs are treated as undirected.

use crate::algorithms::properties::is_bipartite::is_bipartite;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is biregular.
///
/// A bipartite graph is biregular if every vertex in the same
/// partition has the same degree. Returns `false` for non-bipartite
/// graphs.
///
/// Directed graphs are treated as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_biregular};
///
/// // K_{2,3} is biregular: left vertices have degree 3, right have degree 2
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(0, 4).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(1, 4).unwrap();
/// assert!(is_biregular(&g).unwrap());
///
/// // Path P_4: bipartite but not biregular (degrees differ within a side)
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(!is_biregular(&g).unwrap());
/// ```
pub fn is_biregular(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(true);
    }

    let bp = is_bipartite(graph)?;
    if !bp.is_bipartite {
        return Ok(false);
    }

    let mut deg_false: Option<usize> = None;
    let mut deg_true: Option<usize> = None;

    for v in 0..n {
        let d = graph.degree(v)?;
        let side = bp.types[v as usize];
        let slot = if side { &mut deg_true } else { &mut deg_false };
        match *slot {
            None => *slot = Some(d),
            Some(expected) => {
                if d != expected {
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
        assert!(is_biregular(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_biregular(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_biregular(&g).unwrap());
    }

    #[test]
    fn triangle_not_bipartite() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_biregular(&g).unwrap());
    }

    #[test]
    fn c4_biregular() {
        // C_4: bipartite, every vertex degree 2
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_biregular(&g).unwrap());
    }

    #[test]
    fn k23_biregular() {
        // K_{2,3}: left deg=3, right deg=2
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(is_biregular(&g).unwrap());
    }

    #[test]
    fn k33_biregular() {
        // K_{3,3}: regular bipartite → biregular
        let mut g = Graph::with_vertices(6);
        for i in 0..3u32 {
            for j in 3..6u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_biregular(&g).unwrap());
    }

    #[test]
    fn path_p3_biregular() {
        // P_3: 0-1-2, bipartite {0,2} vs {1}. Deg of {0,2}=1, deg of {1}=2 → biregular
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_biregular(&g).unwrap());
    }

    #[test]
    fn path_p4_not_biregular() {
        // P_4: 0-1-2-3, bipartite {0,2} vs {1,3}. Deg(0)=1, deg(2)=2 → not biregular
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_biregular(&g).unwrap());
    }

    #[test]
    fn star_s4_biregular() {
        // Star S_4: center deg=4, leaves deg=1. Bipartite with each side uniform.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_biregular(&g).unwrap());
    }

    #[test]
    fn edgeless_biregular() {
        // All isolated vertices: bipartite, all degrees 0
        let g = Graph::with_vertices(5);
        assert!(is_biregular(&g).unwrap());
    }

    #[test]
    fn disconnected_biregular() {
        // Two disjoint K_{1,2}: each star has center deg=2, leaves deg=1
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(3, 5).unwrap();
        assert!(is_biregular(&g).unwrap());
    }

    #[test]
    fn disconnected_not_biregular() {
        // K_{1,1} + K_{1,2}: one side has deg 1 and 2
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(2, 4).unwrap();
        // Side false: {0, 2} with degrees 1 and 2 → not biregular
        // (depends on bipartition assignment, but regardless, mixed degrees on one side)
        // Actually, the bipartition may put them on different sides.
        // Let's think: component 0-1: sides {0} and {1}. Component 2-3-4: sides {2} and {3,4}.
        // BFS from 0: 0→false, 1→true. BFS from 2: 2→false, 3→true, 4→true.
        // false side: {0, 2} with deg 1, 2. Not biregular.
        assert!(!is_biregular(&g).unwrap());
    }

    #[test]
    fn cube_q3_biregular() {
        // Q_3 (3-cube): 3-regular bipartite → biregular
        let mut g = Graph::with_vertices(8);
        let edges = [
            (0, 1),
            (0, 2),
            (0, 4),
            (1, 3),
            (1, 5),
            (2, 3),
            (2, 6),
            (3, 7),
            (4, 5),
            (4, 6),
            (5, 7),
            (6, 7),
        ];
        for (u, v) in edges {
            g.add_edge(u, v).unwrap();
        }
        assert!(is_biregular(&g).unwrap());
    }

    #[test]
    fn directed_treated_as_undirected() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(0, 3).unwrap();
        // Undirected view: C_4 bipartite, all deg 2 → biregular
        assert!(is_biregular(&g).unwrap());
    }
}
