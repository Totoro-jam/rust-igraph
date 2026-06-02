//! Dirac's condition for Hamiltonicity (ALGO-PR-118).
//!
//! A simple undirected graph on n ≥ 3 vertices satisfies Dirac's
//! condition if every vertex has degree ≥ n/2. By Dirac's theorem
//! (1952), such a graph is Hamiltonian (contains a Hamiltonian cycle).
//!
//! Returns `false` for directed graphs, graphs with n < 3, or
//! graphs failing the degree condition.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph satisfies Dirac's condition.
///
/// Dirac's condition: every vertex has degree ≥ ⌈n/2⌉ (equivalently
/// ≥ n/2 for integer arithmetic). A graph satisfying this on n ≥ 3
/// vertices is guaranteed to be Hamiltonian.
///
/// Returns `false` for directed graphs or n < 3.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, satisfies_dirac};
///
/// // `K_4`: every vertex has degree 3 ≥ 4/2 = 2
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 {
///     for j in (i+1)..4 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// assert!(satisfies_dirac(&g).unwrap());
///
/// // `C_5`: each vertex has degree 2 < 5/2 = 2.5 → fails
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 0).unwrap();
/// assert!(!satisfies_dirac(&g).unwrap());
/// ```
pub fn satisfies_dirac(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n < 3 {
        return Ok(false);
    }

    let threshold = (n as usize).div_ceil(2);

    for v in 0..n {
        let deg = graph.degree(v)?;
        if deg < threshold {
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
        assert!(!satisfies_dirac(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(!satisfies_dirac(&g).unwrap());
    }

    #[test]
    fn two_vertices() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(!satisfies_dirac(&g).unwrap());
    }

    #[test]
    fn triangle() {
        // K_3: deg=2 ≥ 3/2=1 ✓
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(satisfies_dirac(&g).unwrap());
    }

    #[test]
    fn complete_k4() {
        // K_4: deg=3 ≥ 4/2=2 ✓
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(satisfies_dirac(&g).unwrap());
    }

    #[test]
    fn complete_k5() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(satisfies_dirac(&g).unwrap());
    }

    #[test]
    fn c4() {
        // C_4: deg=2 ≥ 4/2=2 ✓
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(satisfies_dirac(&g).unwrap());
    }

    #[test]
    fn c5_fails() {
        // C_5: deg=2 < 5/2=2 (integer: 2 < 2 is false, but 2 < 2.5)
        // threshold = 5/2 = 2 (integer division). deg=2 ≥ 2 ✓
        // Actually with integer div, 5/2 = 2, and deg=2 ≥ 2 is true.
        // Dirac's condition is deg ≥ n/2 where n/2 is real 2.5.
        // So C_5 does NOT satisfy Dirac (deg 2 < 2.5).
        // With integer: threshold should be ceil(n/2) = 3.
        // Wait, Dirac's theorem states δ(G) ≥ n/2 (real division).
        // For n=5, need deg ≥ 2.5, so min deg must be ≥ 3.
        // C_5 has deg 2 < 3. So does NOT satisfy.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!satisfies_dirac(&g).unwrap());
    }

    #[test]
    fn path_p4_fails() {
        // P_4: endpoint deg=1 < 4/2=2 → fails
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!satisfies_dirac(&g).unwrap());
    }

    #[test]
    fn star_fails() {
        // Star: leaves have deg 1 → fails
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(!satisfies_dirac(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!satisfies_dirac(&g).unwrap());
    }

    #[test]
    fn complete_bipartite_k33() {
        // K_{3,3}: deg=3 ≥ 6/2=3 ✓
        let mut g = Graph::with_vertices(6);
        for i in 0..3u32 {
            for j in 3..6u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(satisfies_dirac(&g).unwrap());
    }

    #[test]
    fn k22_satisfies() {
        // K_{2,2} = C_4: deg=2 ≥ 4/2=2 ✓
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(satisfies_dirac(&g).unwrap());
    }
}
