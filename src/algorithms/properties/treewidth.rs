//! Graph treewidth (ALGO-TR-034).
//!
//! The **treewidth** of a graph measures how tree-like it is.
//! Computing exact treewidth is NP-hard; we provide:
//!
//! - `treewidth_upper_bound`: greedy min-degree elimination ordering.
//! - `treewidth_min_fill`: greedy min-fill elimination ordering.
//! - `elimination_ordering`: the vertex ordering from min-degree heuristic.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute an upper bound on treewidth via min-degree elimination.
///
/// Repeatedly eliminates the vertex with minimum degree, connecting
/// all its neighbours (creating a clique), and tracks the maximum
/// degree at elimination time.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, treewidth_upper_bound};
///
/// // Path graph: treewidth = 1
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// assert_eq!(treewidth_upper_bound(&g).unwrap(), 1);
/// ```
pub fn treewidth_upper_bound(graph: &Graph) -> IgraphResult<u32> {
    let (tw, _) = elimination_order_internal(graph, EliminationHeuristic::MinDegree);
    Ok(tw)
}

/// Compute an upper bound on treewidth via min-fill elimination.
///
/// Repeatedly eliminates the vertex that would introduce the fewest
/// new edges (fill edges) when eliminated.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, treewidth_min_fill};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// assert_eq!(treewidth_min_fill(&g).unwrap(), 1);
/// ```
pub fn treewidth_min_fill(graph: &Graph) -> IgraphResult<u32> {
    let (tw, _) = elimination_order_internal(graph, EliminationHeuristic::MinFill);
    Ok(tw)
}

/// Compute an elimination ordering using the min-degree heuristic.
///
/// Returns the ordering as a sequence of vertex ids.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, elimination_ordering};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// let order = elimination_ordering(&g).unwrap();
/// assert_eq!(order.len(), 4);
/// ```
pub fn elimination_ordering(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let (_, order) = elimination_order_internal(graph, EliminationHeuristic::MinDegree);
    Ok(order)
}

#[derive(Clone, Copy)]
enum EliminationHeuristic {
    MinDegree,
    MinFill,
}

fn elimination_order_internal(graph: &Graph, heuristic: EliminationHeuristic) -> (u32, Vec<u32>) {
    let n = graph.vcount() as usize;
    if n == 0 {
        return (0, Vec::new());
    }

    let mut adj: Vec<Vec<bool>> = vec![vec![false; n]; n];
    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        if ui != vi {
            adj[ui][vi] = true;
            if !graph.is_directed() {
                adj[vi][ui] = true;
            }
        }
    }

    let mut eliminated = vec![false; n];
    let mut order = Vec::with_capacity(n);
    let mut tw: u32 = 0;

    for _ in 0..n {
        let v = match heuristic {
            EliminationHeuristic::MinDegree => pick_min_degree(&adj, &eliminated, n),
            EliminationHeuristic::MinFill => pick_min_fill(&adj, &eliminated, n),
        };

        let nbrs: Vec<usize> = (0..n)
            .filter(|&u| !eliminated[u] && u != v && adj[v][u])
            .collect();

        let deg = nbrs.len() as u32;
        if deg > tw {
            tw = deg;
        }

        for i in 0..nbrs.len() {
            for j in (i + 1)..nbrs.len() {
                let a = nbrs[i];
                let b = nbrs[j];
                adj[a][b] = true;
                adj[b][a] = true;
            }
        }

        eliminated[v] = true;
        order.push(v as u32);
    }

    (tw, order)
}

fn pick_min_degree(adj: &[Vec<bool>], eliminated: &[bool], n: usize) -> usize {
    let mut best = usize::MAX;
    let mut best_v = 0;

    for v in 0..n {
        if eliminated[v] {
            continue;
        }
        let deg = (0..n)
            .filter(|&u| !eliminated[u] && u != v && adj[v][u])
            .count();
        if deg < best {
            best = deg;
            best_v = v;
        }
    }

    best_v
}

fn pick_min_fill(adj: &[Vec<bool>], eliminated: &[bool], n: usize) -> usize {
    let mut best_fill = usize::MAX;
    let mut best_deg = usize::MAX;
    let mut best_v = 0;

    for v in 0..n {
        if eliminated[v] {
            continue;
        }
        let nbrs: Vec<usize> = (0..n)
            .filter(|&u| !eliminated[u] && u != v && adj[v][u])
            .collect();

        let mut fill: usize = 0;
        for i in 0..nbrs.len() {
            for j in (i + 1)..nbrs.len() {
                if !adj[nbrs[i]][nbrs[j]] {
                    fill = fill.saturating_add(1);
                }
            }
        }

        if fill < best_fill || (fill == best_fill && nbrs.len() < best_deg) {
            best_fill = fill;
            best_deg = nbrs.len();
            best_v = v;
        }
    }

    best_v
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

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn grid2x3() -> Graph {
        // 0-1-2
        // |   |   |
        // 3-4-5
        Graph::from_edges(
            &[(0, 1), (1, 2), (3, 4), (4, 5), (0, 3), (1, 4), (2, 5)],
            false,
            Some(6),
        )
        .unwrap()
    }

    fn petersen() -> Graph {
        Graph::from_edges(
            &[
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 4),
                (4, 0),
                (0, 5),
                (1, 6),
                (2, 7),
                (3, 8),
                (4, 9),
                (5, 7),
                (7, 9),
                (9, 6),
                (6, 8),
                (8, 5),
            ],
            false,
            Some(10),
        )
        .unwrap()
    }

    // --- treewidth_upper_bound ---

    #[test]
    fn twub_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(treewidth_upper_bound(&g).unwrap(), 0);
    }

    #[test]
    fn twub_single() {
        let g = Graph::with_vertices(1);
        assert_eq!(treewidth_upper_bound(&g).unwrap(), 0);
    }

    #[test]
    fn twub_no_edges() {
        let g = Graph::with_vertices(5);
        assert_eq!(treewidth_upper_bound(&g).unwrap(), 0);
    }

    #[test]
    fn twub_path4() {
        // Trees have treewidth 1
        assert_eq!(treewidth_upper_bound(&path4()).unwrap(), 1);
    }

    #[test]
    fn twub_star5() {
        // Stars are trees — treewidth 1
        assert_eq!(treewidth_upper_bound(&star5()).unwrap(), 1);
    }

    #[test]
    fn twub_k3() {
        // K_3: treewidth = 2
        assert_eq!(treewidth_upper_bound(&k3()).unwrap(), 2);
    }

    #[test]
    fn twub_k4() {
        // K_4: treewidth = 3
        assert_eq!(treewidth_upper_bound(&k4()).unwrap(), 3);
    }

    #[test]
    fn twub_cycle4() {
        // Cycles: treewidth = 2
        assert_eq!(treewidth_upper_bound(&cycle4()).unwrap(), 2);
    }

    #[test]
    fn twub_cycle5() {
        assert_eq!(treewidth_upper_bound(&cycle5()).unwrap(), 2);
    }

    #[test]
    fn twub_grid2x3() {
        // 2x3 grid: treewidth = 2
        let tw = treewidth_upper_bound(&grid2x3()).unwrap();
        assert!(tw >= 2);
        assert!(tw <= 3);
    }

    #[test]
    fn twub_petersen() {
        // Petersen: treewidth = 4
        let tw = treewidth_upper_bound(&petersen()).unwrap();
        assert!(tw >= 4);
        assert!(tw <= 6);
    }

    // --- treewidth_min_fill ---

    #[test]
    fn twmf_path4() {
        assert_eq!(treewidth_min_fill(&path4()).unwrap(), 1);
    }

    #[test]
    fn twmf_k3() {
        assert_eq!(treewidth_min_fill(&k3()).unwrap(), 2);
    }

    #[test]
    fn twmf_k4() {
        assert_eq!(treewidth_min_fill(&k4()).unwrap(), 3);
    }

    #[test]
    fn twmf_cycle5() {
        assert_eq!(treewidth_min_fill(&cycle5()).unwrap(), 2);
    }

    #[test]
    fn twmf_star5() {
        assert_eq!(treewidth_min_fill(&star5()).unwrap(), 1);
    }

    // --- elimination_ordering ---

    #[test]
    fn eo_length() {
        let g = path4();
        let order = elimination_ordering(&g).unwrap();
        assert_eq!(order.len(), 4);
    }

    #[test]
    fn eo_permutation() {
        let g = k4();
        let order = elimination_ordering(&g).unwrap();
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![0, 1, 2, 3]);
    }

    #[test]
    fn eo_empty() {
        let g = Graph::with_vertices(0);
        assert!(elimination_ordering(&g).unwrap().is_empty());
    }

    // --- cross-consistency ---

    #[test]
    fn min_fill_leq_min_degree() {
        // min-fill is often tighter than min-degree
        for g in &[path4(), k3(), cycle4(), cycle5(), star5()] {
            let bound_degree = treewidth_upper_bound(g).unwrap();
            let bound_fill = treewidth_min_fill(g).unwrap();
            // Both are upper bounds; min-fill often matches or beats min-degree
            assert!(bound_fill <= bound_degree.saturating_add(1));
        }
    }

    #[test]
    fn tw_at_least_one_for_edges() {
        for g in &[path4(), k3(), k4(), cycle4(), cycle5()] {
            assert!(treewidth_upper_bound(g).unwrap() >= 1);
        }
    }

    #[test]
    fn tw_leq_n_minus_1() {
        for g in &[path4(), k3(), k4(), cycle5(), star5()] {
            let n = g.vcount();
            let tw = treewidth_upper_bound(g).unwrap();
            assert!(tw < n);
        }
    }

    #[test]
    fn complete_graph_tw_is_n_minus_1() {
        // treewidth(K_n) = n-1
        for n in 2_u32..=5 {
            let edges: Vec<(u32, u32)> = (0..n)
                .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
                .collect();
            let g = Graph::from_edges(&edges, false, Some(n)).unwrap();
            assert_eq!(
                treewidth_upper_bound(&g).unwrap(),
                n - 1,
                "tw(K_{n}) should be {}",
                n - 1
            );
        }
    }

    #[test]
    fn tree_tw_is_one() {
        // Any tree has treewidth 1
        let trees = vec![
            path4(),
            star5(),
            Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (2, 4)], false, Some(5)).unwrap(),
        ];
        for g in &trees {
            assert_eq!(treewidth_upper_bound(g).unwrap(), 1);
        }
    }
}
