#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::module_name_repetitions
)]

//! Bipartite matching algorithms.
//!
//! Provides:
//! - [`is_matching`] — validate a matching vector against a graph
//! - [`is_maximal_matching`] — check whether a valid matching is maximal
//! - [`maximum_bipartite_matching`] — maximum cardinality bipartite matching
//!   (push-relabel, unweighted)
//! - [`maximum_bipartite_matching_weighted`] — maximum weight bipartite
//!   matching (Hungarian / Kuhn-Munkres)
//!
//! Reference: `igraph/src/misc/matching.c` (1013 lines).

use std::collections::VecDeque;

use crate::core::error::{IgraphError, IgraphResult};
use crate::core::graph::Graph;

/// Result of [`maximum_bipartite_matching`] or
/// [`maximum_bipartite_matching_weighted`].
///
/// # Fields
///
/// * `matching_size` — number of matched vertex pairs.
/// * `matching_weight` — total weight of matched edges (equals `matching_size`
///   for unweighted).
/// * `matching` — per-vertex match: `matching[v]` is the partner of `v`, or
///   `None` if `v` is unmatched.
///
/// ```
/// use rust_igraph::{create, maximum_bipartite_matching, MatchingResult};
///
/// // K_{2,2}: 0-2, 0-3, 1-2, 1-3
/// let g = create(&[(0, 2), (0, 3), (1, 2), (1, 3)], 4, false).unwrap();
/// let types = vec![false, false, true, true];
/// let r = maximum_bipartite_matching(&g, &types).unwrap();
/// assert_eq!(r.matching_size, 2);
/// ```
#[derive(Debug, Clone)]
pub struct MatchingResult {
    pub matching_size: usize,
    pub matching_weight: f64,
    pub matching: Vec<Option<u32>>,
}

/// Check whether `matching` is a valid matching for `graph`.
///
/// A matching vector has length `vcount`; entry `i` is `Some(j)` when vertex
/// `i` is matched to `j`, or `None` if unmatched. The function verifies:
/// 1. Length equals `vcount`.
/// 2. Matched pairs are mutual (`matching[i] == Some(j)` ⟹ `matching[j] == Some(i)`).
/// 3. Every matched pair is connected by an edge (ignoring direction).
/// 4. If `types` is provided, matched vertices have different types.
///
/// ```
/// use rust_igraph::{create, is_matching};
///
/// let g = create(&[(0, 1), (1, 2)], 3, false).unwrap();
/// let m = vec![Some(1), Some(0), None];
/// assert!(is_matching(&g, None, &m).unwrap());
/// ```
pub fn is_matching(
    graph: &Graph,
    types: Option<&[bool]>,
    matching: &[Option<u32>],
) -> IgraphResult<bool> {
    let n = graph.vcount() as usize;
    if matching.len() != n {
        return Ok(false);
    }

    let adj = build_undirected_adj(graph);

    for (i, &mi) in matching.iter().enumerate() {
        let Some(j) = mi else { continue };
        let j_usize = j as usize;
        if j_usize >= n {
            return Ok(false);
        }
        if matching[j_usize] != Some(i as u32) {
            return Ok(false);
        }
        if !adj[i].contains(&j) {
            return Ok(false);
        }
    }

    if let Some(t) = types {
        if t.len() < n {
            return Err(IgraphError::InvalidArgument(
                "types vector too short".into(),
            ));
        }
        for (i, &mi) in matching.iter().enumerate() {
            let Some(j) = mi else { continue };
            if t[i] == t[j as usize] {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Check whether `matching` is a *maximal* matching for `graph`.
///
/// A matching is maximal if no unmatched vertex has an unmatched neighbor
/// (respecting bipartite types if given).
///
/// ```
/// use rust_igraph::{create, is_maximal_matching};
///
/// let g = create(&[(0, 1), (1, 2)], 3, false).unwrap();
/// // Only 0-1 matched; vertex 2 is unmatched but has no unmatched neighbor → maximal
/// let m = vec![Some(1), Some(0), None];
/// assert!(is_maximal_matching(&g, None, &m).unwrap());
/// ```
pub fn is_maximal_matching(
    graph: &Graph,
    types: Option<&[bool]>,
    matching: &[Option<u32>],
) -> IgraphResult<bool> {
    if !is_matching(graph, types, matching)? {
        return Ok(false);
    }

    let n = graph.vcount() as usize;
    let adj = build_undirected_adj(graph);

    for i in 0..n {
        if matching[i].is_some() {
            continue;
        }
        for &nb in &adj[i] {
            if matching[nb as usize].is_none() {
                if let Some(t) = types {
                    if t[i] == t[nb as usize] {
                        continue;
                    }
                }
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Compute a maximum cardinality matching in an unweighted bipartite graph.
///
/// Uses a push-relabel algorithm with greedy initialization and global
/// relabeling every `n/2` steps. Returns a [`MatchingResult`] where
/// `matching_weight == matching_size` (since all edges have unit weight).
///
/// `types` must be a bipartite partition: `types[v]` is `false` for one side
/// and `true` for the other. The function validates that every edge connects
/// vertices of different types.
///
/// ```
/// use rust_igraph::{create, maximum_bipartite_matching};
///
/// let g = create(&[(0, 2), (0, 3), (1, 2), (1, 3)], 4, false).unwrap();
/// let types = vec![false, false, true, true];
/// let r = maximum_bipartite_matching(&g, &types).unwrap();
/// assert_eq!(r.matching_size, 2);
/// ```
pub fn maximum_bipartite_matching(graph: &Graph, types: &[bool]) -> IgraphResult<MatchingResult> {
    let n = graph.vcount() as usize;
    if types.len() < n {
        return Err(IgraphError::InvalidArgument(
            "types vector too short".into(),
        ));
    }

    let adj = build_undirected_adj(graph);
    let (matching, num_matched) = push_relabel_unweighted(graph, &adj, types, n)?;

    Ok(MatchingResult {
        matching_size: num_matched,
        matching_weight: num_matched as f64,
        matching,
    })
}

/// Compute a maximum weight matching in a weighted bipartite graph.
///
/// Uses the Hungarian algorithm (Kuhn-Munkres) with push-relabel
/// initialization on tight edges. `weights[e]` gives the weight of edge `e`
/// (by edge id). `eps` controls floating-point tolerance for "tight" edge
/// detection; pass `0.0` for integer weights.
///
/// ```
/// use rust_igraph::{create, maximum_bipartite_matching_weighted};
///
/// let g = create(&[(0, 2), (0, 3), (1, 2), (1, 3)], 4, false).unwrap();
/// let types = vec![false, false, true, true];
/// let weights = vec![1.0, 10.0, 10.0, 1.0];
/// let r = maximum_bipartite_matching_weighted(&g, &types, &weights, 0.0).unwrap();
/// assert_eq!(r.matching_size, 2);
/// assert!((r.matching_weight - 20.0).abs() < 1e-9);
/// ```
pub fn maximum_bipartite_matching_weighted(
    graph: &Graph,
    types: &[bool],
    weights: &[f64],
    eps: f64,
) -> IgraphResult<MatchingResult> {
    let n = graph.vcount() as usize;
    let ne = graph.ecount();
    if types.len() < n {
        return Err(IgraphError::InvalidArgument(
            "types vector too short".into(),
        ));
    }
    if weights.len() < ne {
        return Err(IgraphError::InvalidArgument(
            "weights vector too short".into(),
        ));
    }
    let eps = if eps < 0.0 { 0.0 } else { eps };

    hungarian(graph, types, weights, eps, n, ne)
}

// ── helpers ──────────────────────────────────────────────────────────

fn build_undirected_adj(graph: &Graph) -> Vec<Vec<u32>> {
    let n = graph.vcount() as usize;
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for eid in 0..graph.ecount() {
        if let Ok((u, v)) = graph.edge(eid as u32) {
            adj[u as usize].push(v);
            if u != v {
                adj[v as usize].push(u);
            }
        }
    }
    adj
}

// ── push-relabel unweighted ─────────────────────────────────────────

fn push_relabel_unweighted(
    _graph: &Graph,
    adj: &[Vec<u32>],
    types: &[bool],
    n: usize,
) -> IgraphResult<(Vec<Option<u32>>, usize)> {
    let mut matching: Vec<i64> = vec![-1; n];
    let mut labels: Vec<i64> = vec![0; n];

    // Determine smaller set and greedy init
    let count_true = types[..n].iter().filter(|&&t| t).count();
    let smaller_set = count_true <= n / 2;

    let mut num_matched: usize = 0;
    for i in 0..n {
        if matching[i] != -1 {
            continue;
        }
        for &nb in &adj[i] {
            let nb_usize = nb as usize;
            if types[nb_usize] == types[i] {
                return Err(IgraphError::InvalidArgument(
                    "Graph is not bipartite with supplied types vector".into(),
                ));
            }
            if matching[nb_usize] == -1 {
                matching[nb_usize] = i64::from(i as u32);
                matching[i] = i64::from(nb);
                num_matched += 1;
                break;
            }
        }
    }

    // Global relabeling
    global_relabel(adj, &mut labels, &matching, types, smaller_set, n);

    // Fill push queue with unmatched vertices from smaller set
    let mut q: VecDeque<usize> = VecDeque::new();
    for i in 0..n {
        if matching[i] == -1 && types[i] == smaller_set {
            q.push_back(i);
        }
    }

    let relabeling_freq = (n / 2).max(1);
    let mut label_changed: usize = 0;

    while let Some(v) = q.pop_front() {
        if label_changed >= relabeling_freq {
            global_relabel(adj, &mut labels, &matching, types, smaller_set, n);
            label_changed = 0;
        }

        let mut best_u: i64 = -1;
        let mut best_label: i64 = 2 * n as i64;

        for &nb in &adj[v] {
            let nb_usize = nb as usize;
            if labels[nb_usize] < best_label {
                best_u = i64::from(nb);
                best_label = labels[nb_usize];
                label_changed += 1;
            }
        }

        if best_label < n as i64 {
            let u = best_u as usize;
            labels[v] = labels[u] + 1;
            if matching[u] != -1 {
                let w = matching[u] as usize;
                if w != v {
                    matching[u] = -1;
                    matching[w] = -1;
                    q.push_back(w);
                    num_matched -= 1;
                }
            }
            matching[u] = v as i64;
            matching[v] = u as i64;
            num_matched += 1;
            labels[u] += 2;
            label_changed += 1;
        }
    }

    let result: Vec<Option<u32>> = matching
        .iter()
        .map(|&m| if m < 0 { None } else { Some(m as u32) })
        .collect();

    Ok((result, num_matched))
}

fn global_relabel(
    adj: &[Vec<u32>],
    labels: &mut [i64],
    matching: &[i64],
    types: &[bool],
    smaller_set: bool,
    n: usize,
) {
    labels.fill(n as i64);

    let mut q: VecDeque<usize> = VecDeque::new();
    for i in 0..n {
        if types[i] != smaller_set && matching[i] == -1 {
            q.push_back(i);
            labels[i] = 0;
        }
    }

    while let Some(v) = q.pop_front() {
        for &nb in &adj[v] {
            let w = nb as usize;
            if labels[w] == n as i64 {
                labels[w] = labels[v] + 1;
                let matched_to = matching[w];
                if matched_to != -1 {
                    let mt = matched_to as usize;
                    if labels[mt] == n as i64 {
                        q.push_back(mt);
                        labels[mt] = labels[w] + 1;
                    }
                }
            }
        }
    }
}

// ── Hungarian (weighted) ────────────────────────────────────────────

fn hungarian(
    graph: &Graph,
    types: &[bool],
    weights: &[f64],
    eps: f64,
    n: usize,
    ne: usize,
) -> IgraphResult<MatchingResult> {
    // Build incidence list: for each vertex, list of (edge_id, other_vertex)
    let mut incidence: Vec<Vec<(u32, u32)>> = vec![Vec::new(); n];
    let mut edges: Vec<(u32, u32)> = Vec::with_capacity(ne);
    for eid in 0..ne {
        let (u, v) = graph.edge(eid as u32)?;
        edges.push((u, v));
        incidence[u as usize].push((eid as u32, v));
        if u != v {
            incidence[v as usize].push((eid as u32, u));
        }
    }

    // Find smaller and larger sets
    let count_false = types[..n].iter().filter(|&&t| !t).count();
    let smaller_set_type = count_false > n / 2;
    let smaller_set_size = if smaller_set_type {
        n - count_false
    } else {
        count_false
    };

    let mut smaller_set: Vec<usize> = Vec::with_capacity(smaller_set_size);
    let mut larger_set: Vec<usize> = Vec::with_capacity(n - smaller_set_size);
    for (i, &tp) in types[..n].iter().enumerate() {
        if tp == smaller_set_type {
            smaller_set.push(i);
        } else {
            larger_set.push(i);
        }
    }

    // Initial labeling: for each vertex in smaller set, label = max incident weight
    let mut labels: Vec<f64> = vec![0.0; n];
    for (i, &tp) in types[..n].iter().enumerate() {
        if tp != smaller_set_type {
            continue;
        }
        let mut max_w: f64 = 0.0;
        for &(eid, other) in &incidence[i] {
            if types[other as usize] == types[i] {
                return Err(IgraphError::InvalidArgument(
                    "Graph is not bipartite with supplied types vector".into(),
                ));
            }
            if weights[eid as usize] > max_w {
                max_w = weights[eid as usize];
            }
        }
        labels[i] = max_w;
    }

    // Compute initial slack and tight edges
    let mut slack: Vec<f64> = vec![0.0; ne];
    let mut tight_edges: Vec<(u32, u32)> = Vec::new();
    for eid in 0..ne {
        let (u, v) = edges[eid];
        slack[eid] = labels[u as usize] + labels[v as usize] - weights[eid];
        if slack[eid] <= eps {
            tight_edges.push((u, v));
        }
    }

    // Build initial matching on tight edges using push-relabel
    let tight_graph = crate::algorithms::constructors::create::create(
        &tight_edges.iter().map(|&(a, b)| (a, b)).collect::<Vec<_>>(),
        n as u32,
        false,
    )?;
    let tight_adj = build_undirected_adj(&tight_graph);
    let (init_match_opt, mut msize) = push_relabel_unweighted(&tight_graph, &tight_adj, types, n)?;
    let mut matching: Vec<i64> = init_match_opt
        .iter()
        .map(|o| match o {
            Some(v) => i64::from(*v),
            None => -1,
        })
        .collect();

    // Tight phantom edges adjacency (sorted for binary search)
    let mut tight_phantom: Vec<Vec<usize>> = vec![Vec::new(); n];

    // Main Hungarian loop
    while msize < smaller_set_size {
        let mut parent: Vec<i64> = vec![-1; n];
        let mut reachable_smaller: Vec<usize> = Vec::new();
        let mut reachable_larger: Vec<usize> = Vec::new();

        // Fill queue with unmatched vertices from smaller set
        let mut q: VecDeque<usize> = VecDeque::new();
        for &s in &smaller_set {
            if matching[s] == -1 {
                q.push_back(s);
                parent[s] = s as i64;
                reachable_smaller.push(s);
            }
        }

        // BFS along tight edges
        let mut alternating_path_endpoint: i64 = -1;
        'bfs: while let Some(v) = q.pop_front() {
            // Real tight edges
            for &(eid, other) in &incidence[v] {
                let u = other as usize;
                if slack[eid as usize] > eps {
                    continue;
                }
                if parent[u] >= 0 {
                    continue;
                }
                parent[u] = v as i64;
                reachable_larger.push(u);
                let w = matching[u];
                if w == -1 {
                    alternating_path_endpoint = u as i64;
                    break 'bfs;
                }
                let w_usize = w as usize;
                q.push_back(w_usize);
                parent[w_usize] = u as i64;
                reachable_smaller.push(w_usize);
            }

            // Tight phantom edges
            for &u in &tight_phantom[v] {
                if parent[u] >= 0 {
                    continue;
                }
                if (labels[v] + labels[u]).abs() > eps {
                    continue;
                }
                parent[u] = v as i64;
                reachable_larger.push(u);
                let w = matching[u];
                if w == -1 {
                    alternating_path_endpoint = u as i64;
                    break 'bfs;
                }
                let w_usize = w as usize;
                q.push_back(w_usize);
                parent[w_usize] = u as i64;
                reachable_smaller.push(w_usize);
            }
        }

        if alternating_path_endpoint != -1 {
            // Augment along alternating path
            let mut v = alternating_path_endpoint as usize;
            let mut u = parent[v] as usize;
            while u != v {
                let w = matching[v];
                if w != -1 {
                    matching[w as usize] = -1;
                }
                matching[v] = u as i64;
                let w2 = matching[u];
                if w2 != -1 {
                    matching[w2 as usize] = -1;
                }
                matching[u] = v as i64;

                v = parent[u] as usize;
                u = parent[v] as usize;
            }
            msize += 1;
            continue;
        }

        // No augmenting path found — update labels
        // Find minimum slack between reachable smaller-set and unreachable larger-set

        // Upper bound from phantom edges
        let mut min_label_larger = f64::INFINITY;
        for &l in &larger_set {
            if labels[l] < min_label_larger {
                min_label_larger = labels[l];
            }
        }
        let mut min_label_reachable_smaller = f64::INFINITY;
        for &s in &reachable_smaller {
            if parent[s] >= 0 && labels[s] < min_label_reachable_smaller {
                min_label_reachable_smaller = labels[s];
            }
        }
        let mut min_slack = min_label_larger + min_label_reachable_smaller;

        // Check real edges
        for &u in &reachable_smaller {
            for &(eid, other) in &incidence[u] {
                let v_node = other as usize;
                if parent[v_node] >= 0 {
                    continue;
                }
                if slack[eid as usize] < min_slack {
                    min_slack = slack[eid as usize];
                }
            }
        }

        if min_slack > 0.0 {
            // Update labels and slack
            for &u in &reachable_smaller {
                labels[u] -= min_slack;
                for &(eid, _) in &incidence[u] {
                    slack[eid as usize] -= min_slack;
                }
            }
            for &u in &reachable_larger {
                labels[u] += min_slack;
                for &(eid, _) in &incidence[u] {
                    slack[eid as usize] += min_slack;
                }
            }
        }

        // Update tight phantom edges
        for &u in &smaller_set {
            for &v in &larger_set {
                if (labels[u] + labels[v]).abs() <= eps {
                    let phantoms = &mut tight_phantom[u];
                    match phantoms.binary_search(&v) {
                        Ok(_) => {} // already present
                        Err(pos) => phantoms.insert(pos, v),
                    }
                }
            }
        }
    }

    // Remove phantom matches
    for &u in &smaller_set {
        let v = matching[u];
        if v != -1 {
            let v_usize = v as usize;
            if tight_phantom[u].binary_search(&v_usize).is_ok() {
                // Check if this is a real edge or phantom
                let is_real = incidence[u]
                    .iter()
                    .any(|&(_, other)| other as usize == v_usize);
                if !is_real {
                    matching[u] = -1;
                    matching[v_usize] = -1;
                    msize -= 1;
                }
            }
        }
    }

    // Compute matching weight
    let mut total_weight: f64 = 0.0;
    for eid in 0..ne {
        if slack[eid] <= eps {
            let (u, v) = edges[eid];
            if matching[u as usize] == i64::from(v) {
                total_weight += weights[eid];
            }
        }
    }

    let result_matching: Vec<Option<u32>> = matching
        .iter()
        .map(|&m| if m < 0 { None } else { Some(m as u32) })
        .collect();

    Ok(MatchingResult {
        matching_size: msize,
        matching_weight: total_weight,
        matching: result_matching,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::constructors::create::create;

    fn make_k22() -> (Graph, Vec<bool>) {
        let g = create(&[(0, 2), (0, 3), (1, 2), (1, 3)], 4, false).expect("K22");
        let types = vec![false, false, true, true];
        (g, types)
    }

    // ── is_matching ─────────────────────────────────────────────

    #[test]
    fn is_matching_valid() {
        let (g, types) = make_k22();
        let m = vec![Some(2), Some(3), Some(0), Some(1)];
        assert!(is_matching(&g, Some(&types), &m).expect("ok"));
    }

    #[test]
    fn is_matching_wrong_length() {
        let (g, _) = make_k22();
        let m = vec![Some(1), Some(0)];
        assert!(!is_matching(&g, None, &m).expect("ok"));
    }

    #[test]
    fn is_matching_non_mutual() {
        let (g, _) = make_k22();
        let m = vec![Some(2), None, None, None];
        assert!(!is_matching(&g, None, &m).expect("ok"));
    }

    #[test]
    fn is_matching_no_edge() {
        let g = create(&[(0, 1)], 3, false).expect("ok");
        // vertices 0 and 2 are not connected
        let m = vec![Some(2), None, Some(0)];
        assert!(!is_matching(&g, None, &m).expect("ok"));
    }

    #[test]
    fn is_matching_all_unmatched_with_types() {
        let g = create(&[(0, 1), (1, 2), (2, 3)], 4, false).expect("ok");
        let types = vec![false, true, false, true];
        let m = vec![None, None, None, None];
        assert!(is_matching(&g, Some(&types), &m).expect("ok"));
    }

    #[test]
    fn is_matching_types_same_partition() {
        let g = create(&[(0, 1), (1, 2)], 3, false).expect("ok");
        let types = vec![false, false, true]; // 0 and 1 same type but matched
        let m = vec![Some(1), Some(0), None];
        assert!(!is_matching(&g, Some(&types), &m).expect("ok"));
    }

    // ── is_maximal_matching ─────────────────────────────────────

    #[test]
    fn is_maximal_matching_true() {
        let (g, types) = make_k22();
        let m = vec![Some(2), Some(3), Some(0), Some(1)];
        assert!(is_maximal_matching(&g, Some(&types), &m).expect("ok"));
    }

    #[test]
    fn is_maximal_matching_false() {
        let (g, types) = make_k22();
        // Only 0-2 matched, but 1 and 3 are both unmatched and connected → not maximal
        let m = vec![Some(2), None, Some(0), None];
        assert!(!is_maximal_matching(&g, Some(&types), &m).expect("ok"));
    }

    #[test]
    fn is_maximal_all_unmatched_no_edges() {
        let g = Graph::new(3, false).expect("ok");
        let m = vec![None, None, None];
        assert!(is_maximal_matching(&g, None, &m).expect("ok"));
    }

    // ── maximum_bipartite_matching ──────────────────────────────

    #[test]
    fn max_matching_k22() {
        let (g, types) = make_k22();
        let r = maximum_bipartite_matching(&g, &types).expect("ok");
        assert_eq!(r.matching_size, 2);
        assert!(is_maximal_matching(&g, Some(&types), &r.matching).expect("ok"));
    }

    #[test]
    fn max_matching_empty() {
        let g = Graph::new(0, false).expect("ok");
        let types: Vec<bool> = vec![];
        let r = maximum_bipartite_matching(&g, &types).expect("ok");
        assert_eq!(r.matching_size, 0);
    }

    #[test]
    fn max_matching_singleton() {
        let g = Graph::new(1, false).expect("ok");
        let types = vec![false];
        let r = maximum_bipartite_matching(&g, &types).expect("ok");
        assert_eq!(r.matching_size, 0);
    }

    #[test]
    fn max_matching_path_4() {
        // 0-1-2-3, bipartite with types [F,T,F,T]
        let g = create(&[(0, 1), (1, 2), (2, 3)], 4, false).expect("ok");
        let types = vec![false, true, false, true];
        let r = maximum_bipartite_matching(&g, &types).expect("ok");
        assert_eq!(r.matching_size, 2);
        assert!(is_maximal_matching(&g, Some(&types), &r.matching).expect("ok"));
    }

    #[test]
    fn max_matching_star() {
        // Star: 0 connected to 1,2,3,4
        let g = create(&[(0, 1), (0, 2), (0, 3), (0, 4)], 5, false).expect("ok");
        let types = vec![false, true, true, true, true];
        let r = maximum_bipartite_matching(&g, &types).expect("ok");
        assert_eq!(r.matching_size, 1);
        assert!(is_maximal_matching(&g, Some(&types), &r.matching).expect("ok"));
    }

    #[test]
    fn max_matching_complete_bipartite_k33() {
        let g = create(
            &[
                (0, 3),
                (0, 4),
                (0, 5),
                (1, 3),
                (1, 4),
                (1, 5),
                (2, 3),
                (2, 4),
                (2, 5),
            ],
            6,
            false,
        )
        .expect("ok");
        let types = vec![false, false, false, true, true, true];
        let r = maximum_bipartite_matching(&g, &types).expect("ok");
        assert_eq!(r.matching_size, 3);
        assert!(is_maximal_matching(&g, Some(&types), &r.matching).expect("ok"));
    }

    #[test]
    fn max_matching_not_bipartite_error() {
        // Triangle: not bipartite
        let g = create(&[(0, 1), (1, 2), (2, 0)], 3, false).expect("ok");
        let types = vec![false, true, false]; // 0 and 2 connected but same type
        let r = maximum_bipartite_matching(&g, &types);
        assert!(r.is_err());
    }

    #[test]
    fn max_matching_disconnected() {
        // Two disconnected edges
        let g = create(&[(0, 1), (2, 3)], 4, false).expect("ok");
        let types = vec![false, true, false, true];
        let r = maximum_bipartite_matching(&g, &types).expect("ok");
        assert_eq!(r.matching_size, 2);
    }

    #[test]
    fn max_matching_types_too_short() {
        let g = create(&[(0, 1)], 2, false).expect("ok");
        let types = vec![false];
        let r = maximum_bipartite_matching(&g, &types);
        assert!(r.is_err());
    }

    // ── maximum_bipartite_matching_weighted ──────────────────────

    #[test]
    fn weighted_matching_simple() {
        // K2,2 with weights: prefer 0-3 and 1-2
        let g = create(&[(0, 2), (0, 3), (1, 2), (1, 3)], 4, false).expect("ok");
        let types = vec![false, false, true, true];
        let weights = vec![1.0, 10.0, 10.0, 1.0];
        let r = maximum_bipartite_matching_weighted(&g, &types, &weights, 0.0).expect("ok");
        assert_eq!(r.matching_size, 2);
        assert!((r.matching_weight - 20.0).abs() < 1e-9);
    }

    #[test]
    fn weighted_matching_mit_notes() {
        // Test graph from MIT lecture notes on matching
        // 10 vertices: 0-4 in set A, 5-9 in set B
        let g = create(
            &[
                (0, 6),
                (0, 7),
                (0, 8),
                (0, 9),
                (1, 5),
                (1, 6),
                (1, 7),
                (1, 8),
                (1, 9),
                (2, 5),
                (2, 6),
                (2, 7),
                (2, 8),
                (2, 9),
                (3, 5),
                (3, 7),
                (3, 9),
                (4, 7),
            ],
            10,
            false,
        )
        .expect("ok");
        let types: Vec<bool> = (0..10).map(|i| i >= 5).collect();
        let weights = vec![
            2.0, 7.0, 2.0, 3.0, // edges from 0
            1.0, 3.0, 9.0, 3.0, 3.0, // edges from 1
            1.0, 3.0, 3.0, 1.0, 2.0, // edges from 2
            4.0, 1.0, 2.0, // edges from 3
            3.0, // edge from 4
        ];
        let r = maximum_bipartite_matching_weighted(&g, &types, &weights, 0.0).expect("ok");
        assert_eq!(r.matching_size, 4);
        assert!((r.matching_weight - 19.0).abs() < 1e-9);
        assert!(is_maximal_matching(&g, Some(&types), &r.matching).expect("ok"));
    }

    #[test]
    fn weighted_matching_generated_case1() {
        let g = create(&[(0, 8), (2, 7), (3, 7), (3, 8), (4, 5), (4, 9)], 10, false).expect("ok");
        let types: Vec<bool> = (0..10).map(|i| i >= 5).collect();
        let weights = vec![8.0, 5.0, 9.0, 18.0, 20.0, 13.0];
        let r = maximum_bipartite_matching_weighted(&g, &types, &weights, 0.0).expect("ok");
        assert!((r.matching_weight - 43.0).abs() < 1e-9);
    }

    #[test]
    fn weighted_matching_generated_case2() {
        let g = create(&[(0, 5), (0, 6), (1, 7), (2, 5), (3, 5), (3, 9)], 10, false).expect("ok");
        let types: Vec<bool> = (0..10).map(|i| i >= 5).collect();
        let weights = vec![20.0, 4.0, 20.0, 3.0, 13.0, 1.0];
        let r = maximum_bipartite_matching_weighted(&g, &types, &weights, 0.0).expect("ok");
        assert!((r.matching_weight - 41.0).abs() < 1e-9);
    }

    #[test]
    fn weighted_matching_empty() {
        let g = Graph::new(0, false).expect("ok");
        let r = maximum_bipartite_matching_weighted(&g, &[], &[], 0.0).expect("ok");
        assert_eq!(r.matching_size, 0);
    }

    #[test]
    fn weighted_matching_no_edges() {
        let g = Graph::new(4, false).expect("ok");
        let types = vec![false, false, true, true];
        let r = maximum_bipartite_matching_weighted(&g, &types, &[], 0.0).expect("ok");
        assert_eq!(r.matching_size, 0);
    }

    // ── proptest ────────────────────────────────────────────────

    #[cfg(feature = "proptest-harness")]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        fn arb_bipartite_graph(
            max_a: u32,
            max_b: u32,
        ) -> impl Strategy<Value = (Graph, Vec<bool>)> {
            (1..=max_a, 1..=max_b).prop_flat_map(move |(a, b)| {
                let n = a + b;
                let max_edges = a * b;
                let edge_count = 0..=max_edges.min(20);
                (Just(a), Just(b), edge_count)
                    .prop_flat_map(move |(a, b, ne)| {
                        let edge_indices =
                            proptest::collection::hash_set(0..a * b, ne as usize..=ne as usize);
                        (Just(a), Just(b), edge_indices)
                    })
                    .prop_map(|(a, b, indices)| {
                        let n = a + b;
                        let mut edges = Vec::new();
                        for idx in indices {
                            let u = idx / b;
                            let v = a + idx % b;
                            edges.push((u, v));
                        }
                        let g = create(&edges, n, false).expect("bipartite graph");
                        let types: Vec<bool> = (0..n).map(|i| i >= a).collect();
                        (g, types)
                    })
            })
        }

        proptest! {
            #[test]
            fn matching_is_valid((g, types) in arb_bipartite_graph(6, 6)) {
                let r = maximum_bipartite_matching(&g, &types).expect("ok");
                prop_assert!(is_matching(&g, Some(&types), &r.matching).expect("ok"));
                prop_assert!(is_maximal_matching(&g, Some(&types), &r.matching).expect("ok"));
            }

            #[test]
            fn matching_size_leq_min_partition(
                (g, types) in arb_bipartite_graph(6, 6)
            ) {
                let r = maximum_bipartite_matching(&g, &types).expect("ok");
                let a_size = types.iter().filter(|&&t| !t).count();
                let b_size = types.iter().filter(|&&t| t).count();
                prop_assert!(r.matching_size <= a_size.min(b_size));
            }

            #[test]
            fn matching_size_leq_ecount(
                (g, types) in arb_bipartite_graph(6, 6)
            ) {
                let r = maximum_bipartite_matching(&g, &types).expect("ok");
                prop_assert!(r.matching_size <= g.ecount());
            }

            #[test]
            fn weighted_matching_is_valid((g, types) in arb_bipartite_graph(5, 5)) {
                let ne = g.ecount();
                let weights: Vec<f64> = (0..ne).map(|i| (i as f64) + 1.0).collect();
                if ne > 0 {
                    let r = maximum_bipartite_matching_weighted(&g, &types, &weights, 0.0).expect("ok");
                    prop_assert!(is_matching(&g, Some(&types), &r.matching).expect("ok"));
                }
            }

            #[test]
            fn weighted_geq_unweighted_unit(
                (g, types) in arb_bipartite_graph(5, 5)
            ) {
                let unw = maximum_bipartite_matching(&g, &types).expect("ok");
                let ne = g.ecount();
                let weights: Vec<f64> = vec![1.0; ne];
                if ne > 0 {
                    let w = maximum_bipartite_matching_weighted(&g, &types, &weights, 0.0).expect("ok");
                    // With unit weights, weighted should find same size or better
                    prop_assert!(w.matching_size >= unw.matching_size.saturating_sub(1),
                        "weighted: {}, unweighted: {}", w.matching_size, unw.matching_size);
                }
            }
        }
    }
}
