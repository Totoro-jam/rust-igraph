//! Graph bandwidth (ALGO-TR-033).
//!
//! The **bandwidth** of a labeling `f: V → {0..n-1}` is
//! `max |f(u) - f(v)|` over all edges `(u,v)`.
//! The **bandwidth of a graph** `B(G)` is the minimum bandwidth
//! over all labelings.
//!
//! Computing `B(G)` is NP-hard; we provide:
//! - `bandwidth_of_labeling`: compute bandwidth for a given labeling.
//! - `bandwidth_lower_bound`: lower bound via max degree.
//! - `bandwidth`: brute-force minimum (small graphs only, ≤ ~10).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the bandwidth of a specific labeling.
///
/// The labeling is given as a permutation: `labeling[i]` is the
/// label assigned to vertex `i`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bandwidth_of_labeling};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// // identity labeling: max |i-j| over edges = max(1,1,1) = 1
/// assert_eq!(bandwidth_of_labeling(&g, &[0,1,2,3]).unwrap(), 1);
/// // reversed labeling: max |3-2|,|2-1|,|1-0| = 1
/// assert_eq!(bandwidth_of_labeling(&g, &[3,2,1,0]).unwrap(), 1);
/// ```
pub fn bandwidth_of_labeling(graph: &Graph, labeling: &[u32]) -> IgraphResult<u32> {
    let n = graph.vcount() as usize;
    if labeling.len() != n {
        return Err(crate::core::IgraphError::InvalidArgument(format!(
            "bandwidth_of_labeling: labeling length {} != vcount {n}",
            labeling.len()
        )));
    }

    let mut bw: u32 = 0;
    for (u, v) in graph.edges() {
        let lu = labeling[u as usize];
        let lv = labeling[v as usize];
        let diff = lu.abs_diff(lv);
        if diff > bw {
            bw = diff;
        }
    }

    Ok(bw)
}

/// Compute a lower bound on graph bandwidth.
///
/// Uses the degree-based bound: `B(G) >= ceil(max_degree / 2)`.
/// Also: for connected graphs, `B(G) >= ceil((n-1) / diameter)`.
///
/// Returns the maximum of available lower bounds.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bandwidth_lower_bound};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// assert!(bandwidth_lower_bound(&g).unwrap() >= 1);
/// ```
pub fn bandwidth_lower_bound(graph: &Graph) -> IgraphResult<u32> {
    let n = graph.vcount() as usize;
    if n <= 1 {
        return Ok(0);
    }

    let mut max_deg: usize = 0;
    for v in 0..n as u32 {
        let d = graph.degree(v)?;
        if d > max_deg {
            max_deg = d;
        }
    }

    let deg_bound = (max_deg.saturating_add(1)) / 2;

    Ok(deg_bound as u32)
}

/// Compute the exact graph bandwidth `B(G)`.
///
/// Tries all permutations of vertices and returns the minimum
/// bandwidth. Only feasible for very small graphs (≤ ~10 vertices).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bandwidth};
///
/// // Path graph: optimal bandwidth is 1
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// assert_eq!(bandwidth(&g).unwrap(), 1);
/// ```
pub fn bandwidth(graph: &Graph) -> IgraphResult<u32> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }
    if graph.ecount() == 0 {
        return Ok(0);
    }

    let edges: Vec<(usize, usize)> = graph
        .edges()
        .map(|(u, v)| (u as usize, v as usize))
        .collect();

    let mut perm: Vec<u32> = (0..n as u32).collect();
    let mut best = n as u32;

    permute_and_minimize(&edges, &mut perm, 0, &mut best);

    Ok(best)
}

fn permute_and_minimize(
    edges: &[(usize, usize)],
    perm: &mut Vec<u32>,
    depth: usize,
    best: &mut u32,
) {
    let n = perm.len();

    if depth == n {
        let bw = compute_bw(edges, perm);
        if bw < *best {
            *best = bw;
        }
        return;
    }

    for i in depth..n {
        perm.swap(depth, i);

        let partial_bw = partial_bandwidth(edges, perm, depth);
        if partial_bw < *best {
            permute_and_minimize(edges, perm, depth.saturating_add(1), best);
        }

        perm.swap(depth, i);
    }
}

fn compute_bw(edges: &[(usize, usize)], perm: &[u32]) -> u32 {
    let mut bw: u32 = 0;
    for &(u, v) in edges {
        let lu = perm[u];
        let lv = perm[v];
        let diff = lu.abs_diff(lv);
        if diff > bw {
            bw = diff;
        }
    }
    bw
}

fn partial_bandwidth(edges: &[(usize, usize)], perm: &[u32], depth: usize) -> u32 {
    let mut bw: u32 = 0;
    for &(u, v) in edges {
        if u <= depth && v <= depth {
            let lu = perm[u];
            let lv = perm[v];
            let diff = lu.abs_diff(lv);
            if diff > bw {
                bw = diff;
            }
        }
    }
    bw
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

    // --- bandwidth_of_labeling ---

    #[test]
    fn bol_identity() {
        let g = path4();
        assert_eq!(bandwidth_of_labeling(&g, &[0, 1, 2, 3]).unwrap(), 1);
    }

    #[test]
    fn bol_reversed() {
        let g = path4();
        assert_eq!(bandwidth_of_labeling(&g, &[3, 2, 1, 0]).unwrap(), 1);
    }

    #[test]
    fn bol_bad_labeling() {
        let g = path4();
        assert_eq!(bandwidth_of_labeling(&g, &[0, 3, 1, 2]).unwrap(), 3);
    }

    #[test]
    fn bol_k3() {
        let g = k3();
        assert_eq!(bandwidth_of_labeling(&g, &[0, 1, 2]).unwrap(), 2);
    }

    #[test]
    fn bol_wrong_length() {
        let g = path4();
        assert!(bandwidth_of_labeling(&g, &[0, 1]).is_err());
    }

    #[test]
    fn bol_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(bandwidth_of_labeling(&g, &[]).unwrap(), 0);
    }

    #[test]
    fn bol_no_edges() {
        let g = Graph::with_vertices(3);
        assert_eq!(bandwidth_of_labeling(&g, &[0, 1, 2]).unwrap(), 0);
    }

    // --- bandwidth_lower_bound ---

    #[test]
    fn blb_path4() {
        assert!(bandwidth_lower_bound(&path4()).unwrap() >= 1);
    }

    #[test]
    fn blb_k4() {
        let lb = bandwidth_lower_bound(&k4()).unwrap();
        assert!(lb >= 2);
    }

    #[test]
    fn blb_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(bandwidth_lower_bound(&g).unwrap(), 0);
    }

    #[test]
    fn blb_single() {
        let g = Graph::with_vertices(1);
        assert_eq!(bandwidth_lower_bound(&g).unwrap(), 0);
    }

    // --- bandwidth ---

    #[test]
    fn bw_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(bandwidth(&g).unwrap(), 0);
    }

    #[test]
    fn bw_single() {
        let g = Graph::with_vertices(1);
        assert_eq!(bandwidth(&g).unwrap(), 0);
    }

    #[test]
    fn bw_no_edges() {
        let g = Graph::with_vertices(3);
        assert_eq!(bandwidth(&g).unwrap(), 0);
    }

    #[test]
    fn bw_single_edge() {
        let g = Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap();
        assert_eq!(bandwidth(&g).unwrap(), 1);
    }

    #[test]
    fn bw_path4() {
        assert_eq!(bandwidth(&path4()).unwrap(), 1);
    }

    #[test]
    fn bw_k3() {
        assert_eq!(bandwidth(&k3()).unwrap(), 2);
    }

    #[test]
    fn bw_k4() {
        // B(K_n) = n-1: every pair is an edge, max label diff is n-1
        assert_eq!(bandwidth(&k4()).unwrap(), 3);
    }

    #[test]
    fn bw_cycle4() {
        assert_eq!(bandwidth(&cycle4()).unwrap(), 2);
    }

    #[test]
    fn bw_cycle5() {
        assert_eq!(bandwidth(&cycle5()).unwrap(), 2);
    }

    #[test]
    fn bw_star5() {
        assert_eq!(bandwidth(&star5()).unwrap(), 2);
    }

    // --- cross-consistency ---

    #[test]
    fn bw_geq_lower_bound() {
        for g in &[path4(), k3(), k4(), cycle4(), cycle5(), star5()] {
            let bw = bandwidth(g).unwrap();
            let lb = bandwidth_lower_bound(g).unwrap();
            assert!(bw >= lb, "bandwidth {bw} < lower bound {lb}");
        }
    }

    #[test]
    fn bw_leq_identity_labeling() {
        for g in &[path4(), k3(), k4(), cycle4()] {
            let n = g.vcount() as usize;
            let identity: Vec<u32> = (0..n as u32).collect();
            let id_bw = bandwidth_of_labeling(g, &identity).unwrap();
            let opt_bw = bandwidth(g).unwrap();
            assert!(opt_bw <= id_bw);
        }
    }

    #[test]
    fn path_bandwidth_is_one() {
        for n in 2_u32..=6 {
            let edges: Vec<(u32, u32)> = (0..n - 1).map(|i| (i, i + 1)).collect();
            let g = Graph::from_edges(&edges, false, Some(n)).unwrap();
            assert_eq!(bandwidth(&g).unwrap(), 1);
        }
    }

    #[test]
    fn complete_graph_bandwidth() {
        // B(K_n) = n-1: every pair is an edge
        let g2 = Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap();
        assert_eq!(bandwidth(&g2).unwrap(), 1);
        assert_eq!(bandwidth(&k3()).unwrap(), 2);
        assert_eq!(bandwidth(&k4()).unwrap(), 3);
    }
}
