//! Gem-free graph predicate (ALGO-PR-098).
//!
//! A graph is gem-free if it contains no induced subgraph isomorphic
//! to the gem graph (also called the fan `F_{1,3}`): a `P_4` plus a
//! universal vertex adjacent to all four path vertices. The gem has
//! 5 vertices and 7 edges.
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is gem-free.
///
/// The gem (fan `F_{1,3}`) is `P_4` plus a vertex adjacent to all
/// four path vertices. A gem-free graph has no induced gem.
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_gem_free};
///
/// // Triangle is gem-free (too few vertices for a gem)
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_gem_free(&g).unwrap());
/// ```
pub fn is_gem_free(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n < 5 {
        return Ok(true);
    }

    let n_usize = n as usize;
    let mut adj = vec![vec![false; n_usize]; n_usize];
    let mut nbrs_list: Vec<Vec<u32>> = Vec::with_capacity(n_usize);

    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            adj[v as usize][w as usize] = true;
        }
        nbrs_list.push(nbrs);
    }

    // A gem is: vertices {u, a, b, c, d} where a-b-c-d is an induced P_4
    // and u is adjacent to all of {a, b, c, d}.
    // Strategy: for each vertex u with degree ≥ 4, check if N(u)
    // contains an induced P_4.
    for u in 0..n {
        let u_idx = u as usize;
        if nbrs_list[u_idx].len() < 4 {
            continue;
        }

        let nbrs_u = &nbrs_list[u_idx];

        // Check all ordered 4-tuples (a, b, c, d) from N(u) for induced P_4
        // a-b adjacent, b-c adjacent, c-d adjacent,
        // a-c NOT adjacent, b-d NOT adjacent, a-d NOT adjacent
        for (i, &a) in nbrs_u.iter().enumerate() {
            let ai = a as usize;
            for &b in &nbrs_u[i + 1..] {
                let bi = b as usize;
                if !adj[ai][bi] {
                    continue;
                }
                // a-b adjacent; look for c adjacent to b but not a
                for &c in nbrs_u {
                    if c == a || c == b {
                        continue;
                    }
                    let ci = c as usize;
                    if !adj[bi][ci] || adj[ai][ci] {
                        continue;
                    }
                    // a-b-c is an induced P_3; look for d adjacent to c
                    // but not a and not b
                    for &d in nbrs_u {
                        if d == a || d == b || d == c {
                            continue;
                        }
                        let di = d as usize;
                        if adj[ci][di] && !adj[bi][di] && !adj[ai][di] {
                            return Ok(false);
                        }
                    }
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
        assert!(is_gem_free(&g).unwrap());
    }

    #[test]
    fn small_graphs() {
        let g = Graph::with_vertices(4);
        assert!(is_gem_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_gem_free(&g).unwrap());
    }

    #[test]
    fn gem_graph() {
        // Gem: P_4 = 1-2-3-4, universal vertex 0
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(!is_gem_free(&g).unwrap());
    }

    #[test]
    fn k5() {
        // K_5 is gem-free: every 5-vertex induced subgraph is K_5,
        // which has too many edges for a gem
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_gem_free(&g).unwrap());
    }

    #[test]
    fn star_k15() {
        // Star K_{1,4}: vertex 0 connected to 1,2,3,4
        // N(0) = {1,2,3,4}, all mutually non-adjacent → no P_4 in N(0)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_gem_free(&g).unwrap());
    }

    #[test]
    fn c5_gem_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_gem_free(&g).unwrap());
    }

    #[test]
    fn path_p5_gem_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_gem_free(&g).unwrap());
    }

    #[test]
    fn wheel_w5_not_gem_free() {
        // W_5: hub 0, rim 1-2-3-4-5-1
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 1).unwrap();
        // Hub 0 has N(0) = {1,2,3,4,5}
        // In N(0): 1-2-3-4 is P_4 (1-2 adj, 2-3 adj, 3-4 adj,
        // 1-3 not adj, 2-4 not adj, 1-4 not adj via rim) → gem
        assert!(!is_gem_free(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_gem_free(&g).unwrap());
    }

    #[test]
    fn fan_3_plus_universal() {
        // Universal vertex 0, path 1-2-3, plus extra vertex 4 connected to 0 only
        // N(0) = {1,2,3,4}. In N(0): 1-2-3 but 4 isolated from 1,2,3
        // No P_4 in N(0) since 4 is isolated
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_gem_free(&g).unwrap());
    }
}
