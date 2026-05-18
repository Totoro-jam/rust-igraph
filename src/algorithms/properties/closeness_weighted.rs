//! Weighted closeness centrality (ALGO-PR-007b).
//!
//! Counterpart of `igraph_closeness(_, _, _, _, igraph_vss_all(),
//! IGRAPH_OUT, &weights, /*normalized=*/true)` from
//! `references/igraph/src/centrality/closeness.c`. The weighted
//! variant uses Dijkstra (instead of BFS) for per-vertex shortest
//! distances, then divides the count of reachable vertices by the
//! sum of those distances.
//!
//! Phase-1 minimal slice: undirected (treated as `IGRAPH_ALL` via
//! `Graph::neighbors`) / directed-OUT, normalized = true. Other modes
//! (`IN`, weighted-non-normalized) ship later — same loop body, just
//! changes which neighbour list is followed.
//!
//! Edge weights must be non-negative and finite; violations propagate
//! the [`crate::IgraphError::InvalidArgument`] that `dijkstra_distances`
//! raises.

use crate::algorithms::paths::dijkstra::dijkstra_distances;
use crate::core::{Graph, IgraphResult};

/// Per-vertex weighted closeness centrality.
///
/// Returns `Vec<Option<f64>>`: `Some(reach / sum_dist)` if the vertex
/// has at least one reachable neighbour with finite total distance,
/// `None` for isolated vertices.
///
/// `weights[e]` is the weight of edge `e`; `weights.len()` must equal
/// `graph.ecount()`. All weights must be `>= 0` and finite.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, closeness_weighted};
///
/// // Star with non-uniform weights.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();   // weight 1.0
/// g.add_edge(0, 2).unwrap();   // weight 2.0
/// g.add_edge(0, 3).unwrap();   // weight 3.0
/// let c = closeness_weighted(&g, &[1.0, 2.0, 3.0]).unwrap();
/// // Centre: 3 reachable, sum = 1+2+3 = 6 → 0.5.
/// assert!((c[0].unwrap() - 0.5).abs() < 1e-12);
/// ```
pub fn closeness_weighted(graph: &Graph, weights: &[f64]) -> IgraphResult<Vec<Option<f64>>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let mut out = Vec::with_capacity(n_us);
    for v in 0..n {
        let d = dijkstra_distances(graph, v, weights)?;
        let mut sum_dist: f64 = 0.0;
        let mut reach: u64 = 0;
        let v_us = v as usize;
        for (target, val) in d.iter().enumerate() {
            if target == v_us {
                continue;
            }
            if let Some(dist) = val {
                sum_dist += *dist;
                reach += 1;
            }
        }
        if reach == 0 || sum_dist == 0.0 {
            // Isolated, or all reachable distances are zero (only possible
            // with a degenerate graph carrying zero-weight edges to the
            // source's own neighbours and nothing else). Match upstream's
            // NaN by reporting None.
            out.push(None);
        } else {
            #[allow(clippy::cast_precision_loss)]
            let val = (reach as f64) / sum_dist;
            out.push(Some(val));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: Option<f64>, b: Option<f64>, tol: f64) {
        match (a, b) {
            (Some(x), Some(y)) => {
                assert!((x - y).abs() < tol, "{x} vs {y}");
            }
            (None, None) => {}
            _ => panic!("{a:?} vs {b:?}"),
        }
    }

    #[test]
    fn empty_graph_yields_empty() {
        let g = Graph::with_vertices(0);
        assert!(closeness_weighted(&g, &[]).unwrap().is_empty());
    }

    #[test]
    fn isolated_vertices_all_none() {
        let g = Graph::with_vertices(3);
        let c = closeness_weighted(&g, &[]).unwrap();
        assert_eq!(c, vec![None, None, None]);
    }

    #[test]
    fn singleton_is_none() {
        let g = Graph::with_vertices(1);
        assert_eq!(closeness_weighted(&g, &[]).unwrap(), vec![None]);
    }

    #[test]
    fn star_with_uniform_weights_matches_unweighted() {
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        let c = closeness_weighted(&g, &[1.0, 1.0, 1.0]).unwrap();
        // Centre: 3 reachable, sum=3 → 1.0.
        approx_eq(c[0], Some(1.0), 1e-12);
        // Leaves: 3 reachable (centre at 1, two leaves at 2 each), sum=5 → 0.6.
        for &leaf_val in &c[1..4] {
            approx_eq(leaf_val, Some(0.6), 1e-12);
        }
    }

    #[test]
    fn star_with_non_uniform_weights() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let c = closeness_weighted(&g, &[1.0, 2.0, 3.0]).unwrap();
        // Centre: sum=1+2+3=6 → 3/6 = 0.5.
        approx_eq(c[0], Some(0.5), 1e-12);
        // Leaf 1: paths to {0,2,3} cost {1, 1+2=3, 1+3=4} sum=8 → 3/8.
        approx_eq(c[1], Some(3.0 / 8.0), 1e-12);
        // Leaf 2: paths to {0,1,3} cost {2, 2+1=3, 2+3=5} sum=10 → 0.3.
        approx_eq(c[2], Some(0.3), 1e-12);
        // Leaf 3: paths to {0,1,2} cost {3, 3+1=4, 3+2=5} sum=12 → 0.25.
        approx_eq(c[3], Some(0.25), 1e-12);
    }

    #[test]
    fn directed_path_uses_out_edges() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let c = closeness_weighted(&g, &[1.0, 1.0]).unwrap();
        // 0 reaches {1,2}, sum=1+2=3 → 2/3
        approx_eq(c[0], Some(2.0 / 3.0), 1e-12);
        // 1 reaches {2}, sum=1 → 1.0
        approx_eq(c[1], Some(1.0), 1e-12);
        // 2 reaches none → None
        assert_eq!(c[2], None);
    }

    #[test]
    fn disconnected_components_within_component_only() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let c = closeness_weighted(&g, &[1.0, 1.0]).unwrap();
        approx_eq(c[0], Some(2.0 / 3.0), 1e-12);
        approx_eq(c[1], Some(1.0), 1e-12);
        approx_eq(c[2], Some(2.0 / 3.0), 1e-12);
        assert_eq!(c[3], None);
    }

    #[test]
    fn unit_weights_match_unweighted() {
        // Validate the equivalence to unweighted closeness on K3.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let cw = closeness_weighted(&g, &[1.0; 3]).unwrap();
        let cu = crate::closeness(&g).unwrap();
        for (a, b) in cw.iter().zip(cu.iter()) {
            approx_eq(*a, *b, 1e-12);
        }
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(closeness_weighted(&g, &[]).is_err());
        assert!(closeness_weighted(&g, &[1.0, 2.0]).is_err());
    }

    #[test]
    fn negative_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(closeness_weighted(&g, &[-1.0]).is_err());
    }
}
