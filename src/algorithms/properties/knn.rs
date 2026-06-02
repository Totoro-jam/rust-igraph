//! Average nearest-neighbour degree (ALGO-PR-005 + ALGO-PR-005b).
//!
//! For each vertex `v`, the average degree over `v`'s neighbours.
//! Counterpart of `igraph_avg_nearest_neighbor_degree(_, vss_all(),
//! IGRAPH_ALL, IGRAPH_ALL, &knn, &knnk, weights)` from
//! `references/igraph/src/properties/degrees.c:263`.
//!
//! Phase-1 minimal slice: undirected (or `IGRAPH_ALL` mode for
//! directed input). Three exposed entry points:
//!
//! - [`avg_nearest_neighbor_degree`] — unweighted per-vertex (PR-005)
//! - [`avg_nearest_neighbor_degree_weighted`] — weighted per-vertex
//!   (PR-005b)
//! - [`knnk`] / [`knnk_weighted`] — per-degree aggregate `k_nn(k)`
//!   (PR-005b)
//!
//! All variants return `Vec<Option<f64>>` where `None` corresponds to
//! upstream's `IGRAPH_NAN` (no neighbours / no vertices of that degree).

use crate::core::{Graph, IgraphError, IgraphResult};

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

/// Weighted average nearest-neighbour degree (Barrat formula).
///
/// `result[v] = Some( (1/s_v) Σ_{u ∼ v} w(v,u) · deg(u) )` where
/// `s_v = Σ_{u ∼ v} w(v,u)` is `v`'s strength (sum of incident edge
/// weights, with self-loops counted twice for undirected graphs to
/// match upstream's `IGRAPH_LOOPS` convention).
///
/// Returns `None` for vertices with strength 0 (no neighbours, or all
/// incident weights are 0). Weights must have length `graph.ecount()`
/// and contain only finite, non-negative values; otherwise this returns
/// `IgraphError`. Reference: Barrat et al., PNAS 101 3747 (2004),
/// equation (6).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, avg_nearest_neighbor_degree_weighted};
///
/// // Triangle 0-1-2 with weights (1,2,4) on edges (0,1),(1,2),(2,0).
/// // Vertex 0 incident to e0=1.0 (→1) and e2=4.0 (→2). deg[1]=deg[2]=2.
/// // s_0 = 1+4 = 5; sum = 1*2 + 4*2 = 10; knn[0] = 10/5 = 2.0.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let r = avg_nearest_neighbor_degree_weighted(&g, &[1.0, 2.0, 4.0]).unwrap();
/// assert_eq!(r, vec![Some(2.0); 3]);
/// ```
pub fn avg_nearest_neighbor_degree_weighted(
    graph: &Graph,
    weights: &[f64],
) -> IgraphResult<Vec<Option<f64>>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let m = graph.ecount();
    if weights.len() != m {
        return Err(IgraphError::Internal(
            "weights length does not match ecount in knn_weighted",
        ));
    }
    for &w in weights {
        if !w.is_finite() || w < 0.0 {
            return Err(IgraphError::Internal(
                "weights must be finite and non-negative in knn_weighted",
            ));
        }
    }

    let mut deg: Vec<u32> = Vec::with_capacity(n_us);
    for v in 0..n {
        deg.push(
            u32::try_from(graph.degree(v)?)
                .map_err(|_| IgraphError::Internal("degree exceeds u32 in knn_weighted"))?,
        );
    }

    let mut out: Vec<Option<f64>> = Vec::with_capacity(n_us);
    for v in 0..n {
        // Iterate over incident edges directly — `neighbors()` merges
        // out/in lists in sorted order while `incident()` concatenates
        // them, so the two are NOT positionally aligned. Use
        // `edge_other()` to get the corresponding neighbour vertex.
        let inc = graph.incident(v)?;
        let mut sum = 0.0_f64;
        let mut strength = 0.0_f64;
        for &e in &inc {
            let u = graph.edge_other(e, v)?;
            let w = weights[e as usize];
            strength += w;
            sum += w * f64::from(deg[u as usize]);
        }
        if strength > 0.0 {
            out.push(Some(sum / strength));
        } else {
            out.push(None);
        }
    }
    Ok(out)
}

/// Per-degree average nearest-neighbour degree, `k_nn(k)`.
///
/// Counterpart of `igraph_avg_nearest_neighbor_degree(_, _, _, _,
/// NULL, &knnk, NULL)`. The output has length `maxdeg(graph)`; entry
/// `result[k-1]` is the mean of `knn[v]` over all vertices with
/// degree `k`. Returns `None` for degrees that no vertex has
/// (matches upstream's `IGRAPH_NAN`).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, knnk};
///
/// // Star K_{1,3}: leaves (deg 1) all have knn=3, centre (deg 3) has knn=1.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// let r = knnk(&g).unwrap();
/// assert_eq!(r, vec![Some(3.0), None, Some(1.0)]);
/// ```
pub fn knnk(graph: &Graph) -> IgraphResult<Vec<Option<f64>>> {
    let n = graph.vcount();
    let knn = avg_nearest_neighbor_degree(graph)?;
    let mut max_deg: u32 = 0;
    for v in 0..n {
        let d = u32::try_from(graph.degree(v)?)
            .map_err(|_| IgraphError::Internal("degree exceeds u32 in knnk"))?;
        if d > max_deg {
            max_deg = d;
        }
    }
    if max_deg == 0 {
        return Ok(Vec::new());
    }
    let max_deg_us = max_deg as usize;
    let mut sums: Vec<f64> = vec![0.0; max_deg_us];
    let mut counts: Vec<u32> = vec![0; max_deg_us];
    for v in 0..n {
        if let Some(k) = knn[v as usize] {
            let d = graph.degree(v)?;
            sums[d - 1] += k;
            counts[d - 1] += 1;
        }
    }
    Ok(sums
        .iter()
        .zip(counts.iter())
        .map(|(&s, &c)| if c == 0 { None } else { Some(s / f64::from(c)) })
        .collect())
}

/// Per-degree weighted average nearest-neighbour degree.
///
/// Counterpart of `igraph_avg_nearest_neighbor_degree(_, _, _, _,
/// NULL, &knnk, weights)`. Output indexed by *degree* (not strength),
/// `result[k-1]` is `Σ_{deg(v)=k} sum_v / Σ_{deg(v)=k} strength_v`,
/// matching upstream's pooling formula. Same input requirements as
/// [`avg_nearest_neighbor_degree_weighted`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, knnk_weighted};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let k = knnk_weighted(&g, &[1.0, 1.0]).unwrap();
/// assert!(!k.is_empty());
/// ```
pub fn knnk_weighted(graph: &Graph, weights: &[f64]) -> IgraphResult<Vec<Option<f64>>> {
    let n = graph.vcount();
    let m = graph.ecount();
    if weights.len() != m {
        return Err(IgraphError::Internal(
            "weights length does not match ecount in knnk_weighted",
        ));
    }
    for &w in weights {
        if !w.is_finite() || w < 0.0 {
            return Err(IgraphError::Internal(
                "weights must be finite and non-negative in knnk_weighted",
            ));
        }
    }
    let mut max_deg: u32 = 0;
    let mut deg: Vec<u32> = Vec::with_capacity(n as usize);
    for v in 0..n {
        let d = u32::try_from(graph.degree(v)?)
            .map_err(|_| IgraphError::Internal("degree exceeds u32 in knnk_weighted"))?;
        deg.push(d);
        if d > max_deg {
            max_deg = d;
        }
    }
    if max_deg == 0 {
        return Ok(Vec::new());
    }
    let max_deg_us = max_deg as usize;
    let mut sums: Vec<f64> = vec![0.0; max_deg_us];
    let mut strs: Vec<f64> = vec![0.0; max_deg_us];

    for v in 0..n {
        let inc = graph.incident(v)?;
        if inc.is_empty() {
            continue;
        }
        let mut sum_v = 0.0_f64;
        let mut str_v = 0.0_f64;
        for &e in &inc {
            let u = graph.edge_other(e, v)?;
            let w = weights[e as usize];
            str_v += w;
            sum_v += w * f64::from(deg[u as usize]);
        }
        let bucket = deg[v as usize] as usize - 1;
        sums[bucket] += sum_v;
        strs[bucket] += str_v;
    }

    Ok(sums
        .iter()
        .zip(strs.iter())
        .map(|(&s, &t)| if t > 0.0 { Some(s / t) } else { None })
        .collect())
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
    fn weighted_uniform_weights_match_unweighted() {
        // Path 0-1-2-3-4 with all unit weights — must match unweighted knn.
        let mut g = Graph::with_vertices(5);
        for i in 0..4u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let weights = vec![1.0; 4];
        let unweighted = avg_nearest_neighbor_degree(&g).unwrap();
        let weighted = avg_nearest_neighbor_degree_weighted(&g, &weights).unwrap();
        assert_eq!(unweighted.len(), weighted.len());
        for i in 0..unweighted.len() {
            match (unweighted[i], weighted[i]) {
                (Some(a), Some(b)) => assert!((a - b).abs() < 1e-12),
                (None, None) => {}
                _ => panic!("uniform weights diverged at vertex {i}"),
            }
        }
    }

    #[test]
    fn weighted_triangle_unequal_weights() {
        // Triangle 0-1-2: edges (0,1)=1, (1,2)=2, (2,0)=4.
        // deg[0]=deg[1]=deg[2]=2.
        // Vertex 0: incident to e0=1.0 (→1) + e2=4.0 (→2). s=5; sum=2+8=10. knn=2.
        // Vertex 1: incident to e0=1.0 (→0) + e1=2.0 (→2). s=3; sum=2+4=6. knn=2.
        // Vertex 2: incident to e1=2.0 (→1) + e2=4.0 (→0). s=6; sum=4+8=12. knn=2.
        // All vertices have neighbours of identical degree (2), so weighted
        // average must equal 2.0 regardless of weighting.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let r = avg_nearest_neighbor_degree_weighted(&g, &[1.0, 2.0, 4.0]).unwrap();
        assert_eq!(r, vec![Some(2.0); 3]);
    }

    #[test]
    fn weighted_isolated_vertex_yields_none() {
        let g = Graph::with_vertices(2);
        let r = avg_nearest_neighbor_degree_weighted(&g, &[]).unwrap();
        assert_eq!(r, vec![None, None]);
    }

    #[test]
    fn weighted_zero_weight_edge_yields_none() {
        // Vertex 0 has only one incident edge, weight 0 → strength 0 → None.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let r = avg_nearest_neighbor_degree_weighted(&g, &[0.0]).unwrap();
        assert_eq!(r, vec![None, None]);
    }

    #[test]
    fn weighted_invalid_weights_length_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(avg_nearest_neighbor_degree_weighted(&g, &[]).is_err());
    }

    #[test]
    fn weighted_negative_weights_error() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(avg_nearest_neighbor_degree_weighted(&g, &[-1.0]).is_err());
    }

    #[test]
    fn knnk_star_distinct_degrees() {
        // Star K_{1,3}: degrees [3, 1, 1, 1]. Centre's knn=1, leaves' knn=3.
        // knnk[0] (deg 1) = avg of three 3.0 = 3.0.
        // knnk[1] (deg 2) = None (no deg-2 vertex).
        // knnk[2] (deg 3) = 1.0.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let r = knnk(&g).unwrap();
        assert_eq!(r, vec![Some(3.0), None, Some(1.0)]);
    }

    #[test]
    fn knnk_path_5() {
        // Path 0-1-2-3-4. Degrees: 1,2,2,2,1.
        // knn = [2.0, 1.5, 2.0, 1.5, 2.0].
        // knnk[0] (deg 1) = (2.0 + 2.0) / 2 = 2.0 (vertices 0, 4).
        // knnk[1] (deg 2) = (1.5 + 2.0 + 1.5) / 3 = 5.0/3 (vertices 1, 2, 3).
        let mut g = Graph::with_vertices(5);
        for i in 0..4u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let r = knnk(&g).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0], Some(2.0));
        assert!((r[1].unwrap() - 5.0_f64 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn knnk_empty_graph_yields_empty() {
        let g = Graph::with_vertices(0);
        assert!(knnk(&g).unwrap().is_empty());
    }

    #[test]
    fn knnk_no_edges_yields_empty() {
        let g = Graph::with_vertices(5);
        // maxdeg = 0 → empty.
        assert!(knnk(&g).unwrap().is_empty());
    }

    #[test]
    fn knnk_weighted_uniform_matches_knnk() {
        let mut g = Graph::with_vertices(5);
        for i in 0..4u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let unw = knnk(&g).unwrap();
        let w = knnk_weighted(&g, &[1.0; 4]).unwrap();
        assert_eq!(unw.len(), w.len());
        for i in 0..unw.len() {
            match (unw[i], w[i]) {
                (Some(a), Some(b)) => assert!((a - b).abs() < 1e-12),
                (None, None) => {}
                _ => panic!("knnk_weighted uniform diverged at idx {i}"),
            }
        }
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
