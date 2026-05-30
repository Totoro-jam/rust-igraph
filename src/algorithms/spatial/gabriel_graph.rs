//! Gabriel graph of a spatial point set (ALGO-GEO-003).
//!
//! Counterpart of `igraph_gabriel_graph()` from
//! `references/igraph/src/spatial/beta_skeleton.cpp:744`.
//!
//! In the Gabriel graph of a point set, two points `A` and `B` are
//! connected if and only if no other point `C` lies inside the closed ball
//! that has the segment `AB` as a diameter. Equivalently, the edge `A-B`
//! is kept when every other point `C` satisfies
//! `dist²(C, midpoint(A, B)) >= (dist(A, B) / 2)²`.
//!
//! The Gabriel graph is the lune-based β-skeleton with `β = 1`; it is
//! connected and, in 2D, planar. Arbitrary dimensionality is supported.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Closed-ball tolerance, mirroring the C reference's
/// `#define TOLERANCE (128 * DBL_EPSILON)`. A point that sits exactly on
/// the diametral sphere counts as *inside* (closed ball), so the boundary
/// removes the edge — matching `igraph`'s `is_closed = true` filter and
/// keeping degenerate lattices (e.g. a square grid, whose cell corners are
/// co-circular) free of spurious diagonals.
const TOLERANCE: f64 = 128.0 * f64::EPSILON;

/// Build the Gabriel graph of a point set.
///
/// `points` holds one row per point with a shared, arbitrary
/// dimensionality (inferred from the first row). The result is an
/// undirected [`Graph`] on `points.len()` vertices whose edges are exactly
/// the Gabriel-adjacent pairs.
///
/// This is an `O(n²·d)` candidate enumeration with an `O(n·d)` empty-ball
/// test per pair (`O(n³·d)` overall), which reproduces the reference
/// filter exactly: the Gabriel graph is a subgraph of the Delaunay
/// triangulation, so testing every pair yields the same edge set as the
/// reference's Delaunay-pruned candidate superset.
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
/// use rust_igraph::gabriel_graph;
///
/// // The four corners of a unit square: the Gabriel graph keeps the four
/// // sides and drops both diagonals (each diagonal's midpoint has the two
/// // opposite corners sitting on the diametral circle).
/// let pts = vec![
///     vec![0.0, 0.0], // 0
///     vec![1.0, 0.0], // 1
///     vec![0.0, 1.0], // 2
///     vec![1.0, 1.0], // 3
/// ];
/// let g = gabriel_graph(&pts).unwrap();
/// assert_eq!(g.vcount(), 4);
/// assert_eq!(g.ecount(), 4); // sides only, no diagonals
/// ```
// Single-character names (`a`, `b`, `c` for the three points, `d` for the
// dimension axis) are the natural geometric notation here.
#[allow(clippy::many_single_char_names)]
pub fn gabriel_graph(points: &[Vec<f64>]) -> IgraphResult<Graph> {
    let n = points.len();

    let dim = if n == 0 { 0 } else { points[0].len() };
    if n > 0 {
        if dim == 0 {
            return Err(IgraphError::InvalidArgument(
                "gabriel_graph: points must not be zero-dimensional".into(),
            ));
        }
        for (i, row) in points.iter().enumerate().skip(1) {
            if row.len() != dim {
                return Err(IgraphError::InvalidArgument(format!(
                    "gabriel_graph: point row {i} has dimension {} but expected {dim}",
                    row.len()
                )));
            }
        }
    }

    let n_u32 = u32::try_from(n)
        .map_err(|_| IgraphError::InvalidArgument("gabriel_graph: too many points".into()))?;
    let mut graph = Graph::with_vertices(n_u32);

    // (1 + TOLERANCE)² scaling applied to the squared radius, matching the
    // reference `beta_radius = sqr_dist * r² * tol²` with `r = 0.5`.
    let tol_sq = (1.0 + TOLERANCE) * (1.0 + TOLERANCE);

    for i in 0..n {
        let a = &points[i];
        for j in (i + 1)..n {
            let b = &points[j];

            let mut dist_sq = 0.0_f64;
            for d in 0..dim {
                let t = a[d] - b[d];
                dist_sq += t * t;
            }
            // Squared radius of the diametral ball, inflated by tol² so a
            // boundary point falls strictly inside and removes the edge.
            let radius_sq = dist_sq * 0.25 * tol_sq;

            let mut empty = true;
            for (k, c) in points.iter().enumerate() {
                if k == i || k == j {
                    continue;
                }
                let mut to_mid_sq = 0.0_f64;
                for d in 0..dim {
                    let mid = 0.5 * (a[d] + b[d]);
                    let t = c[d] - mid;
                    to_mid_sq += t * t;
                }
                if to_mid_sq < radius_sq {
                    empty = false;
                    break;
                }
            }

            if empty {
                let u = u32::try_from(i)
                    .map_err(|_| IgraphError::Internal("gabriel_graph: vertex id overflow"))?;
                let v = u32::try_from(j)
                    .map_err(|_| IgraphError::Internal("gabriel_graph: vertex id overflow"))?;
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
        let g = gabriel_graph(&[]).expect("empty ok");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn single_point() {
        let g = gabriel_graph(&[vec![0.5, 0.5]]).expect("singleton ok");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn two_points_always_connected() {
        let g = gabriel_graph(&[vec![0.0, 0.0], vec![3.0, 4.0]]).expect("pair ok");
        assert_eq!(g.vcount(), 2);
        assert_eq!(edge_set(&g), vec![(0, 1)]);
    }

    #[test]
    fn unit_square_keeps_sides_drops_diagonals() {
        let pts = vec![
            vec![0.0, 0.0], // 0
            vec![1.0, 0.0], // 1
            vec![0.0, 1.0], // 2
            vec![1.0, 1.0], // 3
        ];
        let g = gabriel_graph(&pts).expect("square ok");
        assert_eq!(edge_set(&g), vec![(0, 1), (0, 2), (1, 3), (2, 3)]);
    }

    #[test]
    fn collinear_midpoint_breaks_edge() {
        // Three collinear points: the outer pair's midpoint coincides with
        // the middle point, so the long edge is removed; the two short
        // edges remain (a path).
        let pts = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![2.0, 0.0]];
        let g = gabriel_graph(&pts).expect("collinear ok");
        assert_eq!(edge_set(&g), vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn equilateral_triangle_is_complete() {
        // No point lies inside any side's diametral ball, so all three
        // edges survive.
        let h = 3.0_f64.sqrt() / 2.0;
        let pts = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![0.5, h]];
        let g = gabriel_graph(&pts).expect("triangle ok");
        assert_eq!(edge_set(&g), vec![(0, 1), (0, 2), (1, 2)]);
    }

    #[test]
    fn three_dimensional_points() {
        // A unit cube face vs. its space diagonal: the two endpoints of a
        // 3D pair whose midpoint hosts no other point stay connected.
        let pts = vec![vec![0.0, 0.0, 0.0], vec![1.0, 1.0, 1.0]];
        let g = gabriel_graph(&pts).expect("3d ok");
        assert_eq!(edge_set(&g), vec![(0, 1)]);
    }

    #[test]
    fn zero_dimensional_error() {
        let err = gabriel_graph(&[vec![], vec![]]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn inconsistent_dimensions_error() {
        let err = gabriel_graph(&[vec![0.0, 0.0], vec![1.0]]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    // Coordinates are transcribed verbatim from igraph's own
    // `beta_skeletons.c`; underscores would obscure that they are an
    // authentic anchor, so the readability lint is waived for this fixture.
    #[allow(clippy::unreadable_literal)]
    #[test]
    fn rotated_square_lattice_c_anchor() {
        // Authentic igraph C anchor (beta_skeletons.out, "Gabriel graph of
        // rotated square lattice"): a rotated 4x4 grid whose Gabriel graph
        // is exactly the grid adjacency — no diagonals, because each cell's
        // opposite corners are co-circular (closed-ball boundary).
        let pts = vec![
            vec![-0.3594924531727418, 1.3677591805986329],
            vec![-1.2231182700584293, 1.8718925443115786],
            vec![-2.0867440869441167, 2.3760259080245243],
            vec![-2.950369903829804, 2.8801592717374698],
            vec![0.14464091054020378, 2.2313849974843203],
            vec![-0.7189849063454836, 2.7355183611972658],
            vec![-1.582610723231171, 3.2396517249102117],
            vec![-2.4462365401168586, 3.743785088623157],
            vec![0.6487742742531495, 3.0950108143700077],
            vec![-0.21485154263253792, 3.5991441780829536],
            vec![-1.0784773595182253, 4.103277541795899],
            vec![-1.9421031764039127, 4.607410905508845],
            vec![1.152907637966095, 3.958636631255695],
            vec![0.28928182108040756, 4.462769994968641],
            vec![-0.5743439958052798, 4.966903358681586],
            vec![-1.4379698126909672, 5.4710367223945315],
        ];
        let g = gabriel_graph(&pts).expect("lattice ok");
        let expected: Vec<(u32, u32)> = vec![
            (0, 1),
            (0, 4),
            (1, 2),
            (1, 5),
            (2, 3),
            (2, 6),
            (3, 7),
            (4, 5),
            (4, 8),
            (5, 6),
            (5, 9),
            (6, 7),
            (6, 10),
            (7, 11),
            (8, 9),
            (8, 12),
            (9, 10),
            (9, 13),
            (10, 11),
            (10, 14),
            (11, 15),
            (12, 13),
            (13, 14),
            (14, 15),
        ];
        assert_eq!(edge_set(&g), expected);
    }
}
