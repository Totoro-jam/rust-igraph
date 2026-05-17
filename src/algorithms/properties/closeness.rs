//! Closeness centrality (ALGO-PR-007).
//!
//! Counterpart of `igraph_closeness()` from
//! `references/igraph/src/centrality/closeness.c:33+`. The closeness
//! of vertex `v` is the inverse of its mean distance to all reachable
//! vertices.
//!
//! Phase-1 minimal slice: undirected/IGRAPH_OUT-mode unweighted
//! closeness with `normalized = true` (per the upstream default).
//! Reuses the existing [`distances`] primitive for BFS-from-each-vertex.
//!
//! For an isolated vertex (no reachable vertices besides itself) the
//! definition is undefined → returns `None`. For graphs with multiple
//! components, the score is computed within each vertex's reachable
//! set (matches upstream's `unconn = true` semantics).

use crate::algorithms::paths::distances::distances;
use crate::core::{Graph, IgraphResult};

/// Per-vertex closeness centrality (`Vec<Option<f64>>`).
///
/// `result[v]` is `Some(reachable_count / sum_of_distances)` if vertex
/// `v` has at least one reachable neighbour; `None` for isolated
/// vertices (matches upstream's `IGRAPH_NAN`).
///
/// Edge directions follow [`distances`]: out-edges for directed
/// graphs (`IGRAPH_OUT`), full undirected reachability otherwise.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, closeness};
///
/// // Star with centre 0 and 3 leaves: centre has the highest closeness.
/// let mut g = Graph::with_vertices(4);
/// for v in 1..4 { g.add_edge(0, v).unwrap(); }
/// let c = closeness(&g).unwrap();
/// // Centre: 3 reachable, sum_dist = 3 → 3/3 = 1.0.
/// assert_eq!(c[0], Some(1.0));
/// // Leaves: 3 reachable (centre + 2 other leaves), sum = 1 + 2 + 2 = 5 → 3/5 = 0.6.
/// for leaf in 1..4 {
///     assert_eq!(c[leaf], Some(0.6));
/// }
/// ```
pub fn closeness(graph: &Graph) -> IgraphResult<Vec<Option<f64>>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let mut out = Vec::with_capacity(n_us);
    for v in 0..n {
        let d = distances(graph, v)?;
        let mut sum_dist: u64 = 0;
        let mut reach: u64 = 0;
        let v_us = v as usize;
        for (target, &val) in d.iter().enumerate() {
            if target == v_us {
                continue;
            }
            if let Some(dist) = val {
                sum_dist += u64::from(dist);
                reach += 1;
            }
        }
        if reach == 0 || sum_dist == 0 {
            // Isolated (no reachable vertices) — upstream returns NaN.
            // sum_dist == 0 with reach > 0 only happens for degenerate
            // self-loop graphs which we treat as isolated for closeness.
            out.push(None);
        } else {
            // Counts fit in u64; ratio fits in f64 mantissa for any
            // graph that survives u32 vertex ids.
            #[allow(clippy::cast_precision_loss)]
            let val = (reach as f64) / (sum_dist as f64);
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
        assert!(closeness(&g).unwrap().is_empty());
    }

    #[test]
    fn isolated_vertices_all_none() {
        let g = Graph::with_vertices(3);
        let c = closeness(&g).unwrap();
        assert_eq!(c, vec![None, None, None]);
    }

    #[test]
    fn singleton_is_none() {
        let g = Graph::with_vertices(1);
        assert_eq!(closeness(&g).unwrap(), vec![None]);
    }

    #[test]
    fn k3_triangle_uniform_one() {
        // Triangle: every vertex has 2 reachable, sum_dist = 1+1 = 2 → 2/2 = 1.0.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let c = closeness(&g).unwrap();
        for &val in &c {
            approx_eq(val, Some(1.0), 1e-12);
        }
    }

    #[test]
    fn star_centre_has_highest() {
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        let c = closeness(&g).unwrap();
        // Centre: 3 reachable, sum=3 → 1.0.
        approx_eq(c[0], Some(1.0), 1e-12);
        // Leaves: 3 reachable (centre at 1, two leaves at 2 each), sum=5 → 0.6.
        for &leaf_val in &c[1..4] {
            approx_eq(leaf_val, Some(0.6), 1e-12);
        }
    }

    #[test]
    fn path_5_endpoints_lower_than_centre() {
        // 0-1-2-3-4: centre at 2 has min mean distance.
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        let c = closeness(&g).unwrap();
        // Vertex 0: reach=4, sum=1+2+3+4=10 → 0.4
        approx_eq(c[0], Some(0.4), 1e-12);
        // Vertex 1: reach=4, sum=1+1+2+3=7 → 4/7
        approx_eq(c[1], Some(4.0 / 7.0), 1e-12);
        // Vertex 2 (centre): reach=4, sum=2+1+1+2=6 → 4/6
        approx_eq(c[2], Some(4.0 / 6.0), 1e-12);
        // Symmetric for 3 and 4.
        approx_eq(c[3], Some(4.0 / 7.0), 1e-12);
        approx_eq(c[4], Some(0.4), 1e-12);
    }

    #[test]
    fn disconnected_components_within_component_only() {
        // {0-1-2} and {3}: vertex 3 isolated → None.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let c = closeness(&g).unwrap();
        // 0: reach=2 (1,2), sum=1+2=3 → 2/3
        approx_eq(c[0], Some(2.0 / 3.0), 1e-12);
        // 1: reach=2 (0,2), sum=1+1=2 → 1.0
        approx_eq(c[1], Some(1.0), 1e-12);
        approx_eq(c[2], Some(2.0 / 3.0), 1e-12);
        // 3: isolated.
        assert_eq!(c[3], None);
    }

    #[test]
    fn directed_path_uses_out_edges() {
        // 0->1->2: from 0 reach {1,2} sum=1+2=3 → 2/3; from 1 reach {2} sum=1 → 1.0;
        // from 2 reach {} → None.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let c = closeness(&g).unwrap();
        approx_eq(c[0], Some(2.0 / 3.0), 1e-12);
        approx_eq(c[1], Some(1.0), 1e-12);
        assert_eq!(c[2], None);
    }
}
