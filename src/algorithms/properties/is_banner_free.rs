//! Banner-free graph predicate (ALGO-PR-106).
//!
//! A graph is banner-free if it contains no induced banner. The banner
//! (also called flag) is a `C_4` (4-cycle) with one pendant edge:
//! 5 vertices {a, b, c, d, e} with edges {a-b, b-c, c-d, d-a, a-e}
//! and no diagonals or other edges. 5 edges total.
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is banner-free (no induced banner / flag).
///
/// The banner is a `C_4` plus one pendant edge from a cycle vertex.
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_banner_free};
///
/// // `C_4` is banner-free (no pendant)
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// assert!(is_banner_free(&g).unwrap());
///
/// // Banner: `C_4` {0,1,2,3} + pendant 0-4
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// g.add_edge(0, 4).unwrap();
/// assert!(!is_banner_free(&g).unwrap());
/// ```
pub fn is_banner_free(graph: &Graph) -> IgraphResult<bool> {
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

    // Induced banner: C_4 {a,b,c,d} with edges a-b, b-c, c-d, d-a
    // (no diagonals a-c, b-d) plus pendant e adjacent to exactly one
    // cycle vertex, say a, and not adjacent to b, c, d.
    //
    // Strategy: find each induced C_4 (a-b-c-d-a with no diagonals),
    // then check if any cycle vertex has a neighbor outside the cycle
    // that is not adjacent to the other 3 cycle vertices.
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
                // a-b-c path, a and c not adjacent
                for &d in &nbrs_list[ci] {
                    if d == b || d == a {
                        continue;
                    }
                    let di = d as usize;
                    if !adj[ai][di] || adj[bi][di] {
                        continue;
                    }
                    // Induced C_4: a-b-c-d-a (diags a-c and b-d absent)
                    // Check each cycle vertex for a pendant
                    if has_pendant(&adj, &nbrs_list, ai, bi, ci, di) {
                        return Ok(false);
                    }
                    if has_pendant(&adj, &nbrs_list, bi, ai, ci, di) {
                        return Ok(false);
                    }
                    if has_pendant(&adj, &nbrs_list, ci, ai, bi, di) {
                        return Ok(false);
                    }
                    if has_pendant(&adj, &nbrs_list, di, ai, bi, ci) {
                        return Ok(false);
                    }
                }
            }
        }
    }

    Ok(true)
}

/// Check if vertex `v` has a neighbor outside {v, o1, o2, o3} that is
/// not adjacent to o1, o2, or o3.
fn has_pendant(
    adj: &[Vec<bool>],
    nbrs_list: &[Vec<u32>],
    v: usize,
    o1: usize,
    o2: usize,
    o3: usize,
) -> bool {
    for &e_u32 in &nbrs_list[v] {
        let e = e_u32 as usize;
        if e == o1 || e == o2 || e == o3 {
            continue;
        }
        if !adj[o1][e] && !adj[o2][e] && !adj[o3][e] {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_banner_free(&g).unwrap());
    }

    #[test]
    fn small_graphs() {
        let g = Graph::with_vertices(4);
        assert!(is_banner_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_banner_free(&g).unwrap());
    }

    #[test]
    fn c4_banner_free() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_banner_free(&g).unwrap());
    }

    #[test]
    fn banner() {
        // `C_4` {0,1,2,3} + pendant 0-4
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(!is_banner_free(&g).unwrap());
    }

    #[test]
    fn k5_banner_free() {
        // `K_5`: no induced `C_4` (every 4-cycle has a chord)
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_banner_free(&g).unwrap());
    }

    #[test]
    fn path_banner_free() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_banner_free(&g).unwrap());
    }

    #[test]
    fn star_banner_free() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        assert!(is_banner_free(&g).unwrap());
    }

    #[test]
    fn c5_banner_free() {
        // `C_5` has no induced `C_4` → banner-free
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_banner_free(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_banner_free(&g).unwrap());
    }

    #[test]
    fn pendant_connected_to_non_cycle_vertex() {
        // `C_4` + pendant but pendant also connects to another cycle vertex
        // → not an induced banner
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(is_banner_free(&g).unwrap());
    }

    #[test]
    fn cube_not_banner_free() {
        // Cube `Q_3` has induced `C_4`. Each `C_4` vertex has a neighbor
        // outside the cycle → likely has an induced banner.
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
        assert!(!is_banner_free(&g).unwrap());
    }

    #[test]
    fn k33_banner_free() {
        // `K_{3,3}`: has induced `C_4` but every outside vertex connects
        // to exactly 2 cycle vertices, never just 1 → banner-free
        let mut g = Graph::with_vertices(6);
        for i in 0..3u32 {
            for j in 3..6u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_banner_free(&g).unwrap());
    }

    #[test]
    fn c4_with_isolated_pendant_not_banner_free() {
        // `C_4` {0,1,2,3} + pendant 3-4 (4 adj to 3 only)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(!is_banner_free(&g).unwrap());
    }
}
