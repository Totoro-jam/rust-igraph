//! Greedy vertex coloring and chromatic bounds (ALGO-TR-028).
//!
//! - **Greedy coloring**: assign the smallest available colour to each
//!   vertex, processing vertices in the given order. The number of
//!   colours used depends on the vertex ordering.
//! - **Greedy coloring (largest-first)**: process vertices in
//!   descending degree order — a simple heuristic that often uses
//!   fewer colours.
//! - **Chromatic number upper bound**: the greedy largest-first
//!   colouring provides `χ(G) ≤ greedy_colors`.
//! - **Chromatic number lower bound**: `χ(G) ≥ max clique size`
//!   (clique number); approximated here via the largest clique found
//!   by a greedy search.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphError, IgraphResult};

/// Greedy vertex coloring in natural vertex order.
///
/// Assigns each vertex the smallest colour (0-based) not used by any
/// of its already-coloured neighbours. Returns the colour assignment
/// as a `Vec<u32>` indexed by vertex id.
///
/// Works for both directed and undirected graphs (for directed graphs,
/// both in- and out-neighbours are considered as "adjacent").
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, greedy_coloring};
///
/// // Path 0-1-2: greedy gives {0, 1, 0}
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let c = greedy_coloring(&g).unwrap();
/// assert_eq!(c, vec![0, 1, 0]);
/// ```
pub fn greedy_coloring(graph: &Graph) -> IgraphResult<Vec<u32>> {
    greedy_coloring_with_order(graph, None)
}

/// Greedy vertex coloring with a custom vertex processing order.
///
/// If `order` is `None`, processes vertices in natural order (0, 1, …).
/// Otherwise, processes vertices in the order given by the slice.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, greedy_coloring_with_order};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let c = greedy_coloring_with_order(&g, Some(&[2, 1, 0])).unwrap();
/// // All three are mutually adjacent, so 3 colors needed regardless of order
/// let num_colors = *c.iter().max().unwrap() + 1;
/// assert_eq!(num_colors, 3);
/// ```
pub fn greedy_coloring_with_order(graph: &Graph, order: Option<&[u32]>) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }

    if let Some(ord) = order {
        if ord.len() != n {
            return Err(IgraphError::InvalidArgument(
                "greedy_coloring_with_order: order length must equal vertex count".into(),
            ));
        }
    }

    let mut colors = vec![u32::MAX; n];
    let mut used = vec![false; n + 1];

    for idx in 0..n {
        let v = if let Some(ord) = order {
            ord[idx] as usize
        } else {
            idx
        };

        let nbrs = neighbors_both(graph, v as u32)?;
        for &u in &nbrs {
            let ui = u as usize;
            if colors[ui] != u32::MAX {
                let c = colors[ui] as usize;
                if c < used.len() {
                    used[c] = true;
                }
            }
        }

        let mut c = 0_u32;
        while (c as usize) < used.len() && used[c as usize] {
            c = c.saturating_add(1);
        }
        colors[v] = c;

        for &u in &nbrs {
            let ui = u as usize;
            if colors[ui] != u32::MAX {
                let cu = colors[ui] as usize;
                if cu < used.len() {
                    used[cu] = false;
                }
            }
        }
    }

    Ok(colors)
}

/// Greedy coloring with largest-first (LF) ordering.
///
/// Processes vertices in decreasing degree order (ties broken by
/// vertex id). Often produces better colourings than natural order.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, greedy_coloring_largest_first};
///
/// // Cycle C_5: chromatic number is 3
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,4),(4,0)], false, Some(5)
/// ).unwrap();
/// let c = greedy_coloring_largest_first(&g).unwrap();
/// let num_colors = *c.iter().max().unwrap() + 1;
/// assert!(num_colors <= 3);
/// ```
pub fn greedy_coloring_largest_first(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }

    let mut deg_order: Vec<(usize, u32)> = (0..n)
        .map(|v| {
            let d = degree_both(graph, v as u32);
            (d, v as u32)
        })
        .collect();
    deg_order.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));

    let order: Vec<u32> = deg_order.iter().map(|&(_, v)| v).collect();
    greedy_coloring_with_order(graph, Some(&order))
}

/// Count the number of colors used in a coloring.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, greedy_coloring, chromatic_number_greedy};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let c = greedy_coloring(&g).unwrap();
/// assert_eq!(chromatic_number_greedy(&c), 3);
/// ```
pub fn chromatic_number_greedy(coloring: &[u32]) -> u32 {
    if coloring.is_empty() {
        return 0;
    }
    coloring.iter().max().map_or(0, |&m| m + 1)
}

/// Check whether a coloring is valid (proper).
///
/// A proper coloring assigns colours such that no two adjacent vertices
/// share the same colour.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, greedy_coloring, is_proper_coloring};
///
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let c = greedy_coloring(&g).unwrap();
/// assert!(is_proper_coloring(&g, &c).unwrap());
/// ```
pub fn is_proper_coloring(graph: &Graph, coloring: &[u32]) -> IgraphResult<bool> {
    let n = graph.vcount() as usize;
    if coloring.len() != n {
        return Err(IgraphError::InvalidArgument(
            "is_proper_coloring: coloring length must equal vertex count".into(),
        ));
    }

    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        if coloring[ui] == coloring[vi] {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Compute the greedy clique number (lower bound on chromatic number).
///
/// Uses a greedy approach: iteratively pick an uncoloured vertex with
/// the most uncoloured neighbours and add it to a clique if it's
/// adjacent to all current clique members. This gives `ω(G) ≤ χ(G)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, greedy_clique_number};
///
/// // K_4: clique number is 4
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert_eq!(greedy_clique_number(&g).unwrap(), 4);
/// ```
pub fn greedy_clique_number(graph: &Graph) -> IgraphResult<u32> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "greedy_clique_number is defined for undirected graphs only".into(),
        ));
    }

    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let adj = build_adj_set(graph);

    let mut best_clique = 0_u32;
    let mut used = vec![false; n];

    for _ in 0..n {
        let mut clique: Vec<u32> = Vec::new();
        let mut candidates: Vec<u32> = (0..n as u32).filter(|&v| !used[v as usize]).collect();

        while !candidates.is_empty() {
            let mut scored: Vec<(usize, u32)> = candidates
                .iter()
                .map(|&v| {
                    let d = candidates
                        .iter()
                        .filter(|&&c| c != v && is_adj(&adj, n, v, c))
                        .count();
                    (d, v)
                })
                .collect();
            scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));

            let v = scored[0].1;
            clique.push(v);
            candidates.retain(|&c| c != v && is_adj(&adj, n, v, c));
        }

        if clique.len() as u32 > best_clique {
            best_clique = clique.len() as u32;
        }

        if let Some(&first) = clique.first() {
            used[first as usize] = true;
        } else {
            break;
        }
    }

    Ok(best_clique)
}

fn build_adj_set(graph: &Graph) -> Vec<bool> {
    let n = graph.vcount() as usize;
    let mut adj = vec![false; n * n];
    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        adj[ui * n + vi] = true;
        adj[vi * n + ui] = true;
    }
    adj
}

fn is_adj(adj: &[bool], n: usize, u: u32, v: u32) -> bool {
    adj[u as usize * n + v as usize]
}

fn neighbors_both(graph: &Graph, v: u32) -> IgraphResult<Vec<u32>> {
    if graph.is_directed() {
        let mut nbrs = graph.neighbors(v)?;
        if let Ok(in_nbrs) = graph.in_neighbors_vec(v) {
            for u in in_nbrs {
                if !nbrs.contains(&u) {
                    nbrs.push(u);
                }
            }
        }
        Ok(nbrs)
    } else {
        graph.neighbors(v)
    }
}

fn degree_both(graph: &Graph, v: u32) -> usize {
    neighbors_both(graph, v).map_or(0, |n| n.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap()
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

    fn star4() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3)], false, Some(4)).unwrap()
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

    fn bipartite_k23() -> Graph {
        Graph::from_edges(
            &[(0, 2), (0, 3), (0, 4), (1, 2), (1, 3), (1, 4)],
            false,
            Some(5),
        )
        .unwrap()
    }

    // --- greedy_coloring ---

    #[test]
    fn gc_empty() {
        let g = Graph::with_vertices(0);
        let c = greedy_coloring(&g).unwrap();
        assert!(c.is_empty());
    }

    #[test]
    fn gc_single() {
        let g = Graph::with_vertices(1);
        let c = greedy_coloring(&g).unwrap();
        assert_eq!(c, vec![0]);
    }

    #[test]
    fn gc_isolated() {
        let g = Graph::with_vertices(5);
        let c = greedy_coloring(&g).unwrap();
        assert_eq!(c, vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn gc_path3() {
        let g = path3();
        let c = greedy_coloring(&g).unwrap();
        assert_eq!(c, vec![0, 1, 0]);
        assert!(is_proper_coloring(&g, &c).unwrap());
    }

    #[test]
    fn gc_k3() {
        let g = k3();
        let c = greedy_coloring(&g).unwrap();
        assert_eq!(chromatic_number_greedy(&c), 3);
        assert!(is_proper_coloring(&g, &c).unwrap());
    }

    #[test]
    fn gc_k4() {
        let g = k4();
        let c = greedy_coloring(&g).unwrap();
        assert_eq!(chromatic_number_greedy(&c), 4);
        assert!(is_proper_coloring(&g, &c).unwrap());
    }

    #[test]
    fn gc_bipartite() {
        let g = bipartite_k23();
        let c = greedy_coloring(&g).unwrap();
        assert!(chromatic_number_greedy(&c) <= 3);
        assert!(is_proper_coloring(&g, &c).unwrap());
    }

    #[test]
    fn gc_star() {
        let g = star4();
        let c = greedy_coloring(&g).unwrap();
        assert_eq!(chromatic_number_greedy(&c), 2);
        assert!(is_proper_coloring(&g, &c).unwrap());
    }

    // --- greedy_coloring_largest_first ---

    #[test]
    fn gclf_k3() {
        let g = k3();
        let c = greedy_coloring_largest_first(&g).unwrap();
        assert_eq!(chromatic_number_greedy(&c), 3);
        assert!(is_proper_coloring(&g, &c).unwrap());
    }

    #[test]
    fn gclf_cycle5() {
        let g = cycle5();
        let c = greedy_coloring_largest_first(&g).unwrap();
        assert!(chromatic_number_greedy(&c) <= 3);
        assert!(is_proper_coloring(&g, &c).unwrap());
    }

    #[test]
    fn gclf_bipartite() {
        let g = bipartite_k23();
        let c = greedy_coloring_largest_first(&g).unwrap();
        assert_eq!(chromatic_number_greedy(&c), 2);
        assert!(is_proper_coloring(&g, &c).unwrap());
    }

    #[test]
    fn gclf_petersen() {
        let g = petersen();
        let c = greedy_coloring_largest_first(&g).unwrap();
        // Petersen graph: χ = 3; greedy LF should use ≤ 4
        assert!(chromatic_number_greedy(&c) <= 4);
        assert!(is_proper_coloring(&g, &c).unwrap());
    }

    // --- greedy_coloring_with_order ---

    #[test]
    fn gcwo_custom_order() {
        let g = path3();
        let c = greedy_coloring_with_order(&g, Some(&[2, 0, 1])).unwrap();
        assert!(is_proper_coloring(&g, &c).unwrap());
        assert_eq!(chromatic_number_greedy(&c), 2);
    }

    #[test]
    fn gcwo_wrong_length() {
        let g = path3();
        assert!(greedy_coloring_with_order(&g, Some(&[0, 1])).is_err());
    }

    // --- chromatic_number_greedy ---

    #[test]
    fn cng_empty() {
        assert_eq!(chromatic_number_greedy(&[]), 0);
    }

    #[test]
    fn cng_single() {
        assert_eq!(chromatic_number_greedy(&[0]), 1);
    }

    // --- is_proper_coloring ---

    #[test]
    fn ipc_valid() {
        let g = k3();
        assert!(is_proper_coloring(&g, &[0, 1, 2]).unwrap());
    }

    #[test]
    fn ipc_invalid() {
        let g = k3();
        assert!(!is_proper_coloring(&g, &[0, 1, 0]).unwrap());
    }

    #[test]
    fn ipc_wrong_length() {
        let g = k3();
        assert!(is_proper_coloring(&g, &[0, 1]).is_err());
    }

    // --- greedy_clique_number ---

    #[test]
    fn gcn_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(greedy_clique_number(&g).unwrap(), 0);
    }

    #[test]
    fn gcn_single() {
        let g = Graph::with_vertices(1);
        assert_eq!(greedy_clique_number(&g).unwrap(), 1);
    }

    #[test]
    fn gcn_k4() {
        let g = k4();
        assert_eq!(greedy_clique_number(&g).unwrap(), 4);
    }

    #[test]
    fn gcn_cycle5() {
        let g = cycle5();
        let cn = greedy_clique_number(&g).unwrap();
        assert_eq!(cn, 2);
    }

    #[test]
    fn gcn_petersen() {
        let g = petersen();
        let cn = greedy_clique_number(&g).unwrap();
        assert_eq!(cn, 2);
    }

    #[test]
    fn gcn_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(greedy_clique_number(&g).is_err());
    }

    // --- cross-consistency ---

    #[test]
    fn coloring_always_proper() {
        for g in &[path3(), k3(), k4(), cycle5(), star4(), petersen()] {
            let c = greedy_coloring(g).unwrap();
            assert!(is_proper_coloring(g, &c).unwrap());
        }
    }

    #[test]
    fn lf_never_worse_than_max_degree_plus_one() {
        for g in &[path3(), k3(), k4(), cycle5(), star4(), petersen()] {
            let c = greedy_coloring_largest_first(g).unwrap();
            let num_colors = chromatic_number_greedy(&c);
            let max_deg = (0..g.vcount())
                .map(|v| degree_both(g, v) as u32)
                .max()
                .unwrap_or(0);
            assert!(
                num_colors <= max_deg + 1,
                "greedy LF used {num_colors} colors but Δ+1 = {}",
                max_deg + 1
            );
        }
    }

    #[test]
    fn clique_bound_vs_coloring() {
        for g in &[k3(), k4(), cycle5(), petersen()] {
            let cn = greedy_clique_number(g).unwrap();
            let c = greedy_coloring_largest_first(g).unwrap();
            let chi = chromatic_number_greedy(&c);
            assert!(cn <= chi, "clique number {cn} > chromatic bound {chi}");
        }
    }

    // --- directed graph coloring ---

    #[test]
    fn gc_directed() {
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], true, Some(3)).unwrap();
        let c = greedy_coloring(&g).unwrap();
        assert_eq!(chromatic_number_greedy(&c), 3);
        assert!(is_proper_coloring(&g, &c).unwrap());
    }
}
