//! 2D convex hull via Graham scan (ALGO-GEO-001).
//!
//! Counterpart of `igraph_convex_hull_2d()` from
//! `references/igraph/src/spatial/convex_hull.c`.
//!
//! Computes the convex hull of a set of 2D points using the Graham
//! scan algorithm (Cormen et al., Introduction to Algorithms, §33.3).
//! Returns the vertex indices of the hull in counter-clockwise order.

use crate::core::{IgraphError, IgraphResult};

/// Result of [`convex_hull_2d`].
#[derive(Debug, Clone, PartialEq)]
pub struct ConvexHullResult {
    /// Indices of the input points that form the convex hull,
    /// in counter-clockwise order starting from the bottom-most
    /// (then left-most) point.
    pub hull_vertices: Vec<usize>,
    /// Coordinates of the hull vertices in the same order.
    pub hull_coords: Vec<(f64, f64)>,
}

/// Compute the convex hull of a set of 2D points.
///
/// Uses the Graham scan algorithm. The input is a slice of `(x, y)`
/// coordinate pairs. Returns the indices and coordinates of the hull
/// vertices in counter-clockwise order.
///
/// # Errors
///
/// - `InvalidArgument` if any coordinate is NaN.
///
/// # Examples
///
/// ```
/// use rust_igraph::convex_hull_2d;
///
/// let points = vec![(0.0, 0.0), (1.0, 0.0), (0.5, 1.0), (0.5, 0.5)];
/// let hull = convex_hull_2d(&points).unwrap();
/// // The interior point (0.5, 0.5) is not on the hull.
/// assert_eq!(hull.hull_vertices.len(), 3);
/// ```
pub fn convex_hull_2d(points: &[(f64, f64)]) -> IgraphResult<ConvexHullResult> {
    let n = points.len();

    if n == 0 {
        return Ok(ConvexHullResult {
            hull_vertices: Vec::new(),
            hull_coords: Vec::new(),
        });
    }

    for (i, &(x, y)) in points.iter().enumerate() {
        if x.is_nan() || y.is_nan() {
            return Err(IgraphError::InvalidArgument(format!(
                "convex_hull_2d: NaN coordinate at index {i}"
            )));
        }
    }

    if n <= 2 {
        let hull_vertices: Vec<usize> = (0..n).collect();
        let hull_coords: Vec<(f64, f64)> = points.to_vec();
        return Ok(ConvexHullResult {
            hull_vertices,
            hull_coords,
        });
    }

    // Find pivot: lowest y, then leftmost x as tiebreaker.
    let mut pivot_idx: usize = 0;
    for i in 1..n {
        let cmp_y = points[i].1.total_cmp(&points[pivot_idx].1);
        let cmp_x = points[i].0.total_cmp(&points[pivot_idx].0);
        if cmp_y == std::cmp::Ordering::Less
            || (cmp_y == std::cmp::Ordering::Equal && cmp_x == std::cmp::Ordering::Less)
        {
            pivot_idx = i;
        }
    }

    let px = points[pivot_idx].0;
    let py = points[pivot_idx].1;

    // Compute polar angles relative to pivot.
    let mut angles: Vec<f64> = Vec::with_capacity(n);
    for (i, &(x, y)) in points.iter().enumerate() {
        if i == pivot_idx {
            angles.push(f64::NEG_INFINITY);
        } else {
            angles.push(f64::atan2(y - py, x - px));
        }
    }

    // Sort indices by angle; for same angle, keep farthest from pivot.
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        angles[a].total_cmp(&angles[b]).then_with(|| {
            let da = dist_sq(px, py, points[a].0, points[a].1);
            let db = dist_sq(px, py, points[b].0, points[b].1);
            da.total_cmp(&db)
        })
    });

    // Remove collinear points with same angle — keep only the farthest.
    let mut filtered: Vec<usize> = Vec::with_capacity(n);
    let mut i = 0;
    while i < order.len() {
        let mut j = i;
        while j + 1 < order.len()
            && angles[order[j + 1]].total_cmp(&angles[order[i]]) == std::cmp::Ordering::Equal
        {
            j += 1;
        }
        // Among order[i..=j] with same angle, keep the last (farthest due to sort).
        filtered.push(order[j]);
        i = j + 1;
    }

    if filtered.len() <= 2 {
        let hull_vertices = filtered;
        let hull_coords: Vec<(f64, f64)> = hull_vertices.iter().map(|&i| points[i]).collect();
        return Ok(ConvexHullResult {
            hull_vertices,
            hull_coords,
        });
    }

    // Graham scan.
    let mut stack: Vec<usize> = Vec::with_capacity(filtered.len());
    stack.push(filtered[0]);
    stack.push(filtered[1]);

    for &idx in filtered.iter().skip(2) {
        while stack.len() >= 2 {
            let top = stack[stack.len() - 1];
            let below = stack[stack.len() - 2];
            let cp = cross_product(points[below], points[top], points[idx]);
            if cp > 0.0 {
                break;
            }
            stack.pop();
        }
        stack.push(idx);
    }

    let hull_coords: Vec<(f64, f64)> = stack.iter().map(|&i| points[i]).collect();
    Ok(ConvexHullResult {
        hull_vertices: stack,
        hull_coords,
    })
}

fn dist_sq(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    (x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)
}

fn cross_product(o: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - o.0) * (b.1 - o.1) - (b.0 - o.0) * (a.1 - o.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let r = convex_hull_2d(&[]).unwrap();
        assert!(r.hull_vertices.is_empty());
        assert!(r.hull_coords.is_empty());
    }

    #[test]
    fn single_point() {
        let r = convex_hull_2d(&[(1.0, 2.0)]).unwrap();
        assert_eq!(r.hull_vertices, vec![0]);
        assert_eq!(r.hull_coords, vec![(1.0, 2.0)]);
    }

    #[test]
    fn two_points() {
        let r = convex_hull_2d(&[(0.0, 0.0), (1.0, 1.0)]).unwrap();
        assert_eq!(r.hull_vertices.len(), 2);
    }

    #[test]
    fn triangle() {
        let pts = vec![(0.0, 0.0), (1.0, 0.0), (0.5, 1.0)];
        let r = convex_hull_2d(&pts).unwrap();
        assert_eq!(r.hull_vertices.len(), 3);
        let mut sorted = r.hull_vertices.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![0, 1, 2]);
    }

    #[test]
    fn square_with_interior() {
        let pts = vec![
            (0.0, 0.0),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 1.0),
            (0.5, 0.5), // interior
        ];
        let r = convex_hull_2d(&pts).unwrap();
        assert_eq!(r.hull_vertices.len(), 4);
        assert!(!r.hull_vertices.contains(&4));
    }

    #[test]
    fn collinear_points() {
        let pts = vec![(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)];
        let r = convex_hull_2d(&pts).unwrap();
        // Collinear points: hull is just the two endpoints.
        assert_eq!(r.hull_vertices.len(), 2);
        let mut sorted = r.hull_vertices.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![0, 3]);
    }

    #[test]
    fn regular_hexagon() {
        let pts: Vec<(f64, f64)> = (0..6)
            .map(|i| {
                let angle = std::f64::consts::PI / 3.0 * f64::from(i);
                (angle.cos(), angle.sin())
            })
            .collect();
        let r = convex_hull_2d(&pts).unwrap();
        assert_eq!(r.hull_vertices.len(), 6);
    }

    #[test]
    fn nan_coordinate_error() {
        let pts = vec![(0.0, 0.0), (f64::NAN, 1.0)];
        assert!(convex_hull_2d(&pts).is_err());
    }

    #[test]
    fn duplicate_points() {
        let pts = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 0.0)];
        let r = convex_hull_2d(&pts).unwrap();
        assert_eq!(r.hull_vertices.len(), 3);
    }

    #[test]
    fn large_random_circle() {
        // Points on a unit circle should all be on the hull.
        let n: u32 = 20;
        let pts: Vec<(f64, f64)> = (0..n)
            .map(|i| {
                let angle = 2.0 * std::f64::consts::PI * f64::from(i) / f64::from(n);
                (angle.cos(), angle.sin())
            })
            .collect();
        let r = convex_hull_2d(&pts).unwrap();
        assert_eq!(r.hull_vertices.len(), n as usize);
    }

    #[test]
    fn hull_is_ccw() {
        // Verify the hull is in counter-clockwise order.
        let pts = vec![(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0), (1.0, 1.0)];
        let r = convex_hull_2d(&pts).unwrap();
        // Check that all consecutive triples make left turns (positive cross product).
        let hull = &r.hull_coords;
        let len = hull.len();
        for i in 0..len {
            let cp = cross_product(hull[i], hull[(i + 1) % len], hull[(i + 2) % len]);
            assert!(
                cp >= 0.0,
                "hull is not CCW at index {i}: cross product = {cp}"
            );
        }
    }
}
