//! Modularity-based ratio indices (ALGO-TR-117).
//!
//! Measures based on community structure and modularity concepts:
//!
//! - **Modularity upper bound ratio** — actual modularity of a greedy
//!   partition / theoretical maximum modularity
//! - **Community size balance** — entropy of community size distribution
//!   normalized by log(k) where k = number of communities
//! - **Inter-community edge ratio** — fraction of edges that connect
//!   vertices in different communities

#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the modularity upper bound ratio.
///
/// Runs a greedy label-propagation-style partition (assign each vertex
/// to the community of its most frequent neighbor label), then computes
/// modularity Q of that partition divided by the theoretical maximum
/// (1 - 1/k where k is the number of communities found). Values near 1
/// indicate the graph is highly modular; values near 0 indicate weak
/// community structure. Returns 0.0 for trivial or edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, modularity_upper_bound_ratio};
///
/// // Two disconnected K_2s: perfect community structure
/// let g = Graph::from_edges(&[(0,1),(2,3)], false, Some(4)).unwrap();
/// let r = modularity_upper_bound_ratio(&g).unwrap();
/// assert!(r > 0.5);
/// ```
pub fn modularity_upper_bound_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    if n < 2 || m == 0 {
        return Ok(0.0);
    }

    let membership = greedy_communities(graph, n)?;
    let q = compute_modularity(graph, n, m, &membership)?;

    let k = *membership.iter().max().unwrap_or(&0) + 1;
    if k <= 1 {
        return Ok(0.0);
    }

    let q_max = 1.0 - 1.0 / k as f64;
    if q_max < 1e-30 {
        return Ok(0.0);
    }

    Ok((q / q_max).clamp(0.0, 1.0))
}

/// Compute the community size balance.
///
/// Entropy of the community size distribution (from greedy partition)
/// normalized by log(k). Values near 1 indicate balanced community
/// sizes; values near 0 indicate one dominant community. Returns 0.0
/// for trivial graphs or when only one community exists.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, community_size_balance};
///
/// // Two disconnected K_2s: 2 communities of equal size → balance = 1.0
/// let g = Graph::from_edges(&[(0,1),(2,3)], false, Some(4)).unwrap();
/// let r = community_size_balance(&g).unwrap();
/// assert!((r - 1.0).abs() < 0.1);
/// ```
pub fn community_size_balance(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let membership = greedy_communities(graph, n)?;
    let k = *membership.iter().max().unwrap_or(&0) + 1;
    if k <= 1 {
        return Ok(0.0);
    }

    let mut sizes = vec![0_u64; k];
    for &c in &membership {
        sizes[c] += 1;
    }

    let n_f = n as f64;
    let mut entropy = 0.0_f64;
    for &s in &sizes {
        if s > 0 {
            let p = s as f64 / n_f;
            entropy -= p * p.ln();
        }
    }

    let max_entropy = (k as f64).ln();
    if max_entropy < 1e-30 {
        return Ok(0.0);
    }

    Ok(entropy / max_entropy)
}

/// Compute the inter-community edge ratio.
///
/// Fraction of edges whose endpoints belong to different communities
/// (using greedy partition). Values near 0 indicate strong community
/// structure (few inter-community edges); values near 1 indicate weak
/// or no community structure. Returns 0.0 for trivial or edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, inter_community_edge_ratio};
///
/// // Two disconnected K_2s: no inter-community edges → 0.0
/// let g = Graph::from_edges(&[(0,1),(2,3)], false, Some(4)).unwrap();
/// assert!(inter_community_edge_ratio(&g).unwrap() < 0.01);
/// ```
pub fn inter_community_edge_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    if n < 2 || m == 0 {
        return Ok(0.0);
    }

    let membership = greedy_communities(graph, n)?;

    let mut inter_edges = 0_u64;
    for v in 0..n {
        let nbrs = graph.neighbors(v as u32)?;
        for &u in &nbrs {
            let ui = u as usize;
            if ui > v && membership[v] != membership[ui] {
                inter_edges += 1;
            }
        }
    }

    Ok(inter_edges as f64 / m as f64)
}

/// Greedy community detection via label propagation (single pass).
/// Each vertex starts in its own community; iteratively assigns each
/// vertex to the most frequent community among its neighbors.
fn greedy_communities(graph: &Graph, n: usize) -> IgraphResult<Vec<usize>> {
    let mut membership: Vec<usize> = (0..n).collect();

    for _ in 0..10 {
        let mut changed = false;
        for v in 0..n {
            let nbrs = graph.neighbors(v as u32)?;
            if nbrs.is_empty() {
                continue;
            }

            let mut freq: std::collections::HashMap<usize, usize> =
                std::collections::HashMap::new();
            for &u in &nbrs {
                *freq.entry(membership[u as usize]).or_insert(0) += 1;
            }

            let best_community = freq
                .into_iter()
                .max_by_key(|&(_, count)| count)
                .map_or(membership[v], |(comm, _)| comm);

            if best_community != membership[v] {
                membership[v] = best_community;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // Renumber communities to 0..k-1
    let mut mapping: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut next_id = 0_usize;
    for v in 0..n {
        let c = membership[v];
        let new_id = *mapping.entry(c).or_insert_with(|| {
            let id = next_id;
            next_id += 1;
            id
        });
        membership[v] = new_id;
    }

    Ok(membership)
}

/// Compute Newman-Girvan modularity Q for a given partition.
fn compute_modularity(
    graph: &Graph,
    n: usize,
    m: usize,
    membership: &[usize],
) -> IgraphResult<f64> {
    if m == 0 {
        return Ok(0.0);
    }

    let two_m = 2.0 * m as f64;
    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)? as f64);
    }

    let k = *membership.iter().max().unwrap_or(&0) + 1;
    let mut e_cc = vec![0.0_f64; k]; // edges within community c (counted once)
    let mut a_c = vec![0.0_f64; k]; // sum of degrees in community c

    for v in 0..n {
        let c = membership[v];
        a_c[c] += degrees[v];
        let nbrs = graph.neighbors(v as u32)?;
        for &u in &nbrs {
            let ui = u as usize;
            if ui > v && membership[ui] == c {
                e_cc[c] += 1.0;
            }
        }
    }

    let mut q = 0.0_f64;
    for c in 0..k {
        q += e_cc[c] / (m as f64) - (a_c[c] / two_m).powi(2);
    }

    Ok(q)
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

    fn single_edge() -> Graph {
        Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap()
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

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn two_triangles() -> Graph {
        // Two K_3 connected by one edge: clear community structure
        Graph::from_edges(
            &[(0, 1), (0, 2), (1, 2), (2, 3), (3, 4), (3, 5), (4, 5)],
            false,
            Some(6),
        )
        .unwrap()
    }

    fn disconnected_k2s() -> Graph {
        Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap()
    }

    // --- modularity_upper_bound_ratio ---

    #[test]
    fn mubr_empty() {
        assert!(modularity_upper_bound_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mubr_single() {
        assert!(modularity_upper_bound_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mubr_in_01() {
        for g in &[
            single_edge(),
            k3(),
            k4(),
            cycle4(),
            star5(),
            two_triangles(),
        ] {
            let r = modularity_upper_bound_ratio(g).unwrap();
            assert!(r >= -0.01);
            assert!(r <= 1.01);
        }
    }

    #[test]
    fn mubr_disconnected_high() {
        // Disconnected graphs should have high modularity
        let r = modularity_upper_bound_ratio(&disconnected_k2s()).unwrap();
        assert!(r > 0.5);
    }

    #[test]
    fn mubr_finite() {
        for g in &[
            single_edge(),
            k3(),
            k4(),
            cycle4(),
            star5(),
            two_triangles(),
        ] {
            assert!(modularity_upper_bound_ratio(g).unwrap().is_finite());
        }
    }

    // --- community_size_balance ---

    #[test]
    fn csb_empty() {
        assert!(community_size_balance(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn csb_single() {
        assert!(community_size_balance(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn csb_disconnected() {
        // Two K_2s → 2 communities of size 2 → perfect balance
        let r = community_size_balance(&disconnected_k2s()).unwrap();
        assert!((r - 1.0).abs() < 0.1);
    }

    #[test]
    fn csb_in_01() {
        for g in &[
            single_edge(),
            k3(),
            k4(),
            cycle4(),
            star5(),
            two_triangles(),
        ] {
            let r = community_size_balance(g).unwrap();
            assert!(r >= -0.01);
            assert!(r <= 1.01);
        }
    }

    #[test]
    fn csb_finite() {
        for g in &[single_edge(), k3(), k4(), cycle4(), star5()] {
            assert!(community_size_balance(g).unwrap().is_finite());
        }
    }

    // --- inter_community_edge_ratio ---

    #[test]
    fn icer_empty() {
        assert!(inter_community_edge_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn icer_single() {
        assert!(inter_community_edge_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn icer_disconnected() {
        // Two K_2s → 0 inter-community edges
        assert!(
            inter_community_edge_ratio(&disconnected_k2s())
                .unwrap()
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn icer_in_01() {
        for g in &[
            single_edge(),
            k3(),
            k4(),
            cycle4(),
            star5(),
            two_triangles(),
        ] {
            let r = inter_community_edge_ratio(g).unwrap();
            assert!(r >= -0.01);
            assert!(r <= 1.01);
        }
    }

    #[test]
    fn icer_finite() {
        for g in &[single_edge(), k3(), k4(), cycle4(), star5()] {
            assert!(inter_community_edge_ratio(g).unwrap().is_finite());
        }
    }

    // --- cross-consistency ---

    #[test]
    fn disconnected_strong_community() {
        let g = disconnected_k2s();
        // Should have high modularity, balanced sizes, zero inter edges
        assert!(modularity_upper_bound_ratio(&g).unwrap() > 0.5);
        assert!(community_size_balance(&g).unwrap() > 0.8);
        assert!(inter_community_edge_ratio(&g).unwrap() < 0.01);
    }

    #[test]
    fn complete_weak_community() {
        // K_4 has no clear community structure
        let r = inter_community_edge_ratio(&k4()).unwrap();
        // Complete graph in one community → 0 inter edges
        // Or if split, most edges are inter → depends on algorithm
        assert!(r.is_finite());
    }
}
