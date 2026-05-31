//! Graphlet decomposition (ALGO-CL-020).
//!
//! Counterpart of `igraph_graphlets()`, `igraph_graphlets_candidate_basis()`,
//! and `igraph_graphlets_project()` from `references/igraph/src/cliques/glet.c`.
//!
//! Graphlet decomposition models a weighted undirected graph via the union of
//! potentially overlapping dense social groups. Step one finds a candidate
//! basis of cliques at weight thresholds; step two projects the graph onto
//! that basis via an iterative NNLS-like algorithm.
//!
//! Reference: Hossein Azari Soufiani and Edoardo M. Airoldi,
//! "Graphlet decomposition of a weighted network" (2012).

use crate::algorithms::cliques::maximal_cliques;
use crate::algorithms::properties::is_simple::is_simple;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Result of [`graphlets_candidate_basis`]: a list of basis cliques with
/// their associated weight thresholds.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphletBasis {
    /// Each element is a sorted list of vertex IDs forming a basis clique.
    pub cliques: Vec<Vec<VertexId>>,
    /// `thresholds[i]` is the weight threshold at which `cliques[i]` was
    /// found (the minimum edge weight within that clique).
    pub thresholds: Vec<f64>,
}

/// Result of [`graphlets`]: basis cliques with their projected weights,
/// sorted by decreasing weight.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphletDecomposition {
    /// Basis cliques, sorted by decreasing projected weight.
    pub cliques: Vec<Vec<VertexId>>,
    /// `mu[i]` is the projected weight coefficient of `cliques[i]`.
    pub mu: Vec<f64>,
}

/// Find a candidate graphlets basis for a weighted undirected graph.
///
/// The input graph must be simple (no self-loops, no parallel edges). Edge
/// directions are ignored. The algorithm recursively finds maximal cliques
/// at successively higher weight thresholds, then filters out non-maximal
/// cliques that are subsets of larger cliques at the same threshold.
///
/// # Errors
///
/// - `InvalidArgument` if `weights` length doesn't match edge count, or
///   the graph is not simple.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, graphlets_candidate_basis};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap(); // weight 1
/// g.add_edge(1, 2).unwrap(); // weight 2
/// g.add_edge(0, 2).unwrap(); // weight 1
/// g.add_edge(2, 3).unwrap(); // weight 3
///
/// let basis = graphlets_candidate_basis(&g, &[1.0, 2.0, 1.0, 3.0]).unwrap();
/// assert!(!basis.cliques.is_empty());
/// ```
pub fn graphlets_candidate_basis(graph: &Graph, weights: &[f64]) -> IgraphResult<GraphletBasis> {
    validate_graphlets_input(graph, weights)?;

    let n = graph.vcount();
    let ecount = graph.ecount();

    if ecount == 0 {
        return Ok(GraphletBasis {
            cliques: Vec::new(),
            thresholds: Vec::new(),
        });
    }

    let min_thr = weights.iter().copied().fold(f64::INFINITY, f64::min);

    let ids: Vec<VertexId> = (0..n).collect();

    let mut cliques: Vec<Vec<VertexId>> = Vec::new();
    let mut thresholds: Vec<f64> = Vec::new();

    graphlets_recursive(graph, weights, &ids, min_thr, &mut cliques, &mut thresholds)?;

    graphlets_filter(&mut cliques, &mut thresholds);

    Ok(GraphletBasis {
        cliques,
        thresholds,
    })
}

/// Project a graph onto a graphlets basis.
///
/// Performs `niter` iterations of the multiplicative update (NNLS-like).
/// If `start_mu` is `Some(vec)`, uses it as the initial weight vector;
/// otherwise starts from all-ones.
///
/// # Errors
///
/// - `InvalidArgument` if `weights` length doesn't match edge count,
///   `start_mu` length doesn't match clique count, `niter < 0`, or
///   the graph is not simple.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, graphlets_candidate_basis, graphlets_project};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let weights = [1.0, 2.0, 1.0, 3.0];
/// let basis = graphlets_candidate_basis(&g, &weights).unwrap();
/// let mu = graphlets_project(&g, &weights, &basis.cliques, None, 1000).unwrap();
/// assert_eq!(mu.len(), basis.cliques.len());
/// ```
pub fn graphlets_project(
    graph: &Graph,
    weights: &[f64],
    cliques: &[Vec<VertexId>],
    start_mu: Option<&[f64]>,
    niter: u32,
) -> IgraphResult<Vec<f64>> {
    validate_graphlets_input(graph, weights)?;

    let no_cliques = cliques.len();
    if no_cliques == 0 {
        return Ok(Vec::new());
    }

    if let Some(sm) = start_mu {
        if sm.len() != no_cliques {
            return Err(IgraphError::InvalidArgument(format!(
                "start_mu length {} does not match clique count {no_cliques}",
                sm.len()
            )));
        }
    }

    let mut mu = match start_mu {
        Some(sm) => sm.to_vec(),
        None => vec![1.0; no_cliques],
    };

    project_inner(graph, weights, cliques, &mut mu, niter)?;

    Ok(mu)
}

/// Full graphlet decomposition: find basis + project + sort by weight.
///
/// Convenience function that calls [`graphlets_candidate_basis`] and
/// [`graphlets_project`], then sorts the result by decreasing weight.
///
/// # Errors
///
/// - `InvalidArgument` if `weights` length doesn't match edge count, or
///   the graph is not simple.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, graphlets};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let result = graphlets(&g, &[1.0, 2.0, 1.0, 3.0], 1000).unwrap();
/// // Weights are in decreasing order
/// for w in result.mu.windows(2) {
///     assert!(w[0] >= w[1]);
/// }
/// ```
pub fn graphlets(
    graph: &Graph,
    weights: &[f64],
    niter: u32,
) -> IgraphResult<GraphletDecomposition> {
    let basis = graphlets_candidate_basis(graph, weights)?;
    let mu = graphlets_project(graph, weights, &basis.cliques, None, niter)?;

    let mut order: Vec<usize> = (0..mu.len()).collect();
    order.sort_by(|&a, &b| {
        mu[b]
            .partial_cmp(&mu[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let cliques: Vec<Vec<VertexId>> = order.iter().map(|&i| basis.cliques[i].clone()).collect();
    let sorted_mu: Vec<f64> = order.iter().map(|&i| mu[i]).collect();

    Ok(GraphletDecomposition {
        cliques,
        mu: sorted_mu,
    })
}

fn validate_graphlets_input(graph: &Graph, weights: &[f64]) -> IgraphResult<()> {
    if weights.len() != graph.ecount() {
        return Err(IgraphError::InvalidArgument(format!(
            "weights length {} does not match edge count {}",
            weights.len(),
            graph.ecount()
        )));
    }
    if !is_simple(graph)? {
        return Err(IgraphError::InvalidArgument(
            "graphlets require a simple graph".to_string(),
        ));
    }
    Ok(())
}

type SubCliqueResult = (Graph, Vec<f64>, Vec<VertexId>);

// ---------- internal: recursive basis construction ----------

fn graphlets_recursive(
    graph: &Graph,
    weights: &[f64],
    ids: &[VertexId],
    start_thr: f64,
    cliques_out: &mut Vec<Vec<VertexId>>,
    thresholds_out: &mut Vec<f64>,
) -> IgraphResult<()> {
    let n = graph.vcount();

    // Build subgraph from edges >= start_thr by zeroing out adjacency
    // for edges below threshold. We build an undirected adjacency for
    // maximal_cliques which ignores directions.
    let mut sub = Graph::with_vertices(n);
    for (eid, &w) in weights.iter().enumerate() {
        if w >= start_thr {
            #[allow(clippy::cast_possible_truncation)]
            let (src, tgt) = graph.edge(eid as u32)?;
            sub.add_edge(src, tgt)?;
        }
    }

    let my_cliques = maximal_cliques(&sub)?;

    if my_cliques.is_empty() {
        return Ok(());
    }

    // For each clique, find the minimum edge weight (clique threshold)
    // and the next threshold (smallest weight strictly above minimum).
    // Then build a sub-graph of edges >= next_thr for recursion.
    let no_cliques = my_cliques.len();
    let mut clique_thrs = Vec::with_capacity(no_cliques);
    let mut next_thrs = Vec::with_capacity(no_cliques);
    let mut sub_graphs: Vec<Option<SubCliqueResult>> = Vec::with_capacity(no_cliques);

    for clique in &my_cliques {
        let (min_w, next_w) = find_clique_thresholds(graph, weights, clique);
        clique_thrs.push(min_w);
        next_thrs.push(next_w);

        // Build sub-graph for recursion: edges within clique that are >= next_w
        if next_w.is_finite() {
            let sub_result = build_subclique_graph(graph, weights, ids, clique, next_w)?;
            sub_graphs.push(sub_result);
        } else {
            sub_graphs.push(None);
        }
    }

    // Store cliques at the current level, mapping vertex IDs through ids[]
    for (i, clique) in my_cliques.iter().enumerate() {
        let mut mapped: Vec<VertexId> = clique.iter().map(|&v| ids[v as usize]).collect();
        mapped.sort_unstable();
        cliques_out.push(mapped);
        thresholds_out.push(clique_thrs[i]);
    }

    // Recurse into sub-graphs
    for (i, sub_opt) in sub_graphs.into_iter().enumerate() {
        if let Some((sub_g, sub_w, sub_ids)) = sub_opt {
            if sub_g.vcount() > 1 {
                graphlets_recursive(
                    &sub_g,
                    &sub_w,
                    &sub_ids,
                    next_thrs[i],
                    cliques_out,
                    thresholds_out,
                )?;
            }
        }
    }

    Ok(())
}

/// Find the minimum edge weight within a clique and the next-smallest
/// weight strictly above the minimum.
fn find_clique_thresholds(graph: &Graph, weights: &[f64], clique: &[VertexId]) -> (f64, f64) {
    let mut min_weight = f64::INFINITY;
    let mut next_weight = f64::INFINITY;

    let n = clique.len();
    // Check all pairs within the clique
    for i in 0..n {
        for j in (i + 1)..n {
            let vi = clique[i];
            let vj = clique[j];
            if let Some(w) = edge_weight_between(graph, weights, vi, vj) {
                if w < min_weight {
                    next_weight = min_weight;
                    min_weight = w;
                } else if w > min_weight && w < next_weight {
                    next_weight = w;
                }
            }
        }
    }

    (min_weight, next_weight)
}

/// Find the weight of the edge between two vertices (undirected).
fn edge_weight_between(graph: &Graph, weights: &[f64], v1: VertexId, v2: VertexId) -> Option<f64> {
    if let Ok(edges) = graph.incident(v1) {
        for eid in edges {
            if let Ok(other) = graph.edge_other(eid, v1) {
                if other == v2 {
                    return Some(weights[eid as usize]);
                }
            }
        }
    }
    None
}

/// Build a sub-graph from edges within a clique that are >= `next_thr`.
/// Returns `(sub_graph, sub_weights, sub_ids)` where `sub_ids` maps
/// sub-graph vertex IDs back to original vertex IDs via the parent `ids`.
#[allow(clippy::cast_possible_truncation)]
fn build_subclique_graph(
    graph: &Graph,
    weights: &[f64],
    ids: &[VertexId],
    clique: &[VertexId],
    next_thr: f64,
) -> IgraphResult<Option<SubCliqueResult>> {
    let n = graph.vcount() as usize;
    let mut in_clique = vec![false; n];
    for &v in clique {
        in_clique[v as usize] = true;
    }

    let mut edges_within: Vec<(VertexId, VertexId, f64)> = Vec::new();
    for &v in clique {
        if let Ok(inc) = graph.incident(v) {
            for eid in inc {
                if let Ok(other) = graph.edge_other(eid, v) {
                    if other > v && in_clique[other as usize] {
                        edges_within.push((v, other, weights[eid as usize]));
                    }
                }
            }
        }
    }

    // Filter edges >= next_thr and collect incident vertices
    let mut vertex_map: Vec<Option<u32>> = vec![None; n];
    let mut sub_ids: Vec<VertexId> = Vec::new();
    let mut sub_edges: Vec<(u32, u32, f64)> = Vec::new();
    let mut nov: u32 = 0;

    for &(efrom, eto, ew) in &edges_within {
        if ew >= next_thr {
            let from_mapped = *vertex_map[efrom as usize].get_or_insert_with(|| {
                let m = nov;
                sub_ids.push(ids[efrom as usize]);
                nov += 1;
                m
            });
            let to_mapped = *vertex_map[eto as usize].get_or_insert_with(|| {
                let m = nov;
                sub_ids.push(ids[eto as usize]);
                nov += 1;
                m
            });
            sub_edges.push((from_mapped, to_mapped, ew));
        }
    }

    if nov <= 1 || sub_edges.is_empty() {
        return Ok(None);
    }

    let mut sub_g = Graph::with_vertices(nov);
    let mut sub_w = Vec::with_capacity(sub_edges.len());
    for (f, t, w) in sub_edges {
        sub_g.add_edge(f, t)?;
        sub_w.push(w);
    }

    Ok(Some((sub_g, sub_w, sub_ids)))
}

// ---------- internal: filter non-maximal cliques ----------

fn graphlets_filter(cliques: &mut Vec<Vec<VertexId>>, thresholds: &mut Vec<f64>) {
    let n = cliques.len();
    if n <= 1 {
        return;
    }

    // Sort indices by (threshold, clique_size)
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        thresholds[a]
            .partial_cmp(&thresholds[b])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| cliques[a].len().cmp(&cliques[b].len()))
    });

    let mut to_remove = vec![false; n];

    for ii in 0..n.saturating_sub(1) {
        let ri = order[ii];
        if to_remove[ri] {
            continue;
        }
        let thr_i = thresholds[ri];

        for &rj in &order[(ii + 1)..n] {
            if to_remove[rj] {
                continue;
            }
            let thr_j = thresholds[rj];

            if (thr_j - thr_i).abs() > f64::EPSILON * thr_i.abs().max(1.0) {
                break;
            }

            if cliques[ri].len() > cliques[rj].len() {
                continue;
            }

            // Check if cliques[ri] is a subset of cliques[rj]
            if is_sorted_subset(&cliques[ri], &cliques[rj]) {
                to_remove[ri] = true;
                break;
            }
        }
    }

    // Compact
    let mut write = 0;
    for (read, &remove) in to_remove.iter().enumerate() {
        if !remove {
            if write != read {
                cliques.swap(write, read);
                thresholds.swap(write, read);
            }
            write += 1;
        }
    }
    cliques.truncate(write);
    thresholds.truncate(write);
}

/// Check if sorted slice `needle` is a subset of sorted slice `hay`.
fn is_sorted_subset(needle: &[VertexId], hay: &[VertexId]) -> bool {
    if needle.len() > hay.len() {
        return false;
    }
    let mut pi = 0;
    let mut pj = 0;
    let n_i = needle.len();
    let n_j = hay.len();

    while pi < n_i && pj < n_j && (n_i - pi) <= (n_j - pj) {
        match needle[pi].cmp(&hay[pj]) {
            std::cmp::Ordering::Less => return false,
            std::cmp::Ordering::Greater => pj += 1,
            std::cmp::Ordering::Equal => {
                pi += 1;
                pj += 1;
            }
        }
    }
    pi == n_i
}

// ---------- internal: projection (NNLS-like) ----------

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn project_inner(
    graph: &Graph,
    weights: &[f64],
    cliques: &[Vec<VertexId>],
    mu: &mut [f64],
    niter: u32,
) -> IgraphResult<()> {
    let no_of_edges = graph.ecount();
    let no_of_nodes = graph.vcount() as usize;
    let no_cliques = cliques.len();

    // Build vertex→clique index: for each vertex, which cliques contain it
    let mut vcl: Vec<Vec<usize>> = vec![Vec::new(); no_of_nodes];
    for (ci, clique) in cliques.iter().enumerate() {
        for &v in clique {
            vcl[v as usize].push(ci);
        }
    }
    // Sort for merge-intersection
    for v in &mut vcl {
        v.sort_unstable();
    }

    // Build edge→clique list: for each edge, which cliques contain both endpoints
    let mut ecl: Vec<Vec<usize>> = Vec::with_capacity(no_of_edges);
    let mut edge_from_to: Vec<(VertexId, VertexId)> = Vec::with_capacity(no_of_edges);
    for eid in 0..no_of_edges {
        let (from, to) = graph.edge(eid as u32)?;
        edge_from_to.push((from, to));
        let from_cl = &vcl[from as usize];
        let to_cl = &vcl[to as usize];
        ecl.push(sorted_intersection(from_cl, to_cl));
    }

    // Build clique→edge list (inverse of edge→clique)
    let mut cel: Vec<Vec<usize>> = vec![Vec::new(); no_cliques];
    for (eid, cls) in ecl.iter().enumerate() {
        for &ci in cls {
            cel[ci].push(eid);
        }
    }

    // Normalizing factors: n*(n+1)/2 where n is clique size
    let normfact: Vec<f64> = cliques
        .iter()
        .map(|c| {
            let n = c.len() as f64;
            n * (n + 1.0) / 2.0
        })
        .collect();

    // Iterative projection
    let mut new_weights = vec![0.0_f64; no_of_edges];

    for _iter in 0..niter {
        // Compute predicted weights for each edge
        for (eid, cls) in ecl.iter().enumerate() {
            new_weights[eid] = 0.0001;
            for &ci in cls {
                new_weights[eid] += mu[ci];
            }
        }

        // Update mu
        for (ci, edges) in cel.iter().enumerate() {
            let mut sum_ratio = 0.0_f64;
            for &eid in edges {
                sum_ratio += weights[eid] / new_weights[eid];
            }
            mu[ci] *= sum_ratio / normfact[ci];
        }
    }

    Ok(())
}

/// Merge-intersect two sorted slices.
fn sorted_intersection(a: &[usize], b: &[usize]) -> Vec<usize> {
    let mut result = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Equal => {
                result.push(a[i]);
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_with_pendant() -> (Graph, Vec<f64>) {
        // Triangle {0,1,2} + pendant edge {2,3}
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // e0: weight 1
        g.add_edge(1, 2).unwrap(); // e1: weight 2
        g.add_edge(0, 2).unwrap(); // e2: weight 1
        g.add_edge(2, 3).unwrap(); // e3: weight 3
        (g, vec![1.0, 2.0, 1.0, 3.0])
    }

    #[test]
    fn candidate_basis_nonempty() {
        let (g, w) = triangle_with_pendant();
        let basis = graphlets_candidate_basis(&g, &w).unwrap();
        assert!(!basis.cliques.is_empty());
        assert_eq!(basis.cliques.len(), basis.thresholds.len());
    }

    #[test]
    fn candidate_basis_cliques_sorted() {
        let (g, w) = triangle_with_pendant();
        let basis = graphlets_candidate_basis(&g, &w).unwrap();
        for clique in &basis.cliques {
            let mut sorted = clique.clone();
            sorted.sort_unstable();
            assert_eq!(clique, &sorted);
        }
    }

    #[test]
    fn candidate_basis_empty_graph() {
        let g = Graph::with_vertices(5);
        let basis = graphlets_candidate_basis(&g, &[]).unwrap();
        assert!(basis.cliques.is_empty());
        assert!(basis.thresholds.is_empty());
    }

    #[test]
    fn candidate_basis_single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let basis = graphlets_candidate_basis(&g, &[5.0]).unwrap();
        assert_eq!(basis.cliques.len(), 1);
        assert_eq!(basis.cliques[0], vec![0, 1]);
    }

    #[test]
    fn candidate_basis_wrong_weight_length() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert!(graphlets_candidate_basis(&g, &[1.0, 2.0]).is_err());
    }

    #[test]
    fn candidate_basis_non_simple_graph_errors() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap(); // multi-edge
        assert!(graphlets_candidate_basis(&g, &[1.0, 2.0]).is_err());
    }

    #[test]
    fn project_produces_correct_length() {
        let (g, w) = triangle_with_pendant();
        let basis = graphlets_candidate_basis(&g, &w).unwrap();
        let mu = graphlets_project(&g, &w, &basis.cliques, None, 100).unwrap();
        assert_eq!(mu.len(), basis.cliques.len());
    }

    #[test]
    fn project_nonnegative_weights() {
        let (g, w) = triangle_with_pendant();
        let basis = graphlets_candidate_basis(&g, &w).unwrap();
        let mu = graphlets_project(&g, &w, &basis.cliques, None, 100).unwrap();
        for &m in &mu {
            assert!(m >= 0.0, "mu should be non-negative, got {m}");
        }
    }

    #[test]
    fn project_with_start_mu() {
        let (g, w) = triangle_with_pendant();
        let basis = graphlets_candidate_basis(&g, &w).unwrap();
        let start: Vec<f64> = vec![2.0; basis.cliques.len()];
        let mu = graphlets_project(&g, &w, &basis.cliques, Some(&start), 100).unwrap();
        assert_eq!(mu.len(), basis.cliques.len());
    }

    #[test]
    fn project_wrong_start_mu_length() {
        let (g, w) = triangle_with_pendant();
        let basis = graphlets_candidate_basis(&g, &w).unwrap();
        assert!(graphlets_project(&g, &w, &basis.cliques, Some(&[1.0]), 100).is_err());
    }

    #[test]
    fn graphlets_full_sorted_decreasing() {
        let (g, w) = triangle_with_pendant();
        let result = graphlets(&g, &w, 1000).unwrap();
        assert_eq!(result.cliques.len(), result.mu.len());
        for pair in result.mu.windows(2) {
            assert!(
                pair[0] >= pair[1],
                "mu should be sorted decreasing: {} < {}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn graphlets_full_empty() {
        let g = Graph::with_vertices(3);
        let result = graphlets(&g, &[], 100).unwrap();
        assert!(result.cliques.is_empty());
        assert!(result.mu.is_empty());
    }

    #[test]
    fn graphlets_complete_graph_uniform_weights() {
        // K4 with uniform weights: should produce a single 4-clique
        let mut g = Graph::with_vertices(4);
        let mut w = Vec::new();
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
                w.push(1.0);
            }
        }
        let basis = graphlets_candidate_basis(&g, &w).unwrap();
        // All vertices form one maximal clique at threshold 1
        assert!(basis.cliques.iter().any(|c| c.len() == 4));
    }

    #[test]
    fn is_sorted_subset_basic() {
        assert!(is_sorted_subset(&[1, 3], &[1, 2, 3, 4]));
        assert!(!is_sorted_subset(&[1, 5], &[1, 2, 3, 4]));
        assert!(is_sorted_subset(&[], &[1, 2, 3]));
        assert!(!is_sorted_subset(&[1, 2, 3], &[1, 2]));
        assert!(is_sorted_subset(&[1, 2], &[1, 2]));
    }

    #[test]
    fn filter_removes_subsets() {
        let mut cliques = vec![vec![0, 1], vec![0, 1, 2], vec![3, 4]];
        let mut thresholds = vec![1.0, 1.0, 2.0];
        graphlets_filter(&mut cliques, &mut thresholds);
        // {0,1} is a subset of {0,1,2} at the same threshold
        assert!(!cliques.contains(&vec![0, 1]));
        assert!(cliques.contains(&vec![0, 1, 2]));
        assert!(cliques.contains(&vec![3, 4]));
    }

    #[test]
    fn filter_keeps_different_thresholds() {
        let mut cliques = vec![vec![0, 1], vec![0, 1, 2]];
        let mut thresholds = vec![1.0, 2.0];
        graphlets_filter(&mut cliques, &mut thresholds);
        // Different thresholds: both kept
        assert_eq!(cliques.len(), 2);
    }

    #[test]
    fn project_zero_iterations() {
        let (g, w) = triangle_with_pendant();
        let basis = graphlets_candidate_basis(&g, &w).unwrap();
        let mu = graphlets_project(&g, &w, &basis.cliques, None, 0).unwrap();
        // Zero iterations: mu stays at initial all-ones
        for &m in &mu {
            assert!((m - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn graphlets_path_graph() {
        // Path 0-1-2-3 with increasing weights
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let w = [1.0, 2.0, 3.0];
        let result = graphlets(&g, &w, 100).unwrap();
        // Path graph: each edge is a maximal clique
        assert!(!result.cliques.is_empty());
        for c in &result.cliques {
            assert_eq!(c.len(), 2);
        }
    }
}
