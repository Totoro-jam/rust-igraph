//! Delaunay triangulation graph of a spatial point set (ALGO-GEO-009).
//!
//! Pure-Rust counterpart of `igraph_delaunay_graph()` from
//! `references/igraph/src/spatial/delaunay.c`. The C version delegates to
//! Qhull for ≥2D; this port hand-rolls the 1D case (sorted adjacency) and
//! the 2D case (Bowyer-Watson incremental insertion) to stay dependency-free.
//! Dimensionality ≥3 is not currently supported and returns an error.
//!
//! The Delaunay graph of a point set connects two points with an edge
//! whenever they share a face in the Delaunay triangulation: no other point
//! lies inside the circumscribed circle of any triangle.
//!
//! Reference:
//! Bowyer, A. "Computing Dirichlet Tessellations." *Computer Journal*
//! 24.2 (1981): 162–166.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Build the Delaunay triangulation graph of a 1D or 2D point set.
///
/// `points` holds one row per point: each inner `Vec<f64>` is a coordinate
/// vector of shared dimensionality (inferred from the first row). The
/// result is an undirected [`Graph`] on `points.len()` vertices whose
/// edges are exactly the Delaunay-adjacent pairs.
///
/// Duplicate points (identical coordinates) are an error — the Delaunay
/// triangulation is undefined for duplicate sites.
///
/// # Supported dimensions
///
/// - **1D**: Points are sorted by coordinate; consecutive points in the
///   sorted order are connected. `O(n log n)`.
/// - **2D**: Bowyer-Watson incremental insertion. `O(n²)` average,
///   `O(n²)` worst case for pathological inputs.
/// - **≥3D**: Currently unsupported; returns
///   [`IgraphError::Unimplemented`].
///
/// # Errors
///
/// - [`IgraphError::InvalidArgument`] if the points are zero-dimensional
///   (and there is at least one point), if the rows have inconsistent
///   dimensionality, if any coordinate is NaN or infinite, or if there
///   are duplicate points.
/// - [`IgraphError::InvalidArgument`] for dimensionality ≥ 3 (not yet
///   supported).
///
/// # Examples
///
/// ```
/// use rust_igraph::delaunay_graph;
///
/// // Three points forming a triangle: all three pairs are connected.
/// let pts = vec![
///     vec![0.0, 0.0],
///     vec![1.0, 0.0],
///     vec![0.5, 1.0],
/// ];
/// let g = delaunay_graph(&pts).unwrap();
/// assert_eq!(g.vcount(), 3);
/// assert_eq!(g.ecount(), 3);
///
/// // 1D: four points on a line.
/// let pts1d = vec![vec![3.0], vec![1.0], vec![4.0], vec![2.0]];
/// let g1d = delaunay_graph(&pts1d).unwrap();
/// assert_eq!(g1d.vcount(), 4);
/// assert_eq!(g1d.ecount(), 3); // 1-2, 2-3, 3-4 in sorted order
/// ```
pub fn delaunay_graph(points: &[Vec<f64>]) -> IgraphResult<Graph> {
    let n = points.len();
    if n == 0 {
        return Ok(Graph::with_vertices(0));
    }

    let dim = points[0].len();
    if dim == 0 {
        return Err(IgraphError::InvalidArgument(
            "delaunay_graph: points must not be zero-dimensional".into(),
        ));
    }
    for (i, row) in points.iter().enumerate() {
        if row.len() != dim {
            return Err(IgraphError::InvalidArgument(format!(
                "delaunay_graph: point row {i} has dimension {} but expected {dim}",
                row.len()
            )));
        }
        for (j, &c) in row.iter().enumerate() {
            if c.is_nan() || c.is_infinite() {
                return Err(IgraphError::InvalidArgument(format!(
                    "delaunay_graph: coordinate [{i}][{j}] is not finite"
                )));
            }
        }
    }

    if n == 1 {
        return Ok(Graph::with_vertices(1));
    }

    match dim {
        1 => delaunay_1d(points),
        2 => {
            if n == 2 {
                let n_u32 = u32::try_from(n).map_err(|_| {
                    IgraphError::InvalidArgument("delaunay_graph: too many points".into())
                })?;
                let mut g = Graph::with_vertices(n_u32);
                g.add_edge(0, 1)?;
                return Ok(g);
            }
            delaunay_2d(points)
        }
        _ => Err(IgraphError::InvalidArgument(format!(
            "delaunay_graph: {dim}D not supported (only 1D and 2D)"
        ))),
    }
}

fn delaunay_1d(points: &[Vec<f64>]) -> IgraphResult<Graph> {
    let n = points.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        points[a][0]
            .partial_cmp(&points[b][0])
            .expect("NaN filtered")
    });

    let n_u32 = u32::try_from(n)
        .map_err(|_| IgraphError::InvalidArgument("delaunay_graph: too many points".into()))?;
    let mut g = Graph::with_vertices(n_u32);

    for w in order.windows(2) {
        let (a, b) = (w[0], w[1]);
        if (points[a][0] - points[b][0]).abs() < f64::EPSILON * 128.0 {
            return Err(IgraphError::InvalidArgument(
                "delaunay_graph: duplicate points".into(),
            ));
        }
        #[allow(clippy::cast_possible_truncation)]
        g.add_edge(a as u32, b as u32)?;
    }
    Ok(g)
}

fn is_collinear(points: &[Vec<f64>]) -> bool {
    if points.len() < 3 {
        return true;
    }
    let (x0, y0) = (points[0][0], points[0][1]);
    let (x1, y1) = (points[1][0], points[1][1]);
    let (dx, dy) = (x1 - x0, y1 - y0);
    for p in points.iter().skip(2) {
        let cross = dx * (p[1] - y0) - dy * (p[0] - x0);
        if cross.abs() > 1e-10 {
            return false;
        }
    }
    true
}

fn delaunay_collinear(points: &[Vec<f64>]) -> IgraphResult<Graph> {
    let n = points.len();
    let n_u32 = u32::try_from(n)
        .map_err(|_| IgraphError::InvalidArgument("delaunay_graph: too many points".into()))?;

    // Project onto the dominant axis and sort
    let (x0, y0) = (points[0][0], points[0][1]);
    let (x1, y1) = (points[1][0], points[1][1]);
    let (dx, dy) = (x1 - x0, y1 - y0);
    let len = (dx * dx + dy * dy).sqrt();
    let (ux, uy) = if len > 1e-15 {
        (dx / len, dy / len)
    } else {
        (1.0, 0.0)
    };

    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        let pa = ux * points[a][0] + uy * points[a][1];
        let pb = ux * points[b][0] + uy * points[b][1];
        pa.partial_cmp(&pb).expect("NaN filtered")
    });

    let mut g = Graph::with_vertices(n_u32);
    for w in order.windows(2) {
        #[allow(clippy::cast_possible_truncation)]
        g.add_edge(w[0] as u32, w[1] as u32)?;
    }
    Ok(g)
}

// --- Bowyer-Watson 2D Delaunay ---

#[derive(Clone, Copy)]
struct Pt {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy)]
struct Triangle {
    v: [usize; 3],
    alive: bool,
}

struct Circumcircle {
    cx: f64,
    cy: f64,
    r2: f64,
}

fn circumcircle(a: Pt, b: Pt, c: Pt) -> Circumcircle {
    let ax = a.x - c.x;
    let ay = a.y - c.y;
    let bx = b.x - c.x;
    let by = b.y - c.y;
    let d = 2.0 * (ax * by - ay * bx);
    if d.abs() < 1e-30 {
        return Circumcircle {
            cx: f64::INFINITY,
            cy: f64::INFINITY,
            r2: f64::INFINITY,
        };
    }
    let a2 = ax * ax + ay * ay;
    let b2 = bx * bx + by * by;
    let ux = (a2 * by - b2 * ay) / d;
    let uy = (ax * b2 - bx * a2) / d;
    Circumcircle {
        cx: ux + c.x,
        cy: uy + c.y,
        r2: ux * ux + uy * uy,
    }
}

fn in_circumcircle(cc: &Circumcircle, p: Pt) -> bool {
    if cc.r2.is_infinite() {
        return false;
    }
    let dx = p.x - cc.cx;
    let dy = p.y - cc.cy;
    dx * dx + dy * dy < cc.r2
}

fn check_duplicate_2d(points: &[Vec<f64>]) -> IgraphResult<()> {
    let n = points.len();
    let mut sorted_indices: Vec<usize> = (0..n).collect();
    sorted_indices.sort_by(|&a, &b| {
        points[a][0]
            .partial_cmp(&points[b][0])
            .expect("NaN filtered")
            .then_with(|| {
                points[a][1]
                    .partial_cmp(&points[b][1])
                    .expect("NaN filtered")
            })
    });
    for w in sorted_indices.windows(2) {
        let (a, b) = (w[0], w[1]);
        if (points[a][0] - points[b][0]).abs() < 1e-12
            && (points[a][1] - points[b][1]).abs() < 1e-12
        {
            return Err(IgraphError::InvalidArgument(
                "delaunay_graph: duplicate points".into(),
            ));
        }
    }
    Ok(())
}

#[allow(clippy::manual_midpoint)]
fn super_triangle_setup(pts: &[Pt], n: usize) -> (Vec<Pt>, Vec<Triangle>, Vec<Circumcircle>) {
    let mut lo_x = f64::INFINITY;
    let mut hi_x = f64::NEG_INFINITY;
    let mut lo_y = f64::INFINITY;
    let mut hi_y = f64::NEG_INFINITY;
    for p in pts {
        lo_x = lo_x.min(p.x);
        hi_x = hi_x.max(p.x);
        lo_y = lo_y.min(p.y);
        hi_y = hi_y.max(p.y);
    }
    let cx = (lo_x + hi_x) / 2.0;
    let cy = (lo_y + hi_y) / 2.0;

    let big_r = if n <= 20 {
        let mut max_r = 1.0_f64;
        for i in 0..n {
            for j in (i + 1)..n {
                for k in (j + 1)..n {
                    let cc = circumcircle(pts[i], pts[j], pts[k]);
                    if !cc.r2.is_infinite() {
                        max_r = max_r.max(cc.r2.sqrt());
                    }
                }
            }
        }
        3.0 * max_r
    } else {
        let mut max_d2 = 1.0_f64;
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = pts[i].x - pts[j].x;
                let dy = pts[i].y - pts[j].y;
                max_d2 = max_d2.max(dx * dx + dy * dy);
            }
        }
        100.0 * max_d2.sqrt()
    };

    let sqrt3 = 3.0_f64.sqrt();
    let sp = [
        Pt {
            x: cx - big_r * sqrt3,
            y: cy - big_r,
        },
        Pt {
            x: cx + big_r * sqrt3,
            y: cy - big_r,
        },
        Pt {
            x: cx,
            y: cy + 2.0 * big_r,
        },
    ];
    let mut all_pts: Vec<Pt> = pts.to_vec();
    all_pts.extend_from_slice(&sp);
    let tri = Triangle {
        v: [n, n + 1, n + 2],
        alive: true,
    };
    let cc = circumcircle(sp[0], sp[1], sp[2]);
    (all_pts, vec![tri], vec![cc])
}

fn bowyer_watson_insert(
    i: usize,
    all_pts: &[Pt],
    triangles: &mut Vec<Triangle>,
    circles: &mut Vec<Circumcircle>,
) {
    let p = all_pts[i];
    let mut bad = Vec::new();
    for (ti, tri) in triangles.iter().enumerate() {
        if tri.alive && in_circumcircle(&circles[ti], p) {
            bad.push(ti);
        }
    }

    let mut boundary: Vec<[usize; 2]> = Vec::new();
    for &ti in &bad {
        let tri = &triangles[ti];
        for edge_idx in 0..3 {
            let e = [tri.v[edge_idx], tri.v[(edge_idx + 1) % 3]];
            let shared = bad
                .iter()
                .any(|&oti| oti != ti && edge_in_triangle(e, &triangles[oti]));
            if !shared {
                boundary.push(e);
            }
        }
    }

    for &ti in &bad {
        triangles[ti].alive = false;
    }

    for e in &boundary {
        let new_tri = Triangle {
            v: [i, e[0], e[1]],
            alive: true,
        };
        let cc = circumcircle(all_pts[i], all_pts[e[0]], all_pts[e[1]]);
        triangles.push(new_tri);
        circles.push(cc);
    }
}

fn collect_delaunay_edges(triangles: &[Triangle], n: usize) -> IgraphResult<Graph> {
    let mut edge_set = std::collections::BTreeSet::new();
    for tri in triangles {
        if !tri.alive || tri.v[0] >= n || tri.v[1] >= n || tri.v[2] >= n {
            continue;
        }
        for i in 0..3 {
            let (a, b) = (tri.v[i], tri.v[(i + 1) % 3]);
            let edge = if a < b { (a, b) } else { (b, a) };
            edge_set.insert(edge);
        }
    }

    let n_u32 = u32::try_from(n)
        .map_err(|_| IgraphError::InvalidArgument("delaunay_graph: too many points".into()))?;
    let mut g = Graph::with_vertices(n_u32);
    for (a, b) in &edge_set {
        #[allow(clippy::cast_possible_truncation)]
        g.add_edge(*a as u32, *b as u32)?;
    }
    Ok(g)
}

fn delaunay_2d(points: &[Vec<f64>]) -> IgraphResult<Graph> {
    let n = points.len();
    check_duplicate_2d(points)?;

    if is_collinear(points) {
        return delaunay_collinear(points);
    }

    let pts: Vec<Pt> = points.iter().map(|p| Pt { x: p[0], y: p[1] }).collect();
    let (all_pts, mut triangles, mut circles) = super_triangle_setup(&pts, n);

    for i in 0..n {
        bowyer_watson_insert(i, &all_pts, &mut triangles, &mut circles);
    }

    collect_delaunay_edges(&triangles, n)
}

fn edge_in_triangle(e: [usize; 2], tri: &Triangle) -> bool {
    for i in 0..3 {
        let te = [tri.v[i], tri.v[(i + 1) % 3]];
        if (te[0] == e[0] && te[1] == e[1]) || (te[0] == e[1] && te[1] == e[0]) {
            return true;
        }
    }
    false
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn empty_points() {
        let g = delaunay_graph(&[]).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn single_point() {
        let g = delaunay_graph(&[vec![1.0, 2.0]]).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn two_points_2d() {
        let pts = vec![vec![0.0, 0.0], vec![1.0, 0.0]];
        let g = delaunay_graph(&pts).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
    }

    #[test]
    fn triangle_2d() {
        let pts = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![0.5, 1.0]];
        let g = delaunay_graph(&pts).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 3);
    }

    #[test]
    fn square_2d_has_five_edges() {
        // A unit square: the Delaunay triangulation has 4 outer edges + 1
        // diagonal = 5 edges (two triangles). Which diagonal is chosen is
        // implementation-defined but there must be exactly one.
        let pts = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![1.0, 1.0],
            vec![0.0, 1.0],
        ];
        let g = delaunay_graph(&pts).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 5);
    }

    #[test]
    fn regular_pentagon_2d() {
        let pts: Vec<Vec<f64>> = (0..5)
            .map(|i| {
                let angle = 2.0 * std::f64::consts::PI * f64::from(i) / 5.0;
                vec![angle.cos(), angle.sin()]
            })
            .collect();
        let g = delaunay_graph(&pts).unwrap();
        assert_eq!(g.vcount(), 5);
        // Convex pentagon: 5 outer edges + 2 interior diagonals = 7
        // (3 triangles, each with 3 edges, 2 shared internally: 3*3-2*2=5? No:
        // actually 3 triangles = 9 half-edges, 4 shared = 5+2=7 unique edges)
        assert_eq!(g.ecount(), 7);
    }

    #[test]
    fn line_1d() {
        let pts = vec![vec![3.0], vec![1.0], vec![4.0], vec![2.0]];
        let g = delaunay_graph(&pts).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 3);
        // Sorted order: 1(=1.0), 3(=2.0), 0(=3.0), 2(=4.0)
        // Edges: (1,3), (0,3), (0,2)
        assert!(g.get_eid(1, 3).is_ok());
        assert!(g.get_eid(0, 3).is_ok() || g.get_eid(3, 0).is_ok());
        assert!(g.get_eid(0, 2).is_ok() || g.get_eid(2, 0).is_ok());
    }

    #[test]
    fn single_point_1d() {
        let g = delaunay_graph(&[vec![5.0]]).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn two_points_1d() {
        let g = delaunay_graph(&[vec![2.0], vec![7.0]]).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
    }

    #[test]
    fn duplicate_points_1d_error() {
        let pts = vec![vec![1.0], vec![2.0], vec![1.0]];
        assert!(delaunay_graph(&pts).is_err());
    }

    #[test]
    fn duplicate_points_2d_error() {
        let pts = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![1.0, 2.0]];
        assert!(delaunay_graph(&pts).is_err());
    }

    #[test]
    fn zero_dimensional_error() {
        let pts: Vec<Vec<f64>> = vec![vec![]];
        assert!(delaunay_graph(&pts).is_err());
    }

    #[test]
    fn nan_coordinate_error() {
        let pts = vec![vec![0.0, f64::NAN]];
        assert!(delaunay_graph(&pts).is_err());
    }

    #[test]
    fn infinite_coordinate_error() {
        let pts = vec![vec![f64::INFINITY, 0.0]];
        assert!(delaunay_graph(&pts).is_err());
    }

    #[test]
    fn three_d_unsupported() {
        let pts = vec![vec![0.0, 0.0, 0.0], vec![1.0, 0.0, 0.0]];
        assert!(delaunay_graph(&pts).is_err());
    }

    #[test]
    fn collinear_2d() {
        // Three collinear points in 2D: the Delaunay triangulation
        // reduces to a path (2 edges), same as the 1D case projected.
        let pts = vec![vec![0.0, 0.0], vec![1.0, 1.0], vec![2.0, 2.0]];
        let g = delaunay_graph(&pts).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 2);
    }

    #[test]
    fn grid_3x3() {
        // 3x3 grid of points (9 points), Delaunay should produce
        // connected triangulation with euler: V - E + F = 2 for planar
        // 9 vertices, convex hull has 8 edges on boundary
        // F_inner = E - V + 1 for connected planar = E - 9 + 2 = E - 7
        // Each inner face is a triangle (3 edges, each shared by 2 faces)
        // plus outer face. For 9 points in grid: expect 16 edges.
        let mut pts = Vec::new();
        for i in 0..3 {
            for j in 0..3 {
                pts.push(vec![f64::from(i), f64::from(j)]);
            }
        }
        let g = delaunay_graph(&pts).unwrap();
        assert_eq!(g.vcount(), 9);
        // A regular 3x3 grid has 12 grid edges + 4 diagonals = 16 in
        // Delaunay. (Each of 4 interior squares gets one diagonal.)
        assert_eq!(g.ecount(), 16);
    }

    #[test]
    fn cocircular_points() {
        // Four co-circular points: one diagonal must be chosen, giving
        // exactly 5 edges. This tests the epsilon handling.
        let pts = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![1.0, 1.0],
            vec![0.0, 1.0],
        ];
        let g = delaunay_graph(&pts).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 5);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_points_2d(max_n: usize) -> impl Strategy<Value = Vec<Vec<f64>>> {
        (3..=max_n).prop_flat_map(|n| {
            proptest::collection::vec(proptest::collection::vec(-100.0_f64..100.0, 2..=2), n..=n)
        })
    }

    proptest! {
        /// Euler's formula for connected planar graphs: V - E + F = 2.
        /// For a Delaunay triangulation (convex hull = outer face), the
        /// number of triangles T satisfies E = V + T - 1 (when
        /// connected). We just check that the result is a valid graph
        /// with reasonable edge count.
        #[test]
        fn edge_count_in_planar_range(pts in arb_points_2d(8)) {
            // Skip if there are near-duplicates
            let mut sorted: Vec<(usize, &Vec<f64>)> = pts.iter().enumerate().collect();
            sorted.sort_by(|(_, a), (_, b)| {
                a[0].partial_cmp(&b[0]).unwrap()
                    .then_with(|| a[1].partial_cmp(&b[1]).unwrap())
            });
            for w in sorted.windows(2) {
                let (_, a) = w[0];
                let (_, b) = w[1];
                if (a[0] - b[0]).abs() < 1e-10 && (a[1] - b[1]).abs() < 1e-10 {
                    return Ok(());
                }
            }

            let g = delaunay_graph(&pts).unwrap();
            let n = pts.len();
            prop_assert_eq!(g.vcount() as usize, n);
            // Planar graph: E <= 3V - 6 for V >= 3
            if n >= 3 {
                prop_assert!(g.ecount() as usize <= 3 * n - 6,
                    "too many edges: {} > {}", g.ecount(), 3 * n - 6);
            }
            // At least V - 1 edges (connected spanning tree)
            prop_assert!(g.ecount() as usize >= n - 1,
                "too few edges: {} < {}", g.ecount(), n - 1);
        }
    }
}
