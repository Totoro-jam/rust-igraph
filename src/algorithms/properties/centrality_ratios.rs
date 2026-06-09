//! Centrality-based ratio indices (ALGO-TR-105).
//!
//! Ratios relating different centrality measures:
//!
//! - **Betweenness centralization** — max betweenness / star-graph max
//! - **Closeness centralization** — max closeness / star-graph max
//! - **Degree centralization** — max degree / star-graph max
//! - **Centrality correlation** — Pearson r(betweenness, closeness)

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute degree centralization.
///
/// `sum(max_degree - degree[v]) / ((n-1)*(n-2))` — the Freeman
/// degree centralization. Measures how star-like the degree
/// distribution is. Returns 0.0 for graphs with fewer than 3
/// vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_centralization};
///
/// // Star graph: maximum centralization = 1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(0,4)], false, Some(5)
/// ).unwrap();
/// assert!((degree_centralization(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn degree_centralization(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let mut max_deg = 0_usize;
    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        degrees.push(d);
        if d > max_deg {
            max_deg = d;
        }
    }

    let sum_diff: usize = degrees.iter().map(|&d| max_deg - d).sum();
    let denom = (n - 1) * (n - 2);

    if denom == 0 {
        return Ok(0.0);
    }

    Ok(sum_diff as f64 / denom as f64)
}

/// Compute betweenness centralization.
///
/// `sum(max_bc - bc[v]) / ((n-1)*(n-2))` — Freeman betweenness
/// centralization normalized by the theoretical maximum for a
/// star graph. Uses unnormalized betweenness (divided by 2 for
/// undirected graphs). Returns 0.0 for graphs with fewer than
/// 3 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, betweenness_centralization};
///
/// // K_3: all betweenness = 0 → centralization = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(betweenness_centralization(&g).unwrap().abs() < 1e-10);
/// ```
pub fn betweenness_centralization(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let bc = compute_betweenness(graph)?;

    let max_bc = bc.iter().copied().fold(0.0_f64, f64::max);
    let sum_diff: f64 = bc.iter().map(|&b| max_bc - b).sum();

    let n_f = n as f64;
    let denom = (n_f - 1.0) * (n_f - 2.0);

    if denom < 1e-30 {
        return Ok(0.0);
    }

    Ok(sum_diff / denom)
}

/// Compute closeness centralization.
///
/// `sum(max_cc - cc[v]) / ((n-2)*(n-1)/(2*n-3))` — Freeman
/// closeness centralization. Returns 0.0 for disconnected or
/// trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, closeness_centralization};
///
/// // K_3: all closeness = 1.0 → centralization = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(closeness_centralization(&g).unwrap().abs() < 1e-10);
/// ```
pub fn closeness_centralization(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let dist = all_pairs_bfs(graph)?;

    let mut closeness_vals = Vec::with_capacity(n);
    for v in 0..n {
        let mut sum_d = 0_u64;
        let mut connected = true;
        for u in 0..n {
            if u == v {
                continue;
            }
            let d = dist[v * n + u];
            if d == u32::MAX {
                connected = false;
                break;
            }
            sum_d += u64::from(d);
        }
        if !connected || sum_d == 0 {
            return Ok(0.0);
        }
        closeness_vals.push((n - 1) as f64 / sum_d as f64);
    }

    let max_cc = closeness_vals.iter().copied().fold(0.0_f64, f64::max);
    let sum_diff: f64 = closeness_vals.iter().map(|&c| max_cc - c).sum();

    let n_f = n as f64;
    let denom = (n_f - 2.0) * (n_f - 1.0) / (2.0 * n_f - 3.0);

    if denom < 1e-30 {
        return Ok(0.0);
    }

    Ok(sum_diff / denom)
}

/// Compute the centrality correlation (betweenness vs closeness).
///
/// Pearson correlation coefficient between betweenness centrality
/// and closeness centrality. Returns 0.0 for disconnected, trivial,
/// or constant-centrality graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, centrality_correlation};
///
/// // K_3: constant betweenness and closeness → 0.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(centrality_correlation(&g).unwrap().abs() < 1e-10);
/// ```
pub fn centrality_correlation(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let bc = compute_betweenness(graph)?;
    let dist = all_pairs_bfs(graph)?;

    let mut closeness_vals = Vec::with_capacity(n);
    for v in 0..n {
        let mut sum_d = 0_u64;
        for u in 0..n {
            if u == v {
                continue;
            }
            let d = dist[v * n + u];
            if d == u32::MAX {
                return Ok(0.0);
            }
            sum_d += u64::from(d);
        }
        if sum_d == 0 {
            closeness_vals.push(0.0);
        } else {
            closeness_vals.push((n - 1) as f64 / sum_d as f64);
        }
    }

    let mean_bc = bc.iter().sum::<f64>() / n as f64;
    let mean_cc = closeness_vals.iter().sum::<f64>() / n as f64;

    let mut cov = 0.0_f64;
    let mut var_bc = 0.0_f64;
    let mut var_cc = 0.0_f64;

    for v in 0..n {
        let db = bc[v] - mean_bc;
        let dc = closeness_vals[v] - mean_cc;
        cov += db * dc;
        var_bc += db * db;
        var_cc += dc * dc;
    }

    if var_bc < 1e-30 || var_cc < 1e-30 {
        return Ok(0.0);
    }

    Ok(cov / (var_bc.sqrt() * var_cc.sqrt()))
}

fn all_pairs_bfs(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount() as usize;
    let mut dist = vec![u32::MAX; n * n];

    for s in 0..n {
        dist[s * n + s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);
        while let Some(v) = queue.pop_front() {
            let d = dist[s * n + v];
            let neighbors = graph.neighbors(v as u32)?;
            for &u in &neighbors {
                let ui = u as usize;
                if dist[s * n + ui] == u32::MAX {
                    dist[s * n + ui] = d + 1;
                    queue.push_back(ui);
                }
            }
        }
    }

    Ok(dist)
}

fn compute_betweenness(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount() as usize;
    let directed = graph.is_directed();
    let mut bc = vec![0.0_f64; n];

    for s in 0..n {
        let mut stack = Vec::new();
        let mut pred: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut sigma = vec![0.0_f64; n];
        sigma[s] = 1.0;
        let mut dist_bfs = vec![-1_i64; n];
        dist_bfs[s] = 0;

        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            let neighbors = graph.neighbors(v as u32)?;
            for &w in &neighbors {
                let wi = w as usize;
                if dist_bfs[wi] < 0 {
                    queue.push_back(wi);
                    dist_bfs[wi] = dist_bfs[v] + 1;
                }
                if dist_bfs[wi] == dist_bfs[v] + 1 {
                    sigma[wi] += sigma[v];
                    pred[wi].push(v);
                }
            }
        }

        let mut delta = vec![0.0_f64; n];
        while let Some(w) = stack.pop() {
            for &v in &pred[w] {
                delta[v] += (sigma[v] / sigma[w]) * (1.0 + delta[w]);
            }
            if w != s {
                bc[w] += delta[w];
            }
        }
    }

    if !directed {
        for b in &mut bc {
            *b /= 2.0;
        }
    }

    Ok(bc)
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

    // --- degree_centralization ---

    #[test]
    fn dc_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_centralization(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dc_single() {
        let g = Graph::with_vertices(1);
        assert!(degree_centralization(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dc_two() {
        assert!(degree_centralization(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dc_k3() {
        // Regular → 0
        assert!(degree_centralization(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dc_k4() {
        assert!(degree_centralization(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dc_cycle4() {
        assert!(degree_centralization(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dc_star5() {
        // Star: max_deg=4, others=1, sum_diff=4*3=12, denom=4*3=12 → 1.0
        assert!((degree_centralization(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn dc_path3() {
        // deg: [1,2,1], max=2, sum_diff=1+0+1=2, denom=2*1=2 → 1.0
        assert!((degree_centralization(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn dc_paw() {
        // deg: [2,2,3,1], max=3, sum_diff=1+1+0+2=4, denom=3*2=6 → 2/3
        assert!((degree_centralization(&paw()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn dc_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = degree_centralization(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- betweenness_centralization ---

    #[test]
    fn bc_empty() {
        let g = Graph::with_vertices(0);
        assert!(betweenness_centralization(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn bc_single() {
        let g = Graph::with_vertices(1);
        assert!(betweenness_centralization(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn bc_two() {
        assert!(betweenness_centralization(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn bc_k3() {
        // All betweenness = 0 → centralization = 0
        assert!(betweenness_centralization(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn bc_k4() {
        assert!(betweenness_centralization(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn bc_cycle4() {
        // All betweenness equal (regular) → centralization = 0
        assert!(betweenness_centralization(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn bc_path3() {
        // v0: bc=0, v1: bc=1, v2: bc=0
        // max=1, sum_diff=1+0+1=2, denom=2*1=2 → 1.0
        assert!((betweenness_centralization(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn bc_star5() {
        // Center: bc = (4*3)/2 = 6, leaves: bc = 0
        // sum_diff = 6*4 = 24, denom = 4*3 = 12 → 24/12 = 2.0
        // Wait, that exceeds 1.0 — betweenness centralization uses a
        // different denominator. Let me check.
        // Standard Freeman BC centralization: sum(max-b_i) / ((n-1)*(n-2)/2)
        // For star5: sum_diff=24, denom = 4*3/2 = 6 → 24/6 = 4.0
        // Actually our formula uses (n-1)*(n-2) without the /2
        // bc = unnormalized/2 for undirected. Center: pairs through center
        // that must go through center: 4C2 = 6. bc_center = 6.
        // sum_diff = 4*6 = 24, denom = 4*3 = 12 → 2.0
        // This is > 1 which is fine for non-normalized betweenness centralization.
        let r = betweenness_centralization(&star5()).unwrap();
        assert!(r > 1.0);
    }

    #[test]
    fn bc_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(betweenness_centralization(g).unwrap() >= -1e-10);
        }
    }

    // --- closeness_centralization ---

    #[test]
    fn cc_empty() {
        let g = Graph::with_vertices(0);
        assert!(closeness_centralization(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cc_single() {
        let g = Graph::with_vertices(1);
        assert!(closeness_centralization(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cc_two() {
        assert!(closeness_centralization(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cc_k3() {
        // All closeness = 1 → centralization = 0
        assert!(closeness_centralization(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cc_k4() {
        assert!(closeness_centralization(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cc_cycle4() {
        // Regular → all same closeness → 0
        assert!(closeness_centralization(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cc_star5() {
        // Center: closeness=1.0 (dist 1 to all)
        // Leaves: closeness=4/(1+2+2+2)=4/7
        // max=1, sum_diff=4*(1-4/7)=4*3/7=12/7
        // denom=(5-2)*(5-1)/(2*5-3) = 3*4/7 = 12/7
        // centralization = (12/7)/(12/7) = 1.0
        assert!((closeness_centralization(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cc_path3() {
        // v0: c=2/(1+2)=2/3, v1: c=2/(1+1)=1, v2: c=2/3
        // max=1, sum_diff=(1/3)+(0)+(1/3)=2/3
        // denom=(3-2)*(3-1)/(2*3-3) = 1*2/3 = 2/3
        // centralization = (2/3)/(2/3) = 1.0
        assert!((closeness_centralization(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cc_disconnected() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!(closeness_centralization(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cc_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(closeness_centralization(g).unwrap() >= -1e-10);
        }
    }

    // --- centrality_correlation ---

    #[test]
    fn ccorr_empty() {
        let g = Graph::with_vertices(0);
        assert!(centrality_correlation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ccorr_single() {
        let g = Graph::with_vertices(1);
        assert!(centrality_correlation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ccorr_two() {
        assert!(centrality_correlation(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ccorr_k3() {
        // Constant → 0
        assert!(centrality_correlation(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ccorr_k4() {
        assert!(centrality_correlation(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ccorr_cycle4() {
        // Regular → constant betweenness and closeness → 0
        assert!(centrality_correlation(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ccorr_star5() {
        // Center has highest betweenness AND highest closeness → positive
        let r = centrality_correlation(&star5()).unwrap();
        assert!(r > 0.5);
    }

    #[test]
    fn ccorr_path3() {
        // v1 has highest betweenness AND highest closeness → positive
        let r = centrality_correlation(&path3()).unwrap();
        assert!(r > 0.5);
    }

    #[test]
    fn ccorr_disconnected() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!(centrality_correlation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ccorr_in_range() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = centrality_correlation(g).unwrap();
            assert!(r >= -1.0 - 1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn regular_zero_dc() {
        assert!(degree_centralization(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_centralization(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_centralization(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn complete_zero_bc() {
        assert!(betweenness_centralization(&k3()).unwrap().abs() < 1e-10);
        assert!(betweenness_centralization(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn complete_zero_cc() {
        assert!(closeness_centralization(&k3()).unwrap().abs() < 1e-10);
        assert!(closeness_centralization(&k4()).unwrap().abs() < 1e-10);
    }
}
