//! Degree inequality indices (ALGO-TR-091).
//!
//! Inequality and concentration measures over the degree sequence:
//!
//! - **Herfindahl index** `Σ (d(v)/Σd)²` — concentration/monopoly measure
//! - **Theil index** `(1/n) Σ (d/d̄)·ln(d/d̄)` — generalized entropy inequality
//! - **Palma ratio** top 10% share / bottom 40% share of total degree
//! - **Hoover index** `Σ|d(v) - d̄| / (2·Σd)` — Robin Hood index

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the Herfindahl–Hirschman index of the degree sequence.
///
/// `H = Σ_v (d(v) / Σ_u d(u))²`
///
/// Measures concentration: 1/n for a regular graph (perfect equality),
/// 1.0 for a star (one vertex holds all degree mass). Returns 0.0
/// for the empty or edgeless graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_herfindahl};
///
/// // K_3: all degrees 2, total 6 → 3·(2/6)² = 3·(1/9) = 1/3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((degree_herfindahl(&g).unwrap() - 1.0/3.0).abs() < 1e-10);
/// ```
pub fn degree_herfindahl(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut total = 0_u64;
    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        degrees.push(d);
        total = total.saturating_add(d as u64);
    }

    if total == 0 {
        return Ok(0.0);
    }

    let total_f = total as f64;
    let mut hhi = 0.0_f64;
    for &d in &degrees {
        let share = d as f64 / total_f;
        hhi += share * share;
    }

    Ok(hhi)
}

/// Compute the Theil index (GE(1)) of the degree sequence.
///
/// `T = (1/n) Σ_{v: d(v)>0} (d(v)/d̄) · ln(d(v)/d̄)`
///
/// A generalized entropy inequality measure. Zero for regular graphs,
/// higher values indicate more inequality. Vertices with d=0 contribute
/// 0 (by L'Hôpital: x·ln(x) → 0 as x → 0+). Returns 0.0 for the
/// empty or edgeless graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_theil};
///
/// // K_3: all degrees equal → Theil = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_theil(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_theil(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut total = 0_u64;
    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        degrees.push(d);
        total = total.saturating_add(d as u64);
    }

    if total == 0 {
        return Ok(0.0);
    }

    let mean = total as f64 / n as f64;
    let mut theil = 0.0_f64;
    for &d in &degrees {
        if d == 0 {
            continue;
        }
        let ratio = d as f64 / mean;
        theil += ratio * ratio.ln();
    }

    Ok(theil / n as f64)
}

/// Compute the Palma ratio of the degree sequence.
///
/// Ratio of total degree held by the top 10% of vertices to the
/// total degree held by the bottom 40%, where vertices are ranked
/// by degree in ascending order.
///
/// Returns `f64::INFINITY` if the bottom 40% hold zero degree.
/// Returns 0.0 for the empty graph or graphs with fewer than 2
/// vertices (undefined).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_palma};
///
/// // K_3: all degrees equal → top 10% ≈ bottom 40% ratio → 1/n ratios
/// // With 3 vertices: bottom 40% = 1 vertex (d=2), top 10% = 0 vertices → 0/2 = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let p = degree_palma(&g).unwrap();
/// assert!(p.abs() < 1e-10); // top 10% of 3 = 0 vertices → 0
/// ```
pub fn degree_palma(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }
    degrees.sort_unstable();

    let bottom_count = n * 40 / 100;
    let top_start = n.saturating_sub(n * 10 / 100);

    let bottom_sum: u64 = degrees[..bottom_count].iter().map(|&d| d as u64).sum();
    let top_sum: u64 = degrees[top_start..].iter().map(|&d| d as u64).sum();

    if bottom_sum == 0 {
        if top_sum == 0 {
            return Ok(0.0);
        }
        return Ok(f64::INFINITY);
    }

    Ok(top_sum as f64 / bottom_sum as f64)
}

/// Compute the Hoover index (Robin Hood index) of the degree sequence.
///
/// `H = Σ|d(v) - d̄| / (2 · Σd)`
///
/// The maximum fraction of total degree that would need to be
/// redistributed to achieve perfect equality. Ranges from 0
/// (regular graph) to (n-1)/n (star). Returns 0.0 for the empty
/// or edgeless graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_hoover};
///
/// // K_3: all degrees 2, mean=2 → all |d-d̄|=0 → Hoover=0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_hoover(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_hoover(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut total = 0_u64;
    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        degrees.push(d);
        total = total.saturating_add(d as u64);
    }

    if total == 0 {
        return Ok(0.0);
    }

    let mean = total as f64 / n as f64;
    let mut abs_dev_sum = 0.0_f64;
    for &d in &degrees {
        abs_dev_sum += (d as f64 - mean).abs();
    }

    Ok(abs_dev_sum / (2.0 * total as f64))
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

    // --- degree_herfindahl ---

    #[test]
    fn hhi_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_herfindahl(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hhi_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_herfindahl(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hhi_regular() {
        // Regular graph: H = n·(1/n)² = 1/n
        assert!((degree_herfindahl(&k3()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
        assert!((degree_herfindahl(&k4()).unwrap() - 1.0 / 4.0).abs() < 1e-10);
        assert!((degree_herfindahl(&cycle4()).unwrap() - 1.0 / 4.0).abs() < 1e-10);
    }

    #[test]
    fn hhi_single_edge() {
        // (1,1): total=2, each share=0.5 → 2·0.25 = 0.5
        assert!((degree_herfindahl(&single_edge()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn hhi_star5() {
        // degrees: [4,1,1,1,1], total=8
        // (4/8)² + 4·(1/8)² = 0.25 + 4·(1/64) = 0.25 + 0.0625 = 0.3125
        assert!((degree_herfindahl(&star5()).unwrap() - 0.3125).abs() < 1e-10);
    }

    #[test]
    fn hhi_path3() {
        // degrees: [1,2,1], total=4
        // (1/4)² + (2/4)² + (1/4)² = 1/16 + 4/16 + 1/16 = 6/16 = 3/8
        assert!((degree_herfindahl(&path3()).unwrap() - 3.0 / 8.0).abs() < 1e-10);
    }

    #[test]
    fn hhi_paw() {
        // degrees: [2,2,3,1], total=8
        // (2/8)² + (2/8)² + (3/8)² + (1/8)² = 4/64+4/64+9/64+1/64 = 18/64 = 9/32
        assert!((degree_herfindahl(&paw()).unwrap() - 9.0 / 32.0).abs() < 1e-10);
    }

    // --- degree_theil ---

    #[test]
    fn theil_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_theil(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn theil_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_theil(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn theil_regular() {
        // All degrees equal → ratio=1 → ln(1)=0 → T=0
        assert!(degree_theil(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_theil(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_theil(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn theil_single_edge() {
        // Degrees [1,1] → regular → 0
        assert!(degree_theil(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn theil_star5() {
        // degrees [4,1,1,1,1], mean=8/5=1.6
        // v0: (4/1.6)·ln(4/1.6) = 2.5·ln(2.5)
        // 4 leaves: (1/1.6)·ln(1/1.6) = 0.625·ln(0.625)
        // T = (2.5·ln(2.5) + 4·0.625·ln(0.625)) / 5
        let mean = 1.6_f64;
        let r0 = 4.0 / mean;
        let r1 = 1.0 / mean;
        let expected = (r0 * r0.ln() + 4.0 * r1 * r1.ln()) / 5.0;
        assert!((degree_theil(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn theil_path3() {
        // degrees [1,2,1], mean=4/3
        // v0: (1/(4/3))·ln(1/(4/3)) = 0.75·ln(0.75)
        // v1: (2/(4/3))·ln(2/(4/3)) = 1.5·ln(1.5)
        // v2: same as v0
        // T = (2·0.75·ln(0.75) + 1.5·ln(1.5)) / 3
        let mean: f64 = 4.0 / 3.0;
        let r_end: f64 = 1.0 / mean;
        let r_mid: f64 = 2.0 / mean;
        let expected = (2.0 * r_end * r_end.ln() + r_mid * r_mid.ln()) / 3.0;
        assert!((degree_theil(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn theil_paw() {
        // degrees [2,2,3,1], mean=8/4=2
        let mean = 2.0_f64;
        let vals: [f64; 4] = [2.0, 2.0, 3.0, 1.0];
        let mut sum = 0.0_f64;
        for &d in &vals {
            let r: f64 = d / mean;
            sum += r * r.ln();
        }
        let expected = sum / 4.0;
        assert!((degree_theil(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- degree_palma ---

    #[test]
    fn palma_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_palma(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn palma_single() {
        let g = Graph::with_vertices(1);
        assert!(degree_palma(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn palma_single_edge() {
        // 2 vertices, bottom 40% = 0 vertices, top 10% = 0 vertices → 0/0 → 0.0
        let p = degree_palma(&single_edge()).unwrap();
        assert!(p.abs() < 1e-10);
    }

    #[test]
    fn palma_k3() {
        // 3 vertices: bottom_count=3*40/100=1, top_start=3-3*10/100=3-0=3
        // bottom=[2], top=[] → 0/2 = 0
        assert!(degree_palma(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn palma_star5() {
        // 5 vertices sorted: [1,1,1,1,4]
        // bottom_count = 5*40/100 = 2 → bottom=[1,1], sum=2
        // top_start = 5 - 5*10/100 = 5-0 = 5 → top=[], sum=0
        // 0/2 = 0
        assert!(degree_palma(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn palma_10vertices() {
        // 10 vertices: bottom 40% = 4, top 10% = 1
        // Star S_10 (center+9 leaves): degrees sorted [1,1,1,1,1,1,1,1,1,9]
        // bottom 4: [1,1,1,1]=4, top 1: [9]=9
        // Palma = 9/4 = 2.25
        let mut edges = Vec::new();
        for i in 1..10_u32 {
            edges.push((0, i));
        }
        let g = Graph::from_edges(&edges, false, Some(10)).unwrap();
        assert!((degree_palma(&g).unwrap() - 2.25).abs() < 1e-10);
    }

    #[test]
    fn palma_20vertices_regular() {
        // K_4 extended: 20-vertex cycle — all degrees 2
        // bottom 40% = 8 verts each d=2 → sum=16
        // top 10% = 2 verts each d=2 → sum=4
        // Palma = 4/16 = 0.25
        let mut edges = Vec::new();
        for i in 0..20_u32 {
            edges.push((i, (i + 1) % 20));
        }
        let g = Graph::from_edges(&edges, false, Some(20)).unwrap();
        assert!((degree_palma(&g).unwrap() - 0.25).abs() < 1e-10);
    }

    // --- degree_hoover ---

    #[test]
    fn hoover_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_hoover(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hoover_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_hoover(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hoover_regular() {
        // Regular: all d=d̄ → all |d-d̄|=0 → 0
        assert!(degree_hoover(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_hoover(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_hoover(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hoover_single_edge() {
        // [1,1] mean=1 → all |d-mean|=0 → 0
        assert!(degree_hoover(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hoover_star5() {
        // degrees [4,1,1,1,1], mean=8/5=1.6, total=8
        // |4-1.6|=2.4, 4·|1-1.6|=4·0.6=2.4
        // sum_abs_dev = 2.4 + 2.4 = 4.8
        // H = 4.8 / (2·8) = 4.8/16 = 0.3
        assert!((degree_hoover(&star5()).unwrap() - 0.3).abs() < 1e-10);
    }

    #[test]
    fn hoover_path3() {
        // degrees [1,2,1], mean=4/3, total=4
        // |1-4/3|=1/3, |2-4/3|=2/3, |1-4/3|=1/3
        // sum = 1/3 + 2/3 + 1/3 = 4/3
        // H = (4/3) / (2·4) = (4/3)/8 = 1/6
        assert!((degree_hoover(&path3()).unwrap() - 1.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn hoover_paw() {
        // degrees [2,2,3,1], mean=2, total=8
        // |2-2|=0, |2-2|=0, |3-2|=1, |1-2|=1
        // sum=2, H = 2/(2·8) = 2/16 = 1/8
        assert!((degree_hoover(&paw()).unwrap() - 0.125).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn herfindahl_bounds() {
        // H ∈ [1/n, 1] for graphs with edges
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let n = f64::from(g.vcount());
            let h = degree_herfindahl(g).unwrap();
            assert!(h >= 1.0 / n - 1e-10);
            assert!(h <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn theil_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_theil(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn hoover_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let h = degree_hoover(g).unwrap();
            assert!(h >= -1e-10);
            assert!(h <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn palma_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_palma(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn regular_all_zero_inequality() {
        for g in &[k3(), k4(), cycle4()] {
            assert!(degree_theil(g).unwrap().abs() < 1e-10);
            assert!(degree_hoover(g).unwrap().abs() < 1e-10);
        }
    }
}
