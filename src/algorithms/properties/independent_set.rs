//! Independent set heuristics and ratios (ALGO-TR-029).
//!
//! Complements the exact `maximum_independent_set` and
//! `independence_number` in `crate::algorithms::independent_set` /
//! `crate::algorithms::cliques` with:
//!
//! - **Independence ratio**: `α(G) / n`.
//! - **Greedy independent set**: a maximal (not necessarily maximum)
//!   independent set found by greedily picking smallest-degree vertices.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the independence ratio `α(G) / n`.
///
/// Uses the exact `independence_number` from the cliques module.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, independence_ratio};
///
/// // K_3: α = 1, ratio = 1/3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let r = independence_ratio(&g).unwrap();
/// assert!((r - 1.0/3.0).abs() < 0.01);
/// ```
pub fn independence_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }
    let alpha = crate::algorithms::cliques::independence_number(graph)?;
    Ok(f64::from(alpha) / n as f64)
}

/// Find a maximal independent set using a greedy heuristic.
///
/// Iteratively picks the vertex with the smallest degree among
/// remaining candidates, adds it to the independent set, and removes
/// it and all its neighbours from the candidate pool.
///
/// The result is maximal (no vertex can be added without breaking
/// independence) but not necessarily maximum.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, greedy_independent_set, is_independent_vertex_set};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,4),(4,0)], false, Some(5)
/// ).unwrap();
/// let ind = greedy_independent_set(&g).unwrap();
/// assert!(is_independent_vertex_set(&g, &ind).unwrap());
/// assert!(ind.len() >= 2);
/// ```
pub fn greedy_independent_set(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }

    let mut available = vec![true; n];
    let mut result = Vec::new();

    loop {
        let mut best_v = None;
        let mut best_deg = usize::MAX;

        for v in 0..n {
            if !available[v] {
                continue;
            }
            let deg = graph
                .neighbors(v as u32)?
                .iter()
                .filter(|&&u| available[u as usize])
                .count();
            if deg < best_deg {
                best_deg = deg;
                best_v = Some(v as u32);
            }
        }

        let Some(v) = best_v else { break };

        result.push(v);
        available[v as usize] = false;

        let nbrs = graph.neighbors(v)?;
        for &u in &nbrs {
            available[u as usize] = false;
        }
    }

    result.sort_unstable();
    Ok(result)
}

#[cfg(test)]
mod tests {
    fn is_ind(graph: &Graph, vertices: &[u32]) -> bool {
        crate::algorithms::properties::is_clique::is_independent_vertex_set(graph, vertices)
            .unwrap_or(false)
    }

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

    fn cycle5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)], false, Some(5)).unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
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

    // --- independence_ratio ---

    #[test]
    fn ir_k3() {
        let r = independence_ratio(&k3()).unwrap();
        assert!((r - 1.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn ir_k4() {
        let r = independence_ratio(&k4()).unwrap();
        assert!((r - 0.25).abs() < 0.01);
    }

    #[test]
    fn ir_path4() {
        let r = independence_ratio(&path4()).unwrap();
        assert!((r - 0.5).abs() < 0.01);
    }

    #[test]
    fn ir_cycle5() {
        let r = independence_ratio(&cycle5()).unwrap();
        assert!((r - 0.4).abs() < 0.01);
    }

    #[test]
    fn ir_isolated() {
        let g = Graph::with_vertices(5);
        let r = independence_ratio(&g).unwrap();
        assert!((r - 1.0).abs() < 0.01);
    }

    #[test]
    fn ir_empty() {
        let g = Graph::with_vertices(0);
        let r = independence_ratio(&g).unwrap();
        assert!(r.abs() < 1e-10);
    }

    #[test]
    fn ir_star5() {
        let r = independence_ratio(&star5()).unwrap();
        assert!((r - 0.8).abs() < 0.01);
    }

    // --- greedy_independent_set ---

    #[test]
    fn gis_empty() {
        let g = Graph::with_vertices(0);
        assert!(greedy_independent_set(&g).unwrap().is_empty());
    }

    #[test]
    fn gis_isolated() {
        let g = Graph::with_vertices(4);
        let ind = greedy_independent_set(&g).unwrap();
        assert_eq!(ind.len(), 4);
    }

    #[test]
    fn gis_path4() {
        let g = path4();
        let ind = greedy_independent_set(&g).unwrap();
        assert!(is_ind(&g, &ind));
        assert!(ind.len() >= 2);
    }

    #[test]
    fn gis_k4() {
        let g = k4();
        let ind = greedy_independent_set(&g).unwrap();
        assert_eq!(ind.len(), 1);
        assert!(is_ind(&g, &ind));
    }

    #[test]
    fn gis_cycle5() {
        let g = cycle5();
        let ind = greedy_independent_set(&g).unwrap();
        assert!(is_ind(&g, &ind));
        assert!(ind.len() >= 2);
    }

    #[test]
    fn gis_star5() {
        let g = star5();
        let ind = greedy_independent_set(&g).unwrap();
        assert!(is_ind(&g, &ind));
        assert!(ind.len() >= 2);
    }

    #[test]
    fn gis_petersen() {
        let g = petersen();
        let ind = greedy_independent_set(&g).unwrap();
        assert!(is_ind(&g, &ind));
        assert!(ind.len() >= 2);
    }

    #[test]
    fn gis_is_maximal() {
        let g = petersen();
        let ind = greedy_independent_set(&g).unwrap();
        assert!(is_ind(&g, &ind));
        let n = g.vcount() as usize;
        let mut in_set = vec![false; n];
        for &v in &ind {
            in_set[v as usize] = true;
        }
        for v in 0..g.vcount() {
            if in_set[v as usize] {
                continue;
            }
            let nbrs = g.neighbors(v).unwrap();
            let has_neighbor_in_set = nbrs.iter().any(|&u| in_set[u as usize]);
            assert!(
                has_neighbor_in_set,
                "vertex {v} could be added — not maximal"
            );
        }
    }

    // --- cross-consistency ---

    #[test]
    fn greedy_at_least_one_for_nonempty() {
        for n in 1_u32..=5 {
            let g = Graph::with_vertices(n);
            let ind = greedy_independent_set(&g).unwrap();
            assert!(!ind.is_empty());
        }
    }

    #[test]
    fn ratio_between_zero_and_one() {
        for g in &[path4(), k3(), k4(), cycle5(), star5()] {
            let r = independence_ratio(g).unwrap();
            assert!((0.0..=1.0).contains(&r));
        }
    }

    #[test]
    fn greedy_at_most_alpha() {
        for g in &[path4(), k3(), cycle5()] {
            let alpha = crate::algorithms::cliques::independence_number(g).unwrap();
            let ind = greedy_independent_set(g).unwrap();
            assert!(ind.len() as u32 <= alpha);
        }
    }
}
