//! Weighted mean distance / average path length (ALGO-PR-044).
//!
//! Computes the average shortest-path distance across all vertex pairs
//! using Dijkstra's algorithm. Supports directed/undirected graphs with
//! non-negative edge weights.
//!
//! Counterpart of `igraph_i_average_path_length_dijkstra` from
//! `references/igraph/src/paths/shortest_paths.c`.

use crate::algorithms::paths::dijkstra::{DijkstraMode, dijkstra_distances_with_mode};
use crate::core::{Graph, IgraphError, IgraphResult};

/// Compute the weighted mean distance (average shortest path length).
///
/// Runs Dijkstra from every vertex and averages all finite pairwise distances.
///
/// - `weights`: non-negative edge weights (length must equal edge count).
/// - `directed`: if true, only follow edge directions; if false, treat
///   edges as undirected (ignored for undirected graphs).
/// - `unconn`: if true, average only over connected pairs; if false,
///   return `f64::INFINITY` if any pair is disconnected.
///
/// Returns `None` if the graph has fewer than 2 vertices, or if `unconn`
/// is true and no pairs are connected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, mean_distance_weighted};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let weights = vec![1.0, 2.0];
/// let md = mean_distance_weighted(&g, &weights, false, false).unwrap();
/// // pairs: (0,1)=1, (1,0)=1, (0,2)=3, (2,0)=3, (1,2)=2, (2,1)=2
/// // mean = (1+1+3+3+2+2)/6 = 12/6 = 2.0
/// assert!((md.unwrap() - 2.0).abs() < 1e-10);
/// ```
pub fn mean_distance_weighted(
    graph: &Graph,
    weights: &[f64],
    directed: bool,
    unconn: bool,
) -> IgraphResult<Option<f64>> {
    let n = graph.vcount();
    if n < 2 {
        return Ok(None);
    }

    let ecount = graph.ecount();
    if weights.len() != ecount {
        return Err(IgraphError::InvalidArgument(format!(
            "mean_distance_weighted: weight vector length ({}) does not match edge count ({ecount})",
            weights.len()
        )));
    }

    if ecount > 0 {
        for &w in weights {
            if w < 0.0 {
                return Err(IgraphError::InvalidArgument(
                    "mean_distance_weighted: weights must be non-negative".to_string(),
                ));
            }
            if w.is_nan() {
                return Err(IgraphError::InvalidArgument(
                    "mean_distance_weighted: weights must not contain NaN".to_string(),
                ));
            }
        }
    }

    let mode = if graph.is_directed() && directed {
        DijkstraMode::Out
    } else {
        DijkstraMode::All
    };

    let mut sum = 0.0_f64;
    let mut conn_pairs = 0_u64;

    for source in 0..n {
        let dists = dijkstra_distances_with_mode(graph, source, weights, mode)?;
        for (target, dist_opt) in dists.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            if target as u32 == source {
                continue;
            }
            if let Some(d) = dist_opt {
                sum += d;
                conn_pairs += 1;
            }
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let total_pairs = u64::from(n) * (u64::from(n) - 1);

    if unconn {
        if conn_pairs == 0 {
            Ok(None)
        } else {
            #[allow(clippy::cast_precision_loss)]
            Ok(Some(sum / conn_pairs as f64))
        }
    } else if conn_pairs < total_pairs {
        Ok(Some(f64::INFINITY))
    } else {
        #[allow(clippy::cast_precision_loss)]
        Ok(Some(sum / total_pairs as f64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert_eq!(mean_distance_weighted(&g, &[], false, false).unwrap(), None);
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert_eq!(mean_distance_weighted(&g, &[], false, false).unwrap(), None);
    }

    #[test]
    fn path_3_unweighted_equivalent() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let weights = vec![1.0, 1.0];
        let md = mean_distance_weighted(&g, &weights, false, false)
            .unwrap()
            .unwrap();
        // same as unweighted: (1+1+2+2+1+1)/6 = 8/6 = 4/3
        assert!(approx_eq(md, 4.0 / 3.0));
    }

    #[test]
    fn path_3_weighted() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let weights = vec![1.0, 2.0];
        let md = mean_distance_weighted(&g, &weights, false, false)
            .unwrap()
            .unwrap();
        // (0,1)=1, (1,0)=1, (0,2)=3, (2,0)=3, (1,2)=2, (2,1)=2 → 12/6=2
        assert!(approx_eq(md, 2.0));
    }

    #[test]
    fn disconnected_unconn_false() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let weights = vec![1.0, 1.0];
        let md = mean_distance_weighted(&g, &weights, false, false)
            .unwrap()
            .unwrap();
        assert!(md.is_infinite());
    }

    #[test]
    fn disconnected_unconn_true() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let weights = vec![1.0, 1.0];
        let md = mean_distance_weighted(&g, &weights, false, true)
            .unwrap()
            .unwrap();
        // Connected pairs: (0,1),(1,0),(2,3),(3,2) all dist 1 → mean=1.0
        assert!(approx_eq(md, 1.0));
    }

    #[test]
    fn directed_graph() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let weights = vec![1.0, 1.0];
        let md = mean_distance_weighted(&g, &weights, true, true)
            .unwrap()
            .unwrap();
        // Directed: only 0→1=1, 0→2=2, 1→2=1 are reachable
        // mean = (1+2+1)/3 = 4/3
        assert!(approx_eq(md, 4.0 / 3.0));
    }

    #[test]
    fn directed_as_undirected() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let weights = vec![1.0, 1.0];
        let md = mean_distance_weighted(&g, &weights, false, false)
            .unwrap()
            .unwrap();
        // Treats as undirected: all pairs connected, same as path_3 unweighted
        assert!(approx_eq(md, 4.0 / 3.0));
    }

    #[test]
    fn complete_k3_weighted() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let weights = vec![1.0, 10.0, 1.0];
        let md = mean_distance_weighted(&g, &weights, false, false)
            .unwrap()
            .unwrap();
        // Shortest paths: 0-1=1, 0-2=min(10, 1+1)=2, 1-2=1
        // All pairs: (0,1)=1, (1,0)=1, (0,2)=2, (2,0)=2, (1,2)=1, (2,1)=1
        // mean = 8/6 = 4/3
        assert!(approx_eq(md, 4.0 / 3.0));
    }

    #[test]
    fn negative_weight_error() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(mean_distance_weighted(&g, &[-1.0], false, false).is_err());
    }

    #[test]
    fn nan_weight_error() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(mean_distance_weighted(&g, &[f64::NAN], false, false).is_err());
    }

    #[test]
    fn weight_mismatch() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(mean_distance_weighted(&g, &[1.0, 2.0], false, false).is_err());
    }

    #[test]
    fn all_isolated_unconn_true() {
        let g = Graph::with_vertices(5);
        let md = mean_distance_weighted(&g, &[], false, true).unwrap();
        assert_eq!(md, None);
    }
}
