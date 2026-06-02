//! Cubic (3-regular) graph predicate (ALGO-PR-123).
//!
//! A graph is cubic if every vertex has degree exactly 3. Cubic
//! graphs are also called 3-regular or trivalent graphs.
//!
//! The Petersen graph, `K_{3,3}`, and the prism graph are cubic.
//! `K_4` is cubic (each of 4 vertices has degree 3).
//!
//! Directed graphs: returns `false` (degree is ambiguous for
//! directed graphs without specifying in/out/all).

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is cubic (3-regular).
///
/// A graph is cubic if every vertex has degree exactly 3.
/// Returns `false` for directed graphs, empty graphs, or graphs
/// with fewer than 4 vertices (need at least 4 vertices for a
/// cubic graph).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_cubic};
///
/// // `K_4` is cubic: each vertex has degree 3
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 {
///     for j in (i+1)..4 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// assert!(is_cubic(&g).unwrap());
///
/// // `C_4` is 2-regular, not cubic
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// assert!(!is_cubic(&g).unwrap());
/// ```
pub fn is_cubic(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n < 4 {
        return Ok(false);
    }

    for v in 0..n {
        if graph.degree(v)? != 3 {
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
        assert!(!is_cubic(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(!is_cubic(&g).unwrap());
    }

    #[test]
    fn two_vertices() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(!is_cubic(&g).unwrap());
    }

    #[test]
    fn triangle() {
        // 2-regular, not cubic
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_cubic(&g).unwrap());
    }

    #[test]
    fn k4_cubic() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_cubic(&g).unwrap());
    }

    #[test]
    fn c4_not_cubic() {
        // 2-regular
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_cubic(&g).unwrap());
    }

    #[test]
    fn k33_cubic() {
        // K_{3,3}: each vertex has degree 3
        let mut g = Graph::with_vertices(6);
        for i in 0..3u32 {
            for j in 3..6u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_cubic(&g).unwrap());
    }

    #[test]
    fn petersen_graph() {
        // Petersen graph: 10 vertices, each degree 3
        let mut g = Graph::with_vertices(10);
        // Outer cycle
        for i in 0..5u32 {
            g.add_edge(i, (i + 1) % 5).unwrap();
        }
        // Inner pentagram
        for i in 0..5u32 {
            g.add_edge(i + 5, ((i + 2) % 5) + 5).unwrap();
        }
        // Spokes
        for i in 0..5u32 {
            g.add_edge(i, i + 5).unwrap();
        }
        assert!(is_cubic(&g).unwrap());
    }

    #[test]
    fn prism_graph() {
        // Prism (triangular prism): 6 vertices, each degree 3
        let mut g = Graph::with_vertices(6);
        // Two triangles
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        // Connecting edges
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        assert!(is_cubic(&g).unwrap());
    }

    #[test]
    fn star_not_cubic() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(!is_cubic(&g).unwrap());
    }

    #[test]
    fn k5_not_cubic() {
        // K_5: 4-regular, not cubic
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(!is_cubic(&g).unwrap());
    }

    #[test]
    fn edgeless() {
        let g = Graph::with_vertices(5);
        assert!(!is_cubic(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(4, true).unwrap();
        for i in 0..4u32 {
            for j in 0..4 {
                if i != j {
                    g.add_edge(i, j).unwrap();
                }
            }
        }
        assert!(!is_cubic(&g).unwrap());
    }
}
