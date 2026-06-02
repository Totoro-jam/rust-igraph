//! Co-chordal graph predicate (ALGO-PR-124).
//!
//! A graph is co-chordal if its complement is chordal. Every
//! co-chordal graph is weakly chordal. Every complete graph is
//! co-chordal (complement is edgeless, hence chordal). Every
//! edgeless graph is co-chordal (complement is complete, hence
//! chordal).
//!
//! Directed graphs are treated as undirected.

use crate::algorithms::chordality::is_chordal;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is co-chordal (complement is chordal).
///
/// Builds the complement graph and runs the chordality test on it.
///
/// Directed graphs are treated as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_co_chordal};
///
/// // `K_5`: complement is edgeless (chordal) → co-chordal
/// let mut g = Graph::with_vertices(5);
/// for i in 0..5u32 {
///     for j in (i+1)..5 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// assert!(is_co_chordal(&g).unwrap());
///
/// // `C_5`: complement is also `C_5` → not chordal → not co-chordal
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 0).unwrap();
/// assert!(!is_co_chordal(&g).unwrap());
/// ```
pub fn is_co_chordal(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    if n <= 3 {
        return Ok(true);
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

    let mut comp = Graph::with_vertices(n);
    for v in 0..n {
        for w in (v + 1)..n {
            if !adj[v as usize][w as usize] {
                comp.add_edge(v, w)?;
            }
        }
    }

    Ok(is_chordal(&comp, None)?.chordal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_co_chordal(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_co_chordal(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_co_chordal(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_co_chordal(&g).unwrap());
    }

    #[test]
    fn k4() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_co_chordal(&g).unwrap());
    }

    #[test]
    fn k5() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_co_chordal(&g).unwrap());
    }

    #[test]
    fn edgeless_5() {
        // Complement is K_5 (chordal)
        let g = Graph::with_vertices(5);
        assert!(is_co_chordal(&g).unwrap());
    }

    #[test]
    fn c4() {
        // C_4: complement is 2 disjoint edges (K_2 + K_2), which is chordal
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_co_chordal(&g).unwrap());
    }

    #[test]
    fn c5_not_co_chordal() {
        // C_5 is self-complementary; complement is also C_5 (not chordal)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!is_co_chordal(&g).unwrap());
    }

    #[test]
    fn c6_not_co_chordal() {
        // C_6 complement has induced C_4: 0-3-5-2-0 (no chords 0-5 or 3-2).
        let mut g = Graph::with_vertices(6);
        for i in 0..6u32 {
            g.add_edge(i, (i + 1) % 6).unwrap();
        }
        assert!(!is_co_chordal(&g).unwrap());
    }

    #[test]
    fn path_p5() {
        // P_5: complement has edges 0-2,0-3,0-4,1-3,1-4,2-4.
        // 6 edges on 5 vertices. Degree: 0→3, 1→2, 2→2, 3→2, 4→3.
        // Is it chordal? Need to check for induced C_k (k≥4).
        // Try 0-2-4-1-3-0: edges 0-2✓,2-4✓,4-1✓,1-3✓,3-0✓. All present!
        // Chord: 0-4 ✓. So not an induced C_5.
        // Any induced C_4? 4 vertices. 0-2-4-1: 0-2✓,2-4✓,4-1✓,1-0? No edge.
        // 0-3-1-4: 0-3✓,3-1✓,1-4✓,4-0✓. Chord 0-1? No. Chord 3-4? No.
        // So 0-3-1-4-0 is an induced C_4! Not chordal.
        // P_5 is NOT co-chordal.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(!is_co_chordal(&g).unwrap());
    }

    #[test]
    fn star_s4() {
        // Star S_4: center 0, leaves 1,2,3,4.
        // Complement: 0 isolated, leaves form K_4 (chordal).
        // So star IS co-chordal.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_co_chordal(&g).unwrap());
    }

    #[test]
    fn directed_treated_as_undirected() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_co_chordal(&g).unwrap());
    }
}
