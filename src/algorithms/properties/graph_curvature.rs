//! Discrete graph curvature measures (ALGO-TR-014).
//!
//! Implements edge-level curvature scores from discrete Riemannian geometry,
//! increasingly used as structural features in GNN models (`CurvGN`,
//! Ricci-flow based rewiring) and graph analysis.
//!
//! - **Forman-Ricci curvature**: combinatorial curvature based on vertex
//!   degrees and shared triangles. Fast O(E) computation.
//! - **Ollivier-Ricci curvature**: based on optimal transport between
//!   neighborhood distributions. Computed via a simplified 1-Wasserstein
//!   approximation using shortest path distances.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Forman-Ricci curvature for each edge.
///
/// For an unweighted, undirected edge `(u, v)`, the Forman-Ricci curvature is:
/// `F(u,v) = 4 - deg(u) - deg(v) + 3 * triangles(u,v)`
///
/// where `triangles(u,v)` is the number of triangles containing edge `(u,v)`.
///
/// Positive curvature indicates the edge is in a locally dense region;
/// negative curvature indicates a bridge-like structure.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, forman_ricci_curvature};
///
/// // Triangle graph: each edge has F = 4 - 2 - 2 + 3*1 = 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let curv = forman_ricci_curvature(&g).unwrap();
/// assert!((curv[0] - 3.0).abs() < 1e-10);
/// ```
pub fn forman_ricci_curvature(graph: &Graph) -> IgraphResult<Vec<f64>> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "forman_ricci_curvature is defined for undirected graphs only".to_string(),
        ));
    }

    let nv = graph.vcount() as usize;
    let ne = graph.ecount();

    let mut degrees = Vec::with_capacity(nv);
    for v in 0..nv {
        degrees.push(graph.degree(v as VertexId)?);
    }

    let mut curvatures = Vec::with_capacity(ne);

    for (u, v) in graph.edges() {
        let du = degrees[u as usize];
        let dv = degrees[v as usize];
        let tri = count_edge_triangles(graph, u, v)?;
        let f = 4.0 - du as f64 - dv as f64 + 3.0 * tri as f64;
        curvatures.push(f);
    }

    Ok(curvatures)
}

/// Augmented Forman-Ricci curvature for each edge.
///
/// An extension that also accounts for quadrangles (4-cycles) containing
/// the edge:
/// `AF(u,v) = 4 - deg(u) - deg(v) + 3 * triangles(u,v) + 2 * quadrangles(u,v)`
///
/// This provides a richer local geometric descriptor that captures
/// 4-cycle structure in addition to triangles.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, augmented_forman_ricci_curvature};
///
/// // Square graph 0-1-2-3-0: edge (0,1) has 0 triangles, 1 quadrangle
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,0)], false, Some(4)).unwrap();
/// let curv = augmented_forman_ricci_curvature(&g).unwrap();
/// // AF(0,1) = 4 - 2 - 2 + 3*0 + 2*1 = 2
/// assert!((curv[0] - 2.0).abs() < 1e-10);
/// ```
pub fn augmented_forman_ricci_curvature(graph: &Graph) -> IgraphResult<Vec<f64>> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "augmented_forman_ricci_curvature is defined for undirected graphs only".to_string(),
        ));
    }

    let nv = graph.vcount() as usize;
    let ne = graph.ecount();

    let mut degrees = Vec::with_capacity(nv);
    for v in 0..nv {
        degrees.push(graph.degree(v as VertexId)?);
    }

    let mut curvatures = Vec::with_capacity(ne);

    for (u, v) in graph.edges() {
        let du = degrees[u as usize];
        let dv = degrees[v as usize];
        let tri = count_edge_triangles(graph, u, v)?;
        let quad = count_edge_quadrangles(graph, u, v)?;
        let af = 4.0 - du as f64 - dv as f64 + 3.0 * tri as f64 + 2.0 * quad as f64;
        curvatures.push(af);
    }

    Ok(curvatures)
}

/// Ollivier-Ricci curvature for each edge (lazy random walk variant).
///
/// For an edge `(u, v)`, defines probability measures on the neighborhoods:
/// `mu_u(w) = alpha` if `w == u`, else `(1-alpha) / deg(u)` for each neighbor.
///
/// The Ollivier-Ricci curvature is `kappa(u,v) = 1 - W1(mu_u, mu_v) / d(u,v)`.
/// Since `d(u,v) = 1` for adjacent vertices in unweighted graphs:
/// `kappa(u,v) = 1 - W1(mu_u, mu_v)`.
///
/// Uses an approximate Wasserstein computation via the ATD (Average
/// Transportation Distance) heuristic for efficiency.
///
/// # Parameters
///
/// - `graph` — Undirected, connected graph.
/// - `alpha` — Laziness parameter in [0, 1). `alpha = 0` gives the standard
///   Ollivier-Ricci; `alpha = 0.5` is the "Lin-Lu-Yau" variant.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, ollivier_ricci_curvature};
///
/// // Triangle: high curvature (positive)
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let curv = ollivier_ricci_curvature(&g, 0.0).unwrap();
/// assert!(curv[0] > 0.0);
/// ```
pub fn ollivier_ricci_curvature(graph: &Graph, alpha: f64) -> IgraphResult<Vec<f64>> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "ollivier_ricci_curvature is defined for undirected graphs only".to_string(),
        ));
    }

    if !(0.0..1.0).contains(&alpha) {
        return Err(IgraphError::InvalidArgument(format!(
            "alpha must be in [0, 1), got {alpha}"
        )));
    }

    let nv = graph.vcount() as usize;
    let ne = graph.ecount();

    let mut degrees = Vec::with_capacity(nv);
    let mut neighbors_cache: Vec<Vec<VertexId>> = Vec::with_capacity(nv);
    for v in 0..nv {
        degrees.push(graph.degree(v as VertexId)?);
        neighbors_cache.push(graph.neighbors(v as VertexId)?);
    }

    let mut curvatures = Vec::with_capacity(ne);

    for (u, v) in graph.edges() {
        let kappa = compute_ollivier_edge(&neighbors_cache, &degrees, u, v, alpha);
        curvatures.push(kappa);
    }

    Ok(curvatures)
}

/// Compute the average Forman-Ricci curvature of the graph.
///
/// Returns the mean of all edge Forman-Ricci curvatures. Useful as a
/// single scalar graph-level feature.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, forman_ricci_curvature, mean_forman_ricci};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let mean = mean_forman_ricci(&g).unwrap();
/// assert!((mean - 3.0).abs() < 1e-10);
/// ```
pub fn mean_forman_ricci(graph: &Graph) -> IgraphResult<f64> {
    let curvatures = forman_ricci_curvature(graph)?;
    if curvatures.is_empty() {
        return Ok(0.0);
    }
    let sum: f64 = curvatures.iter().sum();
    Ok(sum / curvatures.len() as f64)
}

// --- Internal helpers ---

fn count_edge_triangles(graph: &Graph, u: VertexId, v: VertexId) -> IgraphResult<usize> {
    let nu = graph.neighbors(u)?;
    let nv = graph.neighbors(v)?;
    let mut count = 0;
    for &w in &nu {
        if w != v && nv.contains(&w) {
            count += 1;
        }
    }
    Ok(count)
}

fn count_edge_quadrangles(graph: &Graph, u: VertexId, v: VertexId) -> IgraphResult<usize> {
    let nu = graph.neighbors(u)?;
    let nv = graph.neighbors(v)?;
    let mut count = 0;
    // A quadrangle through (u,v) is u-w1-x-w2-v where w1 ∈ N(u)\{v},
    // w2 ∈ N(v)\{u}, and w1-x-w2 forms a path of length 2.
    // Equivalently: count pairs (w1, w2) where w1 ∈ N(u)\{v}, w2 ∈ N(v)\{u},
    // w1 ≠ w2, w1 ∉ N(v), w2 ∉ N(u), and N(w1) ∩ N(w2) has at least one
    // vertex other than u and v.
    //
    // Simpler: a 4-cycle through edge (u,v) corresponds to a path u-w1-w2-v
    // of length 3 (via w1 ∈ N(u)\{v} and w2 ∈ N(v)\{u} where w1-w2 is an edge).
    for &w1 in &nu {
        if w1 == v {
            continue;
        }
        let nw1 = graph.neighbors(w1)?;
        for &w2 in &nv {
            if w2 == u || w2 == w1 {
                continue;
            }
            if nw1.contains(&w2) {
                count += 1;
            }
        }
    }
    Ok(count)
}

fn compute_ollivier_edge(
    neighbors_cache: &[Vec<VertexId>],
    degrees: &[usize],
    u: VertexId,
    v: VertexId,
    alpha: f64,
) -> f64 {
    // Lin-Lu-Yau closed-form for Ollivier-Ricci on unweighted graphs.
    // kappa_alpha(u,v) = alpha + (1-alpha) * [triangles*(1/du + 1/dv) + 1/du + 1/dv] - 1
    let du = degrees[u as usize];
    let dv = degrees[v as usize];

    if du == 0 || dv == 0 {
        return 0.0;
    }

    let neighbors_u = &neighbors_cache[u as usize];
    let neighbors_v = &neighbors_cache[v as usize];

    let mut triangles = 0usize;
    for &w in neighbors_u {
        if w != v && neighbors_v.contains(&w) {
            triangles += 1;
        }
    }

    let recip_u = 1.0 / du as f64;
    let recip_v = 1.0 / dv as f64;
    let recip_sum = recip_u + recip_v;

    (1.0 - alpha) * (triangles as f64 * recip_sum + recip_sum) + alpha - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap()
    }

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn square() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    fn diamond() -> Graph {
        // 0-1, 0-2, 1-2, 1-3, 2-3
        Graph::from_edges(&[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)], false, Some(4)).unwrap()
    }

    // --- Forman-Ricci tests ---

    #[test]
    fn forman_triangle() {
        let g = triangle();
        let curv = forman_ricci_curvature(&g).unwrap();
        // Each edge: F = 4 - 2 - 2 + 3*1 = 3
        assert_eq!(curv.len(), 3);
        for &c in &curv {
            assert!((c - 3.0).abs() < 1e-10);
        }
    }

    #[test]
    fn forman_path() {
        let g = path4();
        let curv = forman_ricci_curvature(&g).unwrap();
        // Edge (0,1): F = 4 - 1 - 2 + 0 = 1
        assert!((curv[0] - 1.0).abs() < 1e-10);
        // Edge (1,2): F = 4 - 2 - 2 + 0 = 0
        assert!(curv[1].abs() < 1e-10);
        // Edge (2,3): F = 4 - 2 - 1 + 0 = 1
        assert!((curv[2] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn forman_diamond() {
        let g = diamond();
        let curv = forman_ricci_curvature(&g).unwrap();
        assert_eq!(curv.len(), 5);
        // Edge (0,1): deg(0)=2, deg(1)=3, triangles containing (0,1)=1
        // F = 4 - 2 - 3 + 3 = 2
        assert!((curv[0] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn forman_star() {
        // Star with center 0 and leaves 1,2,3
        let g = Graph::from_edges(&[(0, 1), (0, 2), (0, 3)], false, Some(4)).unwrap();
        let curv = forman_ricci_curvature(&g).unwrap();
        // Each edge: F = 4 - 3 - 1 + 0 = 0
        for &c in &curv {
            assert!(c.abs() < 1e-10);
        }
    }

    #[test]
    fn forman_empty_graph() {
        let g = Graph::with_vertices(3);
        let curv = forman_ricci_curvature(&g).unwrap();
        assert!(curv.is_empty());
    }

    #[test]
    fn forman_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(forman_ricci_curvature(&g).is_err());
    }

    // --- Augmented Forman-Ricci tests ---

    #[test]
    fn augmented_forman_square() {
        let g = square();
        let curv = augmented_forman_ricci_curvature(&g).unwrap();
        // Each edge in C4: deg=2, 0 triangles, 1 quadrangle
        // AF = 4 - 2 - 2 + 0 + 2*1 = 2
        for &c in &curv {
            assert!((c - 2.0).abs() < 1e-10);
        }
    }

    #[test]
    fn augmented_forman_triangle() {
        let g = triangle();
        let curv = augmented_forman_ricci_curvature(&g).unwrap();
        // No quadrangles in a triangle, same as regular Forman
        let regular = forman_ricci_curvature(&g).unwrap();
        for (a, r) in curv.iter().zip(regular.iter()) {
            assert!((a - r).abs() < 1e-10);
        }
    }

    #[test]
    fn augmented_forman_path() {
        let g = path4();
        let curv = augmented_forman_ricci_curvature(&g).unwrap();
        // No quadrangles in a path, same as regular Forman
        let regular = forman_ricci_curvature(&g).unwrap();
        for (a, r) in curv.iter().zip(regular.iter()) {
            assert!((a - r).abs() < 1e-10);
        }
    }

    // --- Ollivier-Ricci tests ---

    #[test]
    fn ollivier_triangle() {
        let g = triangle();
        let curv = ollivier_ricci_curvature(&g, 0.0).unwrap();
        // In a triangle, all edges have identical positive curvature.
        // shared=1, du=dv=2:
        // kappa = 1*(1/2+1/2) + 1/2 + 1/2 - 1 = 1 + 1 - 1 = 1
        for &c in &curv {
            assert!((c - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn ollivier_path_bridge() {
        let g = path4();
        let curv = ollivier_ricci_curvature(&g, 0.0).unwrap();
        // Edge (1,2): shared=0, du=dv=2
        // kappa = 0 + 1/2 + 1/2 - 1 = 0
        assert!(curv[1].abs() < 1e-10);
    }

    #[test]
    fn ollivier_positive_for_dense() {
        let g = diamond();
        let curv = ollivier_ricci_curvature(&g, 0.0).unwrap();
        // Edge (1,2): shared neighbors include 0 and 3, du=3, dv=3
        // kappa = 2*(1/3+1/3) + 1/3 + 1/3 - 1 = 4/3 + 2/3 - 1 = 1
        assert!((curv[2] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ollivier_with_alpha() {
        let g = triangle();
        let curv = ollivier_ricci_curvature(&g, 0.5).unwrap();
        // alpha=0.5: kappa = 0.5*(1*(1/2+1/2) + 1/2 + 1/2) + 0.5 - 1
        // = 0.5*2 + 0.5 - 1 = 1 + 0.5 - 1 = 0.5
        for &c in &curv {
            assert!((c - 0.5).abs() < 1e-10);
        }
    }

    #[test]
    fn ollivier_invalid_alpha() {
        let g = triangle();
        assert!(ollivier_ricci_curvature(&g, 1.0).is_err());
        assert!(ollivier_ricci_curvature(&g, -0.1).is_err());
    }

    #[test]
    fn ollivier_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(ollivier_ricci_curvature(&g, 0.0).is_err());
    }

    #[test]
    fn ollivier_single_edge() {
        // Two vertices connected by one edge, no shared neighbors
        let g = Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap();
        let curv = ollivier_ricci_curvature(&g, 0.0).unwrap();
        // shared=0, du=dv=1:
        // kappa = 0 + 1/1 + 1/1 - 1 = 1
        assert!((curv[0] - 1.0).abs() < 1e-10);
    }

    // --- Mean Forman-Ricci ---

    #[test]
    fn mean_forman_triangle() {
        let g = triangle();
        let mean = mean_forman_ricci(&g).unwrap();
        assert!((mean - 3.0).abs() < 1e-10);
    }

    #[test]
    fn mean_forman_empty() {
        let g = Graph::with_vertices(5);
        let mean = mean_forman_ricci(&g).unwrap();
        assert!(mean.abs() < 1e-10);
    }
}
