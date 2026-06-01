//! `C_4`-free graph predicate (ALGO-PR-101).
//!
//! A graph is `C_4`-free if it contains no induced 4-cycle (also
//! called a chordless 4-cycle or induced square).
//!
//! Note: this checks for *induced* `C_4`, not just any 4-cycle as a
//! subgraph. A `K_4` contains 4-cycles as subgraphs but no
//! *induced* `C_4` (every 4-cycle in `K_4` has a chord).
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is `C_4`-free (no induced 4-cycle).
///
/// An induced `C_4` is a set of 4 vertices {a, b, c, d} with edges
/// {ab, bc, cd, da} and no diagonals (ac, bd both absent).
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_c4_free};
///
/// // Triangle is `C_4`-free
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_c4_free(&g).unwrap());
///
/// // 4-cycle is NOT `C_4`-free
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// assert!(!is_c4_free(&g).unwrap());
/// ```
pub fn is_c4_free(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n < 4 {
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

    // An induced C_4 is {a, b, c, d} with edges ab, bc, cd, da and
    // NO edges ac or bd.
    // Strategy: for each pair (a, b) that are adjacent, and each pair
    // (c, d) where c is adjacent to b but not a, d is adjacent to a
    // and c but not b → induced C_4: a-b-c-d-a.
    for a in 0..n {
        let ai = a as usize;
        for &b in &nbrs_list[ai] {
            if b <= a {
                continue;
            }
            let bi = b as usize;
            // Find c: adjacent to b, not adjacent to a, c > a (avoid double-counting)
            for &c in &nbrs_list[bi] {
                if c == a {
                    continue;
                }
                let ci = c as usize;
                if adj[ai][ci] {
                    continue;
                }
                // Find d: adjacent to both c and a, not adjacent to b,
                // d != b, d != c, d > b (avoid double-counting)
                for &d in &nbrs_list[ci] {
                    if d == b || d == c || d == a {
                        continue;
                    }
                    let di = d as usize;
                    if adj[ai][di] && !adj[bi][di] {
                        return Ok(false);
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
        assert!(is_c4_free(&g).unwrap());
    }

    #[test]
    fn small_graphs() {
        let g = Graph::with_vertices(3);
        assert!(is_c4_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_c4_free(&g).unwrap());
    }

    #[test]
    fn c4() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_c4_free(&g).unwrap());
    }

    #[test]
    fn k4_is_c4_free() {
        // K_4 has 4-cycles as subgraphs but no INDUCED C_4 (all have chords)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_c4_free(&g).unwrap());
    }

    #[test]
    fn c5_not_c4_free() {
        // C_5: 0-1-2-3-4-0. Vertices {0,1,2,4}: edges 0-1, 1-2, 4-0.
        // Missing: 2-4 and 0-2 and 1-4. That's only 3 edges, not a C_4.
        // Actually C_5 has no induced C_4 (any 4 vertices of C_5 induce
        // at most a P_4, not a C_4).
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_c4_free(&g).unwrap());
    }

    #[test]
    fn c6_not_c4_free() {
        // C_6: 0-1-2-3-4-5-0. Vertices {0,1,3,4}: edges 0-1, 3-4.
        // Missing 0-3, 0-4, 1-3, 1-4. Only 2 edges, not C_4.
        // Actually, {0,1,2,3}: edges 0-1, 1-2, 2-3. Missing 0-3. P_4 not C_4.
        // C_6 has no induced C_4 either.
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 0).unwrap();
        assert!(is_c4_free(&g).unwrap());
    }

    #[test]
    fn path_c4_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_c4_free(&g).unwrap());
    }

    #[test]
    fn star_c4_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_c4_free(&g).unwrap());
    }

    #[test]
    fn cube_not_c4_free() {
        // Cube graph Q_3: 8 vertices, bipartite, contains many C_4s
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
        assert!(!is_c4_free(&g).unwrap());
    }

    #[test]
    fn k33_not_c4_free() {
        // K_{3,3}: complete bipartite, full of C_4s
        let mut g = Graph::with_vertices(6);
        for i in 0..3u32 {
            for j in 3..6u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(!is_c4_free(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_c4_free(&g).unwrap());
    }

    #[test]
    fn diamond_c4_free() {
        // Diamond: K_4 minus edge 2-3. No induced C_4.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_c4_free(&g).unwrap());
    }

    #[test]
    fn petersen_not_c4_free() {
        // Petersen graph contains induced C_4? No — Petersen is
        // triangle-free and has girth 5, so no C_3 or C_4.
        // Actually girth 5 means shortest cycle is 5, so no C_4.
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
        assert!(is_c4_free(&g).unwrap());
    }
}
