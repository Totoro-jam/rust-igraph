//! Range-limited betweenness centrality (ALGO-PR-049).
//!
//! Counterpart of `igraph_betweenness_cutoff()` from
//! `references/igraph/src/centrality/betweenness.c:553`.
//!
//! Computes betweenness centrality using only shortest paths whose
//! length does not exceed a given cutoff. Uses Brandes' algorithm
//! with a BFS depth bound.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphResult, VertexId};

/// Range-limited betweenness centrality.
///
/// Computes betweenness centrality for all vertices using only shortest
/// paths of length at most `cutoff`. Shorter cutoffs reduce computation
/// time by limiting BFS depth, capturing local betweenness structure.
///
/// For undirected graphs the result is halved (each unordered pair
/// counted once). This matches igraph's convention.
///
/// # Parameters
///
/// * `graph` — the input graph.
/// * `cutoff` — maximum shortest-path length to consider.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, betweenness_cutoff};
///
/// // Path 0—1—2—3—4
/// let mut g = Graph::with_vertices(5);
/// for i in 0..4u32 { g.add_edge(i, i + 1).unwrap(); }
/// // With cutoff=2, vertex 2 lies on paths of length ≤2:
/// // (1,3) only. (0,2) passes through 1, not 2.
/// let b = betweenness_cutoff(&g, 2).unwrap();
/// // All betweenness ≤ full betweenness
/// assert!(b[2] <= 4.0);
/// ```
pub fn betweenness_cutoff(graph: &Graph, cutoff: u32) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let mut betweenness = vec![0.0_f64; n_us];

    if n == 0 {
        return Ok(betweenness);
    }

    let mut sigma = vec![0.0_f64; n_us];
    let mut dist = vec![-1_i64; n_us];
    let mut pred: Vec<Vec<VertexId>> = vec![Vec::new(); n_us];
    let mut stack: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut delta = vec![0.0_f64; n_us];
    let cutoff_i64 = i64::from(cutoff);

    for s in 0..n {
        sigma.fill(0.0);
        dist.fill(-1);
        for slot in &mut pred {
            slot.clear();
        }
        stack.clear();
        delta.fill(0.0);

        sigma[s as usize] = 1.0;
        dist[s as usize] = 0;
        let mut queue: VecDeque<VertexId> = VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            let v_dist = dist[v as usize];

            if v_dist >= cutoff_i64 {
                continue;
            }

            for w in graph.neighbors(v)? {
                if dist[w as usize] < 0 {
                    queue.push_back(w);
                    dist[w as usize] = v_dist + 1;
                }
                if dist[w as usize] == v_dist + 1 {
                    sigma[w as usize] += sigma[v as usize];
                    pred[w as usize].push(v);
                }
            }
        }

        while let Some(w) = stack.pop() {
            for &v in &pred[w as usize] {
                delta[v as usize] +=
                    (sigma[v as usize] / sigma[w as usize]) * (1.0 + delta[w as usize]);
            }
            if w != s {
                betweenness[w as usize] += delta[w as usize];
            }
        }
    }

    if !graph.is_directed() {
        for slot in &mut betweenness {
            *slot /= 2.0;
        }
    }

    Ok(betweenness)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: &[f64], expected: &[f64], tol: f64) {
        assert_eq!(actual.len(), expected.len(), "length mismatch");
        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert!((a - e).abs() < tol, "vertex {i}: actual={a} expected={e}");
        }
    }

    #[test]
    fn empty_graph() {
        let g = Graph::new(0, false).unwrap();
        let b = betweenness_cutoff(&g, 1).unwrap();
        assert!(b.is_empty());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::new(1, false).unwrap();
        let b = betweenness_cutoff(&g, 1).unwrap();
        assert_eq!(b, vec![0.0]);
    }

    #[test]
    fn disconnected() {
        let g = Graph::new(3, false).unwrap();
        let b = betweenness_cutoff(&g, 5).unwrap();
        assert_eq!(b, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn path_full_cutoff() {
        // 0—1—2—3—4 with cutoff large enough → same as regular betweenness
        let mut g = Graph::new(5, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3), (3, 4)]).unwrap();
        let b = betweenness_cutoff(&g, 100).unwrap();
        close(&b, &[0.0, 3.0, 4.0, 3.0, 0.0], 1e-12);
    }

    #[test]
    fn path_cutoff_1() {
        // 0—1—2—3—4 with cutoff=1: only direct neighbors count.
        // No vertex is on a shortest path of length 1 between two *other* vertices.
        let mut g = Graph::new(5, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3), (3, 4)]).unwrap();
        let b = betweenness_cutoff(&g, 1).unwrap();
        close(&b, &[0.0, 0.0, 0.0, 0.0, 0.0], 1e-12);
    }

    #[test]
    fn path_cutoff_2() {
        // 0—1—2—3—4 with cutoff=2
        // Paths of length ≤ 2: (0,1),(1,2),(2,3),(3,4) length 1 +
        //   (0,2) through 1, (1,3) through 2, (2,4) through 3
        // Betweenness: v1 on (0,2)=1, v2 on (1,3)=1, v3 on (2,4)=1
        let mut g = Graph::new(5, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3), (3, 4)]).unwrap();
        let b = betweenness_cutoff(&g, 2).unwrap();
        close(&b, &[0.0, 1.0, 1.0, 1.0, 0.0], 1e-12);
    }

    #[test]
    fn star_cutoff_2() {
        // Star: 0 connected to 1,2,3,4
        // With cutoff=2, paths (i,j) for i,j in {1,2,3,4} through centre = C(4,2)=6
        let mut g = Graph::new(5, false).unwrap();
        g.add_edges(vec![(0, 1), (0, 2), (0, 3), (0, 4)]).unwrap();
        let b = betweenness_cutoff(&g, 2).unwrap();
        // Centre has betweenness 6.0 (all pairs go through it)
        close(&b, &[6.0, 0.0, 0.0, 0.0, 0.0], 1e-12);
    }

    #[test]
    fn triangle() {
        // Triangle 0—1—2—0: all betweenness = 0 (all paths have length 1)
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 0)]).unwrap();
        let b = betweenness_cutoff(&g, 10).unwrap();
        close(&b, &[0.0, 0.0, 0.0], 1e-12);
    }

    #[test]
    fn directed_path() {
        // 0→1→2→3
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let b = betweenness_cutoff(&g, 100).unwrap();
        // directed: vertex 1 on (0,2),(0,3) = 2; vertex 2 on (0,3),(1,3) = 2
        close(&b, &[0.0, 2.0, 2.0, 0.0], 1e-12);
    }

    #[test]
    fn directed_cutoff_2() {
        // 0→1→2→3 with cutoff=2
        // Paths of length ≤ 2: (0,1),(1,2),(2,3) len 1 + (0,2) through 1, (1,3) through 2
        // v1 on (0,2)=1; v2 on (1,3)=1
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let b = betweenness_cutoff(&g, 2).unwrap();
        close(&b, &[0.0, 1.0, 1.0, 0.0], 1e-12);
    }

    #[test]
    fn cutoff_zero() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let b = betweenness_cutoff(&g, 0).unwrap();
        // No paths of positive length → all zeros
        close(&b, &[0.0, 0.0, 0.0], 1e-12);
    }
}
