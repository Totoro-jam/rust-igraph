//! Subset betweenness centrality (ALGO-PR-051).
//!
//! Counterpart of `igraph_betweenness_subset()` from
//! `references/igraph/src/centrality/betweenness.c:1000`.
//!
//! Computes betweenness centrality considering only shortest paths
//! originating from a given source subset and terminating at a given
//! target subset. Uses a modified Brandes framework where only target
//! vertices seed the back-propagation (+1 contribution).

use std::collections::VecDeque;

use crate::core::{Graph, IgraphResult, VertexId};

fn all_neighbors(graph: &Graph, v: VertexId, directed: bool) -> IgraphResult<Vec<VertexId>> {
    if directed && graph.is_directed() {
        graph.neighbors(v)
    } else if graph.is_directed() {
        let mut out = graph.out_neighbors_vec(v)?;
        out.extend(graph.in_neighbors_vec(v)?);
        Ok(out)
    } else {
        graph.neighbors(v)
    }
}

/// Subset betweenness centrality.
///
/// Computes betweenness for all vertices considering only shortest paths
/// that start in `sources` and end in `targets`.
///
/// For undirected graphs (or when `directed = false`), the result is
/// halved since each unordered pair is counted once.
///
/// # Parameters
///
/// * `graph` — the input graph.
/// * `sources` — source vertices for the shortest paths.
/// * `targets` — target vertices for the shortest paths.
/// * `directed` — whether to follow edge directions (ignored for
///   undirected graphs).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, betweenness_subset};
///
/// // Path 0—1—2—3—4: sources={0,1}, targets={3,4}
/// let mut g = Graph::with_vertices(5);
/// for i in 0..4u32 { g.add_edge(i, i + 1).unwrap(); }
/// let b = betweenness_subset(&g, &[0, 1], &[3, 4], true).unwrap();
/// // Vertex 2 lies on paths (0,3),(0,4),(1,3),(1,4) — betweenness = 4/2 = 2
/// // Vertex 3 lies on paths (0,4),(1,4) — betweenness = 2/2 = 1
/// assert!((b[2] - 2.0).abs() < 1e-12);
/// assert!((b[3] - 1.0).abs() < 1e-12);
/// ```
pub fn betweenness_subset(
    graph: &Graph,
    sources: &[VertexId],
    targets: &[VertexId],
    directed: bool,
) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let mut betweenness = vec![0.0_f64; n_us];

    if n == 0 || sources.is_empty() || targets.is_empty() {
        return Ok(betweenness);
    }

    let mut is_target = vec![false; n_us];
    for &t in targets {
        if (t as usize) < n_us {
            is_target[t as usize] = true;
        }
    }

    let mut sigma = vec![0.0_f64; n_us];
    let mut dist = vec![-1_i64; n_us];
    let mut pred: Vec<Vec<VertexId>> = vec![Vec::new(); n_us];
    let mut stack: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut delta = vec![0.0_f64; n_us];

    for &s in sources {
        if s >= n {
            continue;
        }

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

            for w in all_neighbors(graph, v, directed)? {
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
            let coeff = if is_target[w as usize] {
                (1.0 + delta[w as usize]) / sigma[w as usize]
            } else {
                delta[w as usize] / sigma[w as usize]
            };

            for &v in &pred[w as usize] {
                delta[v as usize] += sigma[v as usize] * coeff;
            }

            if w != s {
                betweenness[w as usize] += delta[w as usize];
            }
        }
    }

    if !directed || !graph.is_directed() {
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
        let b = betweenness_subset(&g, &[0], &[0], true).unwrap();
        assert!(b.is_empty());
    }

    #[test]
    fn no_sources() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let b = betweenness_subset(&g, &[], &[0, 1, 2], true).unwrap();
        close(&b, &[0.0, 0.0, 0.0], 1e-12);
    }

    #[test]
    fn no_targets() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let b = betweenness_subset(&g, &[0, 1, 2], &[], true).unwrap();
        close(&b, &[0.0, 0.0, 0.0], 1e-12);
    }

    #[test]
    fn all_sources_all_targets_undirected() {
        // With all sources and targets, should equal regular betweenness
        // Path 0—1—2—3
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let b = betweenness_subset(&g, &[0, 1, 2, 3], &[0, 1, 2, 3], true).unwrap();
        // Regular betweenness for path: [0, 2, 2, 0]
        close(&b, &[0.0, 2.0, 2.0, 0.0], 1e-12);
    }

    #[test]
    fn path_subset_sources() {
        // 0—1—2—3—4: sources={0}, targets=all
        // BFS from 0 reaches all; paths (0,1),(0,2),(0,3),(0,4)
        // v1 on (0,2),(0,3),(0,4) = 3; v2 on (0,3),(0,4) = 2; v3 on (0,4) = 1
        // Undirected → /2
        let mut g = Graph::new(5, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3), (3, 4)]).unwrap();
        let b = betweenness_subset(&g, &[0], &[0, 1, 2, 3, 4], true).unwrap();
        close(&b, &[0.0, 1.5, 1.0, 0.5, 0.0], 1e-12);
    }

    #[test]
    fn path_subset_targets() {
        // 0—1—2—3—4: sources=all, targets={4}
        // Only paths (s,4) count: (0,4) through 1,2,3; (1,4) through 2,3; (2,4) through 3; (3,4) direct
        // v1: on (0,4) = 1; v2: on (0,4),(1,4) = 2; v3: on (0,4),(1,4),(2,4) = 3
        // Undirected → /2
        let mut g = Graph::new(5, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3), (3, 4)]).unwrap();
        let b = betweenness_subset(&g, &[0, 1, 2, 3, 4], &[4], true).unwrap();
        close(&b, &[0.0, 0.5, 1.0, 1.5, 0.0], 1e-12);
    }

    #[test]
    fn path_both_subsets() {
        // 0—1—2—3—4: sources={0,1}, targets={3,4}
        // From 0: paths to 3 (through 1,2) and 4 (through 1,2,3)
        //   v1: on (0,3),(0,4) = 2; v2: on (0,3),(0,4) = 2; v3: on (0,4) = 1
        // From 1: paths to 3 (through 2) and 4 (through 2,3)
        //   v2: on (1,3),(1,4) = 2; v3: on (1,4) = 1
        // Total: v1=2, v2=4, v3=2
        // Undirected → /2: v1=1, v2=2, v3=1
        let mut g = Graph::new(5, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3), (3, 4)]).unwrap();
        let b = betweenness_subset(&g, &[0, 1], &[3, 4], true).unwrap();
        close(&b, &[0.0, 1.0, 2.0, 1.0, 0.0], 1e-12);
    }

    #[test]
    fn directed_path() {
        // 0→1→2→3: sources={0}, targets={2,3}, directed=true
        // From 0: path to 2 (through 1), path to 3 (through 1,2)
        // v1: on (0,2),(0,3) = 2; v2: on (0,3) = 1
        // Directed → no halving
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let b = betweenness_subset(&g, &[0], &[2, 3], true).unwrap();
        close(&b, &[0.0, 2.0, 1.0, 0.0], 1e-12);
    }

    #[test]
    fn directed_as_undirected() {
        // 0→1→2→3: sources={0}, targets={2,3}, directed=false
        // Same as undirected path — halved
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let b = betweenness_subset(&g, &[0], &[2, 3], false).unwrap();
        close(&b, &[0.0, 1.0, 0.5, 0.0], 1e-12);
    }

    #[test]
    fn triangle() {
        // Triangle 0—1—2—0: sources=all, targets=all → betweenness all 0
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 0)]).unwrap();
        let b = betweenness_subset(&g, &[0, 1, 2], &[0, 1, 2], true).unwrap();
        close(&b, &[0.0, 0.0, 0.0], 1e-12);
    }

    #[test]
    fn star_subset() {
        // Star: 0 connected to 1,2,3,4
        // sources={1,2}, targets={3,4}: paths (1,3),(1,4),(2,3),(2,4) all through 0
        // v0 betweenness = 4 (directed=true keeps undirected → halve)
        let mut g = Graph::new(5, false).unwrap();
        g.add_edges(vec![(0, 1), (0, 2), (0, 3), (0, 4)]).unwrap();
        let b = betweenness_subset(&g, &[1, 2], &[3, 4], true).unwrap();
        close(&b, &[2.0, 0.0, 0.0, 0.0, 0.0], 1e-12);
    }

    #[test]
    fn out_of_range_vertices_ignored() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        // Source 99 is out of range — silently skipped
        let b = betweenness_subset(&g, &[0, 99], &[2], true).unwrap();
        // Only source=0, target=2: path 0-1-2, v1 gets 1 / 2 = 0.5
        close(&b, &[0.0, 0.5, 0.0], 1e-12);
    }
}
