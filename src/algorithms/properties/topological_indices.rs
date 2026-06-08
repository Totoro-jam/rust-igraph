//! Chemical graph topological indices (ALGO-TR-040).
//!
//! Degree-based topological indices used in chemical graph theory and
//! QSAR/QSPR modelling. All are defined for simple undirected graphs;
//! directed graphs are treated as undirected (edge directions ignored).
//!
//! - **First Zagreb index** `M₁ = Σ_v deg(v)²`
//! - **Second Zagreb index** `M₂ = Σ_{(u,v)∈E} deg(u)·deg(v)`
//! - **Randić index** `R = Σ_{(u,v)∈E} 1/√(deg(u)·deg(v))`
//! - **Atom-bond connectivity (ABC) index**
//!   `ABC = Σ_{(u,v)∈E} √((deg(u)+deg(v)-2)/(deg(u)·deg(v)))`
//! - **Harmonic index** `H = Σ_{(u,v)∈E} 2/(deg(u)+deg(v))`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the first Zagreb index `M₁(G) = Σ deg(v)²`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_zagreb_index};
///
/// // Path 0-1-2: degrees [1, 2, 1], M₁ = 1+4+1 = 6
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(first_zagreb_index(&g).unwrap(), 6);
/// ```
pub fn first_zagreb_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    let deg = compute_degrees(graph, n)?;

    let mut m1: u64 = 0;
    for &d in &deg {
        m1 = m1.saturating_add((d as u64).saturating_mul(d as u64));
    }

    Ok(m1)
}

/// Compute the second Zagreb index `M₂(G) = Σ_{(u,v)∈E} deg(u)·deg(v)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_zagreb_index};
///
/// // Path 0-1-2: edge (0,1): 1·2=2, edge (1,2): 2·1=2 → M₂ = 4
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(second_zagreb_index(&g).unwrap(), 4);
/// ```
pub fn second_zagreb_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    let deg = compute_degrees(graph, n)?;

    let mut m2: u64 = 0;
    for (u, v) in graph.edges() {
        let du = deg[u as usize] as u64;
        let dv = deg[v as usize] as u64;
        m2 = m2.saturating_add(du.saturating_mul(dv));
    }

    Ok(m2)
}

/// Compute the Randić connectivity index.
///
/// `R(G) = Σ_{(u,v)∈E} 1/√(deg(u)·deg(v))`
///
/// Isolated vertices and self-loops are ignored.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, randic_index};
///
/// // Path 0-1-2: 1/√(1·2) + 1/√(2·1) = 2/√2 ≈ 1.414
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let r = randic_index(&g).unwrap();
/// assert!((r - std::f64::consts::SQRT_2).abs() < 1e-10);
/// ```
pub fn randic_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let deg = compute_degrees(graph, n)?;

    let mut r: f64 = 0.0;

    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        if ui == vi || deg[ui] == 0 || deg[vi] == 0 {
            continue;
        }
        let product = (deg[ui] as f64) * (deg[vi] as f64);
        r += 1.0 / product.sqrt();
    }

    Ok(r)
}

/// Compute the atom-bond connectivity (ABC) index.
///
/// `ABC(G) = Σ_{(u,v)∈E} √((deg(u)+deg(v)-2)/(deg(u)·deg(v)))`
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, abc_index};
///
/// // Single edge: √((1+1-2)/(1·1)) = 0
/// let g = Graph::from_edges(&[(0,1)], false, Some(2)).unwrap();
/// assert!((abc_index(&g).unwrap() - 0.0).abs() < 1e-10);
/// ```
pub fn abc_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let deg = compute_degrees(graph, n)?;

    let mut abc: f64 = 0.0;

    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        if ui == vi || deg[ui] == 0 || deg[vi] == 0 {
            continue;
        }
        let du = deg[ui] as f64;
        let dv = deg[vi] as f64;
        let numerator = du + dv - 2.0;
        if numerator >= 0.0 {
            abc += (numerator / (du * dv)).sqrt();
        }
    }

    Ok(abc)
}

/// Compute the harmonic index.
///
/// `H(G) = Σ_{(u,v)∈E} 2/(deg(u)+deg(v))`
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, harmonic_graph_index};
///
/// // Path 0-1-2: 2/(1+2) + 2/(2+1) = 4/3 ≈ 1.333
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let h = harmonic_graph_index(&g).unwrap();
/// assert!((h - 4.0/3.0).abs() < 1e-10);
/// ```
pub fn harmonic_graph_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let deg = compute_degrees(graph, n)?;

    let mut h: f64 = 0.0;

    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        if ui == vi {
            continue;
        }
        let sum_deg = deg[ui] as f64 + deg[vi] as f64;
        if sum_deg > 0.0 {
            h += 2.0 / sum_deg;
        }
    }

    Ok(h)
}

fn compute_degrees(graph: &Graph, n: usize) -> IgraphResult<Vec<usize>> {
    let mut deg = vec![0_usize; n];
    for v in 0..n as u32 {
        deg[v as usize] = graph.degree(v)?;
    }
    Ok(deg)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn single_edge() -> Graph {
        Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap()
    }

    // --- first_zagreb_index ---

    #[test]
    fn fzi_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(first_zagreb_index(&g).unwrap(), 0);
    }

    #[test]
    fn fzi_no_edges() {
        let g = Graph::with_vertices(3);
        assert_eq!(first_zagreb_index(&g).unwrap(), 0);
    }

    #[test]
    fn fzi_single_edge() {
        // degrees [1,1] → 1+1 = 2
        assert_eq!(first_zagreb_index(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn fzi_path3() {
        // degrees [1,2,1] → 1+4+1 = 6
        assert_eq!(first_zagreb_index(&path3()).unwrap(), 6);
    }

    #[test]
    fn fzi_path4() {
        // degrees [1,2,2,1] → 1+4+4+1 = 10
        assert_eq!(first_zagreb_index(&path4()).unwrap(), 10);
    }

    #[test]
    fn fzi_k3() {
        // all degree 2 → 3×4 = 12
        assert_eq!(first_zagreb_index(&k3()).unwrap(), 12);
    }

    #[test]
    fn fzi_k4() {
        // all degree 3 → 4×9 = 36
        assert_eq!(first_zagreb_index(&k4()).unwrap(), 36);
    }

    #[test]
    fn fzi_cycle4() {
        // all degree 2 → 4×4 = 16
        assert_eq!(first_zagreb_index(&cycle4()).unwrap(), 16);
    }

    #[test]
    fn fzi_star5() {
        // center deg=4, leaves deg=1 → 16+1+1+1+1 = 20
        assert_eq!(first_zagreb_index(&star5()).unwrap(), 20);
    }

    // --- second_zagreb_index ---

    #[test]
    fn szi_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(second_zagreb_index(&g).unwrap(), 0);
    }

    #[test]
    fn szi_single_edge() {
        // 1·1 = 1
        assert_eq!(second_zagreb_index(&single_edge()).unwrap(), 1);
    }

    #[test]
    fn szi_path3() {
        // (0,1): 1·2=2, (1,2): 2·1=2 → 4
        assert_eq!(second_zagreb_index(&path3()).unwrap(), 4);
    }

    #[test]
    fn szi_k3() {
        // 3 edges, each 2·2=4 → 12
        assert_eq!(second_zagreb_index(&k3()).unwrap(), 12);
    }

    #[test]
    fn szi_k4() {
        // 6 edges, each 3·3=9 → 54
        assert_eq!(second_zagreb_index(&k4()).unwrap(), 54);
    }

    #[test]
    fn szi_star5() {
        // 4 edges, each 4·1=4 → 16
        assert_eq!(second_zagreb_index(&star5()).unwrap(), 16);
    }

    // --- randic_index ---

    #[test]
    fn ri_empty() {
        let g = Graph::with_vertices(0);
        assert!((randic_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn ri_single_edge() {
        // 1/√(1·1) = 1
        assert!((randic_index(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ri_path3() {
        // 1/√2 + 1/√2 = √2
        let r = randic_index(&path3()).unwrap();
        assert!((r - std::f64::consts::SQRT_2).abs() < 1e-10);
    }

    #[test]
    fn ri_k3() {
        // 3 × 1/√(2·2) = 3/2 = 1.5
        let r = randic_index(&k3()).unwrap();
        assert!((r - 1.5).abs() < 1e-10);
    }

    #[test]
    fn ri_k4() {
        // 6 × 1/√(3·3) = 6/3 = 2.0
        let r = randic_index(&k4()).unwrap();
        assert!((r - 2.0).abs() < 1e-10);
    }

    // --- abc_index ---

    #[test]
    fn abc_single_edge() {
        // √((1+1-2)/(1·1)) = 0
        assert!((abc_index(&single_edge()).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn abc_path3() {
        // 2 edges: √((1+2-2)/(1·2)) = √(1/2) each → 2·√(0.5)
        let a = abc_index(&path3()).unwrap();
        assert!((a - 2.0 * (0.5_f64).sqrt()).abs() < 1e-10);
    }

    #[test]
    fn abc_k3() {
        // 3 edges: √((2+2-2)/(2·2)) = √(2/4) = √(0.5) each → 3·√(0.5)
        let a = abc_index(&k3()).unwrap();
        assert!((a - 3.0 * (0.5_f64).sqrt()).abs() < 1e-10);
    }

    // --- harmonic_graph_index ---

    #[test]
    fn hgi_empty() {
        let g = Graph::with_vertices(0);
        assert!((harmonic_graph_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn hgi_single_edge() {
        // 2/(1+1) = 1
        assert!((harmonic_graph_index(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn hgi_path3() {
        // 2/(1+2) + 2/(2+1) = 4/3
        let h = harmonic_graph_index(&path3()).unwrap();
        assert!((h - 4.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn hgi_k3() {
        // 3 × 2/(2+2) = 3 × 0.5 = 1.5
        let h = harmonic_graph_index(&k3()).unwrap();
        assert!((h - 1.5).abs() < 1e-10);
    }

    #[test]
    fn hgi_star5() {
        // 4 × 2/(4+1) = 8/5 = 1.6
        let h = harmonic_graph_index(&star5()).unwrap();
        assert!((h - 1.6).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn m1_equals_sum_over_edges() {
        // Alternative formula: M₁ = Σ_{(u,v)∈E} (deg(u)+deg(v))
        for g in &[path3(), path4(), k3(), k4(), cycle4(), star5()] {
            let m1 = first_zagreb_index(g).unwrap();
            let n = g.vcount() as usize;
            let deg: Vec<usize> = (0..n as u32).map(|v| g.degree(v).unwrap()).collect();

            let mut seen = std::collections::HashSet::new();
            let mut alt: u64 = 0;
            for (u, v) in g.edges() {
                let key = (u.min(v), u.max(v));
                if seen.insert(key) {
                    alt += deg[u as usize] as u64 + deg[v as usize] as u64;
                }
            }
            assert_eq!(m1, alt, "M₁ edge sum formula mismatch");
        }
    }

    #[test]
    fn regular_graph_indices() {
        // For r-regular graph with n vertices, m edges:
        // M₁ = n·r², M₂ = m·r², R = m/r, H = m/r
        let g = cycle4(); // 2-regular, n=4, m=4
        assert_eq!(first_zagreb_index(&g).unwrap(), 16); // 4·4
        assert_eq!(second_zagreb_index(&g).unwrap(), 16); // 4·4
        let r = randic_index(&g).unwrap();
        assert!((r - 2.0).abs() < 1e-10); // 4/2
        let h = harmonic_graph_index(&g).unwrap();
        assert!((h - 2.0).abs() < 1e-10); // 4 × 2/4 = 2
    }
}
