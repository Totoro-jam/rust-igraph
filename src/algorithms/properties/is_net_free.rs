//! Net-free graph predicate (ALGO-PR-125).
//!
//! A graph is net-free if it contains no induced net subgraph.
//! The net graph consists of a triangle with a pendant edge on
//! each vertex (6 vertices, 6 edges). It is also called the
//! 3-sun complement or `S_3`.
//!
//! Net-free graphs appear in structural graph theory: a graph
//! is claw-free and net-free iff it is a proper circular-arc
//! graph (Trotter and Moore, 1976).
//!
//! Directed graphs are treated as undirected.

use crate::core::{Graph, IgraphResult};

/// Check whether a graph is net-free (no induced net subgraph).
///
/// The net has 6 vertices: a triangle {a, b, c} plus pendants
/// {d, e, f} where d-a, e-b, f-c are the pendant edges.
/// The algorithm checks all 6-vertex subsets.
///
/// Directed graphs are treated as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_net_free};
///
/// // Triangle: no induced net (too few vertices for net)
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_net_free(&g).unwrap());
///
/// // Net graph itself: NOT net-free
/// let mut g = Graph::with_vertices(6);
/// // Triangle 0-1-2
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// // Pendants
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 4).unwrap();
/// g.add_edge(2, 5).unwrap();
/// assert!(!is_net_free(&g).unwrap());
/// ```
pub fn is_net_free(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    if n < 6 {
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

    let verts: Vec<usize> = (0..n_usize).collect();
    for (i0, &a) in verts.iter().enumerate() {
        for (i1, &b) in verts.iter().enumerate().skip(i0 + 1) {
            if !adj[a][b] {
                continue;
            }
            for &c in &verts[(i1 + 1)..] {
                if !adj[a][c] || !adj[b][c] {
                    continue;
                }
                // a-b-c is a triangle. Look for 3 distinct pendants.
                if has_net_pendants(&adj, a, b, c, n_usize) {
                    return Ok(false);
                }
            }
        }
    }

    Ok(true)
}

fn has_net_pendants(adj: &[Vec<bool>], a: usize, b: usize, c: usize, n: usize) -> bool {
    let tri = [a, b, c];

    let pend_a: Vec<usize> = (0..n)
        .filter(|&v| !tri.contains(&v) && adj[v][a] && !adj[v][b] && !adj[v][c])
        .collect();
    let pend_b: Vec<usize> = (0..n)
        .filter(|&v| !tri.contains(&v) && !adj[v][a] && adj[v][b] && !adj[v][c])
        .collect();
    let pend_c: Vec<usize> = (0..n)
        .filter(|&v| !tri.contains(&v) && !adj[v][a] && !adj[v][b] && adj[v][c])
        .collect();

    for &da in &pend_a {
        for &db in &pend_b {
            if adj[da][db] {
                continue;
            }
            for &dc in &pend_c {
                if dc != da && dc != db && !adj[dc][da] && !adj[dc][db] {
                    return true;
                }
            }
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
        assert!(is_net_free(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_net_free(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_net_free(&g).unwrap());
    }

    #[test]
    fn net_graph() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        assert!(!is_net_free(&g).unwrap());
    }

    #[test]
    fn k4() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_net_free(&g).unwrap());
    }

    #[test]
    fn k5() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_net_free(&g).unwrap());
    }

    #[test]
    fn c5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_net_free(&g).unwrap());
    }

    #[test]
    fn star() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_net_free(&g).unwrap());
    }

    #[test]
    fn path_p6() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        assert!(is_net_free(&g).unwrap());
    }

    #[test]
    fn triangle_with_one_pendant() {
        // Triangle 0-1-2 + pendant 0-3. Not a net (only 1 pendant).
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(is_net_free(&g).unwrap());
    }

    #[test]
    fn triangle_with_two_pendants() {
        // Triangle 0-1-2 + pendants 0-3, 1-4. Still not a net (need 3).
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(is_net_free(&g).unwrap());
    }

    #[test]
    fn net_with_extra_edges() {
        // Net + extra edge between pendants → no longer induced net
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_net_free(&g).unwrap());
    }

    #[test]
    fn edgeless() {
        let g = Graph::with_vertices(6);
        assert!(is_net_free(&g).unwrap());
    }

    #[test]
    fn directed_treated_as_undirected() {
        let mut g = Graph::new(6, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        assert!(!is_net_free(&g).unwrap());
    }
}
