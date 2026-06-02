//! Outerplanar graph predicate (ALGO-PR-129).
//!
//! A graph is outerplanar if it can be embedded in the plane with all
//! vertices on the outer face. Equivalently, a graph is outerplanar if
//! and only if it has no `K_4` minor and no `K_{2,3}` minor.
//!
//! We check: series-parallel (no `K_4` minor) + no `K_{2,3}` minor.
//! The `K_{2,3}` check uses a modified SP reduction that tags edges as
//! *original* or *synthetic* (created by series contraction). Three or
//! more synthetic parallel edges between the same pair reveal a
//! `K_{2,3}` minor.
//!
//! Every forest and every cycle is outerplanar. Every outerplanar
//! graph is series-parallel (but not vice versa: `K_{2,3}` is
//! series-parallel but not outerplanar).
//!
//! Directed graphs are treated as undirected.

use crate::algorithms::properties::is_series_parallel::is_series_parallel;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is outerplanar.
///
/// Uses the characterization: outerplanar iff no `K_4` minor
/// (series-parallel) and no `K_{2,3}` minor.
///
/// Directed graphs are treated as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_outerplanar};
///
/// // Cycle C_5: outerplanar
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 0).unwrap();
/// assert!(is_outerplanar(&g).unwrap());
///
/// // K_4: NOT outerplanar (has K_4 minor)
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 {
///     for j in (i+1)..4 {
///         g.add_edge(i, j).unwrap();
///     }
/// }
/// assert!(!is_outerplanar(&g).unwrap());
/// ```
pub fn is_outerplanar(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    if n <= 3 {
        return is_series_parallel(graph);
    }

    if !is_series_parallel(graph)? {
        return Ok(false);
    }

    // Quick global edge bound: outerplanar => edges <= 2n - 3.
    let e = graph.ecount();
    let n_usize = n as usize;
    if e > 2 * n_usize - 3 {
        return Ok(false);
    }

    has_no_k23_minor(graph)
}

/// Detect `K_{2,3}` minor via SP reduction with edge tagging.
///
/// Each edge is tagged as *original* (from the input graph) or
/// *synthetic* (created when a degree-2 vertex is series-contracted).
/// A synthetic edge represents a path with at least one internal
/// vertex. If three or more synthetic parallel edges accumulate
/// between any vertex pair, that certifies a `K_{2,3}` minor: the
/// two endpoints are the hubs, and one internal vertex from each of
/// the three paths forms the three branch vertices.
fn has_no_k23_minor(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount() as usize;

    // K_{2,3} minor needs >= 5 vertices.
    if n < 5 {
        return Ok(true);
    }

    let mut adj: Vec<Vec<(usize, bool)>> = vec![vec![]; n];
    for v in 0..graph.vcount() {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            let vv = v as usize;
            let ww = w as usize;
            if vv < ww {
                adj[vv].push((ww, false));
                adj[ww].push((vv, false));
            }
        }
    }

    Ok(!k23_reduce_check(&mut adj, n))
}

/// Returns `true` if a `K_{2,3}` minor is detected.
fn k23_reduce_check(adj: &mut [Vec<(usize, bool)>], n: usize) -> bool {
    loop {
        // Check for >= 3 synthetic parallel edges between any pair.
        for edges in adj.iter() {
            let mut syn_count = std::collections::HashMap::new();
            for &(w, syn) in edges {
                if syn {
                    *syn_count.entry(w).or_insert(0usize) += 1;
                }
            }
            for &count in syn_count.values() {
                if count >= 3 {
                    return true;
                }
            }
        }

        let mut changed = false;

        // Peel vertices with <= 1 distinct neighbor.
        for v in 0..n {
            if adj[v].is_empty() {
                continue;
            }
            let distinct = distinct_neighbors(&adj[v]);
            if distinct.len() <= 1 {
                changed = true;
                for &w in &distinct {
                    adj[w].retain(|&(x, _)| x != v);
                }
                adj[v].clear();
            }
        }

        // Series reduce: vertices with exactly 2 distinct neighbors.
        for v in 0..n {
            if adj[v].is_empty() {
                continue;
            }
            let distinct = distinct_neighbors(&adj[v]);
            if distinct.len() == 2 {
                let a = distinct[0];
                let b = distinct[1];
                changed = true;
                adj[v].clear();
                adj[a].retain(|&(x, _)| x != v);
                adj[b].retain(|&(x, _)| x != v);
                adj[a].push((b, true));
                adj[b].push((a, true));
            }
        }

        if !changed {
            break;
        }
    }

    false
}

fn distinct_neighbors(edges: &[(usize, bool)]) -> Vec<usize> {
    let mut d: Vec<usize> = edges.iter().map(|&(w, _)| w).collect();
    d.sort_unstable();
    d.dedup();
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_outerplanar(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_outerplanar(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_outerplanar(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_outerplanar(&g).unwrap());
    }

    #[test]
    fn path() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_outerplanar(&g).unwrap());
    }

    #[test]
    fn cycle_c5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_outerplanar(&g).unwrap());
    }

    #[test]
    fn k4_not_outerplanar() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(!is_outerplanar(&g).unwrap());
    }

    #[test]
    fn k23_not_outerplanar() {
        // K_{2,3} is series-parallel but NOT outerplanar.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(!is_outerplanar(&g).unwrap());
    }

    #[test]
    fn diamond_outerplanar() {
        // Diamond: K_4 minus one edge. 4 vertices, 5 edges. Outerplanar.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_outerplanar(&g).unwrap());
    }

    #[test]
    fn star() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_outerplanar(&g).unwrap());
    }

    #[test]
    fn tree() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(3, 5).unwrap();
        assert!(is_outerplanar(&g).unwrap());
    }

    #[test]
    fn edgeless() {
        let g = Graph::with_vertices(5);
        assert!(is_outerplanar(&g).unwrap());
    }

    #[test]
    fn maximal_outerplanar_5() {
        // Triangulated pentagon: 7 = 2*5-3 edges.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert!(is_outerplanar(&g).unwrap());
    }

    #[test]
    fn petersen_not_outerplanar() {
        let mut g = Graph::with_vertices(10);
        let edges = [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 0),
            (5, 7),
            (7, 9),
            (9, 6),
            (6, 8),
            (8, 5),
            (0, 5),
            (1, 6),
            (2, 7),
            (3, 8),
            (4, 9),
        ];
        for (u, v) in edges {
            g.add_edge(u, v).unwrap();
        }
        assert!(!is_outerplanar(&g).unwrap());
    }

    #[test]
    fn directed_treated_as_undirected() {
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(is_outerplanar(&g).unwrap());
    }

    #[test]
    fn disconnected_outerplanar() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        assert!(is_outerplanar(&g).unwrap());
    }

    #[test]
    fn subdivided_k23_not_outerplanar() {
        // K_{2,3} with one edge subdivided: still not outerplanar.
        // Hubs: 0, 1. Branches: 2, 3, 4. Edge 0-2 subdivided with vertex 5.
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 5).unwrap();
        g.add_edge(5, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(!is_outerplanar(&g).unwrap());
    }

    #[test]
    fn three_parallel_paths_with_direct_edge() {
        // 0-1 direct + 0-2-1 + 0-3-4-1: outerplanar (only 2 K_{2,3} branches).
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 1).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 1).unwrap();
        assert!(is_outerplanar(&g).unwrap());
    }
}
