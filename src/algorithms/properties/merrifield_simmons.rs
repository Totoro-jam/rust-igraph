//! Merrifield–Simmons index (ALGO-TR-037).
//!
//! The **Merrifield–Simmons index** `σ(G)` is the total number of
//! independent sets (including the empty set). An independent set is a
//! subset of vertices with no two adjacent.
//!
//! Also provides `independent_set_count_sequence` returning the vector
//! `[i(G,0), i(G,1), ..., i(G,α)]` where `i(G,k)` counts the number
//! of independent sets of size *k* and `α` is the independence number.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the Merrifield–Simmons index of a graph.
///
/// Returns the total number of independent sets (including the empty
/// set). For a graph with no edges, `σ(G) = 2^n`.
///
/// Only feasible for small graphs — the count can be exponential.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, merrifield_simmons_index};
///
/// // Path 0-1-2: independent sets: {}, {0}, {1}, {2}, {0,2} → σ = 5
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(merrifield_simmons_index(&g).unwrap(), 5);
/// ```
pub fn merrifield_simmons_index(graph: &Graph) -> IgraphResult<u64> {
    let seq = independent_set_count_sequence(graph)?;
    let total: u64 = seq.iter().sum();
    Ok(total)
}

/// Compute the independence polynomial coefficient sequence.
///
/// Returns `[i(G,0), i(G,1), ..., i(G,n)]` where `i(G,k)` is the
/// number of independent sets of size `k`. Trailing zeros are trimmed.
/// `i(G,0) = 1` always (the empty set).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, independent_set_count_sequence};
///
/// // K_3 (triangle): {}, {0}, {1}, {2} → [1, 3]
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(independent_set_count_sequence(&g).unwrap(), vec![1, 3]);
/// ```
pub fn independent_set_count_sequence(graph: &Graph) -> IgraphResult<Vec<u64>> {
    let n = graph.vcount() as usize;

    let mut counts = vec![0_u64; n.saturating_add(1)];
    counts[0] = 1;

    if n == 0 {
        return Ok(counts);
    }

    let adj = build_adj_matrix(graph, n);

    let mut selected = vec![false; n];
    enumerate_independent_sets(&adj, n, 0, 0, &mut selected, &mut counts);

    while counts.len() > 1 && counts[counts.len() - 1] == 0 {
        counts.pop();
    }

    Ok(counts)
}

fn build_adj_matrix(graph: &Graph, n: usize) -> Vec<bool> {
    let mut adj = vec![false; n * n];
    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        if ui != vi {
            adj[ui * n + vi] = true;
            if !graph.is_directed() {
                adj[vi * n + ui] = true;
            }
        }
    }
    adj
}

fn enumerate_independent_sets(
    adj: &[bool],
    n: usize,
    start: usize,
    k: usize,
    selected: &mut [bool],
    counts: &mut [u64],
) {
    for v in start..n {
        let mut conflict = false;
        for u in 0..v {
            if selected[u] && adj[u * n + v] {
                conflict = true;
                break;
            }
        }
        if conflict {
            continue;
        }

        selected[v] = true;
        let new_k = k.saturating_add(1);
        if new_k < counts.len() {
            counts[new_k] = counts[new_k].saturating_add(1);
            enumerate_independent_sets(adj, n, v.saturating_add(1), new_k, selected, counts);
        }
        selected[v] = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> Graph {
        Graph::with_vertices(0)
    }

    fn single() -> Graph {
        Graph::with_vertices(1)
    }

    fn no_edges3() -> Graph {
        Graph::with_vertices(3)
    }

    fn single_edge() -> Graph {
        Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap()
    }

    fn path3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap()
    }

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

    fn star4() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3)], false, Some(4)).unwrap()
    }

    // --- merrifield_simmons_index ---

    #[test]
    fn msi_empty() {
        // σ(empty) = 1 (just the empty set)
        assert_eq!(merrifield_simmons_index(&empty()).unwrap(), 1);
    }

    #[test]
    fn msi_single() {
        // {}, {0} → 2
        assert_eq!(merrifield_simmons_index(&single()).unwrap(), 2);
    }

    #[test]
    fn msi_no_edges() {
        // σ(E_n) = 2^n = 8 for n=3
        assert_eq!(merrifield_simmons_index(&no_edges3()).unwrap(), 8);
    }

    #[test]
    fn msi_single_edge() {
        // {}, {0}, {1} → 3
        assert_eq!(merrifield_simmons_index(&single_edge()).unwrap(), 3);
    }

    #[test]
    fn msi_path3() {
        // {}, {0}, {1}, {2}, {0,2} → 5
        assert_eq!(merrifield_simmons_index(&path3()).unwrap(), 5);
    }

    #[test]
    fn msi_path4() {
        // P_4: σ follows Fibonacci-like: σ(P_n) = F(n+2)
        // σ(P_4) = 8 ({},{0},{1},{2},{3},{0,2},{0,3},{1,3})
        assert_eq!(merrifield_simmons_index(&path4()).unwrap(), 8);
    }

    #[test]
    fn msi_k3() {
        // K_3: {}, {0}, {1}, {2} → 4
        assert_eq!(merrifield_simmons_index(&k3()).unwrap(), 4);
    }

    #[test]
    fn msi_k4() {
        // K_4: {}, {0}, {1}, {2}, {3} → 5
        assert_eq!(merrifield_simmons_index(&k4()).unwrap(), 5);
    }

    #[test]
    fn msi_cycle4() {
        // C_4: {},{0},{1},{2},{3},{0,2},{1,3} → 7
        assert_eq!(merrifield_simmons_index(&cycle4()).unwrap(), 7);
    }

    #[test]
    fn msi_cycle5() {
        // C_5: 1 + 5 + 5 = 11
        // i(0)=1, i(1)=5, i(2)=5
        assert_eq!(merrifield_simmons_index(&cycle5()).unwrap(), 11);
    }

    #[test]
    fn msi_star4() {
        // Star K_{1,3}: {},{0},{1},{2},{3},{1,2},{1,3},{2,3},{1,2,3} → 9
        assert_eq!(merrifield_simmons_index(&star4()).unwrap(), 9);
    }

    // --- independent_set_count_sequence ---

    #[test]
    fn iscs_empty() {
        assert_eq!(independent_set_count_sequence(&empty()).unwrap(), vec![1]);
    }

    #[test]
    fn iscs_single() {
        assert_eq!(
            independent_set_count_sequence(&single()).unwrap(),
            vec![1, 1]
        );
    }

    #[test]
    fn iscs_no_edges() {
        // E_3: i(0)=1, i(1)=3, i(2)=3, i(3)=1
        assert_eq!(
            independent_set_count_sequence(&no_edges3()).unwrap(),
            vec![1, 3, 3, 1]
        );
    }

    #[test]
    fn iscs_single_edge() {
        assert_eq!(
            independent_set_count_sequence(&single_edge()).unwrap(),
            vec![1, 2]
        );
    }

    #[test]
    fn iscs_path3() {
        // i(0)=1, i(1)=3, i(2)=1
        assert_eq!(
            independent_set_count_sequence(&path3()).unwrap(),
            vec![1, 3, 1]
        );
    }

    #[test]
    fn iscs_k3() {
        assert_eq!(independent_set_count_sequence(&k3()).unwrap(), vec![1, 3]);
    }

    #[test]
    fn iscs_k4() {
        assert_eq!(independent_set_count_sequence(&k4()).unwrap(), vec![1, 4]);
    }

    #[test]
    fn iscs_cycle4() {
        // i(0)=1, i(1)=4, i(2)=2
        assert_eq!(
            independent_set_count_sequence(&cycle4()).unwrap(),
            vec![1, 4, 2]
        );
    }

    #[test]
    fn iscs_cycle5() {
        // i(0)=1, i(1)=5, i(2)=5
        assert_eq!(
            independent_set_count_sequence(&cycle5()).unwrap(),
            vec![1, 5, 5]
        );
    }

    // --- cross-consistency ---

    #[test]
    fn msi_equals_sum_of_sequence() {
        for g in &[path3(), path4(), k3(), k4(), cycle4(), cycle5(), star4()] {
            let sigma = merrifield_simmons_index(g).unwrap();
            let seq = independent_set_count_sequence(g).unwrap();
            let sum: u64 = seq.iter().sum();
            assert_eq!(sigma, sum);
        }
    }

    #[test]
    fn i0_always_one() {
        for g in &[empty(), single(), no_edges3(), path4(), k4()] {
            let seq = independent_set_count_sequence(g).unwrap();
            assert_eq!(seq[0], 1);
        }
    }

    #[test]
    fn i1_equals_vertex_count() {
        for g in &[single(), single_edge(), path3(), k3(), k4(), cycle4()] {
            let seq = independent_set_count_sequence(g).unwrap();
            if seq.len() > 1 {
                assert_eq!(seq[1], u64::from(g.vcount()));
            }
        }
    }

    #[test]
    fn no_edges_is_power_of_two() {
        for n in 0_u32..=5 {
            let g = Graph::with_vertices(n);
            let sigma = merrifield_simmons_index(&g).unwrap();
            assert_eq!(sigma, 1_u64 << n);
        }
    }

    #[test]
    fn complete_graph_sigma() {
        // σ(K_n) = n + 1
        for n in 1_u32..=5 {
            let edges: Vec<(u32, u32)> = (0..n)
                .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
                .collect();
            let g = if edges.is_empty() {
                Graph::with_vertices(n)
            } else {
                Graph::from_edges(&edges, false, Some(n)).unwrap()
            };
            assert_eq!(
                merrifield_simmons_index(&g).unwrap(),
                u64::from(n) + 1,
                "σ(K_{n}) should be {}",
                n + 1
            );
        }
    }

    #[test]
    fn path_sigma_fibonacci() {
        // σ(P_n) = F(n+2) where F(1)=1,F(2)=1,F(3)=2,...
        let fib = [1, 1, 2, 3, 5, 8, 13, 21];
        for n in 1_u32..=6 {
            let edges: Vec<(u32, u32)> = (0..n.saturating_sub(1)).map(|i| (i, i + 1)).collect();
            let g = if n == 1 {
                Graph::with_vertices(1)
            } else {
                Graph::from_edges(&edges, false, Some(n)).unwrap()
            };
            assert_eq!(
                merrifield_simmons_index(&g).unwrap(),
                fib[(n + 1) as usize],
                "σ(P_{n}) should be F({})",
                n + 2
            );
        }
    }
}
