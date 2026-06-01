//! Strongly regular graph predicate (ALGO-PR-082).
//!
//! A graph is strongly regular with parameters (n, k, λ, μ) if:
//! - It has n vertices and is k-regular,
//! - Every pair of adjacent vertices has exactly λ common neighbors,
//! - Every pair of non-adjacent vertices has exactly μ common neighbors.
//!
//! Complete graphs `K_n` are trivially strongly regular (μ is undefined).
//! Edgeless graphs are trivially strongly regular (λ is undefined).
//! For non-trivial strongly regular graphs, 0 < k < n-1.
//!
//! For directed graphs, the function returns `false`.
//! For graphs with self-loops or multi-edges, the function returns `false`.

use crate::algorithms::properties::is_simple::is_simple;
use crate::core::{Graph, IgraphResult};

/// Result of strongly regular graph recognition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StronglyRegularParams {
    /// Number of vertices.
    pub n: u32,
    /// Degree of each vertex.
    pub k: u32,
    /// Number of common neighbors for each pair of adjacent vertices.
    pub lambda: u32,
    /// Number of common neighbors for each pair of non-adjacent vertices.
    pub mu: u32,
}

/// Check whether a graph is strongly regular.
///
/// Returns `Some(params)` if the graph is strongly regular, `None`
/// otherwise.
///
/// Returns `None` for directed graphs and non-simple graphs.
///
/// Complete graphs and edgeless graphs are considered strongly
/// regular (with degenerate parameters).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_strongly_regular};
///
/// // Petersen graph: strongly regular (10, 3, 0, 1)
/// let mut g = Graph::with_vertices(10);
/// // Outer cycle
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 0).unwrap();
/// // Inner pentagram
/// g.add_edge(5, 7).unwrap();
/// g.add_edge(7, 9).unwrap();
/// g.add_edge(9, 6).unwrap();
/// g.add_edge(6, 8).unwrap();
/// g.add_edge(8, 5).unwrap();
/// // Spokes
/// g.add_edge(0, 5).unwrap();
/// g.add_edge(1, 6).unwrap();
/// g.add_edge(2, 7).unwrap();
/// g.add_edge(3, 8).unwrap();
/// g.add_edge(4, 9).unwrap();
/// let params = is_strongly_regular(&g).unwrap().unwrap();
/// assert_eq!(params.n, 10);
/// assert_eq!(params.k, 3);
/// assert_eq!(params.lambda, 0);
/// assert_eq!(params.mu, 1);
///
/// // Path P4 is NOT strongly regular (not regular)
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_strongly_regular(&g).unwrap().is_none());
/// ```
pub fn is_strongly_regular(graph: &Graph) -> IgraphResult<Option<StronglyRegularParams>> {
    if graph.is_directed() {
        return Ok(None);
    }

    let n = graph.vcount();
    if n == 0 {
        return Ok(Some(StronglyRegularParams {
            n: 0,
            k: 0,
            lambda: 0,
            mu: 0,
        }));
    }

    if !is_simple(graph)? {
        return Ok(None);
    }

    // Check regularity
    let deg0 = graph.neighbors(0)?.len();
    for v in 1..n {
        let dv = graph.neighbors(v)?.len();
        if dv != deg0 {
            return Ok(None);
        }
    }
    let k = u32::try_from(deg0).unwrap_or(u32::MAX);

    // Degenerate cases
    if k == 0 {
        // Edgeless graph: strongly regular with λ=0, μ=0
        return Ok(Some(StronglyRegularParams {
            n,
            k: 0,
            lambda: 0,
            mu: 0,
        }));
    }
    if k == n.saturating_sub(1) {
        // Complete graph: strongly regular with λ=n-2, μ=0 (no non-adj pairs)
        return Ok(Some(StronglyRegularParams {
            n,
            k,
            lambda: n.saturating_sub(2),
            mu: 0,
        }));
    }

    // Build adjacency sets for O(1) lookup
    let n_usize = n as usize;
    let mut adj = vec![vec![false; n_usize]; n_usize];
    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for w in nbrs {
            adj[v as usize][w as usize] = true;
        }
    }

    // Check λ and μ parameters
    let mut lambda: Option<u32> = None;
    let mut mu: Option<u32> = None;

    for u in 0..n {
        for v in (u + 1)..n {
            let common = count_common_neighbors(&adj, u as usize, v as usize, n_usize);

            if adj[u as usize][v as usize] {
                // Adjacent pair
                match lambda {
                    None => lambda = Some(common),
                    Some(l) if l != common => return Ok(None),
                    _ => {}
                }
            } else {
                // Non-adjacent pair
                match mu {
                    None => mu = Some(common),
                    Some(m) if m != common => return Ok(None),
                    _ => {}
                }
            }
        }
    }

    Ok(Some(StronglyRegularParams {
        n,
        k,
        lambda: lambda.unwrap_or(0),
        mu: mu.unwrap_or(0),
    }))
}

fn count_common_neighbors(adj: &[Vec<bool>], u: usize, v: usize, n: usize) -> u32 {
    let mut count = 0u32;
    for (au, av) in adj[u][..n].iter().zip(adj[v][..n].iter()) {
        if *au && *av {
            count = count.saturating_add(1);
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let params = is_strongly_regular(&g).unwrap().unwrap();
        assert_eq!(params.n, 0);
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let params = is_strongly_regular(&g).unwrap().unwrap();
        assert_eq!(params.n, 1);
        assert_eq!(params.k, 0);
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let params = is_strongly_regular(&g).unwrap().unwrap();
        assert_eq!(params.n, 2);
        assert_eq!(params.k, 1);
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let params = is_strongly_regular(&g).unwrap().unwrap();
        assert_eq!(params.n, 3);
        assert_eq!(params.k, 2);
        assert_eq!(params.lambda, 1);
    }

    #[test]
    fn k4() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let params = is_strongly_regular(&g).unwrap().unwrap();
        assert_eq!(params.n, 4);
        assert_eq!(params.k, 3);
        assert_eq!(params.lambda, 2);
    }

    #[test]
    fn edgeless() {
        let g = Graph::with_vertices(5);
        let params = is_strongly_regular(&g).unwrap().unwrap();
        assert_eq!(params.n, 5);
        assert_eq!(params.k, 0);
    }

    #[test]
    fn path_not_sr() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_strongly_regular(&g).unwrap().is_none());
    }

    #[test]
    fn c5_is_sr() {
        // C5 is strongly regular (5, 2, 0, 1)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        let params = is_strongly_regular(&g).unwrap().unwrap();
        assert_eq!(params.n, 5);
        assert_eq!(params.k, 2);
        assert_eq!(params.lambda, 0);
        assert_eq!(params.mu, 1);
    }

    #[test]
    fn c4_not_sr() {
        // C4: regular but not strongly regular
        // adj pairs: common nbrs differ (0-1 share {3} if adj... wait)
        // C4: 0-1, 1-2, 2-3, 3-0
        // 0 adj 1: common = |{3}∩{2}| = 0
        // 0 adj 3: common = |{1}∩{2}| = 0
        // non-adj 0,2: common = |{1,3}∩{1,3}| = 2
        // non-adj 1,3: common = |{0,2}∩{0,2}| = 2
        // λ=0, μ=2. k=2, n=4. Check: k(k-1-λ) = 2(2-1-0) = 2, μ(n-k-1) = 2(4-2-1) = 2. Equal! So C4 IS sr(4,2,0,2).
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let params = is_strongly_regular(&g).unwrap().unwrap();
        assert_eq!(params.n, 4);
        assert_eq!(params.k, 2);
        assert_eq!(params.lambda, 0);
        assert_eq!(params.mu, 2);
    }

    #[test]
    fn petersen() {
        let mut g = Graph::with_vertices(10);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        g.add_edge(5, 7).unwrap();
        g.add_edge(7, 9).unwrap();
        g.add_edge(9, 6).unwrap();
        g.add_edge(6, 8).unwrap();
        g.add_edge(8, 5).unwrap();
        g.add_edge(0, 5).unwrap();
        g.add_edge(1, 6).unwrap();
        g.add_edge(2, 7).unwrap();
        g.add_edge(3, 8).unwrap();
        g.add_edge(4, 9).unwrap();
        let params = is_strongly_regular(&g).unwrap().unwrap();
        assert_eq!(params.n, 10);
        assert_eq!(params.k, 3);
        assert_eq!(params.lambda, 0);
        assert_eq!(params.mu, 1);
    }

    #[test]
    fn star_not_sr() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_strongly_regular(&g).unwrap().is_none());
    }

    #[test]
    fn c6_not_sr() {
        // C6: 2-regular
        // adj pair 0-1: common = |{5}∩{2}| = 0
        // non-adj 0-2: common = |{1,5}∩{1,3}| = 1
        // non-adj 0-3: common = |{1,5}∩{2,4}| = 0
        // μ not constant → not strongly regular
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 0).unwrap();
        assert!(is_strongly_regular(&g).unwrap().is_none());
    }

    #[test]
    fn directed_returns_none() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_strongly_regular(&g).unwrap().is_none());
    }

    #[test]
    fn paley5_is_sr() {
        // Paley(5) = C5, already tested
        // Let's test complement of Petersen = Kneser(5,2)
        // which is also strongly regular (10, 6, 3, 4)
        let mut g = Graph::with_vertices(10);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        g.add_edge(5, 7).unwrap();
        g.add_edge(7, 9).unwrap();
        g.add_edge(9, 6).unwrap();
        g.add_edge(6, 8).unwrap();
        g.add_edge(8, 5).unwrap();
        g.add_edge(0, 5).unwrap();
        g.add_edge(1, 6).unwrap();
        g.add_edge(2, 7).unwrap();
        g.add_edge(3, 8).unwrap();
        g.add_edge(4, 9).unwrap();
        let comp = crate::algorithms::operators::complementer::complementer(&g, false).unwrap();
        let params = is_strongly_regular(&comp).unwrap().unwrap();
        assert_eq!(params.n, 10);
        assert_eq!(params.k, 6);
        assert_eq!(params.lambda, 3);
        assert_eq!(params.mu, 4);
    }

    #[test]
    fn two_k3_is_sr() {
        // Two disjoint K3: 2-regular, λ=1 (adj pairs share 1 common nbr),
        // μ=0 (cross-component pairs share 0). sr(6, 2, 1, 0).
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        let params = is_strongly_regular(&g).unwrap().unwrap();
        assert_eq!(params.n, 6);
        assert_eq!(params.k, 2);
        assert_eq!(params.lambda, 1);
        assert_eq!(params.mu, 0);
    }

    #[test]
    fn cube_not_sr() {
        // Cube graph (Q3): 3-regular, 8 vertices
        // adj pair (0,1): N(0)={1,3,4}, N(1)={0,2,5}, common={}→0
        // non-adj (0,2): N(0)={1,3,4}, N(2)={1,3,6}, common={1,3}→2
        // non-adj (0,5): N(0)={1,3,4}, N(5)={1,4,6}, common={1,4}→2
        // non-adj (0,6): N(0)={1,3,4}, N(6)={2,5,7}, common={}→0
        // μ not constant (0 and 2) → not sr
        let mut g = Graph::with_vertices(8);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 6).unwrap();
        g.add_edge(6, 7).unwrap();
        g.add_edge(7, 4).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 5).unwrap();
        g.add_edge(2, 6).unwrap();
        g.add_edge(3, 7).unwrap();
        assert!(is_strongly_regular(&g).unwrap().is_none());
    }
}
