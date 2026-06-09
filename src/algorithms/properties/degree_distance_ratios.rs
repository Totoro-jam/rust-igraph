//! Degree-distance combined ratio indices (ALGO-TR-103).
//!
//! Measures combining degree and distance information:
//!
//! - **Degree-distance correlation** — Pearson correlation between
//!   vertex degree and eccentricity
//! - **Local efficiency ratio** — mean local efficiency / global efficiency
//! - **Degree-closeness ratio** — mean ratio of degree to closeness rank
//! - **Transmission ratio** — mean transmission / max transmission

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the degree-distance correlation.
///
/// Pearson correlation coefficient between vertex degree and
/// eccentricity (maximum shortest-path distance from the vertex).
/// Returns 0.0 for disconnected, trivial, or constant-degree-and-eccentricity
/// graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_distance_correlation};
///
/// // K_3: all same degree and eccentricity → 0.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_distance_correlation(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_distance_correlation(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let dist = all_pairs_bfs(graph)?;

    let mut degrees = Vec::with_capacity(n);
    let mut eccs = Vec::with_capacity(n);

    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);

        let mut ecc = 0_u32;
        for u in 0..n {
            if u == v {
                continue;
            }
            let d = dist[v * n + u];
            if d == u32::MAX {
                return Ok(0.0);
            }
            if d > ecc {
                ecc = d;
            }
        }
        eccs.push(ecc);
    }

    let mean_deg = degrees.iter().sum::<usize>() as f64 / n as f64;
    let mean_ecc = f64::from(eccs.iter().sum::<u32>()) / n as f64;

    let mut cov = 0.0_f64;
    let mut var_deg = 0.0_f64;
    let mut var_ecc = 0.0_f64;

    for v in 0..n {
        let dd = degrees[v] as f64 - mean_deg;
        let de = f64::from(eccs[v]) - mean_ecc;
        cov += dd * de;
        var_deg += dd * dd;
        var_ecc += de * de;
    }

    if var_deg < 1e-30 || var_ecc < 1e-30 {
        return Ok(0.0);
    }

    Ok(cov / (var_deg.sqrt() * var_ecc.sqrt()))
}

/// Compute the local efficiency ratio.
///
/// `mean_local_eff / global_eff` — how the average vertex's local
/// efficiency compares to the global efficiency. Returns 0.0 if
/// global efficiency is zero or for trivial graphs.
///
/// Local efficiency of vertex v is the average inverse distance
/// between v's neighbors (using full-graph distances).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, local_efficiency_ratio};
///
/// // K_3: global_eff=1.0, each vertex's neighbors form K_2 → local_eff=1.0 → ratio=1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((local_efficiency_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn local_efficiency_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let dist = all_pairs_bfs(graph)?;

    let mut global_sum_inv = 0.0_f64;
    let mut global_pairs = 0_u64;
    for v in 0..n {
        for u in (v + 1)..n {
            let d = dist[v * n + u];
            if d != u32::MAX && d > 0 {
                global_sum_inv += 1.0 / f64::from(d);
            }
            global_pairs += 1;
        }
    }

    if global_pairs == 0 {
        return Ok(0.0);
    }
    let global_eff = global_sum_inv / global_pairs as f64;
    if global_eff < 1e-30 {
        return Ok(0.0);
    }

    let mut local_eff_sum = 0.0_f64;
    for v in 0..n {
        let neighbors = graph.neighbors(v as u32)?;
        let nn = neighbors.len();
        if nn < 2 {
            continue;
        }

        let mut sub_sum_inv = 0.0_f64;
        let mut sub_pairs = 0_u64;
        for i in 0..nn {
            let ni = neighbors[i] as usize;
            for j in (i + 1)..nn {
                let nj = neighbors[j] as usize;
                let d = dist[ni * n + nj];
                if d != u32::MAX && d > 0 {
                    sub_sum_inv += 1.0 / f64::from(d);
                }
                sub_pairs += 1;
            }
        }

        if sub_pairs > 0 {
            local_eff_sum += sub_sum_inv / sub_pairs as f64;
        }
    }

    let mean_local_eff = local_eff_sum / n as f64;
    Ok(mean_local_eff / global_eff)
}

/// Compute the transmission ratio.
///
/// `mean_transmission / max_transmission` where transmission of a
/// vertex is the sum of distances from it to all other vertices.
/// Returns 0.0 for disconnected or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, transmission_ratio};
///
/// // K_3: all transmissions = 2, ratio = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((transmission_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn transmission_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let dist = all_pairs_bfs(graph)?;

    let mut transmissions = Vec::with_capacity(n);
    for v in 0..n {
        let mut t = 0_u64;
        for u in 0..n {
            if u == v {
                continue;
            }
            let d = dist[v * n + u];
            if d == u32::MAX {
                return Ok(0.0);
            }
            t += u64::from(d);
        }
        transmissions.push(t);
    }

    let max_t = transmissions.iter().copied().max().unwrap_or(0);
    if max_t == 0 {
        return Ok(0.0);
    }

    let mean_t = transmissions.iter().sum::<u64>() as f64 / n as f64;
    Ok(mean_t / max_t as f64)
}

/// Compute the degree-closeness correlation.
///
/// Pearson correlation between vertex degree and closeness centrality
/// (reciprocal of mean distance). Returns 0.0 for disconnected or
/// trivial graphs, or when degree or closeness has zero variance.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_closeness_correlation};
///
/// // K_3: constant degree and closeness → 0.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_closeness_correlation(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_closeness_correlation(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let dist = all_pairs_bfs(graph)?;

    let mut degrees = Vec::with_capacity(n);
    let mut closeness_vals = Vec::with_capacity(n);

    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);

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

    let mean_deg = degrees.iter().sum::<usize>() as f64 / n as f64;
    let mean_close: f64 = closeness_vals.iter().sum::<f64>() / n as f64;

    let mut cov = 0.0_f64;
    let mut var_deg = 0.0_f64;
    let mut var_close = 0.0_f64;

    for v in 0..n {
        let dd = degrees[v] as f64 - mean_deg;
        let dc = closeness_vals[v] - mean_close;
        cov += dd * dc;
        var_deg += dd * dd;
        var_close += dc * dc;
    }

    if var_deg < 1e-30 || var_close < 1e-30 {
        return Ok(0.0);
    }

    Ok(cov / (var_deg.sqrt() * var_close.sqrt()))
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

    // --- degree_distance_correlation ---

    #[test]
    fn ddc_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_distance_correlation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ddc_single() {
        let g = Graph::with_vertices(1);
        assert!(degree_distance_correlation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ddc_k3() {
        // Regular, constant eccentricity → 0
        assert!(degree_distance_correlation(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ddc_k4() {
        assert!(degree_distance_correlation(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ddc_cycle4() {
        // Regular, constant eccentricity → 0
        assert!(degree_distance_correlation(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ddc_star5() {
        // Center: deg=4, ecc=1; Leaves: deg=1, ecc=2
        // Higher degree → lower eccentricity → negative correlation
        let r = degree_distance_correlation(&star5()).unwrap();
        assert!(r < -0.5);
    }

    #[test]
    fn ddc_path3() {
        // deg: [1,2,1], ecc: [2,1,2] → negative correlation
        let r = degree_distance_correlation(&path3()).unwrap();
        assert!(r < -0.5);
    }

    #[test]
    fn ddc_disconnected() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!(degree_distance_correlation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ddc_in_range() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = degree_distance_correlation(g).unwrap();
            assert!(r >= -1.0 - 1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- local_efficiency_ratio ---

    #[test]
    fn ler_empty() {
        let g = Graph::with_vertices(0);
        assert!(local_efficiency_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ler_single() {
        let g = Graph::with_vertices(1);
        assert!(local_efficiency_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ler_k3() {
        // global=1.0, each vertex's neighbors form K_2, local_eff=1.0 → ratio=1.0
        assert!((local_efficiency_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ler_k4() {
        // global=1.0, each vertex's neighbors form K_3, local_eff=1.0 → ratio=1.0
        assert!((local_efficiency_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ler_star5() {
        // Center: neighbors={1,2,3,4}, all at dist 2 → local_eff=0.5
        // Leaves: 1 neighbor → local_eff=0
        // mean_local=0.1, global=0.7 → ratio=1/7
        let r = local_efficiency_ratio(&star5()).unwrap();
        assert!(r > 0.0);
        assert!(r < 0.5);
    }

    #[test]
    fn ler_single_edge() {
        // Each vertex has 1 neighbor → no subgraph pairs → local_eff = 0
        // But global eff = 1.0 → ratio = 0
        assert!(local_efficiency_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ler_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(local_efficiency_ratio(g).unwrap() >= -1e-10);
        }
    }

    // --- transmission_ratio ---

    #[test]
    fn tr_empty() {
        let g = Graph::with_vertices(0);
        assert!(transmission_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tr_single() {
        let g = Graph::with_vertices(1);
        assert!(transmission_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tr_k3() {
        // All transmissions = 2 → ratio = 1.0
        assert!((transmission_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tr_k4() {
        // All transmissions = 3 → ratio = 1.0
        assert!((transmission_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tr_cycle4() {
        // All transmissions = 1+2+1=4 → ratio = 1.0
        assert!((transmission_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tr_path3() {
        // Trans: [1+2, 1+1, 2+1] = [3, 2, 3]
        // mean=8/3, max=3 → 8/9
        assert!((transmission_ratio(&path3()).unwrap() - 8.0 / 9.0).abs() < 1e-10);
    }

    #[test]
    fn tr_star5() {
        // Center: 1+1+1+1=4; Leaf: 1+2+2+2=7
        // Trans: [4,7,7,7,7], mean=32/5, max=7 → 32/35
        assert!((transmission_ratio(&star5()).unwrap() - 32.0 / 35.0).abs() < 1e-10);
    }

    #[test]
    fn tr_disconnected() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!(transmission_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = transmission_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- degree_closeness_correlation ---

    #[test]
    fn dcc_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_closeness_correlation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dcc_single() {
        let g = Graph::with_vertices(1);
        assert!(degree_closeness_correlation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dcc_k3() {
        // Constant → 0
        assert!(degree_closeness_correlation(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dcc_k4() {
        assert!(degree_closeness_correlation(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dcc_cycle4() {
        assert!(degree_closeness_correlation(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dcc_star5() {
        // Center: deg=4, closeness=1.0 (all at dist 1)
        // Leaf: deg=1, closeness=4/(1+2+2+2)=4/7
        // Higher degree → higher closeness → positive correlation
        let r = degree_closeness_correlation(&star5()).unwrap();
        assert!(r > 0.5);
    }

    #[test]
    fn dcc_path3() {
        // deg: [1,2,1], closeness: [2/(1+2)=2/3, 2/(1+1)=1, 2/(2+1)=2/3]
        // Higher degree → higher closeness → positive
        let r = degree_closeness_correlation(&path3()).unwrap();
        assert!(r > 0.5);
    }

    #[test]
    fn dcc_disconnected() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!(degree_closeness_correlation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dcc_in_range() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = degree_closeness_correlation(g).unwrap();
            assert!(r >= -1.0 - 1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn regular_graphs_zero_corr() {
        // Regular graphs have constant degree → zero degree-distance correlation
        assert!(degree_distance_correlation(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_distance_correlation(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_distance_correlation(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn regular_graphs_unit_transmission() {
        // Regular + vertex-transitive → all same transmission → ratio=1
        assert!((transmission_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((transmission_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((transmission_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }
}
