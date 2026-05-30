//! Relative neighborhood graph of a spatial point set (ALGO-GEO-004).
//!
//! Counterpart of `igraph_relative_neighborhood_graph()` from
//! `references/igraph/src/spatial/beta_skeleton.cpp:798`.
//!
//! In the relative neighborhood graph (RNG) two points `A` and `B` are
//! connected if and only if no other point `C` is *strictly* closer to
//! both of them than they are to each other:
//! `dist(A, B) <= max(dist(C, A), dist(C, B))` for every other point `C`.
//! Equivalently, the open lune (the intersection of the two open balls of
//! radius `dist(A, B)` centred at `A` and at `B`) contains no other point.
//!
//! The RNG is the **open** lune-based β-skeleton with `β = 2`. Because the
//! lune is open, a point sitting exactly on its boundary — at distance
//! `dist(A, B)` from both endpoints, as happens for every edge of a
//! triangular lattice — does *not* remove the edge. This is the sole
//! difference from the closed β = 2 lune skeleton (which would delete those
//! edges) and the reason a triangular lattice keeps its full edge set under
//! the RNG. The RNG is a supergraph of the Euclidean minimum spanning tree
//! and a subgraph of the Gabriel graph, hence connected; arbitrary
//! dimensionality is supported.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Open-lune tolerance, mirroring the C reference's
/// `#define TOLERANCE (128 * DBL_EPSILON)`. For the open lune the squared
/// radius is *deflated* by `(1 - TOLERANCE)²` so a point lying exactly on
/// the lune boundary falls strictly outside and the edge is kept — matching
/// `igraph`'s `is_closed = false` filter and leaving a triangular lattice
/// (whose edges all have a co-equidistant third point) fully connected.
const TOLERANCE: f64 = 128.0 * f64::EPSILON;

/// Build the relative neighborhood graph of a point set.
///
/// `points` holds one row per point with a shared, arbitrary
/// dimensionality (inferred from the first row). The result is an
/// undirected [`Graph`] on `points.len()` vertices whose edges are exactly
/// the relatively-neighborly pairs.
///
/// This is an `O(n²·d)` candidate enumeration with an `O(n·d)` empty-lune
/// test per pair (`O(n³·d)` overall), which reproduces the reference filter
/// exactly: the RNG is a subgraph of the Delaunay triangulation, so testing
/// every pair yields the same edge set as the reference's Delaunay-pruned
/// candidate superset.
///
/// # Errors
///
/// - [`IgraphError::InvalidArgument`] if the points are zero-dimensional
///   (and there is at least one point).
/// - [`IgraphError::InvalidArgument`] if the rows have inconsistent
///   dimensionality.
///
/// # Examples
///
/// ```
/// use rust_igraph::relative_neighborhood_graph;
///
/// // An equilateral triangle: every pair is mutually nearest, and the
/// // third vertex sits exactly on the open lune's boundary, so all three
/// // edges survive (the open lune distinguishes the RNG from the closed
/// // β = 2 skeleton, which would delete them).
/// let h = 3.0_f64.sqrt() / 2.0;
/// let pts = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![0.5, h]];
/// let g = relative_neighborhood_graph(&pts).unwrap();
/// assert_eq!(g.vcount(), 3);
/// assert_eq!(g.ecount(), 3); // complete triangle
/// ```
// Single-character names (`a`, `b`, `c` for the three points, `d` for the
// dimension axis) are the natural geometric notation here; `to_a_sq` /
// `to_b_sq` are the squared distances from a candidate to each endpoint.
#[allow(clippy::many_single_char_names, clippy::similar_names)]
pub fn relative_neighborhood_graph(points: &[Vec<f64>]) -> IgraphResult<Graph> {
    let n = points.len();

    let dim = if n == 0 { 0 } else { points[0].len() };
    if n > 0 {
        if dim == 0 {
            return Err(IgraphError::InvalidArgument(
                "relative_neighborhood_graph: points must not be zero-dimensional".into(),
            ));
        }
        for (i, row) in points.iter().enumerate().skip(1) {
            if row.len() != dim {
                return Err(IgraphError::InvalidArgument(format!(
                    "relative_neighborhood_graph: point row {i} has dimension {} but expected {dim}",
                    row.len()
                )));
            }
        }
    }

    let n_u32 = u32::try_from(n).map_err(|_| {
        IgraphError::InvalidArgument("relative_neighborhood_graph: too many points".into())
    })?;
    let mut graph = Graph::with_vertices(n_u32);

    // (1 - TOLERANCE)² deflation of the squared lune radius, matching the
    // reference `beta_radius = sqr_dist * r² * tol²` with `r = 1` (β = 2)
    // and the open-lune `tol = 1 - TOLERANCE`.
    let tol = 1.0 - TOLERANCE;
    let tol_sq = tol * tol;

    for i in 0..n {
        let a = &points[i];
        for j in (i + 1)..n {
            let b = &points[j];

            let mut dist_sq = 0.0_f64;
            for d in 0..dim {
                let t = a[d] - b[d];
                dist_sq += t * t;
            }
            // Squared lune radius (= dist² for β = 2), deflated by tol² so a
            // boundary point falls strictly outside and keeps the edge.
            let threshold = dist_sq * tol_sq;

            let mut empty = true;
            for (k, c) in points.iter().enumerate() {
                if k == i || k == j {
                    continue;
                }
                let mut to_a_sq = 0.0_f64;
                let mut to_b_sq = 0.0_f64;
                for d in 0..dim {
                    let ta = c[d] - a[d];
                    to_a_sq += ta * ta;
                    let tb = c[d] - b[d];
                    to_b_sq += tb * tb;
                }
                if to_a_sq < threshold && to_b_sq < threshold {
                    empty = false;
                    break;
                }
            }

            if empty {
                let u = u32::try_from(i).map_err(|_| {
                    IgraphError::Internal("relative_neighborhood_graph: vertex id overflow")
                })?;
                let v = u32::try_from(j).map_err(|_| {
                    IgraphError::Internal("relative_neighborhood_graph: vertex id overflow")
                })?;
                graph.add_edge(u, v)?;
            }
        }
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Collect the undirected edge set as a sorted `(min, max)` vec for
    /// order-independent comparison.
    fn edge_set(g: &Graph) -> Vec<(u32, u32)> {
        let mut edges: Vec<(u32, u32)> = (0..g.ecount())
            .map(|e| {
                let (u, v) = g
                    .edge(u32::try_from(e).expect("edge id fits in u32"))
                    .expect("edge");
                (u.min(v), u.max(v))
            })
            .collect();
        edges.sort_unstable();
        edges
    }

    #[test]
    fn empty_point_set() {
        let g = relative_neighborhood_graph(&[]).expect("empty ok");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn single_point() {
        let g = relative_neighborhood_graph(&[vec![0.5, 0.5]]).expect("singleton ok");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn two_points_always_connected() {
        let g = relative_neighborhood_graph(&[vec![0.0, 0.0], vec![3.0, 4.0]]).expect("pair ok");
        assert_eq!(g.vcount(), 2);
        assert_eq!(edge_set(&g), vec![(0, 1)]);
    }

    #[test]
    fn equilateral_triangle_open_lune_keeps_all() {
        // Each vertex sits exactly on the opposite edge's open-lune
        // boundary, so the OPEN lune keeps all three edges (the closed
        // β = 2 lune would delete them). This is the defining RNG case.
        let h = 3.0_f64.sqrt() / 2.0;
        let pts = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![0.5, h]];
        let g = relative_neighborhood_graph(&pts).expect("triangle ok");
        assert_eq!(edge_set(&g), vec![(0, 1), (0, 2), (1, 2)]);
    }

    #[test]
    fn collinear_midpoint_breaks_long_edge() {
        // Three collinear points: the middle point is strictly closer to
        // both ends than they are to each other, so the long edge is
        // removed; the two short edges remain (a path).
        let pts = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![2.0, 0.0]];
        let g = relative_neighborhood_graph(&pts).expect("collinear ok");
        assert_eq!(edge_set(&g), vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn unit_square_keeps_sides_drops_diagonals() {
        let pts = vec![
            vec![0.0, 0.0], // 0
            vec![1.0, 0.0], // 1
            vec![0.0, 1.0], // 2
            vec![1.0, 1.0], // 3
        ];
        let g = relative_neighborhood_graph(&pts).expect("square ok");
        // Each diagonal's opposite corner is strictly closer to both
        // endpoints (dist 1 < diagonal √2), so both diagonals go.
        assert_eq!(edge_set(&g), vec![(0, 1), (0, 2), (1, 3), (2, 3)]);
    }

    #[test]
    fn three_dimensional_points() {
        let pts = vec![vec![0.0, 0.0, 0.0], vec![1.0, 1.0, 1.0]];
        let g = relative_neighborhood_graph(&pts).expect("3d ok");
        assert_eq!(edge_set(&g), vec![(0, 1)]);
    }

    #[test]
    fn zero_dimensional_error() {
        let err = relative_neighborhood_graph(&[vec![], vec![]]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn inconsistent_dimensions_error() {
        let err = relative_neighborhood_graph(&[vec![0.0, 0.0], vec![1.0]]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    // Coordinates are transcribed verbatim from igraph's own
    // `beta_skeletons.c` (`trig_lattice_points`); underscores would obscure
    // that they are an authentic anchor, and the extra digits beyond f64
    // precision are kept to match the source byte-for-byte, so both
    // readability lints are waived.
    #[allow(clippy::unreadable_literal, clippy::excessive_precision)]
    #[test]
    fn triangular_lattice_c_anchor() {
        // Authentic igraph C anchor (beta_skeletons.out, "Relative
        // neighborhood graph, triangular lattice"): a 10-point triangular
        // lattice whose RNG keeps all 18 lattice edges. Every edge has a
        // third lattice point exactly equidistant (on the open-lune
        // boundary), so only the OPEN lune retains them — the closed β = 2
        // skeleton on the same points is empty.
        let pts = vec![
            vec![0.5, 2.5980762113533159402911695122588085504142078807156],
            vec![0.0, 1.7320508075688772935274463415058723669428052538104],
            vec![1.0, 1.7320508075688772935274463415058723669428052538104],
            vec![-0.5, 0.86602540378443864676372317075293618347140262690519],
            vec![0.5, 0.86602540378443864676372317075293618347140262690519],
            vec![1.5, 0.86602540378443864676372317075293618347140262690519],
            vec![-1.0, 0.0],
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![2.0, 0.0],
        ];
        let g = relative_neighborhood_graph(&pts).expect("lattice ok");
        let expected: Vec<(u32, u32)> = vec![
            (0, 1),
            (0, 2),
            (1, 2),
            (1, 3),
            (1, 4),
            (2, 4),
            (2, 5),
            (3, 4),
            (3, 6),
            (3, 7),
            (4, 5),
            (4, 7),
            (4, 8),
            (5, 8),
            (5, 9),
            (6, 7),
            (7, 8),
            (8, 9),
        ];
        assert_eq!(g.vcount(), 10);
        assert_eq!(edge_set(&g), expected);
    }
}
