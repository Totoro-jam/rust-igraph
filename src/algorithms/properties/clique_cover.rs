//! Clique cover (ALGO-TR-032).
//!
//! A **clique cover** (or clique partition) partitions the vertices of
//! a graph into cliques. The **clique cover number** `θ(G)` is the
//! minimum number of cliques needed.
//!
//! - `is_clique_cover`: validate a partition.
//! - `greedy_clique_cover`: heuristic greedy cover.
//! - `clique_cover_number`: brute-force minimum (small graphs only).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Check whether a partition of vertices forms a valid clique cover.
///
/// Each part must be a clique in the graph, and every vertex must
/// appear exactly once.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_clique_cover};
///
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(1,2),(3,4)], false, Some(5)
/// ).unwrap();
/// assert!(is_clique_cover(&g, &[vec![0,1,2], vec![3,4]]).unwrap());
/// assert!(!is_clique_cover(&g, &[vec![0,1,3], vec![2,4]]).unwrap());
/// ```
pub fn is_clique_cover(graph: &Graph, parts: &[Vec<u32>]) -> IgraphResult<bool> {
    let n = graph.vcount() as usize;
    let mut seen = vec![false; n];

    let adj = build_adj(graph, n);

    for part in parts {
        for &v in part {
            let vi = v as usize;
            if vi >= n || seen[vi] {
                return Ok(false);
            }
            seen[vi] = true;
        }

        for i in 0..part.len() {
            for j in (i + 1)..part.len() {
                let ui = part[i] as usize;
                let vi = part[j] as usize;
                if !adj[ui * n + vi] {
                    return Ok(false);
                }
            }
        }
    }

    if seen.iter().any(|&s| !s) {
        return Ok(false);
    }

    Ok(true)
}

/// Find a greedy clique cover.
///
/// Iterates through vertices and greedily adds each to the first
/// existing clique it can extend, or starts a new clique.
///
/// Returns a partition of vertices into cliques.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, greedy_clique_cover, is_clique_cover};
///
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(1,2),(3,4)], false, Some(5)
/// ).unwrap();
/// let cover = greedy_clique_cover(&g).unwrap();
/// assert!(is_clique_cover(&g, &cover).unwrap());
/// ```
pub fn greedy_clique_cover(graph: &Graph) -> IgraphResult<Vec<Vec<u32>>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }

    let adj = build_adj(graph, n);

    let mut degrees: Vec<(usize, u32)> = (0..n as u32)
        .map(|v| {
            let d = graph.degree(v).unwrap_or(0);
            (d, v)
        })
        .collect();
    degrees.sort_by_key(|b| std::cmp::Reverse(b.0));

    let mut cliques: Vec<Vec<u32>> = Vec::new();

    for &(_, v) in &degrees {
        let vi = v as usize;
        let mut placed = false;

        for clique in &mut cliques {
            let fits = clique.iter().all(|&u| {
                let ui = u as usize;
                adj[vi * n + ui]
            });
            if fits {
                clique.push(v);
                placed = true;
                break;
            }
        }

        if !placed {
            cliques.push(vec![v]);
        }
    }

    for clique in &mut cliques {
        clique.sort_unstable();
    }

    Ok(cliques)
}

/// Compute the clique cover number `θ(G)`.
///
/// The minimum number of cliques needed to cover all vertices.
/// Uses brute-force search — only feasible for small graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, clique_cover_number};
///
/// // K_3 + K_2: need exactly 2 cliques
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(1,2),(3,4)], false, Some(5)
/// ).unwrap();
/// assert_eq!(clique_cover_number(&g).unwrap(), 2);
/// ```
pub fn clique_cover_number(graph: &Graph) -> IgraphResult<u32> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let adj = build_adj(graph, n);

    let maximal_cliques = find_all_maximal_cliques(&adj, n);

    let mut best = n as u32;
    cover_search(&maximal_cliques, &mut vec![false; n], 0, 0, &mut best);

    Ok(best)
}

fn build_adj(graph: &Graph, n: usize) -> Vec<bool> {
    let mut adj = vec![false; n * n];
    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        adj[ui * n + vi] = true;
        if !graph.is_directed() {
            adj[vi * n + ui] = true;
        }
    }
    adj
}

fn find_all_maximal_cliques(adj: &[bool], n: usize) -> Vec<Vec<usize>> {
    let mut cliques = Vec::new();
    let mut current = Vec::new();
    let candidates: Vec<usize> = (0..n).collect();
    let excluded: Vec<usize> = Vec::new();
    bron_kerbosch(adj, n, &mut current, &candidates, &excluded, &mut cliques);
    cliques.push(Vec::new());
    cliques.retain(|c| !c.is_empty());

    for v in 0..n {
        let mut found = false;
        for c in &cliques {
            if c.contains(&v) {
                found = true;
                break;
            }
        }
        if !found {
            cliques.push(vec![v]);
        }
    }

    cliques
}

fn bron_kerbosch(
    adj: &[bool],
    n: usize,
    current: &mut Vec<usize>,
    candidates: &[usize],
    excluded: &[usize],
    results: &mut Vec<Vec<usize>>,
) {
    if candidates.is_empty() && excluded.is_empty() {
        if !current.is_empty() {
            results.push(current.clone());
        }
        return;
    }

    let mut cand = candidates.to_vec();
    let mut excl = excluded.to_vec();

    let pivot = cand
        .iter()
        .chain(excl.iter())
        .max_by_key(|&&v| cand.iter().filter(|&&u| adj[v * n + u]).count())
        .copied();

    let Some(pivot_v) = pivot else { return };

    let to_try: Vec<usize> = cand
        .iter()
        .filter(|&&v| !adj[pivot_v * n + v])
        .copied()
        .collect();

    for v in to_try {
        let new_cand: Vec<usize> = cand.iter().filter(|&&u| adj[v * n + u]).copied().collect();
        let new_excl: Vec<usize> = excl.iter().filter(|&&u| adj[v * n + u]).copied().collect();

        current.push(v);
        bron_kerbosch(adj, n, current, &new_cand, &new_excl, results);
        current.pop();

        cand.retain(|&u| u != v);
        excl.push(v);
    }
}

fn cover_search(
    cliques: &[Vec<usize>],
    covered: &mut Vec<bool>,
    depth: u32,
    start: usize,
    best: &mut u32,
) {
    if covered.iter().all(|&c| c) {
        if depth < *best {
            *best = depth;
        }
        return;
    }

    if depth.saturating_add(1) >= *best {
        return;
    }

    let uncovered = covered.iter().position(|&c| !c);
    let Some(target) = uncovered else { return };

    for i in start..cliques.len() {
        if !cliques[i].contains(&target) {
            continue;
        }

        let any_already_uncovered = cliques[i].iter().any(|&v| !covered[v]);
        if !any_already_uncovered {
            continue;
        }

        let mut restored = Vec::new();
        for &v in &cliques[i] {
            if !covered[v] {
                covered[v] = true;
                restored.push(v);
            }
        }

        cover_search(cliques, covered, depth.saturating_add(1), i + 1, best);

        for v in restored {
            covered[v] = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn k3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap()
    }

    fn k4() -> Graph {
        Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap()
    }

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    fn cycle5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)], false, Some(5)).unwrap()
    }

    fn k3_plus_k2() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (1, 2), (3, 4)], false, Some(5)).unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    // --- is_clique_cover ---

    #[test]
    fn icc_valid() {
        let g = k3_plus_k2();
        assert!(is_clique_cover(&g, &[vec![0, 1, 2], vec![3, 4]]).unwrap());
    }

    #[test]
    fn icc_invalid_not_clique() {
        let g = k3_plus_k2();
        assert!(!is_clique_cover(&g, &[vec![0, 1, 3], vec![2, 4]]).unwrap());
    }

    #[test]
    fn icc_missing_vertex() {
        let g = k3_plus_k2();
        assert!(!is_clique_cover(&g, &[vec![0, 1, 2]]).unwrap());
    }

    #[test]
    fn icc_duplicate_vertex() {
        let g = k3_plus_k2();
        assert!(!is_clique_cover(&g, &[vec![0, 1, 2], vec![2, 3, 4]]).unwrap());
    }

    #[test]
    fn icc_singletons() {
        let g = path4();
        assert!(is_clique_cover(&g, &[vec![0], vec![1], vec![2], vec![3]]).unwrap());
    }

    #[test]
    fn icc_empty() {
        let g = Graph::with_vertices(0);
        assert!(is_clique_cover(&g, &[]).unwrap());
    }

    #[test]
    fn icc_edges_as_cliques() {
        let g = path4();
        assert!(is_clique_cover(&g, &[vec![0, 1], vec![2, 3]]).unwrap());
    }

    // --- greedy_clique_cover ---

    #[test]
    fn gcc_empty() {
        let g = Graph::with_vertices(0);
        let cover = greedy_clique_cover(&g).unwrap();
        assert!(cover.is_empty());
    }

    #[test]
    fn gcc_isolated() {
        let g = Graph::with_vertices(3);
        let cover = greedy_clique_cover(&g).unwrap();
        assert!(is_clique_cover(&g, &cover).unwrap());
        assert_eq!(cover.len(), 3);
    }

    #[test]
    fn gcc_k3() {
        let g = k3();
        let cover = greedy_clique_cover(&g).unwrap();
        assert!(is_clique_cover(&g, &cover).unwrap());
        assert_eq!(cover.len(), 1);
    }

    #[test]
    fn gcc_k4() {
        let g = k4();
        let cover = greedy_clique_cover(&g).unwrap();
        assert!(is_clique_cover(&g, &cover).unwrap());
        assert_eq!(cover.len(), 1);
    }

    #[test]
    fn gcc_k3_plus_k2() {
        let g = k3_plus_k2();
        let cover = greedy_clique_cover(&g).unwrap();
        assert!(is_clique_cover(&g, &cover).unwrap());
        assert!(cover.len() <= 3);
    }

    #[test]
    fn gcc_path4() {
        let g = path4();
        let cover = greedy_clique_cover(&g).unwrap();
        assert!(is_clique_cover(&g, &cover).unwrap());
    }

    #[test]
    fn gcc_cycle5() {
        let g = cycle5();
        let cover = greedy_clique_cover(&g).unwrap();
        assert!(is_clique_cover(&g, &cover).unwrap());
    }

    #[test]
    fn gcc_star5() {
        let g = star5();
        let cover = greedy_clique_cover(&g).unwrap();
        assert!(is_clique_cover(&g, &cover).unwrap());
    }

    // --- clique_cover_number ---

    #[test]
    fn ccn_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(clique_cover_number(&g).unwrap(), 0);
    }

    #[test]
    fn ccn_isolated() {
        let g = Graph::with_vertices(3);
        assert_eq!(clique_cover_number(&g).unwrap(), 3);
    }

    #[test]
    fn ccn_k3() {
        assert_eq!(clique_cover_number(&k3()).unwrap(), 1);
    }

    #[test]
    fn ccn_k4() {
        assert_eq!(clique_cover_number(&k4()).unwrap(), 1);
    }

    #[test]
    fn ccn_k3_plus_k2() {
        assert_eq!(clique_cover_number(&k3_plus_k2()).unwrap(), 2);
    }

    #[test]
    fn ccn_path4() {
        assert_eq!(clique_cover_number(&path4()).unwrap(), 2);
    }

    #[test]
    fn ccn_cycle4() {
        assert_eq!(clique_cover_number(&cycle4()).unwrap(), 2);
    }

    #[test]
    fn ccn_cycle5() {
        assert_eq!(clique_cover_number(&cycle5()).unwrap(), 3);
    }

    #[test]
    fn ccn_single_vertex() {
        let g = Graph::with_vertices(1);
        assert_eq!(clique_cover_number(&g).unwrap(), 1);
    }

    #[test]
    fn ccn_single_edge() {
        let g = Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap();
        assert_eq!(clique_cover_number(&g).unwrap(), 1);
    }

    // --- cross-consistency ---

    #[test]
    fn greedy_at_least_optimal() {
        for g in &[k3(), k4(), k3_plus_k2(), path4(), cycle4()] {
            let greedy = greedy_clique_cover(g).unwrap();
            let opt = clique_cover_number(g).unwrap();
            assert!(greedy.len() as u32 >= opt);
        }
    }

    #[test]
    fn cover_number_at_most_n() {
        for g in &[k3(), k4(), path4(), cycle5(), star5()] {
            let theta = clique_cover_number(g).unwrap();
            assert!(theta <= g.vcount());
        }
    }

    #[test]
    fn complete_graph_cover_is_one() {
        for n in 1_u32..=5 {
            let edges: Vec<(u32, u32)> = (0..n)
                .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
                .collect();
            let g = if edges.is_empty() {
                Graph::with_vertices(n)
            } else {
                Graph::from_edges(&edges, false, Some(n)).unwrap()
            };
            assert_eq!(clique_cover_number(&g).unwrap(), 1);
        }
    }
}
