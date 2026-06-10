//! Core-periphery ratio indices (ALGO-TR-110).
//!
//! Measures capturing core-periphery structure via k-core decomposition:
//!
//! - **Core ratio** — fraction of vertices in the maximum core
//! - **Core density** — density of the subgraph induced by max-core vertices
//! - **Periphery fraction** — fraction of vertices with coreness 1
//! - **Core-periphery gradient** — normalized range (`max_core` - 1) / (n - 1)

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the core ratio.
///
/// Fraction of vertices that belong to the maximum k-core (the densest
/// cohesive substructure). Higher values indicate a large dense core.
/// Returns 0.0 for empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, core_ratio};
///
/// // K_4: all vertices have coreness 3 → ratio = 1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((core_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn core_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let cores = compute_coreness(graph)?;
    let max_core = cores.iter().copied().max().unwrap_or(0);
    if max_core == 0 {
        return Ok(0.0);
    }

    let count = cores.iter().filter(|&&c| c == max_core).count();
    Ok(count as f64 / n as f64)
}

/// Compute the core density.
///
/// Edge density of the subgraph induced by vertices in the maximum
/// k-core. For a complete graph this is 1.0. Returns 0.0 for empty
/// graphs or when the max-core has fewer than 2 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, core_density};
///
/// // K_4: max core = all 4 vertices, density = 1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((core_density(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn core_density(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let cores = compute_coreness(graph)?;
    let max_core = cores.iter().copied().max().unwrap_or(0);
    if max_core == 0 {
        return Ok(0.0);
    }

    let mut in_core = vec![false; n];
    let mut core_size = 0_usize;
    for (v, &c) in cores.iter().enumerate() {
        if c == max_core {
            in_core[v] = true;
            core_size += 1;
        }
    }

    if core_size < 2 {
        return Ok(0.0);
    }

    let mut edges_in_core = 0_u64;
    for v in 0..n {
        if !in_core[v] {
            continue;
        }
        let nbrs = graph.neighbors(v as u32)?;
        for &u in &nbrs {
            let ui = u as usize;
            if in_core[ui] && ui > v {
                edges_in_core += 1;
            }
        }
    }

    let max_edges = (core_size * (core_size - 1)) / 2;
    if max_edges == 0 {
        return Ok(0.0);
    }

    Ok(edges_in_core as f64 / max_edges as f64)
}

/// Compute the periphery fraction.
///
/// Fraction of vertices with coreness equal to 1 (the outermost shell).
/// For trees all non-leaf vertices have coreness 1, so this can be high.
/// Returns 0.0 for empty or edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, periphery_fraction};
///
/// // Star_5: center has coreness 1, leaves have coreness 1 → all are periphery
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(0,4)], false, Some(5)
/// ).unwrap();
/// assert!((periphery_fraction(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn periphery_fraction(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let cores = compute_coreness(graph)?;
    let max_core = cores.iter().copied().max().unwrap_or(0);
    if max_core == 0 {
        return Ok(0.0);
    }

    let count = cores.iter().filter(|&&c| c == 1).count();
    Ok(count as f64 / n as f64)
}

/// Compute the core-periphery gradient.
///
/// `(max_coreness - 1) / (n - 1)` — a normalized measure of how many
/// distinct core layers exist. Values near 0 indicate flat structure
/// (all vertices in similar cores); values near 1 indicate deep
/// hierarchical layering. Returns 0.0 for trivial or edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, core_periphery_gradient};
///
/// // K_4: max_coreness=3, n=4 → (3-1)/(4-1) = 2/3
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((core_periphery_gradient(&g).unwrap() - 2.0/3.0).abs() < 1e-10);
/// ```
pub fn core_periphery_gradient(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let cores = compute_coreness(graph)?;
    let max_core = cores.iter().copied().max().unwrap_or(0);
    if max_core <= 1 {
        return Ok(0.0);
    }

    Ok((max_core - 1) as f64 / (n - 1) as f64)
}

/// Compute k-core decomposition (Batagelj-Zaversnik O(m) algorithm).
fn compute_coreness(graph: &Graph) -> IgraphResult<Vec<usize>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }

    let mut deg = Vec::with_capacity(n);
    for v in 0..n {
        deg.push(graph.degree(v as u32)?);
    }

    let max_deg = deg.iter().copied().max().unwrap_or(0);

    // Bin-sort
    let mut bin = vec![0_usize; max_deg + 1];
    for &d in &deg {
        bin[d] += 1;
    }

    let mut start = vec![0_usize; max_deg + 1];
    let mut cumulative = 0_usize;
    for d in 0..=max_deg {
        start[d] = cumulative;
        cumulative += bin[d];
    }

    let mut vert = vec![0_usize; n]; // position → vertex
    let mut pos = vec![0_usize; n]; // vertex → position
    for v in 0..n {
        pos[v] = start[deg[v]];
        vert[pos[v]] = v;
        start[deg[v]] += 1;
    }

    // Reset start
    let mut cumulative = 0_usize;
    for d in 0..=max_deg {
        let count = bin[d];
        start[d] = cumulative;
        cumulative += count;
    }

    let mut core = deg.clone();

    for i in 0..n {
        let v = vert[i];
        let nbrs = graph.neighbors(v as u32)?;
        for &u in &nbrs {
            let ui = u as usize;
            if core[ui] > core[v] {
                let du = core[ui];
                let pu = pos[ui];
                let pw = start[du];
                let w = vert[pw];

                if ui != w {
                    vert[pu] = w;
                    vert[pw] = ui;
                    pos[w] = pu;
                    pos[ui] = pw;
                }

                start[du] += 1;
                core[ui] -= 1;
            }
        }
    }

    Ok(core)
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

    fn isolated4() -> Graph {
        Graph::with_vertices(4)
    }

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
        // 0-1, 1-2, 0-2, 2-3
        // Coreness: 0→2, 1→2, 2→2, 3→1
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn diamond() -> Graph {
        // K4 minus one edge: 0-1, 0-2, 0-3, 1-2, 2-3
        // Degrees: 0→3, 1→2, 2→3, 3→2
        // Coreness: all have coreness 2
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- core_ratio ---

    #[test]
    fn cr_empty() {
        assert!(core_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cr_single() {
        assert!(core_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cr_isolated() {
        assert!(core_ratio(&isolated4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cr_single_edge() {
        // Both vertices have coreness 1 → all in max core → 1.0
        assert!((core_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cr_k3() {
        // All coreness 2 → 1.0
        assert!((core_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cr_k4() {
        // All coreness 3 → 1.0
        assert!((core_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cr_cycle4() {
        // All coreness 2 → 1.0
        assert!((core_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cr_star5() {
        // All coreness 1 → 1.0
        assert!((core_ratio(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cr_paw() {
        // Coreness: {0,1,2}→2, {3}→1. Max core=2, count=3, n=4 → 3/4
        assert!((core_ratio(&paw()).unwrap() - 3.0 / 4.0).abs() < 1e-10);
    }

    #[test]
    fn cr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = core_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- core_density ---

    #[test]
    fn cd_empty() {
        assert!(core_density(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cd_single() {
        assert!(core_density(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cd_k4() {
        // Max core = all 4 vertices, density = 6/6 = 1.0
        assert!((core_density(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cd_k3() {
        // Max core = all 3 vertices, density = 3/3 = 1.0
        assert!((core_density(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cd_cycle4() {
        // All coreness 2, all 4 vertices in max core
        // Edges among them: 4, max possible: 6 → 4/6 = 2/3
        assert!((core_density(&cycle4()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn cd_paw() {
        // Max core (coreness=2): {0,1,2}. Edges among them: (0,1),(1,2),(0,2)=3
        // Max possible: 3. Density = 1.0
        assert!((core_density(&paw()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cd_diamond() {
        // All coreness 2 (4 vertices). Edges: 5, max: 6 → 5/6
        assert!((core_density(&diamond()).unwrap() - 5.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn cd_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = core_density(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- periphery_fraction ---

    #[test]
    fn pf_empty() {
        assert!(periphery_fraction(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn pf_single() {
        assert!(periphery_fraction(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn pf_isolated() {
        assert!(periphery_fraction(&isolated4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn pf_single_edge() {
        // Both coreness 1 → 1.0
        assert!((periphery_fraction(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn pf_star5() {
        // All coreness 1 → 1.0
        assert!((periphery_fraction(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn pf_path3() {
        // All coreness 1 → 1.0
        assert!((periphery_fraction(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn pf_k3() {
        // All coreness 2 → no vertices with coreness 1 → 0.0
        assert!(periphery_fraction(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn pf_k4() {
        // All coreness 3 → 0.0
        assert!(periphery_fraction(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn pf_paw() {
        // Vertex 3 has coreness 1 → 1/4
        assert!((periphery_fraction(&paw()).unwrap() - 1.0 / 4.0).abs() < 1e-10);
    }

    #[test]
    fn pf_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = periphery_fraction(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- core_periphery_gradient ---

    #[test]
    fn cpg_empty() {
        assert!(core_periphery_gradient(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cpg_single() {
        assert!(core_periphery_gradient(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cpg_isolated() {
        assert!(core_periphery_gradient(&isolated4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cpg_single_edge() {
        // max_coreness=1, ≤1 → 0.0
        assert!(core_periphery_gradient(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cpg_star5() {
        // max_coreness=1 → 0.0
        assert!(core_periphery_gradient(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cpg_k3() {
        // max_coreness=2, n=3 → (2-1)/(3-1) = 0.5
        assert!((core_periphery_gradient(&k3()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn cpg_k4() {
        // max_coreness=3, n=4 → (3-1)/(4-1) = 2/3
        assert!((core_periphery_gradient(&k4()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn cpg_cycle4() {
        // max_coreness=2, n=4 → (2-1)/(4-1) = 1/3
        assert!((core_periphery_gradient(&cycle4()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn cpg_paw() {
        // max_coreness=2, n=4 → (2-1)/(4-1) = 1/3
        assert!((core_periphery_gradient(&paw()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn cpg_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(core_periphery_gradient(g).unwrap() >= -1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn complete_full_core() {
        // K_n: all in max core, density = 1.0
        assert!((core_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((core_density(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tree_all_periphery() {
        // Trees: all coreness 1, periphery fraction = 1.0
        assert!((periphery_fraction(&star5()).unwrap() - 1.0).abs() < 1e-10);
        assert!((periphery_fraction(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tree_zero_gradient() {
        // Trees: max_coreness=1 → gradient=0
        assert!(core_periphery_gradient(&star5()).unwrap().abs() < 1e-10);
        assert!(core_periphery_gradient(&path3()).unwrap().abs() < 1e-10);
    }
}
