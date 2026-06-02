//! Ore's condition for Hamiltonicity (ALGO-PR-119).
//!
//! A simple undirected graph on n ≥ 3 vertices satisfies Ore's
//! condition if for every pair of non-adjacent vertices u, v:
//! deg(u) + deg(v) ≥ n. By Ore's theorem (1960), such a graph is
//! Hamiltonian. Ore's condition is weaker than Dirac's (every graph
//! satisfying Dirac also satisfies Ore, but not vice versa).
//!
//! Returns `false` for directed graphs or n < 3.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph satisfies Ore's condition.
///
/// Ore's condition: for every pair of non-adjacent vertices u, v,
/// deg(u) + deg(v) ≥ n. A graph satisfying this on n ≥ 3 vertices
/// is guaranteed to be Hamiltonian.
///
/// Returns `false` for directed graphs or n < 3.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, satisfies_ore};
///
/// // `K_4`: every non-adjacent pair... but K_4 is complete (no
/// // non-adjacent pairs), so Ore is vacuously true
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 {
///     for j in (i+1)..4 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// assert!(satisfies_ore(&g).unwrap());
///
/// // Path `P_4`: endpoints 0,3 are non-adjacent, deg(0)+deg(3)=1+1=2 < 4
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(!satisfies_ore(&g).unwrap());
/// ```
pub fn satisfies_ore(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n < 3 {
        return Ok(false);
    }

    let n_usize = n as usize;

    // Precompute degrees
    let mut degrees = Vec::with_capacity(n_usize);
    for v in 0..n {
        degrees.push(graph.degree(v)?);
    }

    // Build adjacency for non-adjacency check
    let mut adj = vec![vec![false; n_usize]; n_usize];
    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            adj[v as usize][w as usize] = true;
        }
    }

    // Check all non-adjacent pairs
    for u in 0..n_usize {
        for v in (u + 1)..n_usize {
            if !adj[u][v] && degrees[u] + degrees[v] < n_usize {
                return Ok(false);
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
        assert!(!satisfies_ore(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(!satisfies_ore(&g).unwrap());
    }

    #[test]
    fn two_vertices() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(!satisfies_ore(&g).unwrap());
    }

    #[test]
    fn triangle() {
        // K_3: complete, no non-adjacent pairs → vacuously true
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(satisfies_ore(&g).unwrap());
    }

    #[test]
    fn complete_k4() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(satisfies_ore(&g).unwrap());
    }

    #[test]
    fn c4() {
        // C_4: non-adjacent pairs (0,2) and (1,3).
        // deg(0)+deg(2) = 2+2 = 4 ≥ 4 ✓. Same for (1,3). → satisfies Ore
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(satisfies_ore(&g).unwrap());
    }

    #[test]
    fn c5_fails() {
        // C_5: each non-adjacent pair has deg sum 2+2=4 < 5 → fails
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!satisfies_ore(&g).unwrap());
    }

    #[test]
    fn path_p4_fails() {
        // P_4: endpoints deg 1, non-adjacent, sum 1+1=2 < 4 → fails
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!satisfies_ore(&g).unwrap());
    }

    #[test]
    fn star_fails() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        // Non-adj pair (1,2): deg 1+1=2 < 5 → fails
        assert!(!satisfies_ore(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!satisfies_ore(&g).unwrap());
    }

    #[test]
    fn k33_satisfies() {
        // K_{3,3}: non-adj pairs within same part, deg 3+3=6 ≥ 6 ✓
        let mut g = Graph::with_vertices(6);
        for i in 0..3u32 {
            for j in 3..6u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(satisfies_ore(&g).unwrap());
    }

    #[test]
    fn ore_but_not_dirac() {
        // Graph where Ore holds but Dirac fails: n=4, vertex 0 has
        // degree 1 (fails Dirac), but every non-adjacent pair has
        // sufficient degree sum.
        // 0-1, 1-2, 2-3, 1-3. Non-adj pairs: (0,2) deg 1+2=3<4 → fails.
        // Hard to construct Ore-but-not-Dirac on small n.
        // Try: K_4 minus one edge: 0-1, 0-2, 0-3, 1-2, 1-3, missing 2-3.
        // deg(0)=3, deg(1)=3, deg(2)=2, deg(3)=2. Only non-adj pair: (2,3).
        // 2+2=4 ≥ 4 ✓. Dirac: min deg=2 ≥ 4/2=2 ✓. Both hold.
        // Let me try n=5, K5 minus two edges: miss (3,4) and (2,4).
        // 0-1,0-2,0-3,0-4,1-2,1-3,1-4,2-3. deg: 0→4,1→4,2→3,3→3,4→2.
        // Non-adj: (2,4) sum 3+2=5≥5 ✓, (3,4) sum 3+2=5≥5 ✓. Ore ✓.
        // Dirac: min deg=2 < 5/2=3 → fails. Ore-not-Dirac!
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(satisfies_ore(&g).unwrap());
    }

    #[test]
    fn edgeless_fails() {
        let g = Graph::with_vertices(4);
        assert!(!satisfies_ore(&g).unwrap());
    }

    #[test]
    fn p3_fails() {
        // P_3 (n=3): 0-1-2. Non-adj pair (0,2): deg 1+1=2 < 3 → fails
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!satisfies_ore(&g).unwrap());
    }
}
