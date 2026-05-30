//! Lune-based β-skeleton of a spatial point set (ALGO-GEO-006).
//!
//! Counterpart of `igraph_lune_beta_skeleton()` from
//! `references/igraph/src/spatial/beta_skeleton.cpp:444`.
//!
//! For a parameter `β > 0`, two points `A` and `B` are connected when the
//! *lune* of the edge `A-B` contains no other point. The lune is the
//! intersection of two equal balls whose radius and centres are governed by
//! `β`:
//!
//! - For `β ≥ 1` (any dimension) the balls have radius `β/2 · dist(A, B)`
//!   and centres `A + (r−1)(A−B)` and `B + (r−1)(B−A)` with `r = β/2`,
//!   sliding apart along the line `AB` as `β` grows. `β = 1` collapses both
//!   centres onto the midpoint, recovering the [Gabriel
//!   graph](crate::gabriel_graph).
//! - For `β < 1` (2D only) the two centres sit symmetrically off the line
//!   `AB`, perpendicular to it, with `r = 1/(2β)`; larger lunes than the
//!   Gabriel ball, so a denser graph.
//!
//! A larger `β` enlarges the empty region and so yields a sparser graph.
//! The boundary is **closed** (matching the reference's `is_closed = true`):
//! a point sitting exactly on the lune boundary counts as inside and removes
//! the edge.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Closed-region tolerance, mirroring the C reference's
/// `#define TOLERANCE (128 * DBL_EPSILON)`. The squared ball radius is
/// inflated by `(1 + TOLERANCE)²` so a point on the exact boundary falls
/// strictly inside and removes the edge (`is_closed = true`).
const TOLERANCE: f64 = 128.0 * f64::EPSILON;

/// Build the lune-based β-skeleton of a point set.
///
/// `points` holds one row per point with a shared, arbitrary dimensionality
/// (inferred from the first row). `beta` is the skeleton parameter. The
/// result is an undirected [`Graph`] on `points.len()` vertices.
///
/// This is an `O(n²·d)` candidate enumeration with an `O(n·d)` empty-lune
/// test per pair. For `β ≥ 1` the β-skeleton is a subgraph of the Delaunay
/// triangulation, so testing every pair yields the same edge set as the
/// reference's Delaunay-pruned candidate superset; for `β < 1` the reference
/// itself starts from the complete graph, which this matches directly.
///
/// # Errors
///
/// - [`IgraphError::InvalidArgument`] if `beta` is not a finite positive
///   number, if the points are zero-dimensional (with at least one point),
///   or if the rows have inconsistent dimensionality.
/// - [`IgraphError::Unsupported`] if `beta < 1` and the points are not
///   2-dimensional (the perpendicular-centre construction is only defined in
///   2D, matching the reference's `IGRAPH_UNIMPLEMENTED`).
///
/// # Examples
///
/// ```
/// use rust_igraph::lune_beta_skeleton;
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
/// let g = lune_beta_skeleton(&pts, 1.0).unwrap();
/// assert_eq!(g.vcount(), 4);
/// assert_eq!(g.ecount(), 4); // sides only
/// ```
// Single-character names (`a`, `b`, `c` for the three points, `d` for the
// dimension axis) are the natural geometric notation here.
#[allow(clippy::many_single_char_names)]
pub fn lune_beta_skeleton(points: &[Vec<f64>], beta: f64) -> IgraphResult<Graph> {
    if !beta.is_finite() || beta <= 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "lune_beta_skeleton: beta must be a finite positive number, got {beta}"
        )));
    }

    let n = points.len();
    let dim = if n == 0 { 0 } else { points[0].len() };
    if n > 0 {
        if dim == 0 {
            return Err(IgraphError::InvalidArgument(
                "lune_beta_skeleton: points must not be zero-dimensional".into(),
            ));
        }
        for (i, row) in points.iter().enumerate().skip(1) {
            if row.len() != dim {
                return Err(IgraphError::InvalidArgument(format!(
                    "lune_beta_skeleton: point row {i} has dimension {} but expected {dim}",
                    row.len()
                )));
            }
        }
    }

    if beta < 1.0 && n > 0 && dim != 2 {
        return Err(IgraphError::Unsupported(
            "lune_beta_skeleton: beta < 1 is only supported in 2 dimensions",
        ));
    }

    let n_u32 = u32::try_from(n)
        .map_err(|_| IgraphError::InvalidArgument("lune_beta_skeleton: too many points".into()))?;
    let mut graph = Graph::with_vertices(n_u32);

    let r = if beta < 1.0 { 0.5 / beta } else { 0.5 * beta };
    let tol_sq = (1.0 + TOLERANCE) * (1.0 + TOLERANCE);
    // For β < 1 the perpendicular offset scales by sqrt(r² − 1/4); r > 1/2
    // there, so the argument is non-negative.
    let perp_scale = if beta < 1.0 {
        (r * r - 0.25).max(0.0).sqrt()
    } else {
        0.0
    };

    let mut a_center = vec![0.0_f64; dim];
    let mut b_center = vec![0.0_f64; dim];

    for i in 0..n {
        let a = &points[i];
        for j in (i + 1)..n {
            let b = &points[j];

            let mut dist_sq = 0.0_f64;
            for d in 0..dim {
                let t = a[d] - b[d];
                dist_sq += t * t;
            }

            if beta < 1.0 {
                // 2D perpendicular construction (dim == 2 enforced above).
                let mid0 = 0.5 * (a[0] + b[0]);
                let mid1 = 0.5 * (a[1] + b[1]);
                // perp = (a − b) · sqrt(r² − 1/4), rotated 90° CCW: (x,y) → (−y,x).
                let perp0 = -(a[1] - b[1]) * perp_scale;
                let perp1 = (a[0] - b[0]) * perp_scale;
                a_center[0] = mid0 + perp0;
                a_center[1] = mid1 + perp1;
                b_center[0] = mid0 - perp0;
                b_center[1] = mid1 - perp1;
            } else {
                for d in 0..dim {
                    a_center[d] = a[d] + (r - 1.0) * (a[d] - b[d]);
                    b_center[d] = b[d] + (r - 1.0) * (b[d] - a[d]);
                }
            }

            // Inflated squared ball radius; a point inside BOTH balls lies in
            // the lune and removes the edge.
            let beta_radius = dist_sq * r * r * tol_sq;

            let mut empty = true;
            for (k, c) in points.iter().enumerate() {
                if k == i || k == j {
                    continue;
                }
                let mut da = 0.0_f64;
                let mut db = 0.0_f64;
                for d in 0..dim {
                    let ta = c[d] - a_center[d];
                    da += ta * ta;
                    let tb = c[d] - b_center[d];
                    db += tb * tb;
                }
                if da < beta_radius && db < beta_radius {
                    empty = false;
                    break;
                }
            }

            if empty {
                let u = u32::try_from(i)
                    .map_err(|_| IgraphError::Internal("lune_beta_skeleton: vertex id overflow"))?;
                let v = u32::try_from(j)
                    .map_err(|_| IgraphError::Internal("lune_beta_skeleton: vertex id overflow"))?;
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

    /// The first 10 of the 25 raw values reinterpreted as 10 3-D points
    /// (`igraph_matrix_init_array(..., 10, 3, false)` column-major over the
    /// same backing array → here the same flat prefix taken 3 at a time).
    #[allow(clippy::unreadable_literal)]
    fn points_10_3d() -> Vec<Vec<f64>> {
        let flat = [
            0.474217, 0.0314797, 0.208089, 0.439308, 0.967367, 0.530466, 0.177005, 0.426713,
            0.568462, 0.57507, 0.441834, 0.284514, 0.479224, 0.817988, 0.720209, 0.225744,
            0.204941, 0.44297, 0.285318, 0.912984, 0.831097, 0.0176603, 0.827154, 0.472702,
            0.173059, 0.561858, 0.156276, 0.88019, 0.65935, 0.538207,
        ];
        flat.chunks_exact(3).map(<[f64]>::to_vec).collect()
    }

    /// A 10-point triangular lattice (verbatim from `trig_lattice_points`).
    #[allow(clippy::unreadable_literal)]
    fn triangular_lattice() -> Vec<Vec<f64>> {
        vec![
            vec![0.5, 2.598076211353316],
            vec![0.0, 1.7320508075688772],
            vec![1.0, 1.7320508075688772],
            vec![-0.5, 0.8660254037844386],
            vec![0.5, 0.8660254037844386],
            vec![1.5, 0.8660254037844386],
            vec![-1.0, 0.0],
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![2.0, 0.0],
        ]
    }

    #[test]
    fn empty_point_set() {
        let g = lune_beta_skeleton(&[], 1.0).expect("empty ok");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn single_point() {
        let g = lune_beta_skeleton(&[vec![0.5, 0.5]], 2.0).expect("singleton ok");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn two_points_always_connected() {
        let g = lune_beta_skeleton(&[vec![0.0, 0.0], vec![3.0, 4.0]], 2.0).expect("pair ok");
        assert_eq!(edge_set(&g), vec![(0, 1)]);
    }

    #[test]
    fn non_finite_beta_is_error() {
        assert!(matches!(
            lune_beta_skeleton(&[vec![0.0, 0.0]], f64::NAN).unwrap_err(),
            IgraphError::InvalidArgument(_)
        ));
        assert!(matches!(
            lune_beta_skeleton(&[vec![0.0, 0.0]], f64::INFINITY).unwrap_err(),
            IgraphError::InvalidArgument(_)
        ));
    }

    #[test]
    fn non_positive_beta_is_error() {
        assert!(matches!(
            lune_beta_skeleton(&[vec![0.0, 0.0]], 0.0).unwrap_err(),
            IgraphError::InvalidArgument(_)
        ));
        assert!(matches!(
            lune_beta_skeleton(&[vec![0.0, 0.0]], -1.0).unwrap_err(),
            IgraphError::InvalidArgument(_)
        ));
    }

    #[test]
    fn zero_dimensional_error() {
        let err = lune_beta_skeleton(&[vec![], vec![]], 1.0).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn inconsistent_dimensions_error() {
        let err = lune_beta_skeleton(&[vec![0.0, 0.0], vec![1.0]], 1.0).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn small_beta_non_2d_is_unsupported() {
        let pts = vec![vec![0.0, 0.0, 0.0], vec![1.0, 1.0, 1.0]];
        let err = lune_beta_skeleton(&pts, 0.5).unwrap_err();
        assert!(matches!(err, IgraphError::Unsupported(_)));
    }

    #[test]
    fn beta_one_equals_gabriel() {
        // β = 1 is exactly the Gabriel graph; cross-check the dedicated impl.
        let pts = points_25();
        let lune = lune_beta_skeleton(&pts, 1.0).expect("lune ok");
        let gabriel = crate::gabriel_graph(&pts).expect("gabriel ok");
        assert_eq!(edge_set(&lune), edge_set(&gabriel));
    }

    // --- Authentic igraph C anchors (beta_skeletons.out) ---

    #[allow(clippy::unreadable_literal)]
    #[test]
    fn c_anchor_beta2_25_points() {
        let g = lune_beta_skeleton(&points_25(), 2.0).expect("ok");
        let expected = vec![
            (0, 5),
            (0, 22),
            (1, 8),
            (1, 18),
            (2, 16),
            (2, 21),
            (3, 8),
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
            (11, 21),
            (11, 23),
            (12, 13),
            (14, 15),
            (14, 23),
            (16, 17),
            (18, 19),
            (19, 20),
        ];
        assert_eq!(edge_set(&g), expected);
    }

    #[allow(clippy::unreadable_literal)]
    #[test]
    fn c_anchor_gabriel_25_points() {
        let g = lune_beta_skeleton(&points_25(), 1.0).expect("ok");
        let expected = vec![
            (0, 5),
            (0, 19),
            (0, 20),
            (0, 22),
            (1, 8),
            (1, 15),
            (1, 18),
            (1, 19),
            (2, 16),
            (2, 21),
            (3, 8),
            (3, 24),
            (4, 6),
            (4, 12),
            (4, 14),
            (4, 15),
            (4, 17),
            (5, 7),
            (5, 15),
            (5, 18),
            (6, 9),
            (6, 12),
            (6, 17),
            (7, 22),
            (7, 23),
            (8, 12),
            (9, 13),
            (10, 22),
            (11, 16),
            (11, 21),
            (11, 23),
            (12, 13),
            (12, 24),
            (14, 15),
            (14, 17),
            (14, 23),
            (16, 17),
            (18, 19),
            (19, 20),
        ];
        assert_eq!(edge_set(&g), expected);
    }

    #[allow(clippy::unreadable_literal)]
    #[test]
    fn c_anchor_beta2_10_points_3d() {
        let g = lune_beta_skeleton(&points_10_3d(), 2.0).expect("ok");
        let expected = vec![
            (0, 3),
            (0, 5),
            (1, 4),
            (1, 7),
            (2, 4),
            (2, 5),
            (2, 8),
            (3, 8),
            (3, 9),
            (4, 6),
            (4, 9),
            (7, 8),
        ];
        assert_eq!(edge_set(&g), expected);
    }

    #[test]
    fn c_anchor_beta2_triangular_lattice_is_empty() {
        // Closed-boundary witness: at β = 2 every lattice lune boundary hosts
        // a third point, so all candidate edges drop and the graph is empty.
        let g = lune_beta_skeleton(&triangular_lattice(), 2.0).expect("ok");
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 0);
    }
}
