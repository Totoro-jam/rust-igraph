//! Range-limited edge betweenness centrality (ALGO-PR-050).
//!
//! Counterpart of `igraph_edge_betweenness_cutoff()` from
//! `references/igraph/src/centrality/betweenness.c:813+`.
//!
//! Same Brandes framework as edge betweenness but with a BFS depth
//! bound. Only shortest paths of length at most `cutoff` are counted.

use std::collections::VecDeque;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphResult, VertexId};

/// Range-limited edge betweenness centrality.
///
/// Computes edge betweenness considering only shortest paths of length
/// at most `cutoff`. The result is a vector indexed by edge id.
///
/// For undirected graphs the result is halved (each unordered pair
/// counted once).
///
/// # Parameters
///
/// * `graph` — the input graph.
/// * `cutoff` — maximum shortest-path length to consider.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_betweenness_cutoff};
///
/// // Path 0—1—2—3 with cutoff=2
/// let mut g = Graph::with_vertices(4);
/// for i in 0..3u32 { g.add_edge(i, i + 1).unwrap(); }
/// let eb = edge_betweenness_cutoff(&g, 2).unwrap();
/// // Edge 0 (0-1): on paths (0,1) len 1 and (0,2) len 2 → 2 pairs
/// // Edge 1 (1-2): on paths (1,2) len 1 and (0,2) len 2 and (1,3) len 2 → 3 pairs
/// // Edge 2 (2-3): on paths (2,3) len 1 and (1,3) len 2 → 2 pairs
/// assert_eq!(eb, vec![2.0, 3.0, 2.0]);
/// ```
pub fn edge_betweenness_cutoff(graph: &Graph, cutoff: u32) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let m = graph.ecount();
    let mut betweenness_e = vec![0.0_f64; m];

    if n == 0 || m == 0 {
        return Ok(betweenness_e);
    }

    let mut sigma = vec![0.0_f64; n_us];
    let mut dist = vec![-1_i64; n_us];
    let mut pred: Vec<Vec<(VertexId, EdgeId)>> = vec![Vec::new(); n_us];
    let mut stack: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut delta_v = vec![0.0_f64; n_us];
    let cutoff_i64 = i64::from(cutoff);

    for s in 0..n {
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

            if v_dist >= cutoff_i64 {
                continue;
            }

            for e in graph.incident(v)? {
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
            for &(v, e) in &pred[w as usize] {
                let c = (sigma[v as usize] / sigma[w as usize]) * (1.0 + delta_v[w as usize]);
                delta_v[v as usize] += c;
                betweenness_e[e as usize] += c;
            }
        }
    }

    if !graph.is_directed() {
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
        let eb = edge_betweenness_cutoff(&g, 1).unwrap();
        assert!(eb.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::new(3, false).unwrap();
        let eb = edge_betweenness_cutoff(&g, 5).unwrap();
        assert!(eb.is_empty());
    }

    #[test]
    fn path_full_cutoff() {
        // 0—1—2—3: large cutoff → same as regular edge betweenness
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let eb = edge_betweenness_cutoff(&g, 100).unwrap();
        close(&eb, &[3.0, 4.0, 3.0], 1e-12);
    }

    #[test]
    fn path_cutoff_1() {
        // 0—1—2—3 with cutoff=1: only direct edges, each with weight 1
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let eb = edge_betweenness_cutoff(&g, 1).unwrap();
        // Each edge carries exactly 1 pair of length 1
        close(&eb, &[1.0, 1.0, 1.0], 1e-12);
    }

    #[test]
    fn path_cutoff_2() {
        // 0—1—2—3 with cutoff=2
        // Edge 0 (0-1): paths (0,1) len1, (0,2) len2 → 2
        // Edge 1 (1-2): paths (1,2) len1, (0,2) len2, (1,3) len2 → 3
        // Edge 2 (2-3): paths (2,3) len1, (1,3) len2 → 2
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let eb = edge_betweenness_cutoff(&g, 2).unwrap();
        close(&eb, &[2.0, 3.0, 2.0], 1e-12);
    }

    #[test]
    fn triangle() {
        // Triangle: each edge carries 1 pair
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 0)]).unwrap();
        let eb = edge_betweenness_cutoff(&g, 10).unwrap();
        close(&eb, &[1.0, 1.0, 1.0], 1e-12);
    }

    #[test]
    fn star_cutoff_2() {
        // Star: 0 connected to 1,2,3,4
        // Each spoke carries: 1 direct pair + 3 pairs through centre to other leaves
        // = 4 pairs per spoke
        let mut g = Graph::new(5, false).unwrap();
        g.add_edges(vec![(0, 1), (0, 2), (0, 3), (0, 4)]).unwrap();
        let eb = edge_betweenness_cutoff(&g, 2).unwrap();
        close(&eb, &[4.0, 4.0, 4.0, 4.0], 1e-12);
    }

    #[test]
    fn directed_path() {
        // 0→1→2→3 with large cutoff
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let eb = edge_betweenness_cutoff(&g, 100).unwrap();
        // directed: edge 0→1 on (0,1),(0,2),(0,3) = 3
        // edge 1→2 on (0,2),(0,3),(1,2),(1,3) = 4
        // edge 2→3 on (0,3),(1,3),(2,3) = 3
        close(&eb, &[3.0, 4.0, 3.0], 1e-12);
    }

    #[test]
    fn directed_cutoff_2() {
        // 0→1→2→3 with cutoff=2
        // Edge 0→1: (0,1) len1, (0,2) len2 → 2
        // Edge 1→2: (1,2) len1, (0,2) len2, (1,3) len2 → 3
        // Edge 2→3: (2,3) len1, (1,3) len2 → 2
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let eb = edge_betweenness_cutoff(&g, 2).unwrap();
        close(&eb, &[2.0, 3.0, 2.0], 1e-12);
    }

    #[test]
    fn cutoff_zero() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let eb = edge_betweenness_cutoff(&g, 0).unwrap();
        close(&eb, &[0.0, 0.0], 1e-12);
    }
}
