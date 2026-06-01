//! `C_5`-free graph predicate (ALGO-PR-102).
//!
//! A graph is `C_5`-free if it contains no induced 5-cycle (chordless
//! pentagon).
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is `C_5`-free (no induced 5-cycle).
///
/// An induced `C_5` is a set of 5 vertices forming a cycle with
/// exactly 5 edges and no chords (diagonals).
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_c5_free};
///
/// // Triangle is `C_5`-free
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_c5_free(&g).unwrap());
///
/// // 5-cycle is NOT `C_5`-free
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 0).unwrap();
/// assert!(!is_c5_free(&g).unwrap());
/// ```
pub fn is_c5_free(graph: &Graph) -> IgraphResult<bool> {
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

    // Induced C_5: a-b-c-d-e-a with exactly these 5 edges and no
    // chords (a-c, a-d, b-d, b-e, c-e all absent).
    // Strategy: for each edge (a,b), find c adjacent to b but not a,
    // then d adjacent to c but not a or b, then e adjacent to both d
    // and a but not b or c.
    for a in 0..n {
        let ai = a as usize;
        for &b in &nbrs_list[ai] {
            if b <= a {
                continue;
            }
            let bi = b as usize;
            for &c in &nbrs_list[bi] {
                if c == a {
                    continue;
                }
                let ci = c as usize;
                if adj[ai][ci] {
                    continue;
                }
                for &d in &nbrs_list[ci] {
                    if d == a || d == b {
                        continue;
                    }
                    let di = d as usize;
                    if adj[ai][di] || adj[bi][di] {
                        continue;
                    }
                    for &e in &nbrs_list[di] {
                        if e == a || e == b || e == c {
                            continue;
                        }
                        let ei = e as usize;
                        if adj[ai][ei] && !adj[bi][ei] && !adj[ci][ei] {
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
        assert!(is_c5_free(&g).unwrap());
    }

    #[test]
    fn small_graphs() {
        let g = Graph::with_vertices(4);
        assert!(is_c5_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_c5_free(&g).unwrap());
    }

    #[test]
    fn c4_is_c5_free() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_c5_free(&g).unwrap());
    }

    #[test]
    fn c5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!is_c5_free(&g).unwrap());
    }

    #[test]
    fn k5_is_c5_free() {
        // `K_5` has 5-cycles as subgraphs but no INDUCED `C_5` (all have chords)
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_c5_free(&g).unwrap());
    }

    #[test]
    fn c6_not_c5_free() {
        // `C_6` has no induced `C_5` (any 5 vertices of `C_6` induce at most `P_5`)
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 0).unwrap();
        assert!(is_c5_free(&g).unwrap());
    }

    #[test]
    fn path_c5_free() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        assert!(is_c5_free(&g).unwrap());
    }

    #[test]
    fn star_c5_free() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        assert!(is_c5_free(&g).unwrap());
    }

    #[test]
    fn petersen_not_c5_free() {
        // Petersen graph has girth 5, so it contains induced `C_5`
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
        assert!(!is_c5_free(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_c5_free(&g).unwrap());
    }

    #[test]
    fn cube_not_c5_free() {
        // Cube `Q_3` (8 vertices, bipartite, girth 4) has no odd cycles
        // so no induced `C_5`
        let mut g = Graph::with_vertices(8);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 5).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(2, 6).unwrap();
        g.add_edge(3, 7).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(4, 6).unwrap();
        g.add_edge(5, 7).unwrap();
        g.add_edge(6, 7).unwrap();
        assert!(is_c5_free(&g).unwrap());
    }

    #[test]
    fn wheel5_c5_free() {
        // Wheel W_5: hub 0 + cycle 1-2-3-4-5. The outer cycle is C_5
        // but hub 0 is adjacent to all cycle vertices, adding chords.
        // Any 5-vertex subset including the hub can't form C_5 (hub
        // connects to everything). 5-vertex subset without hub is the
        // outer C_5 itself → NOT `C_5`-free.
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
        assert!(!is_c5_free(&g).unwrap());
    }

    #[test]
    fn c5_plus_pendant_not_c5_free() {
        // `C_5` with a pendant edge: still contains `C_5` on {0..4}
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        g.add_edge(0, 5).unwrap();
        assert!(!is_c5_free(&g).unwrap());
    }
}
