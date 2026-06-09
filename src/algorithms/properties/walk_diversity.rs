//! Walk diversity indices (ALGO-TR-095).
//!
//! Walk-based measures that capture connectivity patterns from
//! different perspectives:
//!
//! - **Walk entropy** — Shannon entropy of walk-count distribution
//! - **Walk regularity** — coefficient of variation of walk counts
//! - **Degree laplacian energy** — Σ |d(v) - 2m/n| normalized
//! - **Average neighbor connectivity** — mean of neighbor degree ratios

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the walk entropy of the graph.
///
/// `H = -Σ p(v)·ln(p(v))` where `p(v) = d(v) / (2m)`
///
/// Shannon entropy of the degree distribution treated as a
/// probability distribution over edge endpoints. Higher entropy
/// indicates more uniform degree distribution. Returns 0.0 for
/// graphs with no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, walk_entropy};
///
/// // K_3: d=2 for all, p=1/3 each → H = -3·(1/3)·ln(1/3) = ln(3)
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((walk_entropy(&g).unwrap() - 3.0_f64.ln()).abs() < 1e-10);
/// ```
pub fn walk_entropy(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let mut sum_d: u64 = 0;
    let mut degrees = Vec::with_capacity(n);

    for v in 0..n {
        let d = graph.degree(v as u32)?;
        degrees.push(d);
        sum_d = sum_d.saturating_add(d as u64);
    }

    if sum_d == 0 {
        return Ok(0.0);
    }

    let two_m: f64 = sum_d as f64;
    let mut h: f64 = 0.0;

    for &d in &degrees {
        if d > 0 {
            let p: f64 = d as f64 / two_m;
            h -= p * p.ln();
        }
    }

    Ok(h)
}

/// Compute the walk regularity index of the graph.
///
/// `WR = 1 - CV(d)` where `CV = σ/μ` is the coefficient of variation
/// of the degree sequence.
///
/// Equals 1.0 for regular graphs (all degrees equal) and decreases
/// toward 0 for highly irregular graphs. Returns 1.0 for graphs with
/// fewer than 2 vertices or no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, walk_regularity};
///
/// // K_3: regular → WR = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((walk_regularity(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn walk_regularity(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(1.0);
    }

    let mut sum_d: f64 = 0.0;
    let mut sum_d2: f64 = 0.0;

    for v in 0..n {
        let d: f64 = graph.degree(v as u32)? as f64;
        sum_d += d;
        sum_d2 += d * d;
    }

    if sum_d < 1e-15 {
        return Ok(1.0);
    }

    let mean: f64 = sum_d / n as f64;
    let variance: f64 = sum_d2 / n as f64 - mean * mean;

    if variance < 1e-15 {
        return Ok(1.0);
    }

    let cv: f64 = variance.sqrt() / mean;
    Ok((1.0 - cv).max(0.0))
}

/// Compute the degree Laplacian energy of the graph.
///
/// `DLE = Σ |d(v) - 2m/n| / (n · d_max)`
///
/// Normalized sum of absolute deviations of degrees from their mean.
/// Measures how far the degree sequence is from uniform. Zero for
/// regular graphs. Returns 0.0 for graphs with fewer than 2 vertices
/// or no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_laplacian_energy};
///
/// // K_3: d=2 for all, mean=2 → DLE = 0/... = 0.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_laplacian_energy(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_laplacian_energy(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let mut sum_d: f64 = 0.0;
    let mut d_max: f64 = 0.0;
    let mut degrees = Vec::with_capacity(n);

    for v in 0..n {
        let d: f64 = graph.degree(v as u32)? as f64;
        degrees.push(d);
        sum_d += d;
        if d > d_max {
            d_max = d;
        }
    }

    if sum_d < 1e-15 || d_max < 1e-15 {
        return Ok(0.0);
    }

    let mean: f64 = sum_d / n as f64;
    let mut sum_abs_dev: f64 = 0.0;

    for &d in &degrees {
        sum_abs_dev += (d - mean).abs();
    }

    Ok(sum_abs_dev / (n as f64 * d_max))
}

/// Compute the average neighbor connectivity of the graph.
///
/// `ANC = (1/n) Σ_v (Σ_{u∈N(v)} d(u)) / d(v)`
///
/// For each vertex with degree > 0, the ratio of total neighbor
/// degree to own degree, averaged over all such vertices. For
/// regular graphs equals degree. Returns 0.0 for graphs with no
/// edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, avg_neighbor_connectivity};
///
/// // K_3: each vertex has 2 neighbors each with degree 2
/// // ratio = (2+2)/2 = 2.0 for each, mean = 2.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((avg_neighbor_connectivity(&g).unwrap() - 2.0).abs() < 1e-10);
/// ```
pub fn avg_neighbor_connectivity(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }

    let mut sum_ratio: f64 = 0.0;
    let mut count = 0_u32;

    for v in 0..n {
        let dv = degrees[v];
        if dv == 0 {
            continue;
        }
        let neighbors = graph.neighbors(v as u32)?;
        let mut neighbor_deg_sum: u64 = 0;
        for &u in &neighbors {
            neighbor_deg_sum = neighbor_deg_sum.saturating_add(degrees[u as usize] as u64);
        }
        sum_ratio += neighbor_deg_sum as f64 / dv as f64;
        count += 1;
    }

    if count == 0 {
        return Ok(0.0);
    }

    Ok(sum_ratio / f64::from(count))
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

    // --- walk_entropy ---

    #[test]
    fn we_empty() {
        let g = Graph::with_vertices(0);
        assert!(walk_entropy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn we_isolated() {
        let g = Graph::with_vertices(5);
        assert!(walk_entropy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn we_single_edge() {
        // p(0)=1/2, p(1)=1/2 → H = ln(2)
        assert!((walk_entropy(&single_edge()).unwrap() - 2.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn we_k3() {
        // p=1/3 each → H = ln(3)
        assert!((walk_entropy(&k3()).unwrap() - 3.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn we_k4() {
        // p=1/4 each → H = ln(4)
        assert!((walk_entropy(&k4()).unwrap() - 4.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn we_cycle4() {
        // All d=2, p=1/4 each → H = ln(4)
        assert!((walk_entropy(&cycle4()).unwrap() - 4.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn we_path3() {
        // degrees [1,2,1], 2m=4
        // p = [1/4, 2/4, 1/4]
        // H = -2·(1/4)·ln(1/4) - (1/2)·ln(1/2)
        //   = (1/2)·ln(4) + (1/2)·ln(2)
        //   = ln(2) + (1/2)·ln(2) = (3/2)·ln(2)
        let expected: f64 = 1.5 * 2.0_f64.ln();
        assert!((walk_entropy(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn we_star5() {
        // degrees [4,1,1,1,1], 2m=8
        // p = [4/8, 1/8, 1/8, 1/8, 1/8]
        // H = -(1/2)·ln(1/2) - 4·(1/8)·ln(1/8)
        //   = (1/2)·ln(2) + (1/2)·ln(8)
        //   = (1/2)·ln(2) + (3/2)·ln(2) = 2·ln(2)
        let expected: f64 = 2.0 * 2.0_f64.ln();
        assert!((walk_entropy(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn we_paw() {
        // degrees [2,2,3,1], 2m=8
        // p = [2/8, 2/8, 3/8, 1/8]
        // H = -2·(1/4)·ln(1/4) - (3/8)·ln(3/8) - (1/8)·ln(1/8)
        let p: [f64; 4] = [0.25, 0.25, 0.375, 0.125];
        let expected: f64 = -p.iter().map(|&pi| pi * pi.ln()).sum::<f64>();
        assert!((walk_entropy(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn we_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(walk_entropy(g).unwrap() >= -1e-10);
        }
    }

    // --- walk_regularity ---

    #[test]
    fn wr_empty() {
        let g = Graph::with_vertices(0);
        assert!((walk_regularity(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn wr_single() {
        let g = Graph::with_vertices(1);
        assert!((walk_regularity(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn wr_isolated() {
        let g = Graph::with_vertices(5);
        assert!((walk_regularity(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn wr_regular() {
        // Regular → CV=0 → WR=1.0
        assert!((walk_regularity(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((walk_regularity(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((walk_regularity(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn wr_single_edge() {
        // d=1 for all → regular → 1.0
        assert!((walk_regularity(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn wr_path3() {
        // degrees [1,2,1], mean=4/3, var=(1+4+1)/3-(4/3)²=2-16/9=2/9
        // σ = √(2/9) = √2/3, CV = (√2/3)/(4/3) = √2/4
        // WR = 1 - √2/4
        let cv: f64 = (2.0_f64 / 9.0).sqrt() / (4.0 / 3.0);
        let expected: f64 = 1.0 - cv;
        assert!((walk_regularity(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn wr_star5() {
        // degrees [4,1,1,1,1], mean=8/5=1.6
        // var = (16+1+1+1+1)/5 - 1.6² = 4.0 - 2.56 = 1.44
        // σ = 1.2, CV = 1.2/1.6 = 0.75
        // WR = 1 - 0.75 = 0.25
        assert!((walk_regularity(&star5()).unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn wr_paw() {
        // degrees [2,2,3,1], mean=8/4=2
        // var = (4+4+9+1)/4 - 4 = 4.5 - 4 = 0.5
        // σ = √0.5, CV = √0.5/2
        // WR = 1 - √0.5/2
        let cv: f64 = 0.5_f64.sqrt() / 2.0;
        let expected: f64 = 1.0 - cv;
        assert!((walk_regularity(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn wr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let w = walk_regularity(g).unwrap();
            assert!(w >= -1e-10);
            assert!(w <= 1.0 + 1e-10);
        }
    }

    // --- degree_laplacian_energy ---

    #[test]
    fn dle_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_laplacian_energy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dle_single() {
        let g = Graph::with_vertices(1);
        assert!(degree_laplacian_energy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dle_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_laplacian_energy(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dle_regular() {
        // All d = mean → sum_abs_dev = 0
        assert!(degree_laplacian_energy(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_laplacian_energy(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_laplacian_energy(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dle_single_edge() {
        // d=1 for all → regular → 0
        assert!(degree_laplacian_energy(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dle_path3() {
        // degrees [1,2,1], mean=4/3, d_max=2
        // |1-4/3| + |2-4/3| + |1-4/3| = 1/3 + 2/3 + 1/3 = 4/3
        // DLE = (4/3) / (3·2) = 4/18 = 2/9
        let expected: f64 = 2.0 / 9.0;
        assert!((degree_laplacian_energy(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn dle_star5() {
        // degrees [4,1,1,1,1], mean=8/5=1.6, d_max=4
        // |4-1.6|+4·|1-1.6| = 2.4+4·0.6 = 2.4+2.4 = 4.8
        // DLE = 4.8 / (5·4) = 4.8/20 = 0.24
        assert!((degree_laplacian_energy(&star5()).unwrap() - 0.24).abs() < 1e-10);
    }

    #[test]
    fn dle_paw() {
        // degrees [2,2,3,1], mean=2, d_max=3
        // |2-2|+|2-2|+|3-2|+|1-2| = 0+0+1+1 = 2
        // DLE = 2 / (4·3) = 2/12 = 1/6
        let expected: f64 = 1.0 / 6.0;
        assert!((degree_laplacian_energy(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn dle_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_laplacian_energy(g).unwrap() >= -1e-10);
        }
    }

    // --- avg_neighbor_connectivity ---

    #[test]
    fn anc_empty() {
        let g = Graph::with_vertices(0);
        assert!(avg_neighbor_connectivity(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn anc_isolated() {
        let g = Graph::with_vertices(5);
        assert!(avg_neighbor_connectivity(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn anc_single_edge() {
        // Both vertices: neighbors have d=1, own d=1 → ratio=1, mean=1
        assert!((avg_neighbor_connectivity(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn anc_k3() {
        // Each vertex: 2 neighbors each with d=2, own d=2
        // ratio = (2+2)/2 = 2.0, mean = 2.0
        assert!((avg_neighbor_connectivity(&k3()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn anc_k4() {
        // Each vertex: 3 neighbors each with d=3, own d=3
        // ratio = 9/3 = 3.0, mean = 3.0
        assert!((avg_neighbor_connectivity(&k4()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn anc_cycle4() {
        // Each vertex: 2 neighbors each with d=2, own d=2
        // ratio = 4/2 = 2.0, mean = 2.0
        assert!((avg_neighbor_connectivity(&cycle4()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn anc_path3() {
        // v0: neighbors={1}, d(1)=2, own d=1 → ratio=2/1=2
        // v1: neighbors={0,2}, d(0)=d(2)=1, own d=2 → ratio=2/2=1
        // v2: neighbors={1}, d(1)=2, own d=1 → ratio=2/1=2
        // mean = (2+1+2)/3 = 5/3
        let expected: f64 = 5.0 / 3.0;
        assert!((avg_neighbor_connectivity(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn anc_star5() {
        // v0 (hub): neighbors={1,2,3,4}, all d=1, own d=4 → ratio=4/4=1
        // v1-v4 (leaves): neighbor={0}, d(0)=4, own d=1 → ratio=4/1=4
        // mean = (1+4+4+4+4)/5 = 17/5 = 3.4
        let expected: f64 = 17.0 / 5.0;
        assert!((avg_neighbor_connectivity(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn anc_paw() {
        // degrees [2,2,3,1]
        // v0: neighbors={1,2}, d(1)=2,d(2)=3, own d=2 → ratio=(2+3)/2=2.5
        // v1: neighbors={0,2}, d(0)=2,d(2)=3, own d=2 → ratio=(2+3)/2=2.5
        // v2: neighbors={0,1,3}, d(0)=2,d(1)=2,d(3)=1, own d=3 → ratio=(2+2+1)/3=5/3
        // v3: neighbor={2}, d(2)=3, own d=1 → ratio=3/1=3
        // mean = (2.5+2.5+5/3+3)/4 = (2.5+2.5+1.6667+3)/4 = 9.6667/4
        let v2_ratio: f64 = 5.0 / 3.0;
        let expected: f64 = (2.5 + 2.5 + v2_ratio + 3.0) / 4.0;
        assert!((avg_neighbor_connectivity(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn anc_positive() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(avg_neighbor_connectivity(g).unwrap() > 0.0);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn regular_entropy_maximal() {
        // For n active vertices, regular graph has H = ln(n)
        let h_k3 = walk_entropy(&k3()).unwrap();
        assert!((h_k3 - 3.0_f64.ln()).abs() < 1e-10);

        let h_k4 = walk_entropy(&k4()).unwrap();
        assert!((h_k4 - 4.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn regular_dle_zero() {
        assert!(degree_laplacian_energy(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_laplacian_energy(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_laplacian_energy(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn regular_wr_one() {
        assert!((walk_regularity(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((walk_regularity(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((walk_regularity(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn regular_anc_equals_degree() {
        // For r-regular graphs, ANC = r
        assert!((avg_neighbor_connectivity(&k3()).unwrap() - 2.0).abs() < 1e-10);
        assert!((avg_neighbor_connectivity(&k4()).unwrap() - 3.0).abs() < 1e-10);
        assert!((avg_neighbor_connectivity(&cycle4()).unwrap() - 2.0).abs() < 1e-10);
    }
}
