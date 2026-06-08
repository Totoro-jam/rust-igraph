//! Hamiltonian path and cycle detection (ALGO-TR-031).
//!
//! A **Hamiltonian path** visits every vertex exactly once.
//! A **Hamiltonian cycle** visits every vertex exactly once and returns
//! to the start.
//!
//! Detection is NP-complete, so we use backtracking with pruning.
//! Suitable for small graphs (≤ ~20 vertices).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Check whether a sequence of vertices forms a valid Hamiltonian path.
///
/// A valid Hamiltonian path visits every vertex exactly once and each
/// consecutive pair is connected by an edge.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_hamiltonian_path};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// assert!(is_hamiltonian_path(&g, &[0, 1, 2, 3]).unwrap());
/// assert!(!is_hamiltonian_path(&g, &[0, 2, 1, 3]).unwrap());
/// ```
pub fn is_hamiltonian_path(graph: &Graph, path: &[u32]) -> IgraphResult<bool> {
    let n = graph.vcount() as usize;
    if path.len() != n {
        return Ok(false);
    }

    let mut visited = vec![false; n];
    for &v in path {
        let vi = v as usize;
        if vi >= n || visited[vi] {
            return Ok(false);
        }
        visited[vi] = true;
    }

    for i in 0..path.len() - 1 {
        let nbrs = graph.neighbors(path[i])?;
        if !nbrs.contains(&path[i + 1]) {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Check whether a sequence of vertices forms a valid Hamiltonian cycle.
///
/// A valid Hamiltonian cycle visits every vertex exactly once and the
/// last vertex is adjacent to the first.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_hamiltonian_cycle};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,0)], false, Some(4)
/// ).unwrap();
/// assert!(is_hamiltonian_cycle(&g, &[0, 1, 2, 3]).unwrap());
/// ```
pub fn is_hamiltonian_cycle(graph: &Graph, cycle: &[u32]) -> IgraphResult<bool> {
    let n = graph.vcount() as usize;
    if n < 3 || cycle.len() != n {
        return Ok(false);
    }

    if !is_hamiltonian_path(graph, cycle)? {
        return Ok(false);
    }

    let last = cycle[cycle.len() - 1];
    let first = cycle[0];
    let nbrs = graph.neighbors(last)?;
    Ok(nbrs.contains(&first))
}

/// Find a Hamiltonian path using backtracking.
///
/// Returns `Some(path)` if one exists, `None` otherwise.
/// Only feasible for small graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, hamiltonian_path, is_hamiltonian_path};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// let p = hamiltonian_path(&g).unwrap();
/// assert!(p.is_some());
/// assert!(is_hamiltonian_path(&g, &p.unwrap()).unwrap());
/// ```
pub fn hamiltonian_path(graph: &Graph) -> IgraphResult<Option<Vec<u32>>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Some(Vec::new()));
    }
    if n == 1 {
        return Ok(Some(vec![0]));
    }

    let adj = build_adj(graph, n);
    let mut visited = vec![false; n];
    let mut path = Vec::with_capacity(n);

    for start in 0..n {
        visited[start] = true;
        path.push(start as u32);
        if ham_path_bt(&adj, n, &mut visited, &mut path) {
            return Ok(Some(path));
        }
        path.pop();
        visited[start] = false;
    }

    Ok(None)
}

/// Find a Hamiltonian cycle using backtracking.
///
/// Returns `Some(cycle)` if one exists, `None` otherwise.
/// Only feasible for small graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, hamiltonian_cycle, is_hamiltonian_cycle};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,0)], false, Some(4)
/// ).unwrap();
/// let c = hamiltonian_cycle(&g).unwrap();
/// assert!(c.is_some());
/// assert!(is_hamiltonian_cycle(&g, &c.unwrap()).unwrap());
/// ```
pub fn hamiltonian_cycle(graph: &Graph) -> IgraphResult<Option<Vec<u32>>> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(None);
    }

    let adj = build_adj(graph, n);
    let mut visited = vec![false; n];
    let mut path = Vec::with_capacity(n);

    visited[0] = true;
    path.push(0_u32);
    let result = ham_cycle_bt(&adj, n, &mut visited, &mut path);

    if result { Ok(Some(path)) } else { Ok(None) }
}

/// Check whether a graph has a Hamiltonian path.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, has_hamiltonian_path};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// assert!(has_hamiltonian_path(&g).unwrap());
/// ```
pub fn has_hamiltonian_path(graph: &Graph) -> IgraphResult<bool> {
    Ok(hamiltonian_path(graph)?.is_some())
}

/// Check whether a graph has a Hamiltonian cycle.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, has_hamiltonian_cycle};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,0)], false, Some(4)
/// ).unwrap();
/// assert!(has_hamiltonian_cycle(&g).unwrap());
/// ```
pub fn has_hamiltonian_cycle(graph: &Graph) -> IgraphResult<bool> {
    Ok(hamiltonian_cycle(graph)?.is_some())
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

fn ham_path_bt(adj: &[bool], n: usize, visited: &mut Vec<bool>, path: &mut Vec<u32>) -> bool {
    if path.len() == n {
        return true;
    }

    let last = path[path.len() - 1] as usize;

    let unvisited_count = visited.iter().filter(|&&v| !v).count();
    if unvisited_count == 0 {
        return false;
    }

    for next in 0..n {
        if !visited[next] && adj[last * n + next] {
            if unvisited_count > 1 {
                let reachable = count_reachable_unvisited(adj, n, next, visited);
                if reachable < unvisited_count - 1 {
                    continue;
                }
            }

            visited[next] = true;
            path.push(next as u32);
            if ham_path_bt(adj, n, visited, path) {
                return true;
            }
            path.pop();
            visited[next] = false;
        }
    }

    false
}

fn ham_cycle_bt(adj: &[bool], n: usize, visited: &mut Vec<bool>, path: &mut Vec<u32>) -> bool {
    if path.len() == n {
        let last = path[path.len() - 1] as usize;
        return adj[last * n]; // check edge back to vertex 0
    }

    let last = path[path.len() - 1] as usize;

    for next in 0..n {
        if !visited[next] && adj[last * n + next] {
            visited[next] = true;
            path.push(next as u32);
            if ham_cycle_bt(adj, n, visited, path) {
                return true;
            }
            path.pop();
            visited[next] = false;
        }
    }

    false
}

fn count_reachable_unvisited(adj: &[bool], n: usize, start: usize, visited: &[bool]) -> usize {
    let mut seen = vec![false; n];
    seen[start] = true;
    let mut stack = vec![start];
    let mut count: usize = 0;

    while let Some(v) = stack.pop() {
        for u in 0..n {
            if !seen[u] && !visited[u] && adj[v * n + u] {
                seen[u] = true;
                count = count.saturating_add(1);
                stack.push(u);
            }
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    fn cycle5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)], false, Some(5)).unwrap()
    }

    fn k4() -> Graph {
        Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap()
    }

    fn k3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn disconnected() -> Graph {
        Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap()
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

    // --- is_hamiltonian_path ---

    #[test]
    fn ihp_valid() {
        let g = path4();
        assert!(is_hamiltonian_path(&g, &[0, 1, 2, 3]).unwrap());
    }

    #[test]
    fn ihp_reversed() {
        let g = path4();
        assert!(is_hamiltonian_path(&g, &[3, 2, 1, 0]).unwrap());
    }

    #[test]
    fn ihp_invalid_no_edge() {
        let g = path4();
        assert!(!is_hamiltonian_path(&g, &[0, 2, 1, 3]).unwrap());
    }

    #[test]
    fn ihp_wrong_length() {
        let g = path4();
        assert!(!is_hamiltonian_path(&g, &[0, 1, 2]).unwrap());
    }

    #[test]
    fn ihp_duplicate_vertex() {
        let g = path4();
        assert!(!is_hamiltonian_path(&g, &[0, 1, 1, 3]).unwrap());
    }

    #[test]
    fn ihp_out_of_range() {
        let g = path4();
        assert!(!is_hamiltonian_path(&g, &[0, 1, 2, 10]).unwrap());
    }

    // --- is_hamiltonian_cycle ---

    #[test]
    fn ihc_valid() {
        let g = cycle4();
        assert!(is_hamiltonian_cycle(&g, &[0, 1, 2, 3]).unwrap());
    }

    #[test]
    fn ihc_path_not_cycle() {
        let g = path4();
        assert!(!is_hamiltonian_cycle(&g, &[0, 1, 2, 3]).unwrap());
    }

    #[test]
    fn ihc_too_small() {
        let g = Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap();
        assert!(!is_hamiltonian_cycle(&g, &[0, 1]).unwrap());
    }

    #[test]
    fn ihc_k3() {
        let g = k3();
        assert!(is_hamiltonian_cycle(&g, &[0, 1, 2]).unwrap());
    }

    // --- hamiltonian_path ---

    #[test]
    fn hp_empty() {
        let g = Graph::with_vertices(0);
        let p = hamiltonian_path(&g).unwrap();
        assert!(p.is_some());
        assert!(p.unwrap().is_empty());
    }

    #[test]
    fn hp_single() {
        let g = Graph::with_vertices(1);
        let p = hamiltonian_path(&g).unwrap();
        assert_eq!(p.unwrap(), vec![0]);
    }

    #[test]
    fn hp_path4() {
        let g = path4();
        let p = hamiltonian_path(&g).unwrap().unwrap();
        assert!(is_hamiltonian_path(&g, &p).unwrap());
    }

    #[test]
    fn hp_cycle4() {
        let g = cycle4();
        let p = hamiltonian_path(&g).unwrap().unwrap();
        assert!(is_hamiltonian_path(&g, &p).unwrap());
    }

    #[test]
    fn hp_k4() {
        let g = k4();
        let p = hamiltonian_path(&g).unwrap().unwrap();
        assert!(is_hamiltonian_path(&g, &p).unwrap());
    }

    #[test]
    fn hp_star5() {
        let g = star5();
        assert!(hamiltonian_path(&g).unwrap().is_none());
    }

    #[test]
    fn hp_disconnected() {
        let g = disconnected();
        assert!(hamiltonian_path(&g).unwrap().is_none());
    }

    #[test]
    fn hp_cycle5() {
        let g = cycle5();
        let p = hamiltonian_path(&g).unwrap().unwrap();
        assert!(is_hamiltonian_path(&g, &p).unwrap());
    }

    // --- hamiltonian_cycle ---

    #[test]
    fn hc_cycle4() {
        let g = cycle4();
        let c = hamiltonian_cycle(&g).unwrap().unwrap();
        assert!(is_hamiltonian_cycle(&g, &c).unwrap());
    }

    #[test]
    fn hc_cycle5() {
        let g = cycle5();
        let c = hamiltonian_cycle(&g).unwrap().unwrap();
        assert!(is_hamiltonian_cycle(&g, &c).unwrap());
    }

    #[test]
    fn hc_k4() {
        let g = k4();
        let c = hamiltonian_cycle(&g).unwrap().unwrap();
        assert!(is_hamiltonian_cycle(&g, &c).unwrap());
    }

    #[test]
    fn hc_k3() {
        let g = k3();
        let c = hamiltonian_cycle(&g).unwrap().unwrap();
        assert!(is_hamiltonian_cycle(&g, &c).unwrap());
    }

    #[test]
    fn hc_path4_no_cycle() {
        let g = path4();
        assert!(hamiltonian_cycle(&g).unwrap().is_none());
    }

    #[test]
    fn hc_star5_no_cycle() {
        let g = star5();
        assert!(hamiltonian_cycle(&g).unwrap().is_none());
    }

    #[test]
    fn hc_disconnected_no_cycle() {
        let g = disconnected();
        assert!(hamiltonian_cycle(&g).unwrap().is_none());
    }

    #[test]
    fn hc_too_small() {
        let g = Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap();
        assert!(hamiltonian_cycle(&g).unwrap().is_none());
    }

    // --- has_hamiltonian_path / has_hamiltonian_cycle ---

    #[test]
    fn hhp_path4() {
        assert!(has_hamiltonian_path(&path4()).unwrap());
    }

    #[test]
    fn hhp_star5() {
        assert!(!has_hamiltonian_path(&star5()).unwrap());
    }

    #[test]
    fn hhc_cycle4() {
        assert!(has_hamiltonian_cycle(&cycle4()).unwrap());
    }

    #[test]
    fn hhc_path4() {
        assert!(!has_hamiltonian_cycle(&path4()).unwrap());
    }

    // --- petersen graph ---

    #[test]
    fn petersen_has_path() {
        let g = petersen();
        let p = hamiltonian_path(&g).unwrap().unwrap();
        assert!(is_hamiltonian_path(&g, &p).unwrap());
    }

    #[test]
    fn petersen_no_cycle() {
        let g = petersen();
        assert!(hamiltonian_cycle(&g).unwrap().is_none());
    }

    // --- cross-consistency ---

    #[test]
    fn cycle_implies_path() {
        for g in &[cycle4(), cycle5(), k3(), k4()] {
            if has_hamiltonian_cycle(g).unwrap() {
                assert!(has_hamiltonian_path(g).unwrap());
            }
        }
    }

    #[test]
    fn isolated_vertices() {
        let g = Graph::with_vertices(3);
        assert!(!has_hamiltonian_path(&g).unwrap());
        assert!(!has_hamiltonian_cycle(&g).unwrap());
    }
}
