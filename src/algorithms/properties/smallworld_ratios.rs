//! Small-world ratio indices (ALGO-TR-108).
//!
//! Ratios capturing small-world properties:
//!
//! - **Small-world sigma** — `(C/C_rand) / (L/L_rand)` where C is
//!   clustering and L is average path length
//! - **Small-world omega** — `L_rand/L - C/C_lattice` difference measure
//! - **Clustering path ratio** — `C * n / L` normalized product
//! - **Navigability ratio** — `log(n) / L` how close to logarithmic
//!   path lengths

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the small-world sigma estimate.
///
/// `(C / C_rand) / (L / L_rand)` where:
/// - C is the global clustering coefficient
/// - L is the average shortest path length
/// - `C_rand = mean_degree / n` (expected clustering for Erdos-Renyi)
/// - `L_rand = ln(n) / ln(mean_degree)` (expected path length for ER)
///
/// Values > 1 suggest small-world structure. Returns 0.0 for
/// disconnected, trivial, or sparse graphs where the ratio is
/// undefined.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, smallworld_sigma};
///
/// // K_4: C=1, L=1, C_rand=3/4, L_rand=ln4/ln3
/// // sigma = (1/(3/4)) / (1/(ln4/ln3)) = (4/3) * (ln4/ln3) ≈ 1.59
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!(smallworld_sigma(&g).unwrap() > 1.0);
/// ```
pub fn smallworld_sigma(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 4 {
        return Ok(0.0);
    }

    let (cc, apl) = clustering_and_apl(graph)?;
    if apl < 1e-30 || cc < 1e-30 {
        return Ok(0.0);
    }

    let mean_deg = 2.0 * graph.ecount() as f64 / n as f64;
    if mean_deg < 1.0 + 1e-10 {
        return Ok(0.0);
    }

    let c_rand = mean_deg / n as f64;
    let l_rand = (n as f64).ln() / mean_deg.ln();

    if c_rand < 1e-30 || l_rand < 1e-30 {
        return Ok(0.0);
    }

    let gamma = cc / c_rand;
    let lambda = apl / l_rand;

    if lambda < 1e-30 {
        return Ok(0.0);
    }

    Ok(gamma / lambda)
}

/// Compute the small-world omega estimate.
///
/// `L_rand / L - C / C_lattice` where:
/// - `L_rand = ln(n) / ln(mean_degree)` (ER expected)
/// - `C_lattice = 3/4` (ring lattice approximation for k≥4)
///
/// Values near 0 suggest small-world; negative → lattice-like;
/// positive → random-like. Returns 0.0 for disconnected, trivial,
/// or sparse graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, smallworld_omega};
///
/// // K_4: highly clustered and short paths
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// let w = smallworld_omega(&g).unwrap();
/// assert!(w > -2.0 && w < 2.0);
/// ```
pub fn smallworld_omega(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 4 {
        return Ok(0.0);
    }

    let (cc, apl) = clustering_and_apl(graph)?;
    if apl < 1e-30 {
        return Ok(0.0);
    }

    let mean_deg = 2.0 * graph.ecount() as f64 / n as f64;
    if mean_deg < 1.0 + 1e-10 {
        return Ok(0.0);
    }

    let l_rand = (n as f64).ln() / mean_deg.ln();
    let c_lattice = 0.75_f64;

    let lambda_inv = l_rand / apl;
    let gamma_lat = cc / c_lattice;

    Ok(lambda_inv - gamma_lat)
}

/// Compute the clustering-path ratio.
///
/// `C * n / L` — a normalized product of clustering and inverse
/// path length. Higher values indicate stronger small-world
/// properties. Returns 0.0 for disconnected or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, clustering_path_ratio};
///
/// // K_4: C=1, L=1, n=4 → 4.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((clustering_path_ratio(&g).unwrap() - 4.0).abs() < 1e-10);
/// ```
pub fn clustering_path_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let (cc, apl) = clustering_and_apl(graph)?;
    if apl < 1e-30 {
        return Ok(0.0);
    }

    Ok(cc * n as f64 / apl)
}

/// Compute the navigability ratio.
///
/// `ln(n) / L` — how close the average path length is to the
/// theoretical minimum for a graph with logarithmic diameter.
/// Values near 1 suggest the graph is navigable like a random
/// graph. Returns 0.0 for disconnected or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, navigability_ratio};
///
/// // K_4: L=1, ln(4)≈1.386 → ratio≈1.386
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// let r = navigability_ratio(&g).unwrap();
/// assert!((r - 4.0_f64.ln()).abs() < 1e-10);
/// ```
pub fn navigability_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let (_, apl) = clustering_and_apl(graph)?;
    if apl < 1e-30 {
        return Ok(0.0);
    }

    Ok((n as f64).ln() / apl)
}

fn clustering_and_apl(graph: &Graph) -> IgraphResult<(f64, f64)> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok((0.0, 0.0));
    }

    let mut total_triplets = 0_u64;
    let mut closed_triplets = 0_u64;
    let mut total_dist = 0_u64;
    let mut dist_pairs = 0_u64;

    for v in 0..n {
        let neighbors = graph.neighbors(v as u32)?;
        let d = neighbors.len();

        if d >= 2 {
            let possible = (d * (d - 1)) / 2;
            total_triplets += possible as u64;

            for i in 0..d {
                for j in (i + 1)..d {
                    if graph.has_edge(neighbors[i], neighbors[j]) {
                        closed_triplets += 1;
                    }
                }
            }
        }

        let mut dist = vec![u32::MAX; n];
        dist[v] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(v);
        while let Some(curr) = queue.pop_front() {
            let cd = dist[curr];
            let nbrs = graph.neighbors(curr as u32)?;
            for &u in &nbrs {
                let ui = u as usize;
                if dist[ui] == u32::MAX {
                    dist[ui] = cd + 1;
                    queue.push_back(ui);
                }
            }
        }

        for u in (v + 1)..n {
            if dist[u] == u32::MAX {
                return Ok((0.0, 0.0));
            }
            total_dist += u64::from(dist[u]);
            dist_pairs += 1;
        }
    }

    let cc = if total_triplets > 0 {
        closed_triplets as f64 / total_triplets as f64
    } else {
        0.0
    };

    let apl = if dist_pairs > 0 {
        total_dist as f64 / dist_pairs as f64
    } else {
        0.0
    };

    Ok((cc, apl))
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

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- smallworld_sigma ---

    #[test]
    fn sigma_empty() {
        let g = Graph::with_vertices(0);
        assert!(smallworld_sigma(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sigma_small() {
        let g = Graph::with_vertices(3);
        assert!(smallworld_sigma(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sigma_k4() {
        let r = smallworld_sigma(&k4()).unwrap();
        assert!(r > 1.0);
    }

    #[test]
    fn sigma_cycle4() {
        // C=0 (no triangles), so sigma=0
        assert!(smallworld_sigma(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sigma_star5() {
        // C=0 (no triangles among leaves), so sigma=0
        assert!(smallworld_sigma(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sigma_paw() {
        let r = smallworld_sigma(&paw()).unwrap();
        assert!(r > 0.0);
    }

    #[test]
    fn sigma_disconnected() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!(smallworld_sigma(&g).unwrap().abs() < 1e-10);
    }

    // --- smallworld_omega ---

    #[test]
    fn omega_empty() {
        let g = Graph::with_vertices(0);
        assert!(smallworld_omega(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn omega_small() {
        let g = Graph::with_vertices(3);
        assert!(smallworld_omega(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn omega_k4() {
        let w = smallworld_omega(&k4()).unwrap();
        assert!(w > -2.0 && w < 2.0);
    }

    #[test]
    fn omega_cycle4() {
        let w = smallworld_omega(&cycle4()).unwrap();
        assert!(w.is_finite());
    }

    #[test]
    fn omega_disconnected() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!(smallworld_omega(&g).unwrap().abs() < 1e-10);
    }

    // --- clustering_path_ratio ---

    #[test]
    fn cpr_empty() {
        let g = Graph::with_vertices(0);
        assert!(clustering_path_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cpr_k3() {
        // C=1, L=1, n=3 → 3.0
        assert!((clustering_path_ratio(&k3()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn cpr_k4() {
        // C=1, L=1, n=4 → 4.0
        assert!((clustering_path_ratio(&k4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn cpr_cycle4() {
        // C=0, so ratio=0
        assert!(clustering_path_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cpr_star5() {
        // C=0, so ratio=0
        assert!(clustering_path_ratio(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cpr_path3() {
        // C=0 (no triangles), so ratio=0
        assert!(clustering_path_ratio(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cpr_disconnected() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!(clustering_path_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cpr_nonneg() {
        for g in &[path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(clustering_path_ratio(g).unwrap() >= -1e-10);
        }
    }

    // --- navigability_ratio ---

    #[test]
    fn nr_empty() {
        let g = Graph::with_vertices(0);
        assert!(navigability_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn nr_k3() {
        // L=1, ln(3)/1 = ln(3)
        assert!((navigability_ratio(&k3()).unwrap() - 3.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn nr_k4() {
        // L=1, ln(4)/1 = ln(4)
        assert!((navigability_ratio(&k4()).unwrap() - 4.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn nr_cycle4() {
        // L=(1+2+1+1+2+1)/(6 pairs)=8/6=4/3 ≈ wait no
        // Pairs: (0,1)=1,(0,2)=2,(0,3)=1,(1,2)=1,(1,3)=2,(2,3)=1 → sum=8, pairs=6 → L=4/3
        // ln(4)/(4/3) = 3*ln(4)/4
        let expected = 4.0_f64.ln() / (4.0 / 3.0);
        assert!((navigability_ratio(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn nr_disconnected() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!(navigability_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn nr_positive() {
        for g in &[path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(navigability_ratio(g).unwrap() > 0.0);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn complete_high_sigma() {
        assert!(smallworld_sigma(&k4()).unwrap() > 1.0);
    }

    #[test]
    fn triangle_free_zero_cpr() {
        assert!(clustering_path_ratio(&cycle4()).unwrap().abs() < 1e-10);
        assert!(clustering_path_ratio(&star5()).unwrap().abs() < 1e-10);
        assert!(clustering_path_ratio(&path3()).unwrap().abs() < 1e-10);
    }
}
