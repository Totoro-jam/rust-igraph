//! `P_5`-free graph predicate (ALGO-PR-109).
//!
//! A graph is `P_5`-free if it contains no induced path on 5 vertices.
//! `P_5`-free graphs generalize cographs (which are `P_4`-free) and
//! appear in results on well-quasi-ordering and efficient algorithms.
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is `P_5`-free (no induced path on 5 vertices).
///
/// An induced `P_5` is 5 distinct vertices a-b-c-d-e forming a path
/// with edges {a-b, b-c, c-d, d-e} and no other edges among them.
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_p5_free};
///
/// // `P_4` is `P_5`-free
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_p5_free(&g).unwrap());
///
/// // `P_5` is NOT `P_5`-free
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// assert!(!is_p5_free(&g).unwrap());
/// ```
pub fn is_p5_free(graph: &Graph) -> IgraphResult<bool> {
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

    // Induced P_5: a-b-c-d-e with edges a-b, b-c, c-d, d-e and
    // no other edges among {a,b,c,d,e}:
    //   a-c, a-d, a-e, b-d, b-e, c-e all absent.
    //
    // Strategy: for each edge (b,c), find a adjacent to b but not c,
    // then d adjacent to c but not a or b, then e adjacent to d but
    // not a, b, or c.
    for b in 0..n {
        let bi = b as usize;
        for &c in &nbrs_list[bi] {
            let ci = c as usize;
            for &a in &nbrs_list[bi] {
                if a == c {
                    continue;
                }
                let ai = a as usize;
                if adj[ai][ci] {
                    continue;
                }
                // a-b edge, a not adj to c
                for &d in &nbrs_list[ci] {
                    if d == b {
                        continue;
                    }
                    let di = d as usize;
                    if adj[ai][di] || adj[bi][di] {
                        continue;
                    }
                    // c-d edge, d not adj to a or b
                    for &e in &nbrs_list[di] {
                        if e == c || e == b || e == a {
                            continue;
                        }
                        let ei = e as usize;
                        if !adj[ai][ei] && !adj[bi][ei] && !adj[ci][ei] {
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
        assert!(is_p5_free(&g).unwrap());
    }

    #[test]
    fn small_graphs() {
        let g = Graph::with_vertices(4);
        assert!(is_p5_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_p5_free(&g).unwrap());
    }

    #[test]
    fn p4_is_p5_free() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_p5_free(&g).unwrap());
    }

    #[test]
    fn p5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(!is_p5_free(&g).unwrap());
    }

    #[test]
    fn c5_not_p5_free() {
        // `C_5`: 0-1-2-3-4-0. The subpath 0-1-2-3-4 would be `P_5`
        // if 0-4 were absent. But 0-4 IS present → induced subgraph
        // on {0,1,2,3,4} has edge 0-4. Try {1,2,3,4,0}: edges 1-2,
        // 2-3, 3-4, 4-0. Is 1-0 present? No (edge is 0-1, yes it is).
        // So we have edges 1-2, 2-3, 3-4, 4-0, 0-1 → `C_5`, not `P_5`.
        // Any 5-vertex subset of `C_5` is `C_5` itself. So `C_5` is `P_5`-free.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_p5_free(&g).unwrap());
    }

    #[test]
    fn c6_not_p5_free() {
        // `C_6`: any 5 consecutive vertices form an induced `P_5`
        // e.g., {0,1,2,3,4} has edges 0-1, 1-2, 2-3, 3-4 and no
        // chord (0-4 absent in `C_6`).
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 0).unwrap();
        assert!(!is_p5_free(&g).unwrap());
    }

    #[test]
    fn k5_p5_free() {
        // `K_5`: every pair is adjacent → no induced path of length > 1
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_p5_free(&g).unwrap());
    }

    #[test]
    fn star_p5_free() {
        // Star: every path through the center has length 2. No `P_5`.
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        assert!(is_p5_free(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_p5_free(&g).unwrap());
    }

    #[test]
    fn cograph_is_p5_free() {
        // `K_2` union `K_3`: {0,1} complete, {2,3,4} complete, no edges between.
        // Cographs are `P_4`-free, hence also `P_5`-free.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(2, 4).unwrap();
        assert!(is_p5_free(&g).unwrap());
    }

    #[test]
    fn bull_not_p5_free() {
        // Bull: triangle {0,1,2} + pendants 1-3, 2-4.
        // Path 3-1-0-2-4: edges 3-1, 1-0, 0-2, 2-4. Chords: 1-2 present!
        // So {3,1,0,2,4} is not an induced `P_5` (has chord 1-2).
        // Try another path: 3-1-2-0-? No pendant from 0. So bull is `P_5`-free.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 4).unwrap();
        assert!(is_p5_free(&g).unwrap());
    }

    #[test]
    fn petersen_not_p5_free() {
        // Petersen graph: diameter 2, every pair at distance ≤ 2.
        // But it has induced `P_5` (e.g., any path through two
        // non-adjacent vertices on the outer and inner pentagons).
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
        assert!(!is_p5_free(&g).unwrap());
    }

    #[test]
    fn two_disjoint_edges_p5_free() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_p5_free(&g).unwrap());
    }
}
