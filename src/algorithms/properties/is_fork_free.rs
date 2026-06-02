//! Fork-free graph predicate (ALGO-PR-108).
//!
//! A graph is fork-free if it contains no induced fork (also called
//! cross or star-1,1,2). The fork has 5 vertices and 4 edges: a center
//! vertex with 3 neighbors, one of which has an additional pendant.
//! Equivalently, the fork is `K_{1,3}` (claw) with one edge subdivided.
//!
//! Vertices: {c, a, b, d, e} where c-a, c-b, c-d, d-e are edges
//! (4 edges total). No other edges among these 5 vertices.
//!
//! For directed graphs, the function returns `false`.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is fork-free (no induced fork / cross).
///
/// The fork is a claw (`K_{1,3}`) with one edge subdivided: center c
/// adjacent to a, b, d, and d also adjacent to e. No other edges
/// among {c, a, b, d, e}.
///
/// Returns `false` for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_fork_free};
///
/// // Path `P_4` is fork-free
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_fork_free(&g).unwrap());
///
/// // Fork: center 0, arms 1, 2, 3, pendant 3-4
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// assert!(!is_fork_free(&g).unwrap());
/// ```
pub fn is_fork_free(graph: &Graph) -> IgraphResult<bool> {
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

    // Fork: center c with neighbors {a, b, d} where a, b, d pairwise
    // non-adjacent (induced claw), and d has a neighbor e outside
    // {c, a, b, d} not adjacent to c, a, or b.
    //
    // Strategy: for each vertex c with degree ≥ 3, find triples of
    // pairwise non-adjacent neighbors (induced claw). For each such
    // triple, check if any of the three arms has a pendant forming
    // the fork.
    for c in 0..n {
        let ci = c as usize;
        let cn = &nbrs_list[ci];
        if cn.len() < 3 {
            continue;
        }

        for (i, &a) in cn.iter().enumerate() {
            let ai = a as usize;
            for (j, &b) in cn.iter().enumerate().skip(i + 1) {
                let bi = b as usize;
                if adj[ai][bi] {
                    continue;
                }
                for &d in &cn[(j + 1)..] {
                    let di = d as usize;
                    if adj[ai][di] || adj[bi][di] {
                        continue;
                    }
                    // Induced claw: c-{a,b,d}. Check each arm for pendant.
                    if has_fork_pendant(&adj, &nbrs_list, ai, ci, bi, di) {
                        return Ok(false);
                    }
                    if has_fork_pendant(&adj, &nbrs_list, bi, ci, ai, di) {
                        return Ok(false);
                    }
                    if has_fork_pendant(&adj, &nbrs_list, di, ci, ai, bi) {
                        return Ok(false);
                    }
                }
            }
        }
    }

    Ok(true)
}

/// Check if arm vertex has a pendant neighbor outside the claw.
fn has_fork_pendant(
    adj: &[Vec<bool>],
    nbrs_list: &[Vec<u32>],
    arm: usize,
    center: usize,
    other1: usize,
    other2: usize,
) -> bool {
    for &e_u32 in &nbrs_list[arm] {
        let e = e_u32 as usize;
        if e == center || e == other1 || e == other2 {
            continue;
        }
        if !adj[center][e] && !adj[other1][e] && !adj[other2][e] {
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
        assert!(is_fork_free(&g).unwrap());
    }

    #[test]
    fn small_graphs() {
        let g = Graph::with_vertices(4);
        assert!(is_fork_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_fork_free(&g).unwrap());
    }

    #[test]
    fn fork() {
        // Center 0, arms 1,2,3, pendant 3-4
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(!is_fork_free(&g).unwrap());
    }

    #[test]
    fn claw_is_fork_free() {
        // Claw `K_{1,3}`: only 4 vertices, no room for pendant
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(is_fork_free(&g).unwrap());
    }

    #[test]
    fn claw_with_pendant_is_fork() {
        // Claw + pendant = fork
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(!is_fork_free(&g).unwrap());
    }

    #[test]
    fn k5_fork_free() {
        // `K_5`: no induced claw → no fork
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_fork_free(&g).unwrap());
    }

    #[test]
    fn path_p5_not_fork_free() {
        // `P_5`: 0-1-2-3-4. Vertex 2 has neighbors {1,3}. Only degree 2,
        // not enough for claw. No vertex has degree ≥ 3. So no claw → fork-free.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_fork_free(&g).unwrap());
    }

    #[test]
    fn star_with_subdivided_edge() {
        // Star `S_4` (center 0, leaves 1,2,3,4) with edge 0-1 subdivided:
        // 0-5-1, 0-2, 0-3, 0-4. Center 0 has neighbors {5,2,3,4} → claw
        // {0,5,2,3}. Does 5 have a pendant? 5-1 edge, 1 not adj to 0... wait,
        // we removed 0-1 and added 0-5, 5-1. So 0's neighbors are {5,2,3,4}.
        // 5's neighbors are {0,1}. Claw on {0; 5,2,3}: 5 has pendant 1.
        // 1 not adj to 0? Yes (we removed that edge). Not adj to 2,3? Yes.
        // So this IS a fork.
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 5).unwrap();
        g.add_edge(5, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(!is_fork_free(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_fork_free(&g).unwrap());
    }

    #[test]
    fn pendant_adjacent_to_another_arm() {
        // Claw center 0 arms {1,2,3}, pendant from 3: 3-4.
        // But 4 adjacent to 1 → not an induced fork (4-1 edge present)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(is_fork_free(&g).unwrap());
    }

    #[test]
    fn c5_fork_free() {
        // `C_5`: max degree 2, no claw → fork-free
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_fork_free(&g).unwrap());
    }

    #[test]
    fn star_s5_is_fork_free() {
        // `S_5`: center 0, leaves 1-5. Degree of center is 5.
        // Claw triples: e.g. {1,2,3}. Pendant of 1? 1 has degree 1 (only
        // neighbor is 0), so no pendant → fork-free.
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        assert!(is_fork_free(&g).unwrap());
    }
}
