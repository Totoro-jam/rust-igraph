//! Information-theoretic ratio indices (ALGO-TR-115).
//!
//! Entropy-based measures of graph structure:
//!
//! - **Degree entropy ratio** — Shannon entropy of degree distribution
//!   normalized by log(n)
//! - **Edge distribution entropy** — entropy of the edge-endpoint degree
//!   distribution normalized by log(2m)
//! - **Structural information content** — log2 of the number of distinct
//!   degree classes, normalized by log2(n)

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

/// Compute the degree entropy ratio.
///
/// Shannon entropy of the degree distribution divided by log(n).
/// Measures how uniform the degree distribution is. Returns 1.0 for
/// regular graphs, values < 1 for heterogeneous degree distributions.
/// Returns 0.0 for trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_entropy_ratio};
///
/// // K_3: all degrees equal → H = log(1) = 0... wait, p=1 for one class
/// // Actually all vertices have same degree → 1 class → H=0, but ratio = 0/log(3)?
/// // Better: P(d=2) = 1 → H = -1*log(1) = 0 → ratio = 0
/// // For non-trivial: cycle has uniform degrees too
/// // Let's use star: center deg=4, leaves deg=1
/// let star = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// let r = degree_entropy_ratio(&star).unwrap();
/// assert!(r > 0.0 && r < 1.0);
/// ```
pub fn degree_entropy_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }

    // Count degree frequencies
    let max_deg = degrees.iter().copied().max().unwrap_or(0);
    let mut freq = vec![0_u64; max_deg + 1];
    for &d in &degrees {
        freq[d] += 1;
    }

    let n_f = n as f64;
    let mut entropy = 0.0_f64;
    for &f in &freq {
        if f > 0 {
            let p = f as f64 / n_f;
            entropy -= p * p.ln();
        }
    }

    let max_entropy = n_f.ln();
    if max_entropy < 1e-30 {
        return Ok(0.0);
    }

    Ok(entropy / max_entropy)
}

/// Compute the edge distribution entropy.
///
/// For each edge (u,v), consider the pair (d(u), d(v)) as a sample from
/// the joint degree distribution. Compute Shannon entropy of this
/// distribution normalized by log(2m). Measures how diverse edge
/// types are in terms of endpoint degrees. Returns 0.0 for edgeless
/// or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_distribution_entropy};
///
/// // K_3: all edges connect degree-2 vertices → 1 class → H=0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(edge_distribution_entropy(&g).unwrap().abs() < 1e-10);
/// ```
pub fn edge_distribution_entropy(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }

    // Count edge type frequencies: key = (min_deg, max_deg)
    let mut edge_types: std::collections::HashMap<(usize, usize), u64> =
        std::collections::HashMap::new();

    for v in 0..n {
        let nbrs = graph.neighbors(v as u32)?;
        for &u in &nbrs {
            let ui = u as usize;
            if ui > v {
                let d1 = degrees[v].min(degrees[ui]);
                let d2 = degrees[v].max(degrees[ui]);
                *edge_types.entry((d1, d2)).or_insert(0) += 1;
            }
        }
    }

    let m_f = m as f64;
    let mut entropy = 0.0_f64;
    for &count in edge_types.values() {
        if count > 0 {
            let p = count as f64 / m_f;
            entropy -= p * p.ln();
        }
    }

    let max_entropy = m_f.ln();
    if max_entropy < 1e-30 {
        return Ok(0.0);
    }

    Ok(entropy / max_entropy)
}

/// Compute the structural information content.
///
/// `log2(k) / log2(n)` where k is the number of distinct degree values
/// in the graph. Measures the structural diversity of the vertex roles
/// by degree. Returns 1.0 when every vertex has a unique degree (e.g.
/// path graphs for n≥3). Returns 0.0 for regular graphs or trivial
/// graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, structural_information_content};
///
/// // K_4: all degrees 3 → k=1 → log2(1)/log2(4) = 0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!(structural_information_content(&g).unwrap().abs() < 1e-10);
/// ```
pub fn structural_information_content(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }

    let mut seen = std::collections::HashSet::new();
    for &d in &degrees {
        seen.insert(d);
    }

    let k = seen.len();
    if k <= 1 {
        return Ok(0.0);
    }

    let log2_k = (k as f64).log2();
    let log2_n = (n as f64).log2();

    if log2_n < 1e-30 {
        return Ok(0.0);
    }

    Ok(log2_k / log2_n)
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

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- degree_entropy_ratio ---

    #[test]
    fn der_empty() {
        assert!(degree_entropy_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn der_single() {
        assert!(degree_entropy_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn der_k3() {
        // All same degree → H=0 → ratio=0
        assert!(degree_entropy_ratio(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn der_k4() {
        assert!(degree_entropy_ratio(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn der_cycle4() {
        assert!(degree_entropy_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn der_star5() {
        // Two degree classes: 4(1 vertex) and 1(4 vertices)
        // H = -(1/5)ln(1/5) - (4/5)ln(4/5)
        // max = ln(5)
        let r = degree_entropy_ratio(&star5()).unwrap();
        let expected = (-(1.0_f64 / 5.0) * (1.0_f64 / 5.0).ln()
            - (4.0_f64 / 5.0) * (4.0_f64 / 5.0).ln())
            / 5.0_f64.ln();
        assert!((r - expected).abs() < 1e-10);
    }

    #[test]
    fn der_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = degree_entropy_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- edge_distribution_entropy ---

    #[test]
    fn ede_empty() {
        assert!(edge_distribution_entropy(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ede_single() {
        assert!(edge_distribution_entropy(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ede_k3() {
        // All edges (2,2) → 1 class → H=0
        assert!(edge_distribution_entropy(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ede_k4() {
        // All edges (3,3) → 1 class → H=0
        assert!(edge_distribution_entropy(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ede_cycle4() {
        // All edges (2,2) → 1 class → H=0
        assert!(edge_distribution_entropy(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ede_star5() {
        // All edges (1,4) → 1 class → H=0
        assert!(edge_distribution_entropy(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ede_paw() {
        // Edges: (0,1)→(2,2), (0,2)→(2,3), (1,2)→(2,3), (2,3)→(3,1)=(1,3)
        // Types: (2,2)→1, (2,3)→2, (1,3)→1 → 3 classes, m=4
        let r = edge_distribution_entropy(&paw()).unwrap();
        assert!(r > 0.0);
        assert!(r <= 1.0 + 1e-10);
    }

    #[test]
    fn ede_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = edge_distribution_entropy(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- structural_information_content ---

    #[test]
    fn sic_empty() {
        assert!(structural_information_content(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sic_single() {
        assert!(structural_information_content(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sic_k3() {
        // 1 degree class → 0
        assert!(structural_information_content(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sic_k4() {
        assert!(structural_information_content(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sic_cycle4() {
        assert!(structural_information_content(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sic_star5() {
        // 2 degree classes (1 and 4) → log2(2)/log2(5) = 1/log2(5)
        let r = structural_information_content(&star5()).unwrap();
        let expected = 1.0 / 5.0_f64.log2();
        assert!((r - expected).abs() < 1e-10);
    }

    #[test]
    fn sic_path4() {
        // Degrees: 1,2,2,1 → 2 classes → log2(2)/log2(4) = 1/2 = 0.5
        let r = structural_information_content(&path4()).unwrap();
        assert!((r - 0.5).abs() < 1e-10);
    }

    #[test]
    fn sic_paw() {
        // Degrees: 2,2,3,1 → 3 classes → log2(3)/log2(4) = log2(3)/2
        let r = structural_information_content(&paw()).unwrap();
        let expected = 3.0_f64.log2() / 4.0_f64.log2();
        assert!((r - expected).abs() < 1e-10);
    }

    #[test]
    fn sic_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = structural_information_content(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn regular_zero_entropy() {
        // Regular graphs have 1 degree class → entropy = 0
        assert!(degree_entropy_ratio(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_entropy_ratio(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_entropy_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn regular_zero_sic() {
        assert!(structural_information_content(&k3()).unwrap().abs() < 1e-10);
        assert!(structural_information_content(&k4()).unwrap().abs() < 1e-10);
        assert!(structural_information_content(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn regular_zero_edge_entropy() {
        assert!(edge_distribution_entropy(&k3()).unwrap().abs() < 1e-10);
        assert!(edge_distribution_entropy(&k4()).unwrap().abs() < 1e-10);
        assert!(edge_distribution_entropy(&cycle4()).unwrap().abs() < 1e-10);
    }
}
