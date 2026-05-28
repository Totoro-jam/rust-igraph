//! Subset edge betweenness centrality (ALGO-PR-052).
//!
//! Counterpart of `igraph_edge_betweenness_subset()` from
//! `references/igraph/src/centrality/betweenness.c:1227`.
//!
//! Computes edge betweenness considering only shortest paths from a
//! given source subset to a given target subset. Uses a modified
//! Brandes framework where only target vertices seed the +1 in
//! back-propagation.

use std::collections::VecDeque;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphResult, VertexId};

/// Subset edge betweenness centrality.
///
/// Computes edge betweenness considering only shortest paths that
/// originate in `sources` and terminate in `targets`. The result is a
/// vector indexed by edge id.
///
/// For undirected graphs (or when `directed = false`), the result is
/// halved.
///
/// # Parameters
///
/// * `graph` — the input graph.
/// * `sources` — source vertices.
/// * `targets` — target vertices.
/// * `directed` — whether to follow edge directions (ignored for
///   undirected graphs).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_betweenness_subset};
///
/// // Path 0—1—2—3: sources={0}, targets={2,3}
/// let mut g = Graph::with_vertices(4);
/// for i in 0..3u32 { g.add_edge(i, i + 1).unwrap(); }
/// let eb = edge_betweenness_subset(&g, &[0], &[2, 3], true).unwrap();
/// // Edge 0 (0-1): on paths (0,2),(0,3) → 2
/// // Edge 1 (1-2): on paths (0,2),(0,3) → 2
/// // Edge 2 (2-3): on path (0,3) → 1
/// // Undirected → halved: [1.0, 1.0, 0.5]
/// assert!((eb[0] - 1.0).abs() < 1e-12);
/// assert!((eb[1] - 1.0).abs() < 1e-12);
/// assert!((eb[2] - 0.5).abs() < 1e-12);
/// ```
pub fn edge_betweenness_subset(
    graph: &Graph,
    sources: &[VertexId],
    targets: &[VertexId],
    directed: bool,
) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let m = graph.ecount();
    let mut betweenness_e = vec![0.0_f64; m];

    if n == 0 || m == 0 || sources.is_empty() || targets.is_empty() {
        return Ok(betweenness_e);
    }

    let mut is_target = vec![false; n_us];
    for &t in targets {
        if (t as usize) < n_us {
            is_target[t as usize] = true;
        }
    }

    let mut sigma = vec![0.0_f64; n_us];
    let mut dist = vec![-1_i64; n_us];
    let mut pred: Vec<Vec<(VertexId, EdgeId)>> = vec![Vec::new(); n_us];
    let mut stack: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut delta_v = vec![0.0_f64; n_us];
    let use_directed = directed && graph.is_directed();

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
        delta_v.fill(0.0);

        sigma[s as usize] = 1.0;
        dist[s as usize] = 0;
        let mut queue: VecDeque<VertexId> = VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            let v_dist = dist[v as usize];

            let edges = if use_directed {
                graph.incident(v)?
            } else if graph.is_directed() {
                let mut e = graph.incident(v)?;
                e.extend(graph.incident_in(v)?);
                e
            } else {
                graph.incident(v)?
            };

            for e in edges {
                let w = graph.edge_other(e, v)?;
                if dist[w as usize] < 0 {
                    queue.push_back(w);
                    dist[w as usize] = v_dist + 1;
                }
                if dist[w as usize] == v_dist + 1 {
                    sigma[w as usize] += sigma[v as usize];
                    pred[w as usize].push((v, e));
                }
            }
        }

        while let Some(w) = stack.pop() {
            let coeff = if is_target[w as usize] {
                (1.0 + delta_v[w as usize]) / sigma[w as usize]
            } else {
                delta_v[w as usize] / sigma[w as usize]
            };

            for &(v, e) in &pred[w as usize] {
                let c = sigma[v as usize] * coeff;
                delta_v[v as usize] += c;
                betweenness_e[e as usize] += c;
            }
        }
    }

    if !directed || !graph.is_directed() {
        for slot in &mut betweenness_e {
            *slot /= 2.0;
        }
    }

    Ok(betweenness_e)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: &[f64], expected: &[f64], tol: f64) {
        assert_eq!(actual.len(), expected.len(), "length mismatch");
        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert!((a - e).abs() < tol, "edge {i}: actual={a} expected={e}");
        }
    }

    #[test]
    fn empty_graph() {
        let g = Graph::new(0, false).unwrap();
        let eb = edge_betweenness_subset(&g, &[0], &[0], true).unwrap();
        assert!(eb.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::new(3, false).unwrap();
        let eb = edge_betweenness_subset(&g, &[0], &[1, 2], true).unwrap();
        assert!(eb.is_empty());
    }

    #[test]
    fn no_sources() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let eb = edge_betweenness_subset(&g, &[], &[0, 1, 2], true).unwrap();
        close(&eb, &[0.0, 0.0], 1e-12);
    }

    #[test]
    fn no_targets() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let eb = edge_betweenness_subset(&g, &[0, 1, 2], &[], true).unwrap();
        close(&eb, &[0.0, 0.0], 1e-12);
    }

    #[test]
    fn all_sources_all_targets_path() {
        // 0—1—2—3 with all vertices as sources and targets → regular edge betweenness
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let eb = edge_betweenness_subset(&g, &[0, 1, 2, 3], &[0, 1, 2, 3], true).unwrap();
        close(&eb, &[3.0, 4.0, 3.0], 1e-12);
    }

    #[test]
    fn path_subset_sources_targets() {
        // 0—1—2—3: sources={0}, targets={2,3}
        // From 0: path to 2 uses edges 0,1; path to 3 uses edges 0,1,2
        // Edge 0: 2 paths; Edge 1: 2 paths; Edge 2: 1 path
        // Undirected → /2: [1.0, 1.0, 0.5]
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let eb = edge_betweenness_subset(&g, &[0], &[2, 3], true).unwrap();
        close(&eb, &[1.0, 1.0, 0.5], 1e-12);
    }

    #[test]
    fn directed_path() {
        // 0→1→2→3: sources={0}, targets={2,3}, directed
        // From 0: path to 2 uses edges 0,1; path to 3 uses edges 0,1,2
        // Edge 0: 2; Edge 1: 2; Edge 2: 1
        // Directed → no halving
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let eb = edge_betweenness_subset(&g, &[0], &[2, 3], true).unwrap();
        close(&eb, &[2.0, 2.0, 1.0], 1e-12);
    }

    #[test]
    fn triangle() {
        // Triangle 0—1—2: sources=all, targets=all
        // All shortest paths have length 1, so each edge carries 1 pair
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 0)]).unwrap();
        let eb = edge_betweenness_subset(&g, &[0, 1, 2], &[0, 1, 2], true).unwrap();
        close(&eb, &[1.0, 1.0, 1.0], 1e-12);
    }

    #[test]
    fn star_subset() {
        // Star: 0 connected to 1,2,3,4
        // sources={1,2}, targets={3,4}
        // Paths: (1,3),(1,4),(2,3),(2,4) all go through centre
        // Edge (0-1): on (1,3),(1,4) = 2; Edge (0-2): on (2,3),(2,4) = 2
        // Edge (0-3): on (1,3),(2,3) = 2; Edge (0-4): on (1,4),(2,4) = 2
        // Undirected → /2: all 1.0
        let mut g = Graph::new(5, false).unwrap();
        g.add_edges(vec![(0, 1), (0, 2), (0, 3), (0, 4)]).unwrap();
        let eb = edge_betweenness_subset(&g, &[1, 2], &[3, 4], true).unwrap();
        close(&eb, &[1.0, 1.0, 1.0, 1.0], 1e-12);
    }

    #[test]
    fn out_of_range_ignored() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let eb = edge_betweenness_subset(&g, &[0, 99], &[2], true).unwrap();
        // Only source=0, target=2: edges 0,1 both on path
        // Undirected → /2: [0.5, 0.5]
        close(&eb, &[0.5, 0.5], 1e-12);
    }
}
