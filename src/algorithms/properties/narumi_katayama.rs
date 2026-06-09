//! Narumi-Katayama index and multiplicative variants (ALGO-TR-052).
//!
//! - **Narumi-Katayama index** `NK(G) = Π_{v∈V, d(v)≥1} d(v)`
//!   Product of all non-zero degrees. Introduced by Narumi & Katayama (1984).
//! - **First multiplicative Zagreb index** `Π₁(G) = Π_{v∈V} d(v)²`
//!   Introduced by Todeschini & Consonni (2010).
//! - **Second multiplicative Zagreb index** `Π₂(G) = Π_{(u,v)∈E} d(u)·d(v)`
//!   Introduced by Todeschini & Consonni (2010).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the Narumi-Katayama index.
///
/// `NK(G) = Π_{v∈V, d(v)≥1} d(v)`
///
/// The product of all non-zero vertex degrees. Isolated vertices
/// (degree 0) are excluded from the product. Returns 1.0 for the
/// empty graph or an edgeless graph (empty product).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, narumi_katayama_index};
///
/// // K_3: degrees [2,2,2] → NK = 2·2·2 = 8
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((narumi_katayama_index(&g).unwrap() - 8.0).abs() < 1e-10);
/// ```
pub fn narumi_katayama_index(graph: &Graph) -> IgraphResult<f64> {
    let mut nk = 1.0_f64;

    for v in 0..graph.vcount() {
        let d = graph.degree(v)?;
        if d > 0 {
            nk *= d as f64;
        }
    }

    Ok(nk)
}

/// Compute the first multiplicative Zagreb index.
///
/// `Π₁(G) = Π_{v∈V, d(v)≥1} d(v)²`
///
/// The product of squared degrees over vertices with non-zero degree.
/// Returns 1.0 for the empty graph or edgeless graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_multiplicative_zagreb};
///
/// // K_3: degrees [2,2,2] → Π₁ = 4·4·4 = 64
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((first_multiplicative_zagreb(&g).unwrap() - 64.0).abs() < 1e-10);
/// ```
pub fn first_multiplicative_zagreb(graph: &Graph) -> IgraphResult<f64> {
    let mut pi1 = 1.0_f64;

    for v in 0..graph.vcount() {
        let d = graph.degree(v)?;
        if d > 0 {
            let df = d as f64;
            pi1 *= df * df;
        }
    }

    Ok(pi1)
}

/// Compute the second multiplicative Zagreb index.
///
/// `Π₂(G) = Π_{(u,v)∈E} d(u) · d(v)`
///
/// The product of degree products over all edges. Self-loops are
/// skipped. Returns 1.0 if there are no (non-loop) edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_multiplicative_zagreb};
///
/// // K_3: 3 edges, each (2·2)=4 → Π₂ = 4³ = 64
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((second_multiplicative_zagreb(&g).unwrap() - 64.0).abs() < 1e-10);
/// ```
pub fn second_multiplicative_zagreb(graph: &Graph) -> IgraphResult<f64> {
    let mut pi2 = 1.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        pi2 *= du * dv;
    }

    Ok(pi2)
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

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- narumi_katayama_index ---

    #[test]
    fn nk_empty() {
        let g = Graph::with_vertices(0);
        assert!((narumi_katayama_index(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn nk_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((narumi_katayama_index(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn nk_isolated_vertices() {
        let g = Graph::with_vertices(5);
        assert!((narumi_katayama_index(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn nk_single_edge() {
        // degrees [1,1] → NK = 1
        assert!((narumi_katayama_index(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn nk_path3() {
        // degrees [1,2,1] → NK = 1·2·1 = 2
        assert!((narumi_katayama_index(&path3()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn nk_path4() {
        // degrees [1,2,2,1] → NK = 1·2·2·1 = 4
        assert!((narumi_katayama_index(&path4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn nk_path5() {
        // degrees [1,2,2,2,1] → NK = 1·2·2·2·1 = 8
        assert!((narumi_katayama_index(&path5()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn nk_k3() {
        // degrees [2,2,2] → NK = 8
        assert!((narumi_katayama_index(&k3()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn nk_k4() {
        // degrees [3,3,3,3] → NK = 81
        assert!((narumi_katayama_index(&k4()).unwrap() - 81.0).abs() < 1e-10);
    }

    #[test]
    fn nk_cycle4() {
        // degrees [2,2,2,2] → NK = 16
        assert!((narumi_katayama_index(&cycle4()).unwrap() - 16.0).abs() < 1e-10);
    }

    #[test]
    fn nk_cycle5() {
        // degrees [2,2,2,2,2] → NK = 32
        assert!((narumi_katayama_index(&cycle5()).unwrap() - 32.0).abs() < 1e-10);
    }

    #[test]
    fn nk_star5() {
        // degrees [4,1,1,1,1] → NK = 4
        assert!((narumi_katayama_index(&star5()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn nk_paw() {
        // degrees [2,2,3,1] → NK = 2·2·3·1 = 12
        assert!((narumi_katayama_index(&paw()).unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn nk_regular_is_r_pow_n() {
        // r-regular graph: NK = r^n
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let n = f64::from(g.vcount());
            let r = g.degree(0).unwrap() as f64;
            let expected = r.powf(n);
            assert!((narumi_katayama_index(g).unwrap() - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn nk_with_isolated() {
        // 0-1 with isolated vertex 2
        let g = Graph::from_edges(&[(0, 1)], false, Some(3)).unwrap();
        // degrees [1,1,0] → NK = 1·1 = 1 (isolated vertex excluded)
        assert!((narumi_katayama_index(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    // --- first_multiplicative_zagreb ---

    #[test]
    fn pi1_empty() {
        let g = Graph::with_vertices(0);
        assert!((first_multiplicative_zagreb(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn pi1_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((first_multiplicative_zagreb(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn pi1_single_edge() {
        // degrees [1,1] → Π₁ = 1²·1² = 1
        assert!((first_multiplicative_zagreb(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn pi1_path3() {
        // degrees [1,2,1] → Π₁ = 1·4·1 = 4
        assert!((first_multiplicative_zagreb(&path3()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn pi1_path4() {
        // degrees [1,2,2,1] → Π₁ = 1·4·4·1 = 16
        assert!((first_multiplicative_zagreb(&path4()).unwrap() - 16.0).abs() < 1e-10);
    }

    #[test]
    fn pi1_k3() {
        // degrees [2,2,2] → Π₁ = 4³ = 64
        assert!((first_multiplicative_zagreb(&k3()).unwrap() - 64.0).abs() < 1e-10);
    }

    #[test]
    fn pi1_k4() {
        // degrees [3,3,3,3] → Π₁ = 9⁴ = 6561
        assert!((first_multiplicative_zagreb(&k4()).unwrap() - 6561.0).abs() < 1e-10);
    }

    #[test]
    fn pi1_cycle4() {
        // degrees [2,2,2,2] → Π₁ = 4⁴ = 256
        assert!((first_multiplicative_zagreb(&cycle4()).unwrap() - 256.0).abs() < 1e-10);
    }

    #[test]
    fn pi1_star5() {
        // degrees [4,1,1,1,1] → Π₁ = 16·1·1·1·1 = 16
        assert!((first_multiplicative_zagreb(&star5()).unwrap() - 16.0).abs() < 1e-10);
    }

    #[test]
    fn pi1_paw() {
        // degrees [2,2,3,1] → Π₁ = 4·4·9·1 = 144
        assert!((first_multiplicative_zagreb(&paw()).unwrap() - 144.0).abs() < 1e-10);
    }

    #[test]
    fn pi1_is_nk_squared() {
        // Π₁ = Π d(v)² = (Π d(v))² = NK² for graphs without isolated vertices
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let nk = narumi_katayama_index(g).unwrap();
            let pi1 = first_multiplicative_zagreb(g).unwrap();
            assert!((pi1 - nk * nk).abs() < 1e-6);
        }
    }

    #[test]
    fn pi1_regular_formula() {
        // r-regular: Π₁ = (r²)^n = r^(2n)
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let n = f64::from(g.vcount());
            let r = g.degree(0).unwrap() as f64;
            let expected = r.powf(2.0 * n);
            assert!((first_multiplicative_zagreb(g).unwrap() - expected).abs() < 1e-4);
        }
    }

    // --- second_multiplicative_zagreb ---

    #[test]
    fn pi2_empty() {
        let g = Graph::with_vertices(0);
        assert!((second_multiplicative_zagreb(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn pi2_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((second_multiplicative_zagreb(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn pi2_single_edge() {
        // (1·1) = 1 → Π₂ = 1
        assert!((second_multiplicative_zagreb(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn pi2_path3() {
        // (0,1): 1·2=2, (1,2): 2·1=2 → Π₂ = 4
        assert!((second_multiplicative_zagreb(&path3()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn pi2_path4() {
        // (0,1):1·2=2, (1,2):2·2=4, (2,3):2·1=2 → Π₂ = 2·4·2 = 16
        assert!((second_multiplicative_zagreb(&path4()).unwrap() - 16.0).abs() < 1e-10);
    }

    #[test]
    fn pi2_k3() {
        // 3 edges, each 2·2=4 → Π₂ = 4³ = 64
        assert!((second_multiplicative_zagreb(&k3()).unwrap() - 64.0).abs() < 1e-10);
    }

    #[test]
    fn pi2_k4() {
        // 6 edges, each 3·3=9 → Π₂ = 9⁶ = 531441
        assert!((second_multiplicative_zagreb(&k4()).unwrap() - 531_441.0).abs() < 1e-6);
    }

    #[test]
    fn pi2_cycle4() {
        // 4 edges, each 2·2=4 → Π₂ = 4⁴ = 256
        assert!((second_multiplicative_zagreb(&cycle4()).unwrap() - 256.0).abs() < 1e-10);
    }

    #[test]
    fn pi2_cycle5() {
        // 5 edges, each 2·2=4 → Π₂ = 4⁵ = 1024
        assert!((second_multiplicative_zagreb(&cycle5()).unwrap() - 1024.0).abs() < 1e-10);
    }

    #[test]
    fn pi2_star5() {
        // 4 edges, each 4·1=4 → Π₂ = 4⁴ = 256
        assert!((second_multiplicative_zagreb(&star5()).unwrap() - 256.0).abs() < 1e-10);
    }

    #[test]
    fn pi2_paw() {
        // degrees [2,2,3,1]
        // (0,1):4, (0,2):6, (1,2):6, (2,3):3 → Π₂ = 4·6·6·3 = 432
        assert!((second_multiplicative_zagreb(&paw()).unwrap() - 432.0).abs() < 1e-10);
    }

    #[test]
    fn pi2_regular_formula() {
        // r-regular: Π₂ = (r²)^m = r^(2m)
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = r.powf(2.0 * m);
            assert!((second_multiplicative_zagreb(g).unwrap() - expected).abs() < 1e-4);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn all_positive() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(narumi_katayama_index(g).unwrap() > 0.0);
            assert!(first_multiplicative_zagreb(g).unwrap() > 0.0);
            assert!(second_multiplicative_zagreb(g).unwrap() > 0.0);
        }
    }

    #[test]
    fn nk_leq_pi1() {
        // NK = Π d(v) ≤ Π d(v)² = Π₁ for d(v)≥1
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let nk = narumi_katayama_index(g).unwrap();
            let pi1 = first_multiplicative_zagreb(g).unwrap();
            assert!(nk <= pi1 + 1e-8);
        }
    }

    #[test]
    fn log_pi2_equals_sum_log_deg_products() {
        // ln(Π₂) = Σ_{(u,v)∈E} ln(d_u·d_v) = second Zagreb using logs
        for g in &[path3(), k3(), cycle4(), star5(), paw()] {
            let pi2 = second_multiplicative_zagreb(g).unwrap();
            let mut log_sum = 0.0_f64;
            for (u, v) in g.edges() {
                if u == v {
                    continue;
                }
                let du = g.degree(u).unwrap() as f64;
                let dv = g.degree(v).unwrap() as f64;
                log_sum += (du * dv).ln();
            }
            assert!((pi2.ln() - log_sum).abs() < 1e-8);
        }
    }
}
