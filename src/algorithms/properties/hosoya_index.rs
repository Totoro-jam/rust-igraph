//! Hosoya index (ALGO-TR-036).
//!
//! The **Hosoya index** (Z-index) of a graph is the total number of
//! matchings, including the empty matching. A *k*-matching is a set
//! of *k* pairwise non-adjacent edges. The Hosoya index equals
//! `Σ_{k=0..⌊n/2⌋} m(G, k)` where `m(G, k)` is the number of
//! *k*-matchings.
//!
//! Also provides `matching_count_sequence` returning the full vector
//! `[m(G,0), m(G,1), ..., m(G,⌊n/2⌋)]`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the Hosoya index (Z-index) of a graph.
///
/// The Hosoya index equals the total number of matchings (including
/// the empty matching). For a graph with no edges, `Z(G) = 1`.
///
/// Only feasible for small/sparse graphs — the value grows
/// exponentially.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, hosoya_index};
///
/// // Path 0-1-2: matchings are {}, {01}, {12} → Z = 3
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(hosoya_index(&g).unwrap(), 3);
/// ```
pub fn hosoya_index(graph: &Graph) -> IgraphResult<u64> {
    let seq = matching_count_sequence(graph)?;
    let total: u64 = seq.iter().sum();
    Ok(total)
}

/// Compute the matching count sequence `[m(G,0), m(G,1), …]`.
///
/// `m(G, k)` is the number of *k*-matchings (sets of *k* pairwise
/// non-adjacent edges). `m(G, 0) = 1` always.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, matching_count_sequence};
///
/// // K_3 (triangle): m(0)=1, m(1)=3 → [1, 3]
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(matching_count_sequence(&g).unwrap(), vec![1, 3]);
/// ```
pub fn matching_count_sequence(graph: &Graph) -> IgraphResult<Vec<u64>> {
    let n = graph.vcount() as usize;
    let edges: Vec<(usize, usize)> = graph
        .edges()
        .map(|(u, v)| (u as usize, v as usize))
        .collect();

    let max_k = n / 2;
    let mut counts = vec![0_u64; max_k.saturating_add(1)];
    counts[0] = 1;

    if edges.is_empty() {
        return Ok(counts);
    }

    let unique_edges = deduplicate_edges(&edges, graph.is_directed());
    let m = unique_edges.len();

    let mut used = vec![false; n];
    enumerate_matchings(&unique_edges, m, 0, 0, &mut used, &mut counts);

    Ok(counts)
}

fn deduplicate_edges(edges: &[(usize, usize)], directed: bool) -> Vec<(usize, usize)> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for &(u, v) in edges {
        if u == v {
            continue;
        }
        let key = if directed {
            (u, v)
        } else {
            (u.min(v), u.max(v))
        };
        if seen.insert(key) {
            result.push(key);
        }
    }
    result
}

fn enumerate_matchings(
    edges: &[(usize, usize)],
    m: usize,
    start: usize,
    k: usize,
    used: &mut [bool],
    counts: &mut [u64],
) {
    for i in start..m {
        let (u, v) = edges[i];
        if used[u] || used[v] {
            continue;
        }
        used[u] = true;
        used[v] = true;
        let new_k = k.saturating_add(1);
        if new_k < counts.len() {
            counts[new_k] = counts[new_k].saturating_add(1);
            enumerate_matchings(edges, m, i.saturating_add(1), new_k, used, counts);
        }
        used[u] = false;
        used[v] = false;
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

    fn no_edges() -> Graph {
        Graph::with_vertices(4)
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

    fn path5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)], false, Some(5)).unwrap()
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

    // --- hosoya_index ---

    #[test]
    fn hi_empty() {
        assert_eq!(hosoya_index(&empty()).unwrap(), 1);
    }

    #[test]
    fn hi_single() {
        assert_eq!(hosoya_index(&single()).unwrap(), 1);
    }

    #[test]
    fn hi_no_edges() {
        assert_eq!(hosoya_index(&no_edges()).unwrap(), 1);
    }

    #[test]
    fn hi_single_edge() {
        // {}, {01} → 2
        assert_eq!(hosoya_index(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn hi_path3() {
        // {}, {01}, {12} → 3
        assert_eq!(hosoya_index(&path3()).unwrap(), 3);
    }

    #[test]
    fn hi_path4() {
        // P_4: Z = 5 (Fibonacci pattern: Z(P_n) = F(n+1))
        // {}, {01}, {12}, {23}, {01,23} → 5
        assert_eq!(hosoya_index(&path4()).unwrap(), 5);
    }

    #[test]
    fn hi_path5() {
        // P_5: Z = 8 (F(6) = 8)
        assert_eq!(hosoya_index(&path5()).unwrap(), 8);
    }

    #[test]
    fn hi_k3() {
        // K_3: {}, {01}, {02}, {12} → 4
        assert_eq!(hosoya_index(&k3()).unwrap(), 4);
    }

    #[test]
    fn hi_k4() {
        // K_4: 6 edges, perfect matchings = 3
        // m(0)=1, m(1)=6, m(2)=3 → Z = 10
        assert_eq!(hosoya_index(&k4()).unwrap(), 10);
    }

    #[test]
    fn hi_cycle4() {
        // C_4: {}, {01},{12},{23},{30}, {01,23},{12,30} → 7
        assert_eq!(hosoya_index(&cycle4()).unwrap(), 7);
    }

    #[test]
    fn hi_cycle5() {
        // C_5: m(0)=1, m(1)=5, m(2)=5 → Z = 11
        assert_eq!(hosoya_index(&cycle5()).unwrap(), 11);
    }

    #[test]
    fn hi_star4() {
        // Star K_{1,3}: 3 edges, no 2-matching possible
        // {}, {01}, {02}, {03} → 4
        assert_eq!(hosoya_index(&star4()).unwrap(), 4);
    }

    // --- matching_count_sequence ---

    #[test]
    fn mcs_empty() {
        assert_eq!(matching_count_sequence(&empty()).unwrap(), vec![1]);
    }

    #[test]
    fn mcs_single_edge() {
        assert_eq!(matching_count_sequence(&single_edge()).unwrap(), vec![1, 1]);
    }

    #[test]
    fn mcs_path3() {
        assert_eq!(matching_count_sequence(&path3()).unwrap(), vec![1, 2]);
    }

    #[test]
    fn mcs_path4() {
        // m(0)=1, m(1)=3, m(2)=1
        assert_eq!(matching_count_sequence(&path4()).unwrap(), vec![1, 3, 1]);
    }

    #[test]
    fn mcs_k4() {
        // m(0)=1, m(1)=6, m(2)=3
        assert_eq!(matching_count_sequence(&k4()).unwrap(), vec![1, 6, 3]);
    }

    #[test]
    fn mcs_cycle4() {
        // m(0)=1, m(1)=4, m(2)=2
        assert_eq!(matching_count_sequence(&cycle4()).unwrap(), vec![1, 4, 2]);
    }

    #[test]
    fn mcs_cycle5() {
        // m(0)=1, m(1)=5, m(2)=5
        assert_eq!(matching_count_sequence(&cycle5()).unwrap(), vec![1, 5, 5]);
    }

    // --- cross-consistency ---

    #[test]
    fn hosoya_equals_sum_of_sequence() {
        for g in &[path3(), path4(), path5(), k3(), k4(), cycle4(), cycle5()] {
            let z = hosoya_index(g).unwrap();
            let seq = matching_count_sequence(g).unwrap();
            let sum: u64 = seq.iter().sum();
            assert_eq!(z, sum);
        }
    }

    #[test]
    fn m0_always_one() {
        for g in &[empty(), single(), no_edges(), path4(), k4()] {
            let seq = matching_count_sequence(g).unwrap();
            assert_eq!(seq[0], 1);
        }
    }

    #[test]
    fn m1_equals_edge_count() {
        for g in &[single_edge(), path3(), path4(), k3(), k4(), cycle4()] {
            let seq = matching_count_sequence(g).unwrap();
            if seq.len() > 1 {
                assert_eq!(seq[1], g.ecount() as u64);
            }
        }
    }

    #[test]
    fn path_hosoya_fibonacci() {
        // Z(P_n) = F(n+1) where F is Fibonacci (F(1)=1, F(2)=1, F(3)=2, ...)
        let fib = [1, 1, 2, 3, 5, 8, 13];
        for n in 1_u32..=6 {
            let edges: Vec<(u32, u32)> = (0..n.saturating_sub(1)).map(|i| (i, i + 1)).collect();
            let g = if n == 1 {
                Graph::with_vertices(1)
            } else {
                Graph::from_edges(&edges, false, Some(n)).unwrap()
            };
            assert_eq!(
                hosoya_index(&g).unwrap(),
                fib[n as usize],
                "Z(P_{n}) should be F({})",
                n + 1
            );
        }
    }
}
