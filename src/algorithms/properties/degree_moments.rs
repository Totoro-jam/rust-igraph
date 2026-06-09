//! Degree distribution moments (ALGO-TR-082).
//!
//! Higher-order statistical moments and inequality measures of the
//! degree sequence, complementing `degree_cv` and `bell_index`:
//!
//! - **Degree skewness** `γ₁ = (1/n)Σ((d(v)-d̄)/σ)³`
//! - **Degree kurtosis** (excess) `γ₂ = (1/n)Σ((d(v)-d̄)/σ)⁴ - 3`
//! - **Degree Gini coefficient** — inequality of the degree sequence
//! - **Degree max-deviation** `Δ_max = max_v |d(v) - d̄|`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the skewness of the degree distribution.
///
/// `γ₁ = (1/n) Σ_v ((d(v) - d̄) / σ)³`
///
/// Measures asymmetry of the degree distribution. Positive skewness
/// means a right tail (few high-degree hubs), negative means left tail.
/// Returns 0.0 for graphs with fewer than 3 vertices or zero variance.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_skewness};
///
/// // K_3: all degrees equal → skewness = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_skewness(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_skewness(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let degrees = collect_degrees(graph)?;
    let mean = mean_of(&degrees);
    let variance = variance_of(&degrees, mean);

    if variance <= 0.0 {
        return Ok(0.0);
    }

    let sigma = variance.sqrt();
    let mut m3 = 0.0_f64;
    for &d in &degrees {
        let z = (d - mean) / sigma;
        m3 += z * z * z;
    }

    Ok(m3 / n as f64)
}

/// Compute the excess kurtosis of the degree distribution.
///
/// `γ₂ = (1/n) Σ_v ((d(v) - d̄) / σ)⁴ - 3`
///
/// Measures the "tailedness" of the degree distribution relative
/// to a normal distribution (excess kurtosis = 0 for normal).
/// Returns 0.0 for graphs with fewer than 3 vertices or zero variance.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_kurtosis};
///
/// // K_4: all degrees equal → kurtosis = -3 (minimal, degenerate)
/// // Actually: zero variance → returns 0.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4),
/// ).unwrap();
/// assert!(degree_kurtosis(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_kurtosis(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let degrees = collect_degrees(graph)?;
    let mean = mean_of(&degrees);
    let variance = variance_of(&degrees, mean);

    if variance <= 0.0 {
        return Ok(0.0);
    }

    let sigma = variance.sqrt();
    let mut m4 = 0.0_f64;
    for &d in &degrees {
        let z = (d - mean) / sigma;
        let z2 = z * z;
        m4 += z2 * z2;
    }

    Ok(m4 / n as f64 - 3.0)
}

/// Compute the Gini coefficient of the degree sequence.
///
/// `Gini = (Σ_i Σ_j |d_i - d_j|) / (2 n² d̄)`
///
/// Measures inequality: 0 = perfect equality (all degrees the same),
/// approaching 1 = maximal inequality (star-like).
/// Returns 0.0 for edgeless graphs (d̄ = 0) or single-vertex graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_gini};
///
/// // K_3: all degrees = 2 → Gini = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_gini(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_gini(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let degrees = collect_degrees(graph)?;
    let mean = mean_of(&degrees);

    if mean <= 0.0 {
        return Ok(0.0);
    }

    let mut sum_abs_diff = 0.0_f64;
    for i in 0..n {
        for j in 0..n {
            sum_abs_diff += (degrees[i] - degrees[j]).abs();
        }
    }

    let nf = n as f64;
    Ok(sum_abs_diff / (2.0 * nf * nf * mean))
}

/// Compute the maximum degree deviation from the mean.
///
/// `Δ_max = max_v |d(v) - d̄|`
///
/// The largest absolute deviation of any vertex's degree from the
/// mean degree. Returns 0.0 for empty or edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_max_deviation};
///
/// // Star S_5: center d=4, leaves d=1, d̄=1.6 → max|4-1.6|=2.4
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert!((degree_max_deviation(&g).unwrap() - 2.4).abs() < 1e-10);
/// ```
pub fn degree_max_deviation(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let degrees = collect_degrees(graph)?;
    let mean = mean_of(&degrees);

    let mut max_dev = 0.0_f64;
    for &d in &degrees {
        let dev = (d - mean).abs();
        if dev > max_dev {
            max_dev = dev;
        }
    }

    Ok(max_dev)
}

fn collect_degrees(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount() as usize;
    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)? as f64);
    }
    Ok(degrees)
}

fn mean_of(vals: &[f64]) -> f64 {
    if vals.is_empty() {
        return 0.0;
    }
    vals.iter().sum::<f64>() / vals.len() as f64
}

fn variance_of(vals: &[f64], mean: f64) -> f64 {
    if vals.is_empty() {
        return 0.0;
    }
    let n = vals.len() as f64;
    vals.iter().map(|&x| (x - mean) * (x - mean)).sum::<f64>() / n
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

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- degree_skewness ---

    #[test]
    fn skew_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_skewness(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn skew_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_skewness(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn skew_regular_zero() {
        assert!(degree_skewness(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_skewness(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_skewness(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn skew_single_edge() {
        assert!(degree_skewness(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn skew_star5_positive() {
        // Star: one high-degree hub → positive skewness
        assert!(degree_skewness(&star5()).unwrap() > 0.0);
    }

    #[test]
    fn skew_path3() {
        // degrees: [1,2,1], mean=4/3, σ²=(1-4/3)²+(2-4/3)²+(1-4/3)² / 3
        // = (1/9+4/9+1/9)/3 = (6/9)/3 = 2/9
        // σ = √(2/9)
        // z_0 = (1-4/3)/√(2/9) = (-1/3)/(√2/3) = -1/√2
        // z_1 = (2-4/3)/√(2/9) = (2/3)/(√2/3) = 2/√2 = √2
        // z_2 = z_0 = -1/√2
        // m3 = ((-1/√2)³ + (√2)³ + (-1/√2)³) / 3
        //    = (-1/(2√2) + 2√2 - 1/(2√2)) / 3
        //    = (-1/√2 + 2√2) / 3
        //    = (-√2/2 + 2√2) / 3
        //    = (3√2/2) / 3
        //    = √2/2
        let expected = std::f64::consts::SQRT_2 / 2.0;
        assert!((degree_skewness(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn skew_paw() {
        // degrees: [2,2,3,1], mean=2, σ²=(0+0+1+1)/4=0.5, σ=1/√2
        // z: [0,0,√2,-√2]
        // m3 = (0+0+2√2-2√2)/4 = 0
        assert!(degree_skewness(&paw()).unwrap().abs() < 1e-10);
    }

    // --- degree_kurtosis ---

    #[test]
    fn kurt_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_kurtosis(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn kurt_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_kurtosis(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn kurt_regular_zero() {
        assert!(degree_kurtosis(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_kurtosis(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_kurtosis(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn kurt_single_edge() {
        // degrees: [1,1], mean=1, σ=0 → 0
        assert!(degree_kurtosis(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn kurt_path3() {
        // z: [-1/√2, √2, -1/√2]
        // z⁴: [1/4, 4, 1/4]
        // m4 = (1/4+4+1/4)/3 = 4.5/3 = 1.5
        // excess kurtosis = 1.5 - 3 = -1.5
        assert!((degree_kurtosis(&path3()).unwrap() - (-1.5)).abs() < 1e-10);
    }

    #[test]
    fn kurt_paw() {
        // degrees: [2,2,3,1], mean=2, σ²=0.5
        // z: [0,0,√2,-√2]
        // z⁴: [0,0,4,4]
        // m4 = 8/4 = 2
        // excess = 2 - 3 = -1
        assert!((degree_kurtosis(&paw()).unwrap() - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn kurt_star5() {
        // degrees: [4,1,1,1,1], mean=8/5=1.6
        // d-mean: [2.4,-0.6,-0.6,-0.6,-0.6]
        // σ² = (5.76+0.36+0.36+0.36+0.36)/5 = 7.2/5 = 1.44
        // σ = 1.2
        // z: [2,-.5,-.5,-.5,-.5]
        // z⁴: [16,.0625,.0625,.0625,.0625]
        // m4 = (16+4·0.0625)/5 = 16.25/5 = 3.25
        // excess = 3.25 - 3 = 0.25
        assert!((degree_kurtosis(&star5()).unwrap() - 0.25).abs() < 1e-10);
    }

    // --- degree_gini ---

    #[test]
    fn gini_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_gini(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gini_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_gini(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gini_regular_zero() {
        assert!(degree_gini(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_gini(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_gini(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gini_single_edge() {
        assert!(degree_gini(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gini_star5() {
        // degrees: [4,1,1,1,1], mean=1.6
        // Σ|d_i-d_j|: center vs each leaf: 4·3=12 (×2 for symmetry=24)
        // leaf vs leaf: all 0
        // total = 24
        // Gini = 24 / (2·25·1.6) = 24/80 = 0.3
        assert!((degree_gini(&star5()).unwrap() - 0.3).abs() < 1e-10);
    }

    #[test]
    fn gini_path3() {
        // degrees: [1,2,1], mean=4/3
        // |d_i-d_j|:
        //   (0,0)=0 (0,1)=1 (0,2)=0
        //   (1,0)=1 (1,1)=0 (1,2)=1
        //   (2,0)=0 (2,1)=1 (2,2)=0
        // total = 4
        // Gini = 4 / (2·9·4/3) = 4/24 = 1/6
        let expected = 1.0 / 6.0;
        assert!((degree_gini(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn gini_paw() {
        // degrees: [2,2,3,1], mean=2
        // |d_i-d_j|:
        //   (0,1)=0 (0,2)=1 (0,3)=1 → row sum=2
        //   (1,0)=0 (1,2)=1 (1,3)=1 → 2
        //   (2,0)=1 (2,1)=1 (2,3)=2 → 4
        //   (3,0)=1 (3,1)=1 (3,2)=2 → 4
        // total = 12
        // Gini = 12 / (2·16·2) = 12/64 = 3/16
        let expected = 3.0 / 16.0;
        assert!((degree_gini(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn gini_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let val = degree_gini(g).unwrap();
            assert!(val >= -1e-10);
            assert!(val <= 1.0 + 1e-10);
        }
    }

    // --- degree_max_deviation ---

    #[test]
    fn maxdev_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_max_deviation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn maxdev_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_max_deviation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn maxdev_regular_zero() {
        assert!(degree_max_deviation(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_max_deviation(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_max_deviation(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn maxdev_single_edge() {
        assert!(degree_max_deviation(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn maxdev_star5() {
        // mean=1.6, max|4-1.6|=2.4
        assert!((degree_max_deviation(&star5()).unwrap() - 2.4).abs() < 1e-10);
    }

    #[test]
    fn maxdev_path3() {
        // mean=4/3, max|2-4/3|=2/3
        let expected = 2.0 / 3.0;
        assert!((degree_max_deviation(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn maxdev_paw() {
        // mean=2, max|3-2|=1
        assert!((degree_max_deviation(&paw()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn maxdev_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_max_deviation(g).unwrap() >= -1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn skew_kurt_consistent() {
        // For symmetric distributions, skewness = 0
        // Paw has symmetric degree dist around mean
        assert!(degree_skewness(&paw()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gini_zero_for_regular() {
        // All regular graphs: Gini = 0
        for g in &[k3(), k4(), cycle4()] {
            assert!(degree_gini(g).unwrap().abs() < 1e-10);
        }
    }
}
