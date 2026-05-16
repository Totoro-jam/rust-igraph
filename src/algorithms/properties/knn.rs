//! Average nearest-neighbour degree (ALGO-PR-005).
//!
//! For each vertex `v`, the average degree over `v`'s neighbours.
//! Counterpart of `igraph_avg_nearest_neighbor_degree(_, vss_all(),
//! IGRAPH_ALL, IGRAPH_ALL, &knn, NULL, NULL)` from
//! `references/igraph/src/properties/degrees.c:263`.
//!
//! Phase-1 minimal slice: unweighted, undirected (or `IGRAPH_ALL` mode
//! for directed input). Returns `Vec<Option<f64>>` where `None`
//! indicates the vertex has no neighbours (matches upstream's
//! `IGRAPH_NAN`). The `knnk` aggregate (per-degree mean) and
//! mode-aware variants ship in PR-005b.

use crate::core::{Graph, IgraphResult};

/// Average nearest-neighbour degree, per vertex.
///
/// `result[v] = Some(d)` where `d` is the mean degree over `v`'s
/// neighbours; `None` if `v` has no neighbours. Self-loops are
/// counted under upstream's `IGRAPH_LOOPS` convention (each loop
/// counts twice for undirected degree).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, avg_nearest_neighbor_degree};
///
/// // Star with centre 0 and leaves 1-2-3:
/// // Centre's neighbours have degree 1 each → knn[0] = 1.
/// // Leaves' single neighbour (centre) has degree 3 → knn[leaf] = 3.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// let knn = avg_nearest_neighbor_degree(&g).unwrap();
/// assert_eq!(knn, vec![Some(1.0), Some(3.0), Some(3.0), Some(3.0)]);
/// ```
pub fn avg_nearest_neighbor_degree(graph: &Graph) -> IgraphResult<Vec<Option<f64>>> {
    let n = graph.vcount();
    let n_us = n as usize;

    // Pre-cache per-vertex degree (LOOPS-counted; matches upstream's
    // IGRAPH_LOOPS default).
    let mut deg: Vec<u32> = Vec::with_capacity(n_us);
    for v in 0..n {
        deg.push(
            u32::try_from(graph.degree(v)?)
                .map_err(|_| crate::core::IgraphError::Internal("degree exceeds u32 in knn"))?,
        );
    }

    let mut out: Vec<Option<f64>> = Vec::with_capacity(n_us);
    for v in 0..n {
        let neis = graph.neighbors(v)?;
        if neis.is_empty() {
            out.push(None);
            continue;
        }
        let mut sum: u64 = 0;
        for &w in &neis {
            sum += u64::from(deg[w as usize]);
        }
        // sum / nv. nv ≤ |E|·2 ≤ 2 * 2^31 — fits f64.
        #[allow(clippy::cast_precision_loss)]
        let avg = (sum as f64) / (neis.len() as f64);
        out.push(Some(avg));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_yields_empty_vec() {
        let g = Graph::with_vertices(0);
        assert!(avg_nearest_neighbor_degree(&g).unwrap().is_empty());
    }

    #[test]
    fn isolated_vertices_have_none() {
        let g = Graph::with_vertices(3);
        assert_eq!(
            avg_nearest_neighbor_degree(&g).unwrap(),
            vec![None, None, None]
        );
    }

    #[test]
    fn star_centre_has_avg_1_leaves_have_3() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert_eq!(
            avg_nearest_neighbor_degree(&g).unwrap(),
            vec![Some(1.0), Some(3.0), Some(3.0), Some(3.0)]
        );
    }

    #[test]
    fn path_5_endpoints_see_internal_neighbours() {
        // 0-1-2-3-4. Degrees: 1,2,2,2,1.
        // knn[0] = deg[1] / 1 = 2.
        // knn[1] = (deg[0] + deg[2]) / 2 = (1 + 2)/2 = 1.5.
        // knn[2] = (deg[1] + deg[3]) / 2 = (2+2)/2 = 2.
        // knn[3] = (deg[2] + deg[4]) / 2 = (2+1)/2 = 1.5.
        // knn[4] = deg[3] / 1 = 2.
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        assert_eq!(
            avg_nearest_neighbor_degree(&g).unwrap(),
            vec![Some(2.0), Some(1.5), Some(2.0), Some(1.5), Some(2.0)]
        );
    }

    #[test]
    fn k4_uniform_degree_3() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert_eq!(avg_nearest_neighbor_degree(&g).unwrap(), vec![Some(3.0); 4]);
    }

    #[test]
    fn self_loop_inflates_neighbour_degree() {
        // Vertex 0 has a self-loop and an edge to 1: degree 0 = 3
        // (LOOPS_TWICE), degree 1 = 1.
        // 0's neighbours via `neighbors()`: [0, 0, 1] (self-loop reported twice
        // + 1 once). knn[0] = (deg[0] + deg[0] + deg[1]) / 3 = (3+3+1)/3 = 7/3.
        // 1's neighbours: [0]; knn[1] = deg[0] = 3.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let r = avg_nearest_neighbor_degree(&g).unwrap();
        let seven_thirds = 7.0_f64 / 3.0;
        assert_eq!(r[0], Some(seven_thirds));
        assert_eq!(r[1], Some(3.0));
    }
}
