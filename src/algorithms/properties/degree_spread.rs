//! Degree spread measures (ALGO-TR-083).
//!
//! Descriptive statistics of the degree sequence that capture how
//! spread-out or concentrated the vertex degrees are:
//!
//! - **Degree range** `R(G) = d_max − d_min`
//! - **Degree span ratio** `DSR(G) = (d_max − d_min) / d̄` (0 for edgeless)
//! - **Degree median** — the median of the degree sequence
//! - **Degree IQR** `IQR(G) = Q₃ − Q₁` (interquartile range)

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the range of the degree sequence.
///
/// `R(G) = d_max - d_min`
///
/// Returns 0 for empty or single-vertex graphs. For regular graphs,
/// the range is 0.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_range};
///
/// // Star S_5: d_max=4, d_min=1 → range=3
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert_eq!(degree_range(&g).unwrap(), 3);
/// ```
pub fn degree_range(graph: &Graph) -> IgraphResult<usize> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let mut d_min = usize::MAX;
    let mut d_max = 0_usize;
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d < d_min {
            d_min = d;
        }
        if d > d_max {
            d_max = d;
        }
    }

    Ok(d_max - d_min)
}

/// Compute the degree span ratio.
///
/// `DSR(G) = (d_max - d_min) / d̄`
///
/// Normalizes the degree range by the mean degree. Returns 0.0 for
/// edgeless or empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_span_ratio};
///
/// // K_3: all degrees equal → span ratio = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_span_ratio(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_span_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut d_min = usize::MAX;
    let mut d_max = 0_usize;
    let mut sum = 0_usize;
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d < d_min {
            d_min = d;
        }
        if d > d_max {
            d_max = d;
        }
        sum += d;
    }

    let mean = sum as f64 / n as f64;
    if mean <= 0.0 {
        return Ok(0.0);
    }

    Ok((d_max - d_min) as f64 / mean)
}

/// Compute the median of the degree sequence.
///
/// Returns the median degree. For even-sized sequences, returns the
/// average of the two middle values. Returns 0.0 for empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_median};
///
/// // Path P_3: degrees [1,2,1] sorted → [1,1,2], median = 1
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((degree_median(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn degree_median(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }
    degrees.sort_unstable();

    if n % 2 == 1 {
        Ok(degrees[n / 2] as f64)
    } else {
        Ok(f64::midpoint(
            degrees[n / 2 - 1] as f64,
            degrees[n / 2] as f64,
        ))
    }
}

/// Compute the interquartile range (IQR) of the degree sequence.
///
/// `IQR(G) = Q₃ - Q₁`
///
/// Uses linear interpolation for quartiles. Returns 0.0 for graphs
/// with fewer than 2 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_iqr};
///
/// // K_3: all degrees = 2 → IQR = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_iqr(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_iqr(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)? as f64);
    }
    degrees.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let q1 = percentile(&degrees, 0.25);
    let q3 = percentile(&degrees, 0.75);

    Ok(q3 - q1)
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return 0.0;
    }
    if n == 1 {
        return sorted[0];
    }

    let idx = p * (n - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;

    if lo == hi {
        sorted[lo]
    } else {
        let frac = idx - lo as f64;
        sorted[lo] * (1.0 - frac) + sorted[hi] * frac
    }
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

    // --- degree_range ---

    #[test]
    fn range_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(degree_range(&g).unwrap(), 0);
    }

    #[test]
    fn range_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(degree_range(&g).unwrap(), 0);
    }

    #[test]
    fn range_regular_zero() {
        assert_eq!(degree_range(&k3()).unwrap(), 0);
        assert_eq!(degree_range(&k4()).unwrap(), 0);
        assert_eq!(degree_range(&cycle4()).unwrap(), 0);
    }

    #[test]
    fn range_single_edge() {
        assert_eq!(degree_range(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn range_star5() {
        assert_eq!(degree_range(&star5()).unwrap(), 3);
    }

    #[test]
    fn range_path3() {
        assert_eq!(degree_range(&path3()).unwrap(), 1);
    }

    #[test]
    fn range_paw() {
        // degrees: 2,2,3,1 → range=3-1=2
        assert_eq!(degree_range(&paw()).unwrap(), 2);
    }

    // --- degree_span_ratio ---

    #[test]
    fn span_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_span_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn span_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_span_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn span_regular_zero() {
        assert!(degree_span_ratio(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_span_ratio(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_span_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn span_single_edge() {
        assert!(degree_span_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn span_star5() {
        // range=3, mean=1.6 → 3/1.6 = 1.875
        assert!((degree_span_ratio(&star5()).unwrap() - 1.875).abs() < 1e-10);
    }

    #[test]
    fn span_path3() {
        // range=1, mean=4/3 → 3/4 = 0.75
        assert!((degree_span_ratio(&path3()).unwrap() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn span_paw() {
        // range=2, mean=2 → 1.0
        assert!((degree_span_ratio(&paw()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn span_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_span_ratio(g).unwrap() >= -1e-10);
        }
    }

    // --- degree_median ---

    #[test]
    fn median_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_median(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn median_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_median(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn median_k3() {
        // all degree 2
        assert!((degree_median(&k3()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn median_k4() {
        // all degree 3
        assert!((degree_median(&k4()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn median_cycle4() {
        // all degree 2
        assert!((degree_median(&cycle4()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn median_single_edge() {
        // [1,1] → 1
        assert!((degree_median(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn median_star5() {
        // sorted: [1,1,1,1,4] → median=1
        assert!((degree_median(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn median_path3() {
        // sorted: [1,1,2] → median=1
        assert!((degree_median(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn median_paw() {
        // sorted: [1,2,2,3] → median=(2+2)/2=2
        assert!((degree_median(&paw()).unwrap() - 2.0).abs() < 1e-10);
    }

    // --- degree_iqr ---

    #[test]
    fn iqr_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_iqr(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn iqr_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_iqr(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn iqr_regular_zero() {
        assert!(degree_iqr(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_iqr(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_iqr(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn iqr_single_edge() {
        assert!(degree_iqr(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn iqr_star5() {
        // sorted: [1,1,1,1,4], n=5
        // Q1 at 0.25*(5-1)=1.0 → sorted[1]=1
        // Q3 at 0.75*(5-1)=3.0 → sorted[3]=1
        // IQR = 1-1 = 0
        assert!(degree_iqr(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn iqr_paw() {
        // sorted: [1,2,2,3], n=4
        // Q1 at 0.25*3=0.75 → 1*0.25 + 2*0.75 = 1.75
        // Q3 at 0.75*3=2.25 → 2*0.75 + 3*0.25 = 2.25
        // IQR = 2.25 - 1.75 = 0.5
        assert!((degree_iqr(&paw()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn iqr_path3() {
        // sorted: [1,1,2], n=3
        // Q1 at 0.25*2=0.5 → 1*0.5 + 1*0.5 = 1
        // Q3 at 0.75*2=1.5 → 1*0.5 + 2*0.5 = 1.5
        // IQR = 1.5 - 1.0 = 0.5
        assert!((degree_iqr(&path3()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn iqr_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_iqr(g).unwrap() >= -1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn range_le_max_degree() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let n = g.vcount() as usize;
            let mut d_max = 0_usize;
            for v in 0..n {
                let d = g.degree(v as u32).unwrap();
                if d > d_max {
                    d_max = d;
                }
            }
            assert!(degree_range(g).unwrap() <= d_max);
        }
    }

    #[test]
    fn median_between_min_max() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let n = g.vcount() as usize;
            let mut d_min = usize::MAX;
            let mut d_max = 0_usize;
            for v in 0..n {
                let d = g.degree(v as u32).unwrap();
                if d < d_min {
                    d_min = d;
                }
                if d > d_max {
                    d_max = d;
                }
            }
            let med = degree_median(g).unwrap();
            assert!(med >= d_min as f64 - 1e-10);
            assert!(med <= d_max as f64 + 1e-10);
        }
    }
}
