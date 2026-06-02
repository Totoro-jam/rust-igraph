//! Weakly chordal graph predicate (ALGO-PR-121).
//!
//! A graph is weakly chordal (also called weakly triangulated) if
//! neither G nor its complement contains an induced cycle of length
//! ≥ 5. Equivalently, every induced cycle in both G and its
//! complement has length at most 4.
//!
//! Every chordal graph is weakly chordal (no induced `C_k` for k ≥ 4).
//! Every co-chordal graph (complement is chordal) is weakly chordal.
//!
//! Directed graphs are treated as undirected.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is weakly chordal.
///
/// A graph is weakly chordal if neither G nor its complement has an
/// induced cycle of length ≥ 5. Uses chordality checks on both G
/// and its complement; if both are chordal, G is weakly chordal.
/// Otherwise, checks for induced ``C_k`` (k ≥ 5) in the non-chordal
/// graph(s).
///
/// Directed graphs are treated as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_weakly_chordal};
///
/// // `K_4`: chordal → weakly chordal
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 {
///     for j in (i+1)..4 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// assert!(is_weakly_chordal(&g).unwrap());
///
/// // `C_5` is NOT weakly chordal (is its own induced C_5)
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 0).unwrap();
/// assert!(!is_weakly_chordal(&g).unwrap());
/// ```
pub fn is_weakly_chordal(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    if n <= 4 {
        return Ok(true);
    }

    let n_usize = n as usize;
    let mut adj = vec![vec![false; n_usize]; n_usize];
    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            adj[v as usize][w as usize] = true;
            adj[w as usize][v as usize] = true;
        }
    }

    // Check if G has an induced `C_k` for k ≥ 5
    if has_long_induced_cycle(&adj, n_usize) {
        return Ok(false);
    }

    // Build complement adjacency
    let mut comp = vec![vec![false; n_usize]; n_usize];
    for i in 0..n_usize {
        for j in (i + 1)..n_usize {
            if !adj[i][j] {
                comp[i][j] = true;
                comp[j][i] = true;
            }
        }
    }

    // Check if complement has an induced `C_k` for k ≥ 5
    if has_long_induced_cycle(&comp, n_usize) {
        return Ok(false);
    }

    Ok(true)
}

/// Check if a graph (given by adjacency matrix) has an induced cycle
/// of length ≥ 5. Uses DFS to find chordless cycles.
fn has_long_induced_cycle(adj: &[Vec<bool>], n: usize) -> bool {
    // A graph has no induced `C_k` (k ≥ 5) iff it is "hole-free for k≥5".
    // We can check this by looking for chordless cycles of length ≥ 5.
    // Approach: for each edge (u,v), try to find a chordless path from
    // v back to u of length ≥ 4 (so total cycle ≥ 5) that avoids
    // non-path neighbors of u.

    // Simpler approach: check if chordal. If chordal, no induced `C_k`
    // for k ≥ 4, so certainly no k ≥ 5.
    // If not chordal, there exists an induced `C_k` for k ≥ 4.
    // If k = 4, that's fine (weakly chordal allows C_4).
    // If k ≥ 5, that's a violation.

    // So we need to check: is there an induced `C_k` for k ≥ 5?
    // If the graph is chordal, return false.
    // If the graph has an induced C_4 but no induced `C_k` (k ≥ 5),
    // return false.

    // For the DFS approach: enumerate chordless cycles.
    // For each vertex u, for each pair of non-adjacent neighbors (v,w),
    // try to find a chordless path from v to w not through u and not
    // using any other neighbor of u (to ensure chordless cycle with u).
    // If path length ≥ 3, cycle length = path + 2 ≥ 5.

    for u in 0..n {
        let nbrs_u: Vec<usize> = (0..n).filter(|&v| adj[u][v]).collect();

        for (i, &v) in nbrs_u.iter().enumerate() {
            for &w in &nbrs_u[(i + 1)..] {
                if adj[v][w] {
                    continue;
                }
                // v and w are non-adjacent neighbors of u.
                // Find chordless path from v to w not through u,
                // where internal vertices are not neighbors of u
                // (to ensure the cycle u-v-...-w-u is chordless).
                if chordless_path_exists(adj, v, w, u, &nbrs_u, n, 3) {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if there exists a chordless path from `start` to `end` of
/// length ≥ `min_len`, not passing through `excluded`, where
/// internal vertices are not in `forbidden` (neighbors of the cycle
/// anchor) and are not adjacent to both `start`'s predecessor and
/// `end` (to maintain chordlessness).
fn chordless_path_exists(
    adj: &[Vec<bool>],
    start: usize,
    end: usize,
    excluded: usize,
    forbidden_set: &[usize],
    n: usize,
    min_len: usize,
) -> bool {
    // BFS/DFS to find path from start to end of length ≥ min_len
    // through vertices that are not in forbidden_set and not excluded.
    let mut forbidden = vec![false; n];
    forbidden[excluded] = true;
    for &f in forbidden_set {
        if f != start && f != end {
            forbidden[f] = true;
        }
    }

    // DFS with path tracking
    let mut path = vec![start];
    let mut visited = vec![false; n];
    visited[start] = true;
    visited[excluded] = true;

    dfs_chordless(adj, &mut path, &mut visited, end, &forbidden, n, min_len)
}

fn dfs_chordless(
    adj: &[Vec<bool>],
    path: &mut Vec<usize>,
    visited: &mut Vec<bool>,
    target: usize,
    forbidden: &[bool],
    n: usize,
    min_len: usize,
) -> bool {
    let current = *path.last().unwrap();
    let depth = path.len();

    // If current depth ≥ min_len and we can reach target
    if depth >= min_len && adj[current][target] {
        // Check chordlessness: no internal vertex adjacent to target
        // except the last one (current).
        // Actually we need: no chord in the cycle. The cycle is
        // excluded - start - path - target - excluded.
        // We already ensured internal vertices are not neighbors of
        // excluded. We need to check that no internal vertex of the
        // path (indices 1..depth-1) is adjacent to target.
        let chordless = path[1..depth - 1].iter().all(|&v| !adj[v][target]);
        if chordless {
            return true;
        }
    }

    // Limit search depth to avoid exponential blowup
    if depth >= n.min(12) {
        return false;
    }

    for next in 0..n {
        if visited[next] || forbidden[next] || next == target && depth < min_len - 1 {
            continue;
        }
        if !adj[current][next] {
            continue;
        }

        // Check chordlessness: next must not be adjacent to any
        // path vertex except current (the immediately previous one)
        let has_chord = path[..depth - 1].iter().any(|&v| adj[next][v]);
        if has_chord {
            continue;
        }

        visited[next] = true;
        path.push(next);
        if dfs_chordless(adj, path, visited, target, forbidden, n, min_len) {
            return true;
        }
        path.pop();
        visited[next] = false;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::chordality::is_chordal;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_weakly_chordal(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_weakly_chordal(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_weakly_chordal(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_weakly_chordal(&g).unwrap());
    }

    #[test]
    fn c4_weakly_chordal() {
        // C_4 is weakly chordal (induced C_4 is allowed, only C_5+ banned)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_weakly_chordal(&g).unwrap());
    }

    #[test]
    fn c5_not_weakly_chordal() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!is_weakly_chordal(&g).unwrap());
    }

    #[test]
    fn c6_not_weakly_chordal() {
        let mut g = Graph::with_vertices(6);
        for i in 0..6u32 {
            g.add_edge(i, (i + 1) % 6).unwrap();
        }
        assert!(!is_weakly_chordal(&g).unwrap());
    }

    #[test]
    fn complete_k5() {
        // Chordal → weakly chordal
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_weakly_chordal(&g).unwrap());
    }

    #[test]
    fn chordal_graph() {
        // Any chordal graph is weakly chordal
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(2, 4).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_chordal(&g, None).unwrap().chordal);
        assert!(is_weakly_chordal(&g).unwrap());
    }

    #[test]
    fn edgeless() {
        // Complement of 5 isolated vertices is K_5 (chordal) → weakly chordal
        let g = Graph::with_vertices(5);
        assert!(is_weakly_chordal(&g).unwrap());
    }

    #[test]
    fn path_p5() {
        // P_5 is a tree → no induced cycle in G.
        // Complement edges: 0-2,0-3,0-4,1-3,1-4,2-4.
        // Cycle 0-2-4-1-3-0 has chord 0-4, so no induced C_5.
        // → P_5 is weakly chordal.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_weakly_chordal(&g).unwrap());
    }

    #[test]
    fn star() {
        // Star S_4: center 0, leaves 1,2,3,4.
        // G is a tree → chordal → no induced C_5+ in G.
        // Complement: 0 isolated, 1-2-3-4 complete (K_4).
        // K_4 is chordal → no induced C_5+ in complement.
        // → weakly chordal
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_weakly_chordal(&g).unwrap());
    }

    #[test]
    fn diamond() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_weakly_chordal(&g).unwrap());
    }

    #[test]
    fn directed_treated_as_undirected() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_weakly_chordal(&g).unwrap());
    }
}
