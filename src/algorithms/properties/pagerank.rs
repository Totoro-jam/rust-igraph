//! `PageRank` centrality (ALGO-PR-011).
//!
//! Counterpart of `igraph_pagerank()` from
//! `references/igraph/src/centrality/pagerank.c` (the
//! `IGRAPH_PAGERANK_ALGO_POWER` branch). Implemented as standard power
//! iteration:
//!
//! `PR_new[v] = (1 - d) / N + d * (Σ_{u → v} PR[u] / out_deg(u) +
//!              Σ_{sink} PR[sink] / N)`
//!
//! where the second sum handles "dangling" vertices (out-degree 0) by
//! distributing their rank uniformly across the graph.
//!
//! For undirected graphs every edge counts as bidirectional, so
//! `out_deg(u) == deg(u)` and the in-neighbour iteration is symmetric.
//!
//! Phase-1 minimal slice: undirected / `IGRAPH_OUT`, unweighted, damping
//! `0.85`, `eps = 1e-10`, `max_iter = 1000`. Configurable parameters and
//! ARPACK-based variants ship later.

use crate::core::{Graph, IgraphResult};

const DEFAULT_DAMPING: f64 = 0.85;
const DEFAULT_EPS: f64 = 1e-10;
const DEFAULT_MAX_ITER: usize = 1000;

/// `PageRank` scores via power iteration with damping `0.85`.
///
/// Returns a `Vec<f64>` summing approximately to 1. For graphs with
/// `vcount = 0` returns an empty vector.
///
/// Counterpart of
/// `igraph_pagerank(_, IGRAPH_PAGERANK_ALGO_POWER, _, _, vss_all(),
/// /*directed=*/g.is_directed(), 0.85, NULL_weights, NULL_options)`
/// with default convergence parameters.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, pagerank};
///
/// // Triangle: every vertex has identical PageRank ≈ 1/3.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let pr = pagerank(&g).unwrap();
/// assert!((pr[0] - 1.0/3.0).abs() < 1e-10);
/// assert!((pr[1] - 1.0/3.0).abs() < 1e-10);
/// assert!((pr[2] - 1.0/3.0).abs() < 1e-10);
/// ```
pub fn pagerank(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    if n == 0 {
        return Ok(Vec::new());
    }
    if n == 1 {
        return Ok(vec![1.0]);
    }

    let directed = graph.is_directed();

    // Pre-compute per-vertex out-degree and in-neighbour adjacency:
    //   in_adj[v] = list of (u, 1/out_deg(u)) for each in-edge u → v.
    // For undirected graphs we use the symmetric `neighbors()` iteration
    // (each edge contributes both directions).
    let n_f = f64::from(n);
    let mut out_deg = vec![0u64; n_us];
    for v in 0..n {
        out_deg[v as usize] = graph.neighbors_iter(v)?.len() as u64;
    }

    // Build the in-adjacency: for each edge u → v (or u-v undirected, both
    // u→v and v→u), append u to in_adj[v].
    let mut in_adj: Vec<Vec<u32>> = vec![Vec::new(); n_us];
    if directed {
        // Iterate every edge once and record u → v.
        let m = u32::try_from(graph.ecount())
            .map_err(|_| crate::core::IgraphError::Internal("ecount overflows u32"))?;
        for e in 0..m {
            let (u, v) = graph.edge(e)?;
            in_adj[v as usize].push(u);
        }
    } else {
        // Undirected: each (u, v) edge appears once; record both directions.
        let m = u32::try_from(graph.ecount())
            .map_err(|_| crate::core::IgraphError::Internal("ecount overflows u32"))?;
        for e in 0..m {
            let (u, v) = graph.edge(e)?;
            // Self-loop in undirected has degree 2 contribution so we add
            // both directions.
            if u == v {
                in_adj[v as usize].push(u);
                in_adj[v as usize].push(u);
            } else {
                in_adj[u as usize].push(v);
                in_adj[v as usize].push(u);
            }
        }
    }

    // Initial uniform distribution.
    let mut pr = vec![1.0 / n_f; n_us];
    let mut pr_new = vec![0.0_f64; n_us];

    for _ in 0..DEFAULT_MAX_ITER {
        // Compute total dangling-vertex rank for redistribution.
        let mut dangling_sum: f64 = 0.0;
        for v in 0..n_us {
            if out_deg[v] == 0 {
                dangling_sum += pr[v];
            }
        }

        let teleport = (1.0 - DEFAULT_DAMPING) / n_f;
        let dangling_share = DEFAULT_DAMPING * dangling_sum / n_f;

        for v in 0..n_us {
            let mut incoming: f64 = 0.0;
            for &u in &in_adj[v] {
                #[allow(clippy::cast_precision_loss)]
                let denom = out_deg[u as usize] as f64;
                if denom > 0.0 {
                    incoming += pr[u as usize] / denom;
                }
            }
            pr_new[v] = teleport + dangling_share + DEFAULT_DAMPING * incoming;
        }

        // Convergence: L1 norm of the change.
        let mut diff: f64 = 0.0;
        for v in 0..n_us {
            diff += (pr_new[v] - pr[v]).abs();
        }
        std::mem::swap(&mut pr, &mut pr_new);
        if diff < DEFAULT_EPS {
            break;
        }
    }

    Ok(pr)
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
    fn empty_graph_yields_empty() {
        let g = Graph::with_vertices(0);
        assert!(pagerank(&g).unwrap().is_empty());
    }

    #[test]
    fn singleton_yields_one() {
        let g = Graph::with_vertices(1);
        assert_eq!(pagerank(&g).unwrap(), vec![1.0]);
    }

    #[test]
    fn isolated_vertices_uniform() {
        // No edges: all vertices are dangling. The dangling-redistribution
        // term keeps the distribution uniform → 1/n each.
        let g = Graph::with_vertices(4);
        let pr = pagerank(&g).unwrap();
        close(&pr, &[0.25, 0.25, 0.25, 0.25], 1e-9);
    }

    #[test]
    fn triangle_uniform_one_third() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let pr = pagerank(&g).unwrap();
        close(&pr, &[1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0], 1e-9);
    }

    #[test]
    fn directed_4cycle_uniform_quarter() {
        let mut g = Graph::new(4, true).unwrap();
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        let pr = pagerank(&g).unwrap();
        close(&pr, &[0.25, 0.25, 0.25, 0.25], 1e-9);
    }

    #[test]
    fn pagerank_sums_to_one() {
        // K4 minus an edge — verify normalization.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let pr = pagerank(&g).unwrap();
        let total: f64 = pr.iter().sum();
        assert!((total - 1.0).abs() < 1e-9, "sum {total} != 1.0");
    }

    #[test]
    fn star_centre_has_higher_pagerank_than_leaves() {
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        let pr = pagerank(&g).unwrap();
        // Centre receives from 3 leaves, each leaf only from centre → centre > leaf.
        for &leaf in &pr[1..4] {
            assert!(pr[0] > leaf, "centre {} not > leaf {}", pr[0], leaf);
        }
        // Symmetry among leaves.
        close(&pr[1..4], &[pr[1], pr[1], pr[1]], 1e-9);
    }

    #[test]
    fn pagerank_dangling_node_distributes() {
        // Directed: 0 → 1, vertex 1 is dangling. Power iteration plus
        // dangling-redistribution keeps PR finite and summing to 1.
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let pr = pagerank(&g).unwrap();
        let total: f64 = pr.iter().sum();
        assert!((total - 1.0).abs() < 1e-9);
        // Vertex 1 receives flow from 0 and the dangling-redistribution.
        // Vertex 0 only receives the teleport + dangling-redistribution.
        // → pr[1] > pr[0].
        assert!(pr[1] > pr[0]);
    }
}
