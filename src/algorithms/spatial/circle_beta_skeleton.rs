//! Circle-based β-skeleton of a 2-D spatial point set (ALGO-GEO-007).
//!
//! Counterpart of `igraph_circle_beta_skeleton()` from
//! `references/igraph/src/spatial/beta_skeleton.cpp:494`.
//!
//! Like the [lune-based β-skeleton](crate::lune_beta_skeleton), two points
//! `A` and `B` are connected when a β-governed empty region contains no other
//! point. The circle-based variant differs from the lune one **only for
//! `β ≥ 1`**, where it uses two circles whose centres sit perpendicular to the
//! line `AB` (rather than sliding along it) and asks whether their *union* is
//! empty:
//!
//! - For `β ≥ 1`: radius `r = β/2`, centres offset perpendicular to `AB` by
//!   `(A−B)·sqrt(r²−¼)` rotated 90°. The edge survives iff **neither** circle
//!   (radius `r·dist(A,B)`) contains another point — the *union-empty* test.
//!   `β = 1` collapses both centres onto the midpoint, recovering the [Gabriel
//!   graph](crate::gabriel_graph).
//! - For `β < 1`: identical to the lune skeleton — `r = 1/(2β)`, the same
//!   perpendicular centres, and the *intersection-empty* test (a point removes
//!   the edge only when it lies in **both** circles).
//!
//! A larger `β` enlarges the empty region and so yields a sparser graph. The
//! construction is **2-D only** in every regime (the perpendicular-centre
//! rotation is defined only in the plane), matching the reference's
//! unconditional `IGRAPH_UNIMPLEMENTED` for non-2-D input. The boundary is
//! **closed** (matching the reference's `is_closed = true`): a point on the
//! exact circle boundary counts as inside and removes the edge.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Closed-region tolerance, mirroring the C reference's
/// `#define TOLERANCE (128 * DBL_EPSILON)`.
const TOLERANCE: f64 = 128.0 * f64::EPSILON;

/// Build the circle-based β-skeleton of a 2-D point set.
///
/// `points` holds one row per point and must be 2-dimensional (inferred from
/// the first row). `beta` is the skeleton parameter. The result is an
/// undirected [`Graph`] on `points.len()` vertices.
///
/// This is an `O(n²)` candidate enumeration with an `O(n)` empty-region test
/// per pair. For `β ≥ 1` the β-skeleton is a subgraph of the Delaunay
/// triangulation, so testing every pair yields the same edge set as the
/// reference's Delaunay-pruned candidate superset; for `β < 1` the reference
/// itself starts from the complete graph, which this matches directly.
///
/// # Errors
///
/// - [`IgraphError::InvalidArgument`] if `beta` is not a finite positive
///   number, or if the rows have inconsistent dimensionality.
/// - [`IgraphError::Unsupported`] if the points are not 2-dimensional (with at
///   least one point), matching the reference's `IGRAPH_UNIMPLEMENTED`.
///
/// # Examples
///
/// ```
/// use rust_igraph::circle_beta_skeleton;
///
/// // Four corners of a unit square. At β = 1 (the Gabriel graph) the four
/// // sides survive and both diagonals drop: each diagonal's midpoint has the
/// // two opposite corners sitting on the diametral circle.
/// let pts = vec![
///     vec![0.0, 0.0],
///     vec![1.0, 0.0],
///     vec![0.0, 1.0],
///     vec![1.0, 1.0],
/// ];
/// let g = circle_beta_skeleton(&pts, 1.0).unwrap();
/// assert_eq!(g.vcount(), 4);
/// assert_eq!(g.ecount(), 4); // sides only
/// ```
// Single-character names (`a`, `b`, `c` for the three points) are the natural
// geometric notation here.
#[allow(clippy::many_single_char_names)]
#[allow(clippy::similar_names)]
pub fn circle_beta_skeleton(points: &[Vec<f64>], beta: f64) -> IgraphResult<Graph> {
    if !beta.is_finite() || beta <= 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "circle_beta_skeleton: beta must be a finite positive number, got {beta}"
        )));
    }

    let n = points.len();
    let dim = if n == 0 { 0 } else { points[0].len() };
    if n > 0 {
        for (i, row) in points.iter().enumerate().skip(1) {
            if row.len() != dim {
                return Err(IgraphError::InvalidArgument(format!(
                    "circle_beta_skeleton: point row {i} has dimension {} but expected {dim}",
                    row.len()
                )));
            }
        }
        if dim != 2 {
            return Err(IgraphError::Unsupported(
                "circle_beta_skeleton: circle-based beta skeletons are only supported in 2 dimensions",
            ));
        }
    }

    let n_u32 = u32::try_from(n).map_err(|_| {
        IgraphError::InvalidArgument("circle_beta_skeleton: too many points".into())
    })?;
    let mut graph = Graph::with_vertices(n_u32);

    let r = if beta < 1.0 { 0.5 / beta } else { 0.5 * beta };
    // Perpendicular offset scales by sqrt(r² − 1/4); r ≥ 1/2 in both regimes,
    // so the argument is non-negative.
    let perp_scale = (r * r - 0.25).max(0.0).sqrt();

    // For β ≥ 1 the union-empty test inflates the squared radius by a single
    // (1 + TOLERANCE) factor; for β < 1 the intersection-empty test (matching
    // the lune skeleton) inflates by (1 + TOLERANCE)².
    let union_mode = beta >= 1.0;
    let tol = 1.0 + TOLERANCE;
    let radius_factor = if union_mode { tol } else { tol * tol };

    let mut a_center = [0.0_f64; 2];
    let mut b_center = [0.0_f64; 2];

    for i in 0..n {
        let a = &points[i];
        for j in (i + 1)..n {
            let b = &points[j];

            let dx = a[0] - b[0];
            let dy = a[1] - b[1];
            let dist_sq = dx * dx + dy * dy;

            // Perpendicular centres (used in both regimes for this variant).
            let mid0 = 0.5 * (a[0] + b[0]);
            let mid1 = 0.5 * (a[1] + b[1]);
            // perp = (a − b) · sqrt(r² − 1/4), rotated 90° CCW: (x,y) → (−y,x).
            let perp0 = -dy * perp_scale;
            let perp1 = dx * perp_scale;
            a_center[0] = mid0 + perp0;
            a_center[1] = mid1 + perp1;
            b_center[0] = mid0 - perp0;
            b_center[1] = mid1 - perp1;

            let circle_radius = dist_sq * r * r * radius_factor;

            let mut empty = true;
            for (k, c) in points.iter().enumerate() {
                if k == i || k == j {
                    continue;
                }
                let ca0 = c[0] - a_center[0];
                let ca1 = c[1] - a_center[1];
                let da = ca0 * ca0 + ca1 * ca1;
                let cb0 = c[0] - b_center[0];
                let cb1 = c[1] - b_center[1];
                let db = cb0 * cb0 + cb1 * cb1;

                let blocks = if union_mode {
                    // Union empty: a point in EITHER circle removes the edge.
                    da < circle_radius || db < circle_radius
                } else {
                    // Intersection empty: a point in BOTH circles removes it.
                    da < circle_radius && db < circle_radius
                };
                if blocks {
                    empty = false;
                    break;
                }
            }

            if empty {
                let u = u32::try_from(i).map_err(|_| {
                    IgraphError::Internal("circle_beta_skeleton: vertex id overflow")
                })?;
                let v = u32::try_from(j).map_err(|_| {
                    IgraphError::Internal("circle_beta_skeleton: vertex id overflow")
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

    /// Collect the undirected edge set as a sorted `(min, max)` vec.
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

    /// The 25-point 2-D set transcribed verbatim from igraph's
    /// `beta_skeletons.c` (row-major, 25×2).
    #[allow(clippy::unreadable_literal)]
    fn points_25() -> Vec<Vec<f64>> {
        let flat = [
            0.474217, 0.0314797, 0.208089, 0.439308, 0.967367, 0.530466, 0.177005, 0.426713,
            0.568462, 0.57507, 0.441834, 0.284514, 0.479224, 0.817988, 0.720209, 0.225744,
            0.204941, 0.44297, 0.285318, 0.912984, 0.831097, 0.0176603, 0.827154, 0.472702,
            0.173059, 0.561858, 0.156276, 0.88019, 0.65935, 0.538207, 0.570379, 0.518081, 0.900553,
            0.656416, 0.726631, 0.863709, 0.380264, 0.287159, 0.31098, 0.230773, 0.243089,
            0.164584, 0.967974, 0.524992, 0.726605, 0.0724703, 0.739752, 0.447069, 0.0443581,
            0.444839,
        ];
        flat.chunks_exact(2).map(<[f64]>::to_vec).collect()
    }

    #[test]
    fn empty_point_set() {
        let g = circle_beta_skeleton(&[], 1.1).expect("empty ok");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn single_point() {
        let g = circle_beta_skeleton(&[vec![0.5, 0.5]], 2.0).expect("singleton ok");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn two_points_always_connected() {
        let g = circle_beta_skeleton(&[vec![0.0, 0.0], vec![3.0, 4.0]], 1.1).expect("pair ok");
        assert_eq!(edge_set(&g), vec![(0, 1)]);
    }

    #[test]
    fn non_finite_beta_is_error() {
        assert!(matches!(
            circle_beta_skeleton(&[vec![0.0, 0.0]], f64::NAN).unwrap_err(),
            IgraphError::InvalidArgument(_)
        ));
        assert!(matches!(
            circle_beta_skeleton(&[vec![0.0, 0.0]], f64::INFINITY).unwrap_err(),
            IgraphError::InvalidArgument(_)
        ));
    }

    #[test]
    fn non_positive_beta_is_error() {
        assert!(matches!(
            circle_beta_skeleton(&[vec![0.0, 0.0]], 0.0).unwrap_err(),
            IgraphError::InvalidArgument(_)
        ));
        assert!(matches!(
            circle_beta_skeleton(&[vec![0.0, 0.0]], -1.0).unwrap_err(),
            IgraphError::InvalidArgument(_)
        ));
    }

    #[test]
    fn non_2d_is_unsupported() {
        // 2-D only in every regime, unlike the lune skeleton (which allows any
        // dimension for β ≥ 1).
        let pts = vec![vec![0.0, 0.0, 0.0], vec![1.0, 1.0, 1.0]];
        assert!(matches!(
            circle_beta_skeleton(&pts, 2.0).unwrap_err(),
            IgraphError::Unsupported(_)
        ));
        assert!(matches!(
            circle_beta_skeleton(&pts, 0.5).unwrap_err(),
            IgraphError::Unsupported(_)
        ));
    }

    #[test]
    fn inconsistent_dimensions_error() {
        let err = circle_beta_skeleton(&[vec![0.0, 0.0], vec![1.0]], 1.1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn beta_one_equals_gabriel() {
        // β = 1: both perpendicular centres collapse onto the midpoint and the
        // union of the two (identical) circles is the Gabriel diametral ball.
        let pts = points_25();
        let circle = circle_beta_skeleton(&pts, 1.0).expect("circle ok");
        let gabriel = crate::gabriel_graph(&pts).expect("gabriel ok");
        assert_eq!(edge_set(&circle), edge_set(&gabriel));
    }

    // --- Authentic igraph C anchor (beta_skeletons.out) ---

    #[allow(clippy::unreadable_literal)]
    #[test]
    fn c_anchor_beta11_circle_25_points() {
        let g = circle_beta_skeleton(&points_25(), 1.1).expect("ok");
        let expected = vec![
            (0, 22),
            (1, 8),
            (2, 16),
            (2, 21),
            (3, 24),
            (4, 6),
            (4, 15),
            (5, 7),
            (5, 15),
            (5, 18),
            (6, 9),
            (6, 17),
            (7, 22),
            (7, 23),
            (8, 12),
            (9, 13),
            (10, 22),
            (11, 23),
            (12, 13),
            (14, 23),
            (16, 17),
            (18, 19),
            (19, 20),
        ];
        assert_eq!(edge_set(&g), expected);
    }
}
