//! Local scan statistics (ALGO-PR-051 + ALGO-PR-042).
//!
//! Counterparts of `igraph_local_scan_*` from
//! `references/igraph/src/misc/scan.c`.
//!
//! Scan statistics summarise local neighbourhood structure. For each
//! vertex the k-neighbourhood is identified and the edges (or total
//! weight) within that neighbourhood are counted.
//!
//! ## Functions
//!
//! - [`local_scan_1`] — undirected-only 1-neighbourhood edge count
//!   (original ALGO-PR-051, kept for backward compatibility).
//! - [`local_scan_0`] — k=0 (degree / strength).
//! - [`local_scan_1_ecount`] — k=1 with directed-mode support.
//! - [`local_scan_0_them`] — two-graph k=0 scan.
//! - [`local_scan_1_ecount_them`] — two-graph k=1 scan.
//! - [`local_scan_subset_ecount`] — edge count in given vertex subsets.

use crate::algorithms::properties::degree::DegreeMode;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Collect incident edge IDs for a vertex respecting mode.
fn incident_with_mode(graph: &Graph, v: VertexId, mode: DegreeMode) -> IgraphResult<Vec<u32>> {
    if !graph.is_directed() {
        return graph.incident(v);
    }
    match mode {
        DegreeMode::Out => graph.incident(v),
        DegreeMode::In => graph.incident_in(v),
        DegreeMode::All => {
            let mut out = graph.incident(v)?;
            let in_edges = graph.incident_in(v)?;
            out.extend(in_edges);
            Ok(out)
        }
    }
}

/// Collect vertex neighbours respecting mode.
fn neighbors_with_mode(graph: &Graph, v: VertexId, mode: DegreeMode) -> IgraphResult<Vec<u32>> {
    if !graph.is_directed() {
        return graph.neighbors(v);
    }
    match mode {
        DegreeMode::Out => {
            let edges = graph.incident(v)?;
            edges.iter().map(|&e| graph.edge_other(e, v)).collect()
        }
        DegreeMode::In => {
            let edges = graph.incident_in(v)?;
            edges.iter().map(|&e| graph.edge_other(e, v)).collect()
        }
        DegreeMode::All => {
            let out_edges = graph.incident(v)?;
            let in_edges = graph.incident_in(v)?;
            let mut neis = Vec::with_capacity(out_edges.len() + in_edges.len());
            for &e in &out_edges {
                neis.push(graph.edge_other(e, v)?);
            }
            for &e in &in_edges {
                neis.push(graph.edge_other(e, v)?);
            }
            Ok(neis)
        }
    }
}

// ────────────────── local_scan_1 (legacy, backward-compat) ──────────────────

/// For each vertex, count edges within its closed 1-neighborhood.
///
/// The closed 1-neighborhood of vertex `v` is `{v} ∪ neighbors(v)`.
/// This function counts all edges that have both endpoints in that set.
///
/// For undirected graphs, each such edge is counted once per vertex whose
/// neighborhood contains it.
///
/// `weights`: optional edge weights (length must equal `ecount()`).
/// When provided, sums edge weights instead of counting edges.
///
/// Returns a vector of length `vcount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, local_scan_1};
///
/// // Triangle: each vertex's 1-neighborhood is the whole graph.
/// // 3 edges in the neighborhood of each vertex.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let s = local_scan_1(&g, None).unwrap();
/// assert!((s[0] - 3.0).abs() < 1e-10);
/// assert!((s[1] - 3.0).abs() < 1e-10);
/// assert!((s[2] - 3.0).abs() < 1e-10);
/// ```
pub fn local_scan_1(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<Vec<f64>> {
    local_scan_1_ecount(graph, weights, DegreeMode::All)
}

// ──────────────────────── local_scan_0 ──────────────────────────────────────

/// Local scan-0: vertex degree (unweighted) or strength (weighted).
///
/// For `k = 0` the scan statistic is defined as the degree (or
/// weighted strength) of each vertex.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, local_scan_0, DegreeMode};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// let res = local_scan_0(&g, None, DegreeMode::All).unwrap();
/// assert!((res[0] - 3.0).abs() < 1e-12);
/// assert!((res[1] - 1.0).abs() < 1e-12);
/// ```
pub fn local_scan_0(
    graph: &Graph,
    weights: Option<&[f64]>,
    mode: DegreeMode,
) -> IgraphResult<Vec<f64>> {
    if let Some(w) = weights {
        if w.len() != graph.ecount() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight vector length {} != edge count {}",
                w.len(),
                graph.ecount()
            )));
        }
    }

    let n = graph.vcount() as usize;
    let mut res = vec![0.0_f64; n];
    let directed = graph.is_directed();
    let ecount = graph.ecount();

    for eid in 0..ecount {
        let eid_u32 =
            u32::try_from(eid).map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
        let from = graph.edge_source(eid_u32)? as usize;
        let to = graph.edge_target(eid_u32)? as usize;
        let w = weights.map_or(1.0, |ws| ws[eid]);

        if directed {
            match mode {
                DegreeMode::Out => res[from] += w,
                DegreeMode::In => res[to] += w,
                DegreeMode::All => {
                    res[from] += w;
                    res[to] += w;
                }
            }
        } else {
            res[from] += w;
            res[to] += w;
        }
    }

    Ok(res)
}

// ──────────────────────── local_scan_1_ecount ───────────────────────────────

/// Local scan-1 for directed graphs (mode != All): mark+crawl approach.
fn local_scan_1_directed(
    graph: &Graph,
    weights: Option<&[f64]>,
    mode: DegreeMode,
) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_usize = n as usize;
    let mut res = vec![0.0_f64; n_usize];
    let mut marker: Vec<i64> = vec![-1; n_usize];

    for node in 0..n {
        let node_i = i64::from(node);
        let edges1 = incident_with_mode(graph, node, mode)?;

        marker[node as usize] = node_i;
        for &e in &edges1 {
            let nei = graph.edge_other(e, node)?;
            let w = weights.map_or(1.0, |ws| ws[e as usize]);
            marker[nei as usize] = node_i;
            res[node as usize] += w;
        }

        for &e in &edges1 {
            let nei = graph.edge_other(e, node)?;
            if nei == node {
                break;
            }
            let edges2 = incident_with_mode(graph, nei, mode)?;
            for &e2 in &edges2 {
                let nei2 = graph.edge_other(e2, nei)?;
                if marker[nei2 as usize] == node_i {
                    let w2 = weights.map_or(1.0, |ws| ws[e2 as usize]);
                    res[node as usize] += w2;
                }
            }
        }
    }

    Ok(res)
}

/// Local scan-1 for directed graphs with mode = All: unmark-after-visit.
fn local_scan_1_directed_all(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_usize = n as usize;
    let mut res = vec![0.0_f64; n_usize];
    let mut marker: Vec<i64> = vec![-1; n_usize];

    for node in 0..n {
        let node_i = i64::from(node);
        let edges1 = incident_with_mode(graph, node, DegreeMode::All)?;

        for &e in &edges1 {
            let nei = graph.edge_other(e, node)?;
            let w = weights.map_or(1.0, |ws| ws[e as usize]);
            marker[nei as usize] = node_i;
            res[node as usize] += w;
        }

        marker[node as usize] = -1;

        for &e in &edges1 {
            let nei = graph.edge_other(e, node)?;
            if marker[nei as usize] != node_i {
                continue;
            }
            let edges2 = incident_with_mode(graph, nei, DegreeMode::All)?;
            for &e2 in &edges2 {
                let nei2 = graph.edge_other(e2, nei)?;
                if marker[nei2 as usize] == node_i {
                    let w2 = weights.map_or(1.0, |ws| ws[e2 as usize]);
                    res[node as usize] += w2;
                }
            }
            marker[nei as usize] = -1;
        }
    }

    Ok(res)
}

/// Local scan-1 edge count / weight sum in the 1-neighbourhood of
/// every vertex, with directed-mode support.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, local_scan_1_ecount, DegreeMode};
///
/// // Triangle 0-1-2 plus pendant 3-0
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(3, 0).unwrap();
/// let res = local_scan_1_ecount(&g, None, DegreeMode::All).unwrap();
/// assert!((res[0] - 4.0).abs() < 1e-12);
/// assert!((res[3] - 1.0).abs() < 1e-12);
/// ```
pub fn local_scan_1_ecount(
    graph: &Graph,
    weights: Option<&[f64]>,
    mode: DegreeMode,
) -> IgraphResult<Vec<f64>> {
    if let Some(w) = weights {
        if w.len() != graph.ecount() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight vector length {} != edge count {}",
                w.len(),
                graph.ecount()
            )));
        }
    }

    if graph.is_directed() {
        if mode == DegreeMode::All {
            local_scan_1_directed_all(graph, weights)
        } else {
            local_scan_1_directed(graph, weights, mode)
        }
    } else {
        // For undirected graphs, use k-general BFS approach
        crate::algorithms::properties::local_scan_k::local_scan_k_ecount(graph, 1, weights, mode)
    }
}

// ────────────────────── local_scan_0_them ────────────────────────────────────

/// Local scan-0 on two graphs: degree/strength from `them` restricted
/// to edges that also exist in `us`.
///
/// Both graphs must have the same vertex count and directedness.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, local_scan_0_them, DegreeMode};
///
/// let mut us = Graph::with_vertices(3);
/// us.add_edge(0, 1).unwrap();
/// us.add_edge(0, 2).unwrap();
/// let mut them = Graph::with_vertices(3);
/// them.add_edge(0, 1).unwrap();
/// them.add_edge(1, 2).unwrap();
/// let res = local_scan_0_them(&us, &them, None, DegreeMode::All).unwrap();
/// assert!((res[0] - 1.0).abs() < 1e-12);
/// assert!((res[1] - 1.0).abs() < 1e-12);
/// assert!((res[2] - 0.0).abs() < 1e-12);
/// ```
pub fn local_scan_0_them(
    us: &Graph,
    them: &Graph,
    weights_them: Option<&[f64]>,
    mode: DegreeMode,
) -> IgraphResult<Vec<f64>> {
    check_them_compat(us, them, weights_them, "local_scan_0_them")?;

    let is_graph = crate::algorithms::operators::intersection::intersection(us, them)?;

    // For unweighted, just compute scan_0 on the intersection
    if weights_them.is_none() {
        return local_scan_0(&is_graph, None, mode);
    }

    // For weighted, we need to map weights from `them` to the intersection.
    // The intersection result has edge_map2 which maps intersection edges
    // back to `them` edges. But our intersection returns IntersectionResult
    // which only has .graph. We'll do the unweighted path for now and
    // handle weighted via a different approach.
    //
    // Weighted scan-0-them: for each edge in `us`, check if the same edge
    // exists in `them` and if so add its weight.
    let Some(ws) = weights_them else {
        return Err(IgraphError::Internal(
            "weights_them should be Some after guard",
        ));
    };
    let n = us.vcount() as usize;
    let mut res = vec![0.0_f64; n];
    let directed = us.is_directed();
    let us_ecount = us.ecount();

    for eid in 0..us_ecount {
        let eid_u32 =
            u32::try_from(eid).map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
        let from = us.edge_source(eid_u32)?;
        let to = us.edge_target(eid_u32)?;

        // Check if this edge exists in `them`
        if let Some(them_eid) = them.find_eid(from, to)? {
            let w = ws[them_eid as usize];
            if directed {
                match mode {
                    DegreeMode::Out => res[from as usize] += w,
                    DegreeMode::In => res[to as usize] += w,
                    DegreeMode::All => {
                        res[from as usize] += w;
                        res[to as usize] += w;
                    }
                }
            } else {
                res[from as usize] += w;
                res[to as usize] += w;
            }
        }
    }

    Ok(res)
}

// ────────────────── local_scan_1_ecount_them ─────────────────────────────────

/// Local scan-1 edge count on two graphs: count edges from `them` in
/// the 1-neighbourhood defined by `us`.
///
/// Both graphs must have the same vertex count and directedness.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, local_scan_1_ecount_them, DegreeMode};
///
/// let mut us = Graph::with_vertices(3);
/// us.add_edge(0, 1).unwrap();
/// us.add_edge(1, 2).unwrap();
/// let mut them = Graph::with_vertices(3);
/// them.add_edge(0, 1).unwrap();
/// them.add_edge(0, 2).unwrap();
/// them.add_edge(1, 2).unwrap();
/// let res = local_scan_1_ecount_them(&us, &them, None, DegreeMode::All).unwrap();
/// assert!((res[1] - 3.0).abs() < 1e-12);
/// ```
pub fn local_scan_1_ecount_them(
    us: &Graph,
    them: &Graph,
    weights_them: Option<&[f64]>,
    mode: DegreeMode,
) -> IgraphResult<Vec<f64>> {
    check_them_compat(us, them, weights_them, "local_scan_1_ecount_them")?;

    let n = us.vcount();
    let n_usize = n as usize;
    let mut res = vec![0.0_f64; n_usize];
    let mut marker: Vec<i64> = vec![-1; n_usize];
    let undirected_or_all = mode == DegreeMode::All || !us.is_directed();

    for node in 0..n {
        let node_i = i64::from(node);

        marker[node as usize] = node_i;
        let us_neis = neighbors_with_mode(us, node, mode)?;
        for &nei in &us_neis {
            marker[nei as usize] = node_i;
        }

        let them_edges_ego = incident_with_mode(them, node, mode)?;
        for &e in &them_edges_ego {
            let nei = them.edge_other(e, node)?;
            if marker[nei as usize] == node_i {
                let w = weights_them.map_or(1.0, |ws| ws[e as usize]);
                res[node as usize] += w;
            }
        }

        for &us_nei in &us_neis {
            let them_edges = incident_with_mode(them, us_nei, mode)?;
            for &e2 in &them_edges {
                let nei2 = them.edge_other(e2, us_nei)?;
                if marker[nei2 as usize] == node_i {
                    let w = weights_them.map_or(1.0, |ws| ws[e2 as usize]);
                    res[node as usize] += w;
                }
            }
        }

        if undirected_or_all {
            res[node as usize] /= 2.0;
        }
    }

    Ok(res)
}

// ────────────────── local_scan_subset_ecount ─────────────────────────────────

/// Local scan on given vertex subsets: count edges (or sum weights)
/// in the induced subgraph of each subset.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, local_scan_subset_ecount};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let subsets = vec![vec![0, 1, 2], vec![2, 3]];
/// let res = local_scan_subset_ecount(&g, &subsets, None).unwrap();
/// assert!((res[0] - 3.0).abs() < 1e-12);
/// assert!((res[1] - 1.0).abs() < 1e-12);
/// ```
pub fn local_scan_subset_ecount(
    graph: &Graph,
    subsets: &[Vec<VertexId>],
    weights: Option<&[f64]>,
) -> IgraphResult<Vec<f64>> {
    if let Some(w) = weights {
        if w.len() != graph.ecount() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight vector length {} != edge count {}",
                w.len(),
                graph.ecount()
            )));
        }
    }

    let n = graph.vcount();
    let n_usize = n as usize;
    let directed = graph.is_directed();
    let num_subsets = subsets.len();
    let mut res = vec![0.0_f64; num_subsets];
    let mut marker: Vec<i64> = vec![-1; n_usize];

    for (subset_idx, subset) in subsets.iter().enumerate() {
        let subset_i = i64::try_from(subset_idx)
            .map_err(|_| IgraphError::Internal("subset index overflow"))?;

        for &v in subset {
            if v >= n {
                return Err(IgraphError::InvalidArgument(format!(
                    "vertex {v} out of range for graph with {n} vertices"
                )));
            }
            marker[v as usize] = subset_i;
        }

        // Use out-edges (IGRAPH_OUT mode with IGRAPH_LOOPS_TWICE) to
        // count each undirected edge once from each endpoint, then halve.
        for &v in subset {
            let edges = graph.incident(v)?;
            for &e in &edges {
                let nei = graph.edge_other(e, v)?;
                if marker[nei as usize] == subset_i {
                    let w = weights.map_or(1.0, |ws| ws[e as usize]);
                    res[subset_idx] += w;
                }
            }

            if directed {
                let in_edges = graph.incident_in(v)?;
                for &e in &in_edges {
                    let nei = graph.edge_other(e, v)?;
                    if marker[nei as usize] == subset_i {
                        let w = weights.map_or(1.0, |ws| ws[e as usize]);
                        res[subset_idx] += w;
                    }
                }
            }
        }

        res[subset_idx] /= 2.0;
    }

    Ok(res)
}

/// Validate compatibility for two-graph ("them") scan functions.
fn check_them_compat(
    us: &Graph,
    them: &Graph,
    weights_them: Option<&[f64]>,
    name: &str,
) -> IgraphResult<()> {
    if us.vcount() != them.vcount() {
        return Err(IgraphError::InvalidArgument(format!(
            "{name}: vertex count mismatch ({} vs {})",
            us.vcount(),
            them.vcount()
        )));
    }
    if us.is_directed() != them.is_directed() {
        return Err(IgraphError::InvalidArgument(format!(
            "{name}: directedness mismatch"
        )));
    }
    if let Some(w) = weights_them {
        if w.len() != them.ecount() {
            return Err(IgraphError::InvalidArgument(format!(
                "{name}: weight vector length {} != them edge count {}",
                w.len(),
                them.ecount()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    // --- local_scan_1 (legacy) tests ---

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let s = local_scan_1(&g, None).unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(5);
        let s = local_scan_1(&g, None).unwrap();
        assert!(s.iter().all(|&v| close(v, 0.0)));
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let s = local_scan_1(&g, None).unwrap();
        assert!(close(s[0], 3.0));
        assert!(close(s[1], 3.0));
        assert!(close(s[2], 3.0));
    }

    #[test]
    fn path_5() {
        let mut g = Graph::with_vertices(5);
        for i in 0..4u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let s = local_scan_1(&g, None).unwrap();
        assert!(close(s[0], 1.0));
        assert!(close(s[1], 2.0));
        assert!(close(s[2], 2.0));
        assert!(close(s[3], 2.0));
        assert!(close(s[4], 1.0));
    }

    #[test]
    fn star() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let s = local_scan_1(&g, None).unwrap();
        assert!(close(s[0], 3.0));
        assert!(close(s[1], 1.0));
        assert!(close(s[2], 1.0));
        assert!(close(s[3], 1.0));
    }

    #[test]
    fn k4() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let s = local_scan_1(&g, None).unwrap();
        for &val in &s {
            assert!(close(val, 6.0));
        }
    }

    #[test]
    fn two_triangles_with_bridge() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(3, 5).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(2, 3).unwrap();
        let s = local_scan_1(&g, None).unwrap();
        assert!(close(s[0], 3.0));
        assert!(close(s[1], 3.0));
        assert!(close(s[2], 4.0));
        assert!(close(s[3], 4.0));
        assert!(close(s[4], 3.0));
        assert!(close(s[5], 3.0));
    }

    #[test]
    fn weighted() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let w = vec![2.0, 3.0, 5.0];
        let s = local_scan_1(&g, Some(&w)).unwrap();
        assert!(close(s[0], 10.0));
        assert!(close(s[1], 10.0));
        assert!(close(s[2], 10.0));
    }

    #[test]
    fn self_loop() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let s = local_scan_1(&g, None).unwrap();
        assert!(close(s[0], 2.0));
        assert!(close(s[1], 2.0));
    }

    #[test]
    fn weights_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(local_scan_1(&g, Some(&[1.0, 2.0])).is_err());
    }

    // --- local_scan_0 tests ---

    #[test]
    fn scan0_empty() {
        let g = Graph::with_vertices(0);
        let res = local_scan_0(&g, None, DegreeMode::All).unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn scan0_no_edges() {
        let g = Graph::with_vertices(5);
        let res = local_scan_0(&g, None, DegreeMode::All).unwrap();
        assert_eq!(res, vec![0.0; 5]);
    }

    #[test]
    fn scan0_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let res = local_scan_0(&g, None, DegreeMode::All).unwrap();
        for &d in &res {
            assert!(close(d, 2.0));
        }
    }

    #[test]
    fn scan0_weighted() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let w = vec![3.0, 5.0];
        let res = local_scan_0(&g, Some(&w), DegreeMode::All).unwrap();
        assert!(close(res[0], 3.0));
        assert!(close(res[1], 8.0));
        assert!(close(res[2], 5.0));
    }

    #[test]
    fn scan0_directed_out() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let res = local_scan_0(&g, None, DegreeMode::Out).unwrap();
        assert!(close(res[0], 2.0));
        assert!(close(res[1], 1.0));
        assert!(close(res[2], 0.0));
    }

    #[test]
    fn scan0_directed_in() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let res = local_scan_0(&g, None, DegreeMode::In).unwrap();
        assert!(close(res[0], 0.0));
        assert!(close(res[1], 1.0));
        assert!(close(res[2], 2.0));
    }

    // --- local_scan_1_ecount tests ---

    #[test]
    fn scan1e_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let res = local_scan_1_ecount(&g, None, DegreeMode::All).unwrap();
        for &v in &res {
            assert!(close(v, 3.0));
        }
    }

    #[test]
    fn scan1e_path4() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let res = local_scan_1_ecount(&g, None, DegreeMode::All).unwrap();
        assert!(close(res[0], 1.0));
        assert!(close(res[1], 2.0));
        assert!(close(res[2], 2.0));
        assert!(close(res[3], 1.0));
    }

    #[test]
    fn scan1e_directed_out() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let res = local_scan_1_ecount(&g, None, DegreeMode::Out).unwrap();
        assert!(close(res[0], 1.0));
        assert!(close(res[1], 1.0));
        assert!(close(res[2], 0.0));
    }

    #[test]
    fn scan1e_directed_all_cycle() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let res = local_scan_1_ecount(&g, None, DegreeMode::All).unwrap();
        for &v in &res {
            assert!(close(v, 3.0));
        }
    }

    // --- local_scan_0_them tests ---

    #[test]
    fn scan0t_basic() {
        let mut us = Graph::with_vertices(3);
        us.add_edge(0, 1).unwrap();
        us.add_edge(0, 2).unwrap();
        let mut them = Graph::with_vertices(3);
        them.add_edge(0, 1).unwrap();
        them.add_edge(1, 2).unwrap();
        let res = local_scan_0_them(&us, &them, None, DegreeMode::All).unwrap();
        assert!(close(res[0], 1.0));
        assert!(close(res[1], 1.0));
        assert!(close(res[2], 0.0));
    }

    #[test]
    fn scan0t_identical() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let res = local_scan_0_them(&g, &g, None, DegreeMode::All).unwrap();
        let self_scan = local_scan_0(&g, None, DegreeMode::All).unwrap();
        for (a, b) in res.iter().zip(self_scan.iter()) {
            assert!(close(*a, *b));
        }
    }

    #[test]
    fn scan0t_vcount_mismatch() {
        let us = Graph::with_vertices(3);
        let them = Graph::with_vertices(4);
        assert!(local_scan_0_them(&us, &them, None, DegreeMode::All).is_err());
    }

    // --- local_scan_1_ecount_them tests ---

    #[test]
    fn scan1t_basic() {
        let mut us = Graph::with_vertices(3);
        us.add_edge(0, 1).unwrap();
        us.add_edge(1, 2).unwrap();
        let mut them = Graph::with_vertices(3);
        them.add_edge(0, 1).unwrap();
        them.add_edge(0, 2).unwrap();
        them.add_edge(1, 2).unwrap();
        let res = local_scan_1_ecount_them(&us, &them, None, DegreeMode::All).unwrap();
        // v1: nbd in us = {0,1,2}, edges in them within: 0-1,0-2,1-2 = 3
        assert!(close(res[1], 3.0));
    }

    #[test]
    fn scan1t_vcount_mismatch() {
        let us = Graph::with_vertices(3);
        let them = Graph::with_vertices(4);
        assert!(local_scan_1_ecount_them(&us, &them, None, DegreeMode::All).is_err());
    }

    // --- local_scan_subset_ecount tests ---

    #[test]
    fn subset_triangle() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        let subsets = vec![vec![0, 1, 2], vec![2, 3], vec![0, 3]];
        let res = local_scan_subset_ecount(&g, &subsets, None).unwrap();
        assert!(close(res[0], 3.0));
        assert!(close(res[1], 1.0));
        assert!(close(res[2], 0.0));
    }

    #[test]
    fn subset_weighted() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let w = vec![3.0, 7.0];
        let subsets = vec![vec![0, 1, 2]];
        let res = local_scan_subset_ecount(&g, &subsets, Some(&w)).unwrap();
        assert!(close(res[0], 10.0));
    }

    #[test]
    fn subset_empty_subset() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let subsets: Vec<Vec<VertexId>> = vec![vec![]];
        let res = local_scan_subset_ecount(&g, &subsets, None).unwrap();
        assert!(close(res[0], 0.0));
    }

    #[test]
    fn subset_invalid_vertex() {
        let g = Graph::with_vertices(3);
        let subsets = vec![vec![0, 5]];
        assert!(local_scan_subset_ecount(&g, &subsets, None).is_err());
    }

    #[test]
    fn subset_directed() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let subsets = vec![vec![0, 1, 2]];
        let res = local_scan_subset_ecount(&g, &subsets, None).unwrap();
        assert!(close(res[0], 3.0));
    }
}
