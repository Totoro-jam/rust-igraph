//! Entropy-based topological indices (ALGO-TR-076).
//!
//! Shannon entropy applied to the probability distributions induced by
//! classical topological indices. For a bond-additive index
//! `I(G) = Σ f(du,dv)`, define `p_e = f(du,dv) / I(G)`, then:
//!
//! `ENT_I(G) = -Σ p_e · ln(p_e)`
//!
//! - **First Zagreb entropy** weights by `(du + dv)`
//! - **Second Zagreb entropy** weights by `(du · dv)`
//! - **Randić entropy** weights by `1/√(du·dv)`
//! - **ABC entropy** weights by `√((du+dv-2)/(du·dv))`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

fn shannon_entropy(weights: &[f64]) -> f64 {
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        return 0.0;
    }
    let mut h = 0.0_f64;
    for &w in weights {
        if w > 0.0 {
            let p = w / total;
            h -= p * p.ln();
        }
    }
    h
}

/// Compute the first Zagreb entropy.
///
/// `ENT_M1(G) = -Σ_e p_e·ln(p_e)` where `p_e = (du+dv)/M1(G)`.
///
/// Returns 0.0 for edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_zagreb_entropy};
///
/// // K_3: all weights equal (4,4,4) → ln(3)
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((first_zagreb_entropy(&g).unwrap() - 3.0_f64.ln()).abs() < 1e-10);
/// ```
pub fn first_zagreb_entropy(graph: &Graph) -> IgraphResult<f64> {
    let mut weights = Vec::new();

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        weights.push(du + dv);
    }

    Ok(shannon_entropy(&weights))
}

/// Compute the second Zagreb entropy.
///
/// `ENT_M2(G) = -Σ_e p_e·ln(p_e)` where `p_e = (du·dv)/M2(G)`.
///
/// Returns 0.0 for edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_zagreb_entropy};
///
/// // K_3: all weights equal (4,4,4) → ln(3)
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((second_zagreb_entropy(&g).unwrap() - 3.0_f64.ln()).abs() < 1e-10);
/// ```
pub fn second_zagreb_entropy(graph: &Graph) -> IgraphResult<f64> {
    let mut weights = Vec::new();

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let product = du * dv;
        if product > 0.0 {
            weights.push(product);
        }
    }

    Ok(shannon_entropy(&weights))
}

/// Compute the Randić entropy.
///
/// `ENT_R(G) = -Σ_e p_e·ln(p_e)` where `p_e = (1/√(du·dv))/R(G)`.
///
/// Returns 0.0 for edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, randic_entropy};
///
/// // K_3: all weights equal → ln(3)
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((randic_entropy(&g).unwrap() - 3.0_f64.ln()).abs() < 1e-10);
/// ```
pub fn randic_entropy(graph: &Graph) -> IgraphResult<f64> {
    let mut weights = Vec::new();

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let product = du * dv;
        if product > 0.0 {
            weights.push(1.0 / product.sqrt());
        }
    }

    Ok(shannon_entropy(&weights))
}

/// Compute the ABC entropy.
///
/// `ENT_ABC(G) = -Σ_e p_e·ln(p_e)` where
/// `p_e = √((du+dv-2)/(du·dv)) / ABC(G)`.
///
/// Returns 0.0 for edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, abc_entropy};
///
/// // K_3: all weights equal → ln(3)
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((abc_entropy(&g).unwrap() - 3.0_f64.ln()).abs() < 1e-10);
/// ```
pub fn abc_entropy(graph: &Graph) -> IgraphResult<f64> {
    let mut weights = Vec::new();

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let product = du * dv;
        let numer = du + dv - 2.0;
        if product > 0.0 && numer > 0.0 {
            weights.push((numer / product).sqrt());
        }
    }

    Ok(shannon_entropy(&weights))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_edge() -> Graph {
        Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap()
    }

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

    fn cycle5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)], false, Some(5)).unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- first_zagreb_entropy ---

    #[test]
    fn fze_empty() {
        let g = Graph::with_vertices(0);
        assert!(first_zagreb_entropy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fze_isolated() {
        let g = Graph::with_vertices(5);
        assert!(first_zagreb_entropy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fze_single_edge() {
        // 1 edge → 1 weight → entropy 0 (ln(1)=0)
        assert!(first_zagreb_entropy(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fze_k3() {
        // 3 equal weights → ln(3)
        assert!((first_zagreb_entropy(&k3()).unwrap() - 3.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn fze_k4() {
        // 6 equal weights → ln(6)
        assert!((first_zagreb_entropy(&k4()).unwrap() - 6.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn fze_cycle4() {
        // 4 equal weights → ln(4)
        assert!((first_zagreb_entropy(&cycle4()).unwrap() - 4.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn fze_cycle5() {
        // 5 equal weights → ln(5)
        assert!((first_zagreb_entropy(&cycle5()).unwrap() - 5.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn fze_path3() {
        // 2 edges, d=(1,2): weights 3,3 → equal → ln(2)
        assert!((first_zagreb_entropy(&path3()).unwrap() - 2.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn fze_star5() {
        // 4 edges, d=(4,1): all weights 5 → ln(4)
        assert!((first_zagreb_entropy(&star5()).unwrap() - 4.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn fze_paw() {
        // (0,1)d=(2,2):4, (0,2)d=(2,3):5, (1,2)d=(2,3):5, (2,3)d=(3,1):4
        // total=18, p=4/18,5/18,5/18,4/18
        let total = 18.0_f64;
        let expected =
            -2.0 * (4.0 / total) * (4.0 / total).ln() - 2.0 * (5.0 / total) * (5.0 / total).ln();
        assert!((first_zagreb_entropy(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- second_zagreb_entropy ---

    #[test]
    fn sze_empty() {
        let g = Graph::with_vertices(0);
        assert!(second_zagreb_entropy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sze_single_edge() {
        assert!(second_zagreb_entropy(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sze_k3() {
        assert!((second_zagreb_entropy(&k3()).unwrap() - 3.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn sze_k4() {
        assert!((second_zagreb_entropy(&k4()).unwrap() - 6.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn sze_cycle4() {
        assert!((second_zagreb_entropy(&cycle4()).unwrap() - 4.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn sze_path3() {
        // d=(1,2): weights 2,2 → ln(2)
        assert!((second_zagreb_entropy(&path3()).unwrap() - 2.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn sze_star5() {
        // d=(4,1): weights all 4 → ln(4)
        assert!((second_zagreb_entropy(&star5()).unwrap() - 4.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn sze_paw() {
        // (0,1):4, (0,2):6, (1,2):6, (2,3):3
        let total = 19.0_f64;
        let expected = -(4.0 / total) * (4.0 / total).ln()
            - 2.0 * (6.0 / total) * (6.0 / total).ln()
            - (3.0 / total) * (3.0 / total).ln();
        assert!((second_zagreb_entropy(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- randic_entropy ---

    #[test]
    fn re_empty() {
        let g = Graph::with_vertices(0);
        assert!(randic_entropy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn re_single_edge() {
        assert!(randic_entropy(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn re_k3() {
        assert!((randic_entropy(&k3()).unwrap() - 3.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn re_k4() {
        assert!((randic_entropy(&k4()).unwrap() - 6.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn re_cycle4() {
        assert!((randic_entropy(&cycle4()).unwrap() - 4.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn re_path3() {
        // d=(1,2): weights 1/√2, 1/√2 → equal → ln(2)
        assert!((randic_entropy(&path3()).unwrap() - 2.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn re_star5() {
        // d=(4,1): all 1/2 → ln(4)
        assert!((randic_entropy(&star5()).unwrap() - 4.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn re_paw() {
        // (0,1):1/2, (0,2):1/√6, (1,2):1/√6, (2,3):1/√3
        let w0 = 0.5;
        let w1 = 1.0 / 6.0_f64.sqrt();
        let w3 = 1.0 / 3.0_f64.sqrt();
        let total = w0 + 2.0 * w1 + w3;
        let expected = -(w0 / total) * (w0 / total).ln()
            - 2.0 * (w1 / total) * (w1 / total).ln()
            - (w3 / total) * (w3 / total).ln();
        assert!((randic_entropy(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- abc_entropy ---

    #[test]
    fn abce_empty() {
        let g = Graph::with_vertices(0);
        assert!(abc_entropy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn abce_single_edge() {
        // d=(1,1): numer=0 → skipped → 0
        assert!(abc_entropy(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn abce_k3() {
        assert!((abc_entropy(&k3()).unwrap() - 3.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn abce_k4() {
        assert!((abc_entropy(&k4()).unwrap() - 6.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn abce_cycle4() {
        assert!((abc_entropy(&cycle4()).unwrap() - 4.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn abce_path3() {
        // d=(1,2): √(1/2) each → equal → ln(2)
        assert!((abc_entropy(&path3()).unwrap() - 2.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn abce_star5() {
        // d=(4,1): √(3/4) each → equal → ln(4)
        assert!((abc_entropy(&star5()).unwrap() - 4.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn abce_paw() {
        // (0,1):√(2/4)=1/√2, (0,2):√(3/6)=1/√2, (1,2):1/√2, (2,3):√(2/3)
        let w0 = 1.0 / 2.0_f64.sqrt();
        let w3 = (2.0_f64 / 3.0).sqrt();
        let total = 3.0 * w0 + w3;
        let expected = -3.0 * (w0 / total) * (w0 / total).ln() - (w3 / total) * (w3 / total).ln();
        assert!((abc_entropy(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn regular_all_equal_ln_m() {
        // Regular graphs: all edge weights equal → entropy = ln(m)
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let ln_m = (g.ecount() as f64).ln();
            assert!((first_zagreb_entropy(g).unwrap() - ln_m).abs() < 1e-10);
            assert!((second_zagreb_entropy(g).unwrap() - ln_m).abs() < 1e-10);
            assert!((randic_entropy(g).unwrap() - ln_m).abs() < 1e-10);
            assert!((abc_entropy(g).unwrap() - ln_m).abs() < 1e-10);
        }
    }

    #[test]
    fn entropy_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(first_zagreb_entropy(g).unwrap() >= -1e-10);
            assert!(second_zagreb_entropy(g).unwrap() >= -1e-10);
            assert!(randic_entropy(g).unwrap() >= -1e-10);
            assert!(abc_entropy(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn entropy_le_ln_m() {
        // Shannon entropy ≤ ln(m) always (maximum for uniform distribution)
        for g in &[path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let ln_m = (g.ecount() as f64).ln();
            assert!(first_zagreb_entropy(g).unwrap() <= ln_m + 1e-10);
            assert!(second_zagreb_entropy(g).unwrap() <= ln_m + 1e-10);
            assert!(randic_entropy(g).unwrap() <= ln_m + 1e-10);
        }
    }
}
